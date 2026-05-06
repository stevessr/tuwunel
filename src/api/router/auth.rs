mod appservice;
mod server;
mod uiaa;

use std::{any::TypeId, fmt::Debug, time::SystemTime};

use axum::RequestPartsExt;
use axum_extra::{
	TypedHeader,
	headers::{Authorization, authorization::Bearer},
};
use futures::{
	TryFutureExt,
	future::{
		Either::{Left, Right},
		select_ok,
	},
	pin_mut, try_join,
};
use ruma::{
	CanonicalJsonValue, OwnedDeviceId, OwnedServerName, OwnedUserId,
	api::{
		IncomingRequest,
		auth_scheme::{
			AccessToken, AccessTokenOptional, AppserviceToken, AppserviceTokenOptional,
			AuthScheme, NoAccessToken, NoAuthentication,
		},
		client::{
			directory::get_public_rooms,
			profile::{
				get_avatar_url, get_display_name, get_profile, get_profile_field, set_avatar_url,
				set_display_name,
			},
			session::{logout, logout_all},
			voip::get_turn_server_info,
		},
		error::{ErrorKind, UnknownTokenErrorData},
		federation::{authentication::ServerSignatures, openid::get_openid_userinfo},
	},
};
use tuwunel_core::{Err, Error, Result, is_less_than, utils::result::LogDebugErr};
use tuwunel_service::{Services, appservice::RegistrationInfo};

pub(crate) use self::uiaa::auth_uiaa;
use self::{appservice::auth_appservice, server::auth_server};
use super::request::Request;

pub(super) enum Token {
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
pub(super) async fn auth<T>(
	services: &Services,
	request: &mut Request,
	json_body: Option<&CanonicalJsonValue>,
) -> Result<Auth>
where
	T: IncomingRequest + Debug + Send + Sync + 'static,
	T::Authentication: AuthDispatch,
{
	let bearer: Option<TypedHeader<Authorization<Bearer>>> =
		request.parts.extract().await.unwrap_or(None);

	let token = match &bearer {
		| Some(TypedHeader(Authorization(bearer))) => Some(bearer.token()),
		| None => request.query.access_token.as_deref(),
	};

	let token = match find_token(services, token).await? {
		| Token::User((user_id, device_id, expires_at))
			if expires_at.is_some_and(is_less_than!(SystemTime::now())) =>
			Token::Expired((user_id, device_id)),

		| token => token,
	};

	if T::Authentication::SCHEME == Scheme::None {
		check_auth_still_required::<T>(services, &token)?;
	}

	let auth = T::Authentication::dispatch::<T>(services, request, json_body, token).await?;

	try_join!(
		locked_account_check::<T>(services, &auth),
		suspended_account_check::<T>(services, &auth),
	)?;

	Ok(auth)
}

/// MSC3939: 401 `M_USER_LOCKED` for locked accounts; logout endpoints
/// bypass. `soft_logout: true` is emitted by ruma for this errcode.
async fn locked_account_check<T>(services: &Services, auth: &Auth) -> Result
where
	T: IncomingRequest + 'static,
{
	let Some(user_id) = auth.sender_user.as_deref() else {
		return Ok(());
	};

	let id = TypeId::of::<T>();
	let is_logout = id == TypeId::of::<logout::v3::Request>()
		|| id == TypeId::of::<logout_all::v3::Request>();

	if is_logout || !services.users.is_locked(user_id).await {
		return Ok(());
	}

	Err!(Request(UserLocked("This account has been locked.")))
}

/// MSC3823: 403 `M_USER_SUSPENDED` on `set_display_name` / `set_avatar_url`
/// for suspended callers. Companion checks: per-field in the profile
/// handlers, per-PDU in `timeline::build_and_append_pdu`.
async fn suspended_account_check<T>(services: &Services, auth: &Auth) -> Result
where
	T: IncomingRequest + 'static,
{
	let Some(user_id) = auth.sender_user.as_deref() else {
		return Ok(());
	};

	let id = TypeId::of::<T>();
	let blocked = id == TypeId::of::<set_display_name::v3::Request>()
		|| id == TypeId::of::<set_avatar_url::v3::Request>();

	if !blocked || !services.users.is_suspended(user_id).await {
		return Ok(());
	}

	Err!(Request(UserSuspended("Account is suspended.")))
}

/// Tag identifying an [`AuthScheme`] for tuwunel's purposes.
///
/// Ruma's `AuthScheme` is a trait, so endpoint-specific bypasses cannot be
/// expressed as enum match arms anymore. This tag is the value-side handle
/// used to route through `auth()` and to identify the unauthenticated case
/// inside [`check_auth_still_required`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Scheme {
	None,
	AccessToken,
	AccessTokenOptional,
	AppserviceToken,
	AppserviceTokenOptional,
	ServerSignatures,
}

