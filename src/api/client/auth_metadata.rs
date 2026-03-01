use axum::extract::State;
use ruma::{api::client::discovery::get_authorization_server_metadata, serde::Raw};
use serde_json::Value as JsonValue;
use tuwunel_core::{Err, Result, err};

use crate::Ruma;

/// # `GET /_matrix/client/v1/auth_metadata`
///
/// Returns the delegated authorization server metadata when Matrix
/// Authentication Service (MAS) integration is enabled.
pub(crate) async fn get_authorization_server_metadata_route(
	State(services): State<crate::State>,
	_body: Ruma<get_authorization_server_metadata::v1::Request>,
) -> Result<get_authorization_server_metadata::v1::Response> {
	let config = &services
		.server
		.config
		.matrix_authentication_service;
	if !config.enabled {
		return Err!(Request(NotFound("Not found.")));
	}

	let metadata_url = config
		.endpoint
		.join("/.well-known/openid-configuration")?;
	let metadata: JsonValue = services
		.client
		.oauth
		.get(metadata_url)
		.send()
		.await?
		.error_for_status()?
		.json()
		.await?;

	if !metadata.is_object() {
		return Err!(Request(NotJson(
			"Expected JSON object response from MAS metadata endpoint.",
		)));
	}

	// Validate required fields according to Matrix auth metadata requirements.
	let _ = serde_json::from_value::<
		get_authorization_server_metadata::v1::AuthorizationServerMetadata,
	>(metadata.clone())
	.map_err(|e| err!(Request(NotJson("Invalid authorization metadata from MAS: {e}",))))?;

	let raw = Raw::from_json(serde_json::value::to_raw_value(&metadata)?);
	Ok(get_authorization_server_metadata::v1::Response::new(raw))
}
