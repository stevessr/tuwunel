use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use serde_json::json;
use axum_client_ip::InsecureClientIp;
use ruma::OwnedUserId;
use serde::{Deserialize, Serialize};
use tuwunel_core::{Err, Result, err, utils};
use tuwunel_service::Services;


/// OAuth 2.0 token response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct OAuthTokenResponse {
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
pub(super) struct OAuthUserInfo {
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
	let rand = utils::random_string(32);
	let query = uri.query().unwrap_or_default();
	let params = url::form_urlencoded::parse(query.as_bytes()).into_owned().collect::<std::collections::HashMap<_, _>>();
	let redirect_url = params.get("redirectUrl").map(|s| s.as_str()).unwrap_or("");

	// If redirect_url provided, store mapping state -> redirect_url with TTL
	if !redirect_url.is_empty() {
		// lazily-initialized in-memory store
		static STATE_STORE: LazyLock<Mutex<HashMap<String, (String, Instant)>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
		let mut store = STATE_STORE.lock().unwrap();
		// purge expired entries
		let now = Instant::now();
		store.retain(|_, (_, exp)| *exp > now);
		store.insert(rand.clone(), (redirect_url.to_string(), now + Duration::from_secs(300)));
	}

	// state is just the random string
	let state = rand.clone();

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
	let mut params = std::collections::HashMap::new();
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
		| "preferred_username" =>
			userinfo.preferred_username.as_deref().unwrap_or(&userinfo.sub),
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
			let _ = services.users.set_displayname(&user_id, Some(name.clone()));
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
	let params = url::form_urlencoded::parse(query.as_bytes()).into_owned().collect::<std::collections::HashMap<_, _>>();

	let code = match params.get("code") {
		Some(c) => c.as_str(),
		None => return Err!(Request(InvalidParam("Missing code query parameter"))),
	};

	let state = params.get("state").map(|s| s.as_str()).unwrap_or("");

	// perform token exchange
	let token = exchange_code_for_token(&services, code, &services.config.oauth.redirect_uri).await?;

	// validate / get userinfo
	let userinfo = validate_oauth_token(&services, &token.access_token).await?;

	// get or create local user
	let user_id = get_or_create_user(&services, &userinfo).await?;

	// If state contains an embedded redirect URL (we encoded it earlier), extract and redirect
	let mut app_redirect: Option<String> = None;
	if state.contains('|') {
		let mut parts = state.splitn(2, '|');
		let _rand = parts.next().unwrap_or("");
		if let Some(enc) = parts.next() {
			if !enc.is_empty() {
				if let Ok(decoded) = urlencoding::decode(enc) {
					app_redirect = Some(decoded.into_owned());
				}
			}
		}
	}

	// If not embedded in state, check global state store
	if app_redirect.is_none() {
		static STATE_STORE: LazyLock<Mutex<HashMap<String, (String, Instant)>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
		let mut store = STATE_STORE.lock().unwrap();
		let now = Instant::now();
		store.retain(|_, (_, exp)| *exp > now);
		if let Some((redir, _)) = store.remove(state) {
			app_redirect = Some(redir);
		}
	}

	if let Some(mut redirect_to) = app_redirect {
		// append access_token and user_id as query params for the client to consume
		let sep = if redirect_to.contains('?') { '&' } else { '?' };
		redirect_to = format!("{}{}access_token={}&user_id={}", redirect_to, sep, token.access_token, user_id.to_string());
		return Ok(Redirect::temporary(&redirect_to).into_response())
	}

	// fallback: return JSON
	let result = serde_json::json!({
		"user_id": user_id.to_string(),
		"access_token": token.access_token,
		"token_type": token.token_type,
		"state": state,
		"userinfo": userinfo,
	});

	Ok(axum::response::Json(result).into_response())
}



