use ruma::OwnedMxcUri;
use ruma::api::client::session::get_login_types::{
	IdentityProvider, IdentityProviderBrand,
};
use tuwunel_core::{Err, Result, err};
use tuwunel_core::config::{OAuthConfig, OAuthProviderConfig};

#[derive(Clone, Debug)]
pub(crate) struct ResolvedOAuthProvider {
	pub id: String,
	pub name: String,
	pub issuer: String,
	pub client_id: String,
	pub client_secret: String,
	pub redirect_uri: String,
	pub scopes: Vec<String>,
	pub register_user: bool,
	pub authorization_endpoint: Option<String>,
	pub token_endpoint: Option<String>,
	pub userinfo_endpoint: Option<String>,
	pub jwks_uri: Option<String>,
	pub enable_discovery: bool,
	pub subject_claim: String,
	pub displayname_claim: String,
	pub account_management_url: Option<String>,
	pub experimental_msc3861: bool,
}

fn provider_display_name(id: &str, provider: &OAuthProviderConfig) -> String {
	if provider.name.is_empty() { id.to_owned() } else { provider.name.clone() }
}

fn resolve_from_provider(id: &str, provider: &OAuthProviderConfig) -> ResolvedOAuthProvider {
	ResolvedOAuthProvider {
		id: id.to_owned(),
		name: provider_display_name(id, provider),
		issuer: provider.issuer.clone(),
		client_id: provider.client_id.clone(),
		client_secret: provider.client_secret.clone(),
		redirect_uri: provider.redirect_uri.clone(),
		scopes: provider.scopes.clone(),
		register_user: provider.register_user,
		authorization_endpoint: provider.authorization_endpoint.clone(),
		token_endpoint: provider.token_endpoint.clone(),
		userinfo_endpoint: provider.userinfo_endpoint.clone(),
		jwks_uri: provider.jwks_uri.clone(),
		enable_discovery: provider.enable_discovery,
		subject_claim: provider.subject_claim.clone(),
		displayname_claim: provider.displayname_claim.clone(),
		account_management_url: provider.account_management_url.clone(),
		experimental_msc3861: provider.experimental_msc3861,
	}
}

fn resolve_from_legacy(config: &OAuthConfig) -> ResolvedOAuthProvider {
	ResolvedOAuthProvider {
		id: String::new(),
		name: String::new(),
		issuer: config.issuer.clone(),
		client_id: config.client_id.clone(),
		client_secret: config.client_secret.clone(),
		redirect_uri: config.redirect_uri.clone(),
		scopes: config.scopes.clone(),
		register_user: config.register_user,
		authorization_endpoint: config.authorization_endpoint.clone(),
		token_endpoint: config.token_endpoint.clone(),
		userinfo_endpoint: config.userinfo_endpoint.clone(),
		jwks_uri: config.jwks_uri.clone(),
		enable_discovery: config.enable_discovery,
		subject_claim: config.subject_claim.clone(),
		displayname_claim: config.displayname_claim.clone(),
		account_management_url: config.account_management_url.clone(),
		experimental_msc3861: config.experimental_msc3861,
	}
}

pub(crate) fn oauth_identity_providers(config: &OAuthConfig) -> Vec<IdentityProvider> {
	if config.providers.is_empty() {
		return Vec::new();
	}

	config
		.providers
		.iter()
		.map(|(id, provider)| IdentityProvider {
			id: id.clone(),
			name: provider_display_name(id, provider),
			icon: provider
				.icon
				.as_ref()
				.and_then(|value| value.parse::<OwnedMxcUri>().ok()),
			brand: provider
				.brand
				.as_ref()
				.map(|value| IdentityProviderBrand::from(value.as_str())),
		})
		.collect()
}

pub(crate) fn resolve_oauth_provider(
	config: &OAuthConfig,
	idp_id: Option<&str>,
) -> Result<ResolvedOAuthProvider> {
	if !config.providers.is_empty() {
		let selected_id = match idp_id {
			| Some(id) => id.to_owned(),
			| None if config.providers.len() == 1 => config
				.providers
				.keys()
				.next()
				.cloned()
				.unwrap_or_default(),
			| None => {
				if let Some(default_id) = config.default_provider.as_deref() {
					default_id.to_owned()
				} else {
					return Err!(Request(Unknown(
						"Multiple OAuth providers configured; specify idp_id.",
					)));
				}
			},
		};

		let provider = config.providers.get(&selected_id).ok_or_else(|| {
			err!(Request(Unknown("OAuth provider not found.")))
		})?;

		return Ok(resolve_from_provider(&selected_id, provider));
	}

	if idp_id.is_some() {
		return Err!(Request(Unknown(
			"OAuth provider IDs are not configured for this server.",
		)));
	}

	Ok(resolve_from_legacy(config))
}
