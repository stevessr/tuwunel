use axum::extract::State;
use axum_client_ip::InsecureClientIp;
use ruma::OwnedUserId;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tuwunel_core::{Err, Result, err, utils};
use tuwunel_service::Services;

use crate::Ruma;

/// OAuth 2.0 token response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
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
pub struct OAuthUserInfo {
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
) -> Result<String> {
	if !services.config.oauth.enable {
		return Err!(Request(Unknown("OAuth login is not enabled.")));
	}

	let oauth_config = &services.config.oauth;

	// Generate state parameter for CSRF protection
	let state = utils::random_string(32);

	// Store state in session (would need to be implemented)
	// For now, we'll just generate the authorization URL

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

	Ok(auth_url)
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

	// Prepare token request parameters
	let params = json!({
		"grant_type": "authorization_code",
		"code": code,
		"redirect_uri": redirect_uri,
		"client_id": &oauth_config.client_id,
		"client_secret": &oauth_config.client_secret,
	});

	// Make HTTP POST request to token endpoint
	let response = services
		.client
		.default
		.post(token_endpoint)
		.json(&params)
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
			.create(&user_id, None)
			.await
			.map_err(|e| err!(Request(Unknown("Failed to create user: {e}"))))?;

		// Set display name if available
		if let Some(name) = &userinfo.name {
			let _ = services.users.set_displayname(&user_id, Some(name.clone())).await;
		}
	}

	Ok(user_id)
}



