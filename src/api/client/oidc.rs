use std::time::SystemTime;

use axum::{
	Json,
	extract::State,
	response::{IntoResponse, Redirect},
};
use axum_extra::{
	TypedHeader,
	headers::{Authorization, authorization::Bearer},
};
use http::StatusCode;
use ruma::OwnedDeviceId;
use serde::{Deserialize, Serialize};
use tuwunel_core::{Err, Result, err, info, utils};
use tuwunel_service::{
	oauth::oidc_server::{
		DcrRequest, IdTokenClaims, OidcAuthRequest, OidcServer, ProviderMetadata,
	},
	users::device::generate_refresh_token,
};

const OIDC_REQ_ID_LENGTH: usize = 32;

#[derive(Serialize)]
struct AuthIssuerResponse {
	issuer: String,
}

pub(crate) async fn auth_issuer_route(
	State(services): State<crate::State>,
) -> Result<impl IntoResponse> {
	let issuer = oidc_issuer_url(&services)?;
	Ok(Json(AuthIssuerResponse { issuer }))
}

pub(crate) async fn openid_configuration_route(
	State(services): State<crate::State>,
) -> Result<impl IntoResponse> {
	Ok(Json(oidc_metadata(&services)?))
}

fn oidc_metadata(services: &tuwunel_service::Services) -> Result<ProviderMetadata> {
	let issuer = oidc_issuer_url(services)?;
	let base = issuer.trim_end_matches('/').to_owned();

	Ok(ProviderMetadata {
		issuer,
		authorization_endpoint: format!("{base}/_tuwunel/oidc/authorize"),
		token_endpoint: format!("{base}/_tuwunel/oidc/token"),
		registration_endpoint: Some(format!("{base}/_tuwunel/oidc/registration")),
		revocation_endpoint: Some(format!("{base}/_tuwunel/oidc/revoke")),
		jwks_uri: format!("{base}/_tuwunel/oidc/jwks"),
		userinfo_endpoint: Some(format!("{base}/_tuwunel/oidc/userinfo")),
		account_management_uri: Some(format!("{base}/_tuwunel/oidc/account")),
		account_management_actions_supported: Some(vec![
			"org.matrix.profile".to_owned(),
			"org.matrix.sessions_list".to_owned(),
			"org.matrix.session_view".to_owned(),
			"org.matrix.session_end".to_owned(),
			"org.matrix.cross_signing_reset".to_owned(),
		]),
		response_types_supported: vec!["code".to_owned()],
		response_modes_supported: Some(vec!["query".to_owned(), "fragment".to_owned()]),
		grant_types_supported: Some(vec![
			"authorization_code".to_owned(),
			"refresh_token".to_owned(),
		]),
		code_challenge_methods_supported: Some(vec!["S256".to_owned()]),
		token_endpoint_auth_methods_supported: Some(vec![
			"none".to_owned(),
			"client_secret_basic".to_owned(),
			"client_secret_post".to_owned(),
		]),
		scopes_supported: Some(vec![
			"openid".to_owned(),
			"urn:matrix:org.matrix.msc2967.client:api:*".to_owned(),
			"urn:matrix:org.matrix.msc2967.client:device:*".to_owned(),
		]),
		subject_types_supported: Some(vec!["public".to_owned()]),
		id_token_signing_alg_values_supported: Some(vec!["ES256".to_owned()]),
		prompt_values_supported: Some(vec!["create".to_owned()]),
		claim_types_supported: Some(vec!["normal".to_owned()]),
		claims_supported: Some(vec![
			"iss".to_owned(),
			"sub".to_owned(),
			"aud".to_owned(),
			"exp".to_owned(),
			"iat".to_owned(),
			"nonce".to_owned(),
		]),
	})
}