/// Trait routing a concrete [`AuthScheme`] through the per-scheme dispatch.
pub(super) trait AuthDispatch: AuthScheme {
	const SCHEME: Scheme;

	fn dispatch<T>(
		services: &Services,
		request: &mut Request,
		json_body: Option<&CanonicalJsonValue>,
		token: Token,
	) -> impl Future<Output = Result<Auth>> + Send
	where
		T: IncomingRequest + Debug + Send + Sync + 'static;
}

impl AuthDispatch for NoAccessToken {
	const SCHEME: Scheme = Scheme::None;

	async fn dispatch<T>(
		services: &Services,
		request: &mut Request,
		json_body: Option<&CanonicalJsonValue>,
		token: Token,
	) -> Result<Auth>
	where
		T: IncomingRequest + Debug + Send + Sync + 'static,
	{
		<NoAuthentication as AuthDispatch>::dispatch::<T>(services, request, json_body, token)
			.await
	}
}

impl AuthDispatch for NoAuthentication {
	const SCHEME: Scheme = Scheme::None;

	async fn dispatch<T>(
		services: &Services,
		request: &mut Request,
		_json_body: Option<&CanonicalJsonValue>,
		token: Token,
	) -> Result<Auth>
	where
		T: IncomingRequest + Debug + Send + Sync + 'static,
	{
		match token {
			| Token::Invalid
				if request.query.access_token.is_some()
					&& TypeId::of::<T>() == TypeId::of::<get_openid_userinfo::v1::Request>() =>
			{
				// OpenID federation endpoint uses a query param with the same name; drop
				// once query params for user auth are removed from the spec. Required to
				// make the integration manager work.
				Ok(Auth::default())
			},

			| Token::Invalid => unknown_token(),
			| Token::Expired((user_id, device_id)) =>
				expired_token(services, user_id, device_id).await,

			| Token::User(user) => Ok(Auth {
				sender_user: Some(user.0),
				sender_device: Some(user.1),
				_expires_at: user.2,
				..Auth::default()
			}),

			| Token::Appservice(info) => Ok(Auth {
				appservice_info: Some(*info),
				..Auth::default()
			}),

			| Token::None => Ok(Auth::default()),
		}
	}
}

impl AuthDispatch for AccessToken {
	const SCHEME: Scheme = Scheme::AccessToken;

	async fn dispatch<T>(
		services: &Services,
		request: &mut Request,
		_json_body: Option<&CanonicalJsonValue>,
		token: Token,
	) -> Result<Auth>
	where
		T: IncomingRequest + Debug + Send + Sync + 'static,
	{
		match token {
			| Token::Invalid => unknown_token(),
			| Token::Expired((user_id, device_id)) =>
				expired_token(services, user_id, device_id).await,
			| Token::Appservice(info) => Ok(auth_appservice(services, request, info).await?),
			| Token::User(user) => Ok(Auth {
				sender_user: Some(user.0),
				sender_device: Some(user.1),
				_expires_at: user.2,
				..Auth::default()
			}),
			| Token::None
				if TypeId::of::<T>() == TypeId::of::<get_turn_server_info::v3::Request>()
					&& services.server.config.turn_allow_guests =>
				Ok(Auth::default()),

			| Token::None => Err!(Request(MissingToken("Missing access token."))),
		}
	}
}

impl AuthDispatch for AccessTokenOptional {
	const SCHEME: Scheme = Scheme::AccessTokenOptional;

	async fn dispatch<T>(
		services: &Services,
		_request: &mut Request,
		_json_body: Option<&CanonicalJsonValue>,
		token: Token,
	) -> Result<Auth>
	where
		T: IncomingRequest + Debug + Send + Sync + 'static,
	{
		match token {
			| Token::Invalid => unknown_token(),
			| Token::Expired((user_id, device_id)) =>
				expired_token(services, user_id, device_id).await,
			| Token::User(user) => Ok(Auth {
				sender_user: Some(user.0),
				sender_device: Some(user.1),
				_expires_at: user.2,
				..Auth::default()
			}),
			| Token::Appservice(info) => Ok(Auth {
				appservice_info: Some(*info),
				..Auth::default()
			}),
			| Token::None => Ok(Auth::default()),
		}
	}
}

impl AuthDispatch for AppserviceToken {
	const SCHEME: Scheme = Scheme::AppserviceToken;

