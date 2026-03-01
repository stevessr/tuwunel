mod appservice;
mod server;
mod uiaa;

use std::{collections::BTreeSet, fmt::Debug, time::SystemTime};

use axum::RequestPartsExt;
use axum_extra::{
	TypedHeader,
	headers::{Authorization, authorization::Bearer},
};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use ruma::{
	CanonicalJsonValue, OwnedDeviceId, OwnedServerName, OwnedUserId, UserId,
	api::{
		AuthScheme, IncomingRequest, Metadata,
		client::{
			directory::get_public_rooms,
			error::ErrorKind,
			profile::{
				get_avatar_url, get_display_name, get_profile, get_profile_field,
				get_timezone_key,
			},
			voip::get_turn_server_info,
		},
		federation::openid::get_openid_userinfo,
	},
};
use serde::{Deserialize, Serialize};
use tuwunel_core::{Err, Error, Result, err, is_less_than, utils::result::LogDebugErr};
use tuwunel_service::{Services, appservice::RegistrationInfo, users::Register};

pub(crate) use self::uiaa::auth_uiaa;
use self::{appservice::auth_appservice, server::auth_server};
use super::request::Request;

const UNSTABLE_SCOPE_MATRIX_API: &str = "urn:matrix:org.matrix.msc2967.client:api:*";
const STABLE_SCOPE_MATRIX_API: &str = "urn:matrix:client:api:*";
const UNSTABLE_SCOPE_MATRIX_DEVICE_PREFIX: &str = "urn:matrix:org.matrix.msc2967.client:device:";
const STABLE_SCOPE_MATRIX_DEVICE_PREFIX: &str = "urn:matrix:client:device:";
const MAS_SUPPORTS_DEVICE_ID_HEADER: &str = "X-MAS-Supports-Device-Id";

enum Token {
	Appservice(Box<RegistrationInfo>),
	User((OwnedUserId, OwnedDeviceId, Option<SystemTime>)),
	Expired((OwnedUserId, OwnedDeviceId)),
	Invalid,
	None,
}

#[derive(Debug, Default)]
pub(super) struct Auth {
	pub(super) origin: Option<OwnedServerName>,
	pub(super) sender_user: Option<OwnedUserId>,
	pub(super) sender_device: Option<OwnedDeviceId>,
	pub(super) appservice_info: Option<RegistrationInfo>,
	pub(super) _expires_at: Option<SystemTime>,
}