pub(crate) async fn registration_route(
	State(services): State<crate::State>,
	Json(body): Json<DcrRequest>,
) -> Result<impl IntoResponse> {
	let Ok(oidc) = get_oidc_server(&services) else {
		return Err!(Request(NotFound("OIDC server not configured")));
	};

	if body.redirect_uris.is_empty() {
		return Err!(Request(InvalidParam("redirect_uris must not be empty")));
	}

	let reg = oidc.register_client(body)?;
	info!(
		"OIDC client registered: {} ({})",
		reg.client_id,
		reg.client_name.as_deref().unwrap_or("unnamed")
	);

	Ok((
		StatusCode::CREATED,
		Json(
			serde_json::json!({"client_id": reg.client_id, "client_id_issued_at": reg.registered_at, "redirect_uris": reg.redirect_uris, "client_name": reg.client_name, "client_uri": reg.client_uri, "logo_uri": reg.logo_uri, "contacts": reg.contacts, "token_endpoint_auth_method": reg.token_endpoint_auth_method, "grant_types": reg.grant_types, "response_types": reg.response_types, "application_type": reg.application_type, "policy_uri": reg.policy_uri, "tos_uri": reg.tos_uri, "software_id": reg.software_id, "software_version": reg.software_version}),
		),
	))
}

#[derive(Debug, Deserialize)]
pub(crate) struct AuthorizeParams {
	client_id: String,
	redirect_uri: String,
	response_type: String,
	scope: String,
	state: Option<String>,
	nonce: Option<String>,
	code_challenge: Option<String>,
	code_challenge_method: Option<String>,
	#[serde(default, rename = "prompt")]
	_prompt: Option<String>,
}

pub(crate) async fn authorize_route(
	State(services): State<crate::State>,
	request: axum::extract::Request,
) -> Result<impl IntoResponse> {
	let params: AuthorizeParams =
		serde_html_form::from_str(request.uri().query().unwrap_or_default())?;
	let Ok(oidc) = get_oidc_server(&services) else {
		return Err!(Request(NotFound("OIDC server not configured")));
	};

	if params.response_type != "code" {
		return Err!(Request(InvalidParam("Only response_type=code is supported")));
	}

	oidc.validate_redirect_uri(&params.client_id, &params.redirect_uri)
		.await?;

	let req_id = utils::random_string(OIDC_REQ_ID_LENGTH);
	let now = SystemTime::now();

	oidc.store_auth_request(&req_id, &OidcAuthRequest {
		client_id: params.client_id,
		redirect_uri: params.redirect_uri,
		scope: params.scope,
		state: params.state,
		nonce: params.nonce,
		code_challenge: params.code_challenge,
		code_challenge_method: params.code_challenge_method,
		created_at: now,
		expires_at: now
			.checked_add(OidcServer::auth_request_lifetime())
			.unwrap_or(now),
	});

	let default_idp = services
		.config
		.identity_provider
		.values()
		.find(|idp| idp.default)
		.or_else(|| services.config.identity_provider.values().next())
		.ok_or_else(|| err!(Config("identity_provider", "No identity provider configured")))?;
	let idp_id = default_idp.id();

	let base = oidc_issuer_url(&services)?;
	let base = base.trim_end_matches('/');

	let mut complete_url = url::Url::parse(&format!("{base}/_tuwunel/oidc/_complete"))
		.map_err(|_| err!(error!("Failed to build complete URL")))?;
	complete_url
		.query_pairs_mut()
		.append_pair("oidc_req_id", &req_id);

	let mut sso_url =
		url::Url::parse(&format!("{base}/_matrix/client/v3/login/sso/redirect/{idp_id}"))
			.map_err(|_| err!(error!("Failed to build SSO URL")))?;
	sso_url
		.query_pairs_mut()
		.append_pair("redirectUrl", complete_url.as_str());

	Ok(Redirect::temporary(sso_url.as_str()))
}

#[derive(Debug, Deserialize)]
pub(crate) struct CompleteParams {
	oidc_req_id: String,
	#[serde(rename = "loginToken")]
	login_token: String,
}

