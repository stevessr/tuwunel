use crate::client::TOKEN_LENGTH;
use crate::client::oauth_provider::{ResolvedOAuthProvider, resolve_oauth_provider};
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
	redirect_url: Option<String>,
	provider_id: String,
	expires_at: Instant,
}

static STATE_STORE: LazyLock<Mutex<HashMap<String, StateEntry>>> =
	LazyLock::new(|| Mutex::new(HashMap::new()));

fn purge_expired(store: &mut HashMap<String, StateEntry>) {
	let now = Instant::now();
	store.retain(|_, entry| entry.expires_at > now);
}

fn remember_state(state_token: &str, provider_id: &str, redirect_url: Option<&str>) {
	let mut store = STATE_STORE
		.lock()
		.unwrap_or_else(|poison| poison.into_inner());
	purge_expired(&mut store);
	store.insert(
		state_token.to_owned(),
		StateEntry {
			redirect_url: redirect_url.map(str::to_owned),
			provider_id: provider_id.to_owned(),
			expires_at: Instant::now() + STATE_TTL,
		},
	);
}

fn consume_state(state_token: &str) -> Option<StateEntry> {
	let mut store = STATE_STORE
		.lock()
		.unwrap_or_else(|poison| poison.into_inner());
	purge_expired(&mut store);
	store.remove(state_token)
}

fn build_state(
	state_token: &str,
	provider: &ResolvedOAuthProvider,
	redirect_url: &str,
) -> String {
	let encoded_redirect = if redirect_url.is_empty() {
		String::new()
	} else {
		urlencoding::encode(redirect_url).into_owned()
	};

	format!("{}|{}|{}", state_token, provider.id, encoded_redirect)
}

fn parse_state(state: &str) -> (String, Option<String>, Option<String>) {
	let pipe_count = state.matches('|').count();

	if pipe_count == 1 {
		let mut parts = state.splitn(2, '|');
		let token = parts.next().unwrap_or("").to_owned();
		let redirect = parts
			.next()
			.and_then(|value| (!value.is_empty()).then_some(value))
			.and_then(|value| urlencoding::decode(value).ok())
			.map(|value| value.into_owned());
		return (token, None, redirect);
	}

	let mut parts = state.splitn(3, '|');
	let token = parts.next().unwrap_or("").to_owned();
	let provider_id = parts
		.next()
		.and_then(|value| (!value.is_empty()).then_some(value.to_owned()));
	let redirect = parts
		.next()
		.and_then(|value| (!value.is_empty()).then_some(value))
		.and_then(|value| urlencoding::decode(value).ok())
		.map(|value| value.into_owned());
	(token, provider_id, redirect)
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
	let idp_id = params.get("idp_id").map(|s| s.as_str());

	let provider = resolve_oauth_provider(&services.config.oauth, idp_id)?;
	let state = build_state(&state_token, &provider, redirect_url);

	remember_state(
		&state_token,
		&provider.id,
		(!redirect_url.is_empty()).then_some(redirect_url),
	);

	let auth_endpoint = provider
		.authorization_endpoint
		.as_deref()
		.ok_or_else(|| err!(Request(Unknown("OAuth authorization endpoint not configured"))))?;

	// Build authorization URL with parameters
	let scopes = provider.scopes.join(" ");
	let auth_url = format!(
		"{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}",
		auth_endpoint,
		urlencoding::encode(&provider.client_id),
		urlencoding::encode(&provider.redirect_uri),
		urlencoding::encode(&scopes),
		urlencoding::encode(&state)
	);

	// Redirect the client to the authorization URL
	Ok(Redirect::temporary(&auth_url))
}

/// Exchange OAuth authorization code for access token
pub(crate) async fn exchange_code_for_token(
	services: &Services,
	provider: &ResolvedOAuthProvider,
	code: &str,
) -> Result<OAuthTokenResponse> {
	let token_endpoint = provider
		.token_endpoint
		.as_deref()
		.ok_or_else(|| err!(Request(Unknown("OAuth token endpoint not configured"))))?;

	// Prepare token request parameters as form data (RFC 6749)
	let mut params = HashMap::new();
	params.insert("grant_type", "authorization_code");
	params.insert("code", code);
	params.insert("redirect_uri", provider.redirect_uri.as_str());

	// Make HTTP POST request to token endpoint using form encoding and HTTP Basic auth
	let client = &services.client.default;
	let response = client
		.post(token_endpoint)
		.basic_auth(&provider.client_id, Some(&provider.client_secret))
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
	provider: &ResolvedOAuthProvider,
	access_token: &str,
) -> Result<OAuthUserInfo> {
	// Get userinfo endpoint
	let userinfo_endpoint = provider
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
	provider: &ResolvedOAuthProvider,
	userinfo: &OAuthUserInfo,
) -> Result<OwnedUserId> {
	// Extract localpart from userinfo based on configured claim
	let localpart = match provider.subject_claim.as_str() {
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
		if !provider.register_user {
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

	let (state_token, mut provider_id, mut app_redirect) = parse_state(state);
	let state_entry = consume_state(&state_token);
	if let Some(entry) = state_entry {
		if !entry.provider_id.is_empty() {
			provider_id = Some(entry.provider_id);
		}
		if entry.redirect_url.is_some() {
			app_redirect = entry.redirect_url;
		}
	}

	let provider = resolve_oauth_provider(&services.config.oauth, provider_id.as_deref())?;

	// perform token exchange
	let token = exchange_code_for_token(&services, &provider, code).await?;

	// validate / get userinfo
	let userinfo = validate_oauth_token(&services, &provider, &token.access_token).await?;

	// get or create local user
	let user_id = get_or_create_user(&services, &provider, &userinfo).await?;

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
