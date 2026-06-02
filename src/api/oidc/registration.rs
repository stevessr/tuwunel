use axum::{Json, extract::State, response::IntoResponse};
use http::{HeaderMap, StatusCode, header::AUTHORIZATION};
use serde_json::json;
use tuwunel_core::{Err, Result, info};
use tuwunel_service::oauth::server::DcrRequest;
use url::Url;

pub(crate) async fn registration_route(
	State(services): State<crate::State>,
	headers: HeaderMap,
	Json(body): Json<DcrRequest>,
) -> Result<impl IntoResponse> {
	let oidc = services.oauth.get_server()?;
	let config = &services.config;

	if body.redirect_uris.is_empty() {
		return Err!(Request(InvalidParam("redirect_uris must not be empty")));
	}

	// Initial access token (RFC 7591): gate registration when one is configured.
	let required_token = config.oidc_registration_access_token.as_str();
	if !required_token.is_empty() {
		let presented = headers
			.get(AUTHORIZATION)
			.and_then(|value| value.to_str().ok())
			.and_then(|value| value.strip_prefix("Bearer "));

		if presented != Some(required_token) {
			return Err!(Request(Forbidden("A valid initial access token is required")));
		}
	}

	// Redirect-host allowlist (RFC 7591): every redirect_uri host must be listed.
	let allowed = &config.oidc_registration_allowed_redirect_hosts;
	if !allowed.is_empty() {
		let host_allowed = |uri: &String| {
			Url::parse(uri).is_ok_and(|url| {
				url.host_str()
					.is_some_and(|host| allowed.iter().any(|entry| entry.as_str() == host))
			})
		};

		if !body.redirect_uris.iter().all(host_allowed) {
			return Err!(Request(Forbidden(
				"A redirect_uri host is not in the registration allowlist"
			)));
		}
	}

	let reg = oidc.register_client(body).await?;

	info!(
		"OIDC client registered: {} ({})",
		reg.client_id,
		reg.client_name.as_deref().unwrap_or("unnamed")
	);

	Ok((
		StatusCode::CREATED,
		Json(json!({
			"client_id": reg.client_id,
			"client_id_issued_at": reg.registered_at,
			"redirect_uris": reg.redirect_uris,
			"client_name": reg.client_name,
			"client_uri": reg.client_uri,
			"logo_uri": reg.logo_uri,
			"contacts": reg.contacts,
			"token_endpoint_auth_method": reg.token_endpoint_auth_method,
			"grant_types": reg.grant_types,
			"response_types": reg.response_types,
			"application_type": reg.application_type,
			"policy_uri": reg.policy_uri,
			"tos_uri": reg.tos_uri,
			"software_id": reg.software_id,
			"software_version": reg.software_version,
		})),
	))
}