pub(crate) async fn complete_route(
	State(services): State<crate::State>,
	request: axum::extract::Request,
) -> Result<impl IntoResponse> {
	let params: CompleteParams =
		serde_html_form::from_str(request.uri().query().unwrap_or_default())?;
	let Ok(oidc) = get_oidc_server(&services) else {
		return Err!(Request(NotFound("OIDC server not configured")));
	};

	let user_id = services
		.users
		.find_from_login_token(&params.login_token)
		.await
		.map_err(|_| err!(Request(Forbidden("Invalid or expired login token"))))?;
	let auth_req = oidc
		.take_auth_request(&params.oidc_req_id)
		.await?;
	let code = oidc.create_auth_code(&auth_req, user_id);

	let mut redirect_url = url::Url::parse(&auth_req.redirect_uri)
		.map_err(|_| err!(Request(InvalidParam("Invalid redirect_uri"))))?;
	redirect_url
		.query_pairs_mut()
		.append_pair("code", &code);
	if let Some(state) = &auth_req.state {
		redirect_url
			.query_pairs_mut()
			.append_pair("state", state);
	}

	Ok(Redirect::temporary(redirect_url.as_str()))
}

#[derive(Debug, Deserialize)]
pub(crate) struct TokenRequest {
	grant_type: String,
	code: Option<String>,
	redirect_uri: Option<String>,
	client_id: Option<String>,
	code_verifier: Option<String>,
	refresh_token: Option<String>,
	#[serde(rename = "scope")]
	_scope: Option<String>,
}

pub(crate) async fn token_route(
	State(services): State<crate::State>,
	axum::extract::Form(body): axum::extract::Form<TokenRequest>,
) -> impl IntoResponse {
	match body.grant_type.as_str() {
		| "authorization_code" => token_authorization_code(&services, &body)
			.await
			.unwrap_or_else(|e| {
				oauth_error(StatusCode::INTERNAL_SERVER_ERROR, "server_error", &e.to_string())
			}),
		| "refresh_token" => token_refresh(&services, &body)
			.await
			.unwrap_or_else(|e| {
				oauth_error(StatusCode::INTERNAL_SERVER_ERROR, "server_error", &e.to_string())
			}),
		| _ => oauth_error(
			StatusCode::BAD_REQUEST,
			"unsupported_grant_type",
			"Unsupported grant_type",
		),
	}
}

async fn token_authorization_code(
	services: &tuwunel_service::Services,
	body: &TokenRequest,
) -> Result<http::Response<axum::body::Body>> {
	let code = body
		.code
		.as_deref()
		.ok_or_else(|| err!(Request(InvalidParam("code is required"))))?;
	let redirect_uri = body
		.redirect_uri
		.as_deref()
		.ok_or_else(|| err!(Request(InvalidParam("redirect_uri is required"))))?;
	let client_id = body
		.client_id
		.as_deref()
		.ok_or_else(|| err!(Request(InvalidParam("client_id is required"))))?;

	let oidc = get_oidc_server(services)?;
	let session = oidc
		.exchange_auth_code(code, client_id, redirect_uri, body.code_verifier.as_deref())
		.await?;

	let user_id = &session.user_id;
	let (access_token, expires_in) = services.users.generate_access_token(true);
	let refresh_token = generate_refresh_token();

	let client_name = oidc
		.get_client(client_id)
		.await
		.ok()
		.and_then(|c| c.client_name);
	let device_display_name = client_name.as_deref().unwrap_or("OIDC Client");
	let device_id: Option<OwnedDeviceId> =
		extract_device_id(&session.scope).map(OwnedDeviceId::from);
	let device_id = services
		.users
		.create_device(
			user_id,
			device_id.as_deref(),
			(Some(&access_token), expires_in),
			Some(&refresh_token),
			Some(device_display_name),
			None,
		)
		.await?;

	info!("{user_id} logged in via OIDC (device {device_id})");

	let id_token = if session.scope.contains("openid") {
		let now = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap_or_default()
			.as_secs();
		let issuer = oidc_issuer_url(services)?;
		let claims = IdTokenClaims {
			iss: issuer,
			sub: user_id.to_string(),
			aud: client_id.to_owned(),
			exp: now.saturating_add(3600),
			iat: now,
			nonce: session.nonce,
			at_hash: Some(OidcServer::at_hash(&access_token)),
		};
		Some(oidc.sign_id_token(&claims)?)
	} else {
		None
	};

	let mut response = serde_json::json!({"access_token": access_token, "token_type": "Bearer", "scope": session.scope, "refresh_token": refresh_token});
	if let Some(expires_in) = expires_in {
		response["expires_in"] = serde_json::json!(expires_in.as_secs());
	}
	if let Some(id_token) = id_token {
		response["id_token"] = serde_json::json!(id_token);
	}

	Ok(Json(response).into_response())
}