	async fn dispatch<T>(
		services: &Services,
		_request: &mut Request,
		_json_body: Option<&CanonicalJsonValue>,
		token: Token,
	) -> Result<Auth>
	where
		T: IncomingRequest + Debug + Send + Sync + 'static,
	{
		match token {
			| Token::Invalid => unknown_token(),
			| Token::Expired((user_id, device_id)) =>
				expired_token(services, user_id, device_id).await,
			| Token::User(_) =>
				Err!(Request(Unauthorized("Appservice tokens must be used on this endpoint."))),
			| Token::Appservice(info) => Ok(Auth {
				appservice_info: Some(*info),
				..Auth::default()
			}),
			| Token::None => Err!(Request(MissingToken("Missing access token."))),
		}
	}
}

impl AuthDispatch for AppserviceTokenOptional {
	const SCHEME: Scheme = Scheme::AppserviceTokenOptional;

	async fn dispatch<T>(
		services: &Services,
		_request: &mut Request,
		_json_body: Option<&CanonicalJsonValue>,
		token: Token,
	) -> Result<Auth>
	where
		T: IncomingRequest + Debug + Send + Sync + 'static,
	{
		match token {
			| Token::Invalid => unknown_token(),
			| Token::Expired((user_id, device_id)) =>
				expired_token(services, user_id, device_id).await,
			| Token::User(user) => Ok(Auth {
				sender_user: Some(user.0),
				sender_device: Some(user.1),
				_expires_at: user.2,
				..Auth::default()
			}),
			| Token::Appservice(info) => Ok(Auth {
				appservice_info: Some(*info),
				..Auth::default()
			}),
			| Token::None => Ok(Auth::default()),
		}
	}
}

impl AuthDispatch for ServerSignatures {
	const SCHEME: Scheme = Scheme::ServerSignatures;

	async fn dispatch<T>(
		services: &Services,
		request: &mut Request,
		json_body: Option<&CanonicalJsonValue>,
		token: Token,
	) -> Result<Auth>
	where
		T: IncomingRequest + Debug + Send + Sync + 'static,
	{
		match token {
			| Token::Invalid => unknown_token(),
			| Token::Expired((user_id, device_id)) =>
				expired_token(services, user_id, device_id).await,
			| Token::Appservice(_) | Token::User(_) =>
				Err!(Request(Unauthorized("Server signatures must be used on this endpoint."))),
			| Token::None => Ok(auth_server(services, request, json_body).await?),
		}
	}
}

fn unknown_token() -> Result<Auth> {
	Err(Error::BadRequest(
		ErrorKind::UnknownToken(UnknownTokenErrorData::new()),
		"Unknown access token.",
	))
}

async fn expired_token(
	services: &Services,
	user_id: OwnedUserId,
	device_id: OwnedDeviceId,
) -> Result<Auth> {
	services
		.users
		.remove_access_token(&user_id, &device_id)
		.await
		.log_debug_err()
		.ok();

	Err(Error::BadRequest(
		ErrorKind::UnknownToken(UnknownTokenErrorData { soft_logout: true }),
		"Expired access token.",
	))
}

fn check_auth_still_required<T>(services: &Services, token: &Token) -> Result
where
	T: IncomingRequest + 'static,
{
	let id = TypeId::of::<T>();

	let is_profile = id == TypeId::of::<get_profile::v3::Request>()
		|| id == TypeId::of::<get_profile_field::v3::Request>()
		|| id == TypeId::of::<get_display_name::v3::Request>()
		|| id == TypeId::of::<get_avatar_url::v3::Request>();

	let is_public_rooms = id == TypeId::of::<get_public_rooms::v3::Request>();

	if (is_profile
		&& services
			.server
			.config
			.require_auth_for_profile_requests)
		|| (is_public_rooms
			&& !services
				.server
				.config
				.allow_public_room_directory_without_auth)
	{
		match token {
			| Token::Appservice(_) | Token::User(_) => Ok(()),
			| Token::None | Token::Expired(_) | Token::Invalid =>
				Err!(Request(MissingToken("Missing or invalid access token."))),
		}
	} else {
		Ok(())
	}
}

async fn find_token(services: &Services, token: Option<&str>) -> Result<Token> {
	let Some(token) = token else {
		return Ok(Token::None);
	};

	let user_token = services
		.users
		.find_from_token(token)
		.map_ok(Token::User);

	let appservice_token = services
		.appservice
		.find_from_access_token(token)
		.map_ok(Box::new)
		.map_ok(Token::Appservice);

	pin_mut!(user_token, appservice_token);
	match select_ok([Left(user_token), Right(appservice_token)]).await {
		| Err(e) if !e.is_not_found() => Err(e),
		| Ok((token, _)) => Ok(token),
		| _ => Ok(Token::Invalid),
	}
}
