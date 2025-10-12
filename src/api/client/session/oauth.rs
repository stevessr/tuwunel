use axum::extract::State;
use axum_client_ip::InsecureClientIp;
use ruma::OwnedUserId;
use serde::{Deserialize, Serialize};
use tuwunel_core::{Err, Result, err, utils};
use tuwunel_service::Services;

use crate::Ruma;

/// OAuth 2.0 token response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
	pub access_token: String,
	pub token_type: String,
	pub expires_in: Option<u64>,
	pub refresh_token: Option<String>,
	pub scope: Option<String>,
	pub id_token: Option<String>,
}

/// OAuth 2.0 token request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OAuthTokenRequest {
	pub grant_type: String,
	pub code: Option<String>,
	pub redirect_uri: Option<String>,
	pub client_id: Option<String>,
	pub client_secret: Option<String>,
	pub refresh_token: Option<String>,
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

	// Prepare token request
	let token_request = OAuthTokenRequest {
		grant_type: "authorization_code".to_owned(),
		code: Some(code.to_owned()),
		redirect_uri: Some(redirect_uri.to_owned()),
		client_id: Some(oauth_config.client_id.clone()),
		client_secret: Some(oauth_config.client_secret.clone()),
		refresh_token: None,
	};

	// Make HTTP request to token endpoint
	// This would need actual HTTP client implementation
	// For now, return a placeholder error

	Err!(Request(Unknown("Token exchange not yet implemented - requires HTTP client")))
}

/// Validate OAuth token and extract user information
pub(crate) async fn validate_oauth_token(
	services: &Services,
	access_token: &str,
) -> Result<OwnedUserId> {
	let oauth_config = &services.config.oauth;

	// Get userinfo endpoint
	let userinfo_endpoint = oauth_config
		.userinfo_endpoint
		.as_deref()
		.ok_or_else(|| err!(Request(Unknown("OAuth userinfo endpoint not configured"))))?;

	// Make HTTP request to userinfo endpoint
	// This would need actual HTTP client implementation
	// For now, return a placeholder error

	Err!(Request(Unknown("Token validation not yet implemented - requires HTTP client")))
}