#[tracing::instrument(
	level = "trace",
	skip(services, request, json_body),
	err(level = "debug"),
	ret
)]
pub(super) async fn auth(
	services: &Services,
	request: &mut Request,
	json_body: Option<&CanonicalJsonValue>,
	metadata: &Metadata,
) -> Result<Auth> {
	use AuthScheme::{
		AccessToken, AccessTokenOptional, AppserviceToken, AppserviceTokenOptional,
		ServerSignatures,
	};
	use Error::BadRequest;
	use ErrorKind::UnknownToken;
	use Token::{Appservice, Expired, Invalid, User};

	let bearer: Option<TypedHeader<Authorization<Bearer>>> =
		request.parts.extract().await.unwrap_or(None);

	let token = match &bearer {
		| Some(TypedHeader(Authorization(bearer))) => Some(bearer.token()),
		| None => request.query.access_token.as_deref(),
	};

	let token = match find_token(services, token).await? {
		| User((user_id, device_id, expires_at))
			if expires_at.is_some_and(is_less_than!(SystemTime::now())) =>
		{
			Expired((user_id, device_id))
		},

		| token => token,
	};

	if metadata.authentication == AuthScheme::None {
		check_auth_still_required(services, metadata, &token)?;
	}

	match (metadata.authentication, token) {
		| (AuthScheme::None, Invalid)
			if request.query.access_token.is_some()
				&& metadata == &get_openid_userinfo::v1::Request::METADATA =>
		{
			// OpenID federation endpoint uses a query param with the same name, drop this
			// once query params for user auth are removed from the spec. This is
			// required to make integration manager work.
			Ok(Auth::default())
		},

		| (_, Invalid) => {
			Err(BadRequest(UnknownToken { soft_logout: false }, "Unknown access token."))
		},

		| (_, Expired((user_id, device_id))) => {
			services
				.users
				.remove_access_token(&user_id, &device_id)
				.await
				.log_debug_err()
				.ok();

			Err(BadRequest(UnknownToken { soft_logout: true }, "Expired access token."))
		},

		| (AppserviceToken, User(_)) => {
			Err!(Request(Unauthorized("Appservice tokens must be used on this endpoint.")))
		},

		| (ServerSignatures, Appservice(_) | User(_)) => {
			Err!(Request(Unauthorized("Server signatures must be used on this endpoint.")))
		},

		| (ServerSignatures, Token::None) => Ok(auth_server(services, request, json_body).await?),

		| (AccessToken, Appservice(info)) => Ok(auth_appservice(services, request, info).await?),

		| (AccessToken | AppserviceToken, Token::None) => match metadata {
			| &get_turn_server_info::v3::Request::METADATA
				if services.server.config.turn_allow_guests =>
			{
				Ok(Auth::default())
			},

			| _ => Err!(Request(MissingToken("Missing access token."))),
		},

		| (
			AccessToken | AccessTokenOptional | AppserviceTokenOptional | AuthScheme::None,
			User(user),
		) => Ok(Auth {
			sender_user: Some(user.0),
			sender_device: Some(user.1),
			_expires_at: user.2,
			..Auth::default()
		}),

		| (
			AccessTokenOptional | AppserviceTokenOptional | AppserviceToken | AuthScheme::None,
			Appservice(info),
		) => Ok(Auth {
			appservice_info: Some(*info),
			..Auth::default()
		}),

		| (AccessTokenOptional | AppserviceTokenOptional | AuthScheme::None, Token::None) => {
			Ok(Auth::default())
		},
	}
}

fn check_auth_still_required(services: &Services, metadata: &Metadata, token: &Token) -> Result {
	debug_assert_eq!(
		metadata.authentication,
		AuthScheme::None,
		"Expected endpoint to be unauthenticated"
	);

	match metadata {
		| &get_profile::v3::Request::METADATA
		| &get_profile_field::v3::Request::METADATA
		| &get_display_name::v3::Request::METADATA
		| &get_avatar_url::v3::Request::METADATA
		| &get_timezone_key::unstable::Request::METADATA
			if services
				.server
				.config
				.require_auth_for_profile_requests =>
		{
			match token {
				| Token::Appservice(_) | Token::User(_) => Ok(()),
				| Token::None | Token::Expired(_) | Token::Invalid => {
					Err!(Request(MissingToken("Missing or invalid access token.")))
				},
			}
		},
		| &get_public_rooms::v3::Request::METADATA
			if !services
				.server
				.config
				.allow_public_room_directory_without_auth =>
		{
			match token {
				| Token::Appservice(_) | Token::User(_) => Ok(()),
				| Token::None | Token::Expired(_) | Token::Invalid => {
					Err!(Request(MissingToken("Missing or invalid access token.")))
				},
			}
		},
		| _ => Ok(()),
	}
}

async fn find_token(services: &Services, token: Option<&str>) -> Result<Token> {
	let Some(token) = token else {
		return Ok(Token::None);
	};

	match services.users.find_from_token(token).await {
		| Ok(token) => return Ok(Token::User(token)),
		| Err(e) if !e.is_not_found() => return Err(e),
		| Err(_) => {},
	}

	match services
		.appservice
		.find_from_access_token(token)
		.await
	{
		| Ok(info) => return Ok(Token::Appservice(Box::new(info))),
		| Err(e) if !e.is_not_found() => return Err(e),
		| Err(_) => {},
	}

	match find_token_mas(services, token).await? {
		| Some(token) => Ok(Token::User(token)),
		| None => Ok(Token::Invalid),
	}
}

