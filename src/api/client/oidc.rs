use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use tuwunel_core::{Err, Result};

use crate::client::oauth_provider::resolve_oauth_provider;


/// OIDC Discovery metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct OidcDiscovery {
	pub issuer: String,
	pub authorization_endpoint: String,
	pub token_endpoint: String,
	pub userinfo_endpoint: Option<String>,
	pub jwks_uri: Option<String>,
	pub response_types_supported: Vec<String>,
	pub subject_types_supported: Vec<String>,
	pub id_token_signing_alg_values_supported: Vec<String>,
	pub scopes_supported: Option<Vec<String>>,
	pub token_endpoint_auth_methods_supported: Option<Vec<String>>,
	pub claims_supported: Option<Vec<String>>,
	// MSC3861: Account management capabilities
	#[serde(skip_serializing_if = "Option::is_none")]
	pub account_management_uri: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub account_management_actions_supported: Option<Vec<String>>,
}

/// # `GET /.well-known/openid-configuration`
///
/// Returns OIDC discovery metadata if OAuth is enabled
#[tracing::instrument(skip_all, name = "oidc_discovery")]
pub(crate) async fn oidc_discovery_route(
	State(services): State<crate::State>,
) -> Result<Json<OidcDiscovery>> {
	if !services.config.oauth.enable {
		return Err!(Request(Unknown("OIDC discovery is not enabled.")));
	}

	let provider = resolve_oauth_provider(&services.config.oauth, None)?;
	if !provider.enable_discovery {
		return Err!(Request(Unknown("OIDC discovery is not enabled.")));
	}

	let discovery = OidcDiscovery {
		issuer: provider.issuer.clone(),
		authorization_endpoint: provider
			.authorization_endpoint
			.clone()
			.unwrap_or_else(|| format!("{}/oauth2/authorize", provider.issuer)),
		token_endpoint: provider
			.token_endpoint
			.clone()
			.unwrap_or_else(|| format!("{}/oauth2/token", provider.issuer)),
		userinfo_endpoint: provider
			.userinfo_endpoint
			.clone()
			.or_else(|| Some(format!("{}/oauth2/userinfo", provider.issuer))),
		jwks_uri: provider
			.jwks_uri
			.clone()
			.or_else(|| Some(format!("{}/.well-known/jwks.json", provider.issuer))),
		response_types_supported: vec!["code".to_owned()],
		subject_types_supported: vec!["public".to_owned()],
		id_token_signing_alg_values_supported: vec!["RS256".to_owned()],
		scopes_supported: Some(provider.scopes.clone()),
		token_endpoint_auth_methods_supported: Some(vec![
			"client_secret_basic".to_owned(),
			"client_secret_post".to_owned(),
		]),
		claims_supported: Some(vec![
			"sub".to_owned(),
			"name".to_owned(),
			"email".to_owned(),
			"email_verified".to_owned(),
			"preferred_username".to_owned(),
		]),
		// MSC3861: Include account management information if configured
		account_management_uri: provider.account_management_url.clone(),
		account_management_actions_supported: provider
			.account_management_url
			.as_ref()
			.map(|_| vec![
				"org.matrix.profile".to_owned(),
				"org.matrix.sessions_list".to_owned(),
				"org.matrix.session_view".to_owned(),
				"org.matrix.session_end".to_owned(),
			]),
	};

	Ok(Json(discovery))
}

/// Matrix OAuth 2.0 account metadata (MSC2965/MSC3861)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MatrixOAuthAccount {
	pub issuer: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub account: Option<String>,
}

/// # `GET /_matrix/client/unstable/org.matrix.msc2965/auth_issuer`
///
/// Returns the OAuth issuer for this homeserver (MSC2965)
#[tracing::instrument(skip_all, name = "oauth_issuer")]
pub(crate) async fn oauth_issuer_route(
	State(services): State<crate::State>,
) -> Result<Json<MatrixOAuthAccount>> {
	if !services.config.oauth.enable {
		return Err!(Request(Unknown("OAuth is not enabled.")));
	}

	let provider = resolve_oauth_provider(&services.config.oauth, None)?;

	Ok(Json(MatrixOAuthAccount {
		issuer: provider.issuer.clone(),
		account: Some(services.server.name.to_string()),
	}))
}

/// MSC3861: Account management information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct AccountManagementInfo {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub account_management_uri: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub account_management_actions_supported: Option<Vec<String>>,
}

/// # `GET /_matrix/client/unstable/org.matrix.msc3861/account_management`
///
/// Returns account management information (MSC3861)
#[tracing::instrument(skip_all, name = "msc3861_account_management")]
pub(crate) async fn msc3861_account_management_route(
	State(services): State<crate::State>,
) -> Result<Json<AccountManagementInfo>> {
	if !services.config.oauth.enable {
		return Err!(Request(Unknown("MSC3861 is not enabled.")));
	}

	let provider = resolve_oauth_provider(&services.config.oauth, None)?;
	if !provider.experimental_msc3861 {
		return Err!(Request(Unknown("MSC3861 is not enabled.")));
	}

	Ok(Json(AccountManagementInfo {
		account_management_uri: provider.account_management_url.clone(),
		account_management_actions_supported: provider
			.account_management_url
			.as_ref()
			.map(|_| vec![
				"org.matrix.profile".to_owned(),
				"org.matrix.sessions_list".to_owned(),
				"org.matrix.session_view".to_owned(),
				"org.matrix.session_end".to_owned(),
			]),
	}))
}
