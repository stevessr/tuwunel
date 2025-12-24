use crate::client::TOKEN_LENGTH;
use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use axum_client_ip::InsecureClientIp;
use ruma::OwnedUserId;
use serde::{Deserialize, Serialize};
use std::{
	collections::HashMap,
	sync::{LazyLock, Mutex},
	time::{Duration, Instant},
};
use tuwunel_core::{Err, Result, err, utils};
use tuwunel_service::Services;

const STATE_TTL: Duration = Duration::from_secs(300);

#[derive(Debug, Clone)]
struct StateEntry {
	redirect_url: String,
	expires_at: Instant,
}

static STATE_STORE: LazyLock<Mutex<HashMap<String, StateEntry>>> =
	LazyLock::new(|| Mutex::new(HashMap::new()));

fn purge_expired(store: &mut HashMap<String, StateEntry>) {
	let now = Instant::now();
	store.retain(|_, entry| entry.expires_at > now);
}

fn remember_redirect(state_token: &str, redirect_url: &str) {
	let mut store = STATE_STORE
		.lock()
		.unwrap_or_else(|poison| poison.into_inner());
	purge_expired(&mut store);
	store.insert(
		state_token.to_owned(),
		StateEntry {
			redirect_url: redirect_url.to_owned(),
			expires_at: Instant::now() + STATE_TTL,
		},
	);
}

fn consume_redirect(state_token: &str) -> Option<String> {
	let mut store = STATE_STORE
		.lock()
		.unwrap_or_else(|poison| poison.into_inner());
	purge_expired(&mut store);
	store
		.remove(state_token)
		.map(|entry| entry.redirect_url)
}

/// OAuth 2.0 token response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct OAuthTokenResponse {
	pub access_token: String,
	pub token_type: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub expires_in: Option<u64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub refresh_token: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub scope: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub id_token: Option<String>,
}

/// OAuth 2.0 userinfo response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct OAuthUserInfo {
	pub sub: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub email: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub preferred_username: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub email_verified: Option<bool>,
}

/// # `GET /_matrix/client/v3/login/sso/redirect`
///
/// Redirect to the OAuth 2.0 authorization endpoint
#[tracing::instrument(skip_all, fields(%client), name = "oauth_redirect")]
pub(crate) async fn oauth_redirect_route(
	State(services): State<crate::State>,
	InsecureClientIp(client): InsecureClientIp,
	uri: http::Uri,
) -> Result<Redirect> {
	if !services.config.oauth.enable {
		return Err!(Request(Unknown("OAuth login is not enabled.")));
	}

	let oauth_config = &services.config.oauth;

	// Generate state parameter for CSRF protection
	let state_token = utils::random_string(32);
	let query = uri.query().unwrap_or_default();
	let params = url::form_urlencoded::parse(query.as_bytes())
		.into_owned()
		.collect::<HashMap<_, _>>();
	let redirect_url = params
		.get("redirectUrl")
		.map(|s| s.as_str())
		.unwrap_or("");

	// If redirect_url provided, store mapping state -> redirect_url with TTL
	if !redirect_url.is_empty() {
		remember_redirect(&state_token, redirect_url);
	}

	let state = if redirect_url.is_empty() {
		state_token.clone()
	} else {
		format!("{}|{}", state_token, urlencoding::encode(redirect_url))
	};

	let auth_endpoint = oauth_config
		.authorization_endpoint
		.as_deref()
		.ok_or_else(|| err!(Request(Unknown("OAuth authorization endpoint not configured"))))?;

	// Build authorization URL with parameters
	let scopes = oauth_config.scopes.join(" ");
	let auth_url = format!(
		"{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}",
		auth_endpoint,
		urlencoding::encode(&oauth_config.client_id),
		urlencoding::encode(&oauth_config.redirect_uri),
		urlencoding::encode(&scopes),
		urlencoding::encode(&state)
	);

	// Redirect the client to the authorization URL
	Ok(Redirect::temporary(&auth_url))
}