async fn token_refresh(
	services: &tuwunel_service::Services,
	body: &TokenRequest,
) -> Result<http::Response<axum::body::Body>> {
	let refresh_token = body
		.refresh_token
		.as_deref()
		.ok_or_else(|| err!(Request(InvalidParam("refresh_token is required"))))?;
	let (user_id, device_id, _) = services
		.users
		.find_from_token(refresh_token)
		.await
		.map_err(|_| err!(Request(Forbidden("Invalid refresh token"))))?;

	let (new_access_token, expires_in) = services.users.generate_access_token(true);
	let new_refresh_token = generate_refresh_token();
	services
		.users
		.set_access_token(
			&user_id,
			&device_id,
			&new_access_token,
			expires_in,
			Some(&new_refresh_token),
		)
		.await?;

	let mut response = serde_json::json!({"access_token": new_access_token, "token_type": "Bearer", "refresh_token": new_refresh_token});
	if let Some(expires_in) = expires_in {
		response["expires_in"] = serde_json::json!(expires_in.as_secs());
	}

	Ok(Json(response).into_response())
}

#[derive(Debug, Deserialize)]
pub(crate) struct RevokeRequest {
	token: String,
	#[serde(default, rename = "token_type_hint")]
	_token_type_hint: Option<String>,
}

pub(crate) async fn revoke_route(
	State(services): State<crate::State>,
	axum::extract::Form(body): axum::extract::Form<RevokeRequest>,
) -> Result<impl IntoResponse> {
	if let Ok((user_id, device_id, _)) = services.users.find_from_token(&body.token).await {
		services
			.users
			.remove_device(&user_id, &device_id)
			.await;
	}
	Ok(Json(serde_json::json!({})))
}

pub(crate) async fn jwks_route(
	State(services): State<crate::State>,
) -> Result<impl IntoResponse> {
	let oidc = get_oidc_server(&services)?;
	Ok(Json(oidc.jwks()))
}

pub(crate) async fn userinfo_route(
	State(services): State<crate::State>,
	TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> Result<impl IntoResponse> {
	let token = bearer.token();
	let Ok((user_id, _device_id, _expires)) = services.users.find_from_token(token).await else {
		return Err!(Request(Unauthorized("Invalid access token")));
	};
	let displayname = services.users.displayname(&user_id).await.ok();
	let avatar_url = services.users.avatar_url(&user_id).await.ok();
	Ok(Json(
		serde_json::json!({"sub": user_id.to_string(), "name": displayname, "picture": avatar_url}),
	))
}

pub(crate) async fn account_route() -> impl IntoResponse {
	axum::response::Html(
		"<html><body><h1>Account Management</h1><p>Account management is not yet implemented. \
		 Please use your identity provider to manage your account.</p></body></html>",
	)
}

fn oauth_error(
	status: StatusCode,
	error: &str,
	description: &str,
) -> http::Response<axum::body::Body> {
	(
		status,
		Json(serde_json::json!({"error": error, "error_description": description})),
	)
		.into_response()
}

fn get_oidc_server(services: &tuwunel_service::Services) -> Result<&OidcServer> {
	services
		.oauth
		.oidc_server
		.as_deref()
		.ok_or_else(|| err!(Request(NotFound("OIDC server not configured"))))
}

fn oidc_issuer_url(services: &tuwunel_service::Services) -> Result<String> {
	services
		.config
		.well_known
		.client
		.as_ref()
		.map(|url| {
			let s = url.to_string();
			if s.ends_with('/') { s } else { s + "/" }
		})
		.ok_or_else(|| {
			err!(Config("well_known.client", "well_known.client must be set for OIDC server"))
		})
}

fn extract_device_id(scope: &str) -> Option<String> {
	scope
		.split_whitespace()
		.find_map(|s| s.strip_prefix("urn:matrix:org.matrix.msc2967.client:device:"))
		.map(ToOwned::to_owned)
}