#[derive(Debug, Serialize)]
struct MasIntrospectionRequest<'a> {
	token: &'a str,
	token_type_hint: &'static str,
}

#[derive(Debug, Deserialize)]
struct MasIntrospectionResponse {
	active: bool,
	scope: Option<String>,
	username: Option<String>,
	device_id: Option<String>,
}

async fn find_token_mas(
	services: &Services,
	token: &str,
) -> Result<Option<(OwnedUserId, OwnedDeviceId, Option<SystemTime>)>> {
	let config = &services
		.server
		.config
		.matrix_authentication_service;
	if !config.enabled {
		return Ok(None);
	}

	let secret = config
		.get_secret()
		.await?
		.filter(|secret| !secret.is_empty())
		.ok_or_else(|| {
			err!(Config(
				"matrix_authentication_service.secret",
				"MAS integration is enabled but no secret is configured."
			))
		})?;

	let endpoint = config.endpoint.join("/oauth2/introspect")?;
	let response = services
		.client
		.oauth
		.post(endpoint)
		.header(ACCEPT, "application/json")
		.header(CONTENT_TYPE, "application/x-www-form-urlencoded")
		.header(AUTHORIZATION, format!("Bearer {secret}"))
		.header(MAS_SUPPORTS_DEVICE_ID_HEADER, "1")
		.body(serde_html_form::to_string(&MasIntrospectionRequest {
			token,
			token_type_hint: "access_token",
		})?)
		.send()
		.await?
		.error_for_status()?
		.json::<MasIntrospectionResponse>()
		.await?;

	if !response.active {
		return Ok(None);
	}

	let scope = scope_set(response.scope.as_deref());
	if !scope.contains(UNSTABLE_SCOPE_MATRIX_API) && !scope.contains(STABLE_SCOPE_MATRIX_API) {
		return Ok(None);
	}

	let Some(username) = response
		.username
		.as_deref()
		.filter(|username| !username.is_empty())
	else {
		return Ok(None);
	};

	let user_id = match UserId::parse_with_server_name(username, services.globals.server_name()) {
		| Ok(user_id) => user_id,
		| Err(_) => return Ok(None),
	};

	let device_id = match extract_device_id(response.device_id.as_deref(), &scope) {
		| Some(device_id) => OwnedDeviceId::from(device_id),
		| None => return Ok(None),
	};

	if !services.users.exists(&user_id).await {
		services
			.users
			.full_register(Register {
				user_id: Some(&user_id),
				password: Some("*"),
				origin: Some("mas"),
				grant_first_user_admin: true,
				..Default::default()
			})
			.await?;
	}

	if !services
		.users
		.device_exists(&user_id, &device_id)
		.await
	{
		services
			.users
			.create_device(&user_id, Some(&device_id), (None, None), None, None, None)
			.await?;
	}

	Ok(Some((user_id, device_id, None)))
}

fn scope_set(scope: Option<&str>) -> BTreeSet<&str> {
	scope
		.into_iter()
		.flat_map(|scope| scope.split_whitespace())
		.filter(|scope| !scope.is_empty())
		.collect()
}

fn extract_device_id(device_id: Option<&str>, scope: &BTreeSet<&str>) -> Option<String> {
	if let Some(device_id) = device_id.filter(|device_id| !device_id.is_empty()) {
		return Some(device_id.to_owned());
	}

	let mut candidates = scope
		.iter()
		.filter_map(|scope| {
			scope
				.strip_prefix(UNSTABLE_SCOPE_MATRIX_DEVICE_PREFIX)
				.or_else(|| scope.strip_prefix(STABLE_SCOPE_MATRIX_DEVICE_PREFIX))
		})
		.filter(|scope| !scope.is_empty())
		.collect::<BTreeSet<_>>();

	(candidates.len() == 1)
		.then(|| {
			candidates
				.pop_first()
				.expect("length checked before pop")
		})
		.map(ToOwned::to_owned)
}