/// Exchange OAuth authorization code for access token
pub(crate) async fn exchange_code_for_token(
	services: &Services,
	code: &str,
	redirect_uri: &str,
) -> Result<OAuthTokenResponse> {
	let oauth_config = &services.config.oauth;

	let token_endpoint = oauth_config
		.token_endpoint
		.as_deref()
		.ok_or_else(|| err!(Request(Unknown("OAuth token endpoint not configured"))))?;

	// Prepare token request parameters as form data (RFC 6749)
	let mut params = HashMap::new();
	params.insert("grant_type", "authorization_code");
	params.insert("code", code);
	params.insert("redirect_uri", redirect_uri);

	// Make HTTP POST request to token endpoint using form encoding and HTTP Basic auth
	let client = &services.client.default;
	let response = client
		.post(token_endpoint)
		.basic_auth(&oauth_config.client_id, Some(&oauth_config.client_secret))
		.form(&params)
		.send()
		.await
		.map_err(|e| err!(Request(Unknown("Failed to request OAuth token: {e}"))))?;

	if !response.status().is_success() {
		let status = response.status();
		let error_text = response
			.text()
			.await
			.unwrap_or_else(|_| "Unknown error".to_owned());
		return Err!(Request(Unknown(
			"OAuth token request failed with status {status}: {error_text}"
		)));
	}

	let token_response: OAuthTokenResponse = response
		.json()
		.await
		.map_err(|e| err!(Request(Unknown("Failed to parse OAuth token response: {e}"))))?;

	Ok(token_response)
}

/// Validate OAuth token and extract user information
pub(crate) async fn validate_oauth_token(
	services: &Services,
	access_token: &str,
) -> Result<OAuthUserInfo> {
	let oauth_config = &services.config.oauth;

	// Get userinfo endpoint
	let userinfo_endpoint = oauth_config
		.userinfo_endpoint
		.as_deref()
		.ok_or_else(|| err!(Request(Unknown("OAuth userinfo endpoint not configured"))))?;

	// Make HTTP GET request to userinfo endpoint with bearer token
	let response = services
		.client
		.default
		.get(userinfo_endpoint)
		.bearer_auth(access_token)
		.send()
		.await
		.map_err(|e| err!(Request(Unknown("Failed to request user info: {e}"))))?;

	if !response.status().is_success() {
		let status = response.status();
		let error_text = response
			.text()
			.await
			.unwrap_or_else(|_| "Unknown error".to_owned());
		return Err!(Request(Unknown(
			"OAuth userinfo request failed with status {status}: {error_text}"
		)));
	}

	let userinfo: OAuthUserInfo = response
		.json()
		.await
		.map_err(|e| err!(Request(Unknown("Failed to parse OAuth userinfo response: {e}"))))?;

	Ok(userinfo)
}

/// Create or get Matrix user ID from OAuth user info
pub(crate) async fn get_or_create_user(
	services: &Services,
	userinfo: &OAuthUserInfo,
) -> Result<OwnedUserId> {
	let oauth_config = &services.config.oauth;

	// Extract localpart from userinfo based on configured claim
	let localpart = match oauth_config.subject_claim.as_str() {
		| "sub" => &userinfo.sub,
		| "email" => userinfo.email.as_deref().unwrap_or(&userinfo.sub),
		| "preferred_username" => userinfo
			.preferred_username
			.as_deref()
			.unwrap_or(&userinfo.sub),
		| _ => &userinfo.sub,
	};

	// Sanitize localpart to be valid Matrix localpart
	let sanitized_localpart = localpart
		.chars()
		.filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '_' || *c == '-')
		.collect::<String>()
		.to_lowercase();

	// Construct user ID
	let user_id: OwnedUserId = format!("@{}:{}", sanitized_localpart, services.server.name)
		.try_into()
		.map_err(|e| err!(Request(Unknown("Failed to construct user ID: {e}"))))?;

	// Check if user exists
	if !services.users.exists(&user_id).await {
		if !oauth_config.register_user {
			return Err!(Request(Forbidden("User registration via OAuth is disabled")));
		}

		// Create new user
		services
			.users
			.create(&user_id, None, Some("oauth"))
			.await
			.map_err(|e| err!(Request(Unknown("Failed to create user: {e}"))))?;

		// Set display name if available
		if let Some(name) = &userinfo.name {
			// set_displayname is synchronous (writes to DB map), no await
			services
				.users
				.set_displayname(&user_id, Some(name.clone()));
		}
	}

	Ok(user_id)
}

