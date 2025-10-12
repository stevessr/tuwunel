use axum::extract::State;
use axum_client_ip::InsecureClientIp;
use ruma::OwnedUserId;
use tuwunel_core::{Err, Result, err, utils};
use tuwunel_service::Services;

use crate::Ruma;

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