/// OAuth callback handler for provider redirects
#[tracing::instrument(skip_all, fields(%client), name = "oauth_callback")]
pub(crate) async fn oauth_callback_route(
	State(services): State<crate::State>,
	InsecureClientIp(client): InsecureClientIp,
	// query parameters will be parsed by Ruma wrapper normally; for minimal debug we read from Axum
	uri: http::Uri,
) -> Result<axum::response::Response> {
	// parse query
	let query = uri.query().unwrap_or_default();
	let params = url::form_urlencoded::parse(query.as_bytes())
		.into_owned()
		.collect::<HashMap<_, _>>();

	let code = match params.get("code") {
		| Some(c) => c.as_str(),
		| None => return Err!(Request(InvalidParam("Missing code query parameter"))),
	};

	let state = params
		.get("state")
		.map(|s| s.as_str())
		.unwrap_or("");
	let state_token = state
		.split_once('|')
		.map(|(token, _)| token)
		.unwrap_or(state);

	// perform token exchange
	let token =
		exchange_code_for_token(&services, code, &services.config.oauth.redirect_uri).await?;

	// validate / get userinfo
	let userinfo = validate_oauth_token(&services, &token.access_token).await?;

	// get or create local user
	let user_id = get_or_create_user(&services, &userinfo).await?;

	// 生成一次性 SSO login_token
	let login_token = {
		use ruma::UserId;
		let user_id_ref: &UserId = user_id.as_ref();
		let token = utils::random_string(TOKEN_LENGTH);
		let _ = services
			.users
			.create_login_token(user_id_ref, &token);
		token
	};

	// If state contains an embedded redirect URL (we encoded it earlier), extract and redirect
	let mut app_redirect: Option<String> = consume_redirect(state_token);

	// Fall back to embedded state payload when not found in the map
	if app_redirect.is_none() && state.contains('|') {
		let mut parts = state.splitn(2, '|');
		let _ = parts.next();
		if let Some(enc) = parts.next() {
			if !enc.is_empty() {
				if let Ok(decoded) = urlencoding::decode(enc) {
					app_redirect = Some(decoded.into_owned());
				}
			}
		}
	}

	if let Some(mut redirect_to) = app_redirect {
		// If the redirect URL contains a fragment (hash routing, e.g. /#/home),
		// append the token inside the fragment so single-page apps that
		// use hash routing can read it (e.g. https://app.example/#/home?token=...)
		if redirect_to.contains('#') {
			let mut parts = redirect_to.splitn(2, '#');
			let base = parts.next().unwrap_or("");
			let frag = parts.next().unwrap_or("");
			let frag_sep = if frag.contains('?') { '&' } else { '?' };
			redirect_to = format!(
				"{}#{}{}loginToken={}&user_id={}",
				base,
				frag,
				frag_sep,
				urlencoding::encode(&login_token),
				urlencoding::encode(&user_id.to_string())
			);
		} else {
			// 拼接 token 和 user_id，字段名为 loginToken (query param)
			let sep = if redirect_to.contains('?') { '&' } else { '?' };
			redirect_to = format!(
				"{}{}loginToken={}&user_id={}",
				redirect_to,
				sep,
				urlencoding::encode(&login_token),
				urlencoding::encode(&user_id.to_string())
			);
		}

		return Ok(Redirect::temporary(&redirect_to).into_response());
	}

	// fallback: return JSON（同样用 loginToken 字段）
	let result = serde_json::json!({
		"user_id": user_id.to_string(),
		"loginToken": login_token,
		"state": state,
		"userinfo": userinfo,
	});

	Ok(axum::response::Json(result).into_response())
}
