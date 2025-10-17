use axum::{Json, extract::State, response::IntoResponse};
use ruma::api::client::discovery::{
	discover_homeserver::{self, HomeserverInfo},
	discover_support::{self, Contact},
};
use serde::{Deserialize, Serialize};
use tuwunel_core::{Err, Result};

use crate::Ruma;

/// MSC3861: Authentication information for .well-known/matrix/client
#[derive(Clone, Debug, Serialize, Deserialize)]
struct AuthenticationInfo {
	issuer: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	account: Option<String>,
}

/// Extended response for .well-known/matrix/client with MSC3861 support
#[derive(Clone, Debug, Serialize, Deserialize)]
struct ExtendedClientDiscovery {
	#[serde(rename = "m.homeserver")]
	homeserver: HomeserverInfo,
	#[serde(rename = "m.identity_server", skip_serializing_if = "Option::is_none")]
	identity_server: Option<ruma::api::client::discovery::discover_homeserver::IdentityServerInfo>,
	#[serde(rename = "org.matrix.msc3575.proxy", skip_serializing_if = "Option::is_none")]
	tile_server: Option<ruma::api::client::discovery::discover_homeserver::SlidingSyncProxyInfo>,
	// MSC3861: OAuth authentication information
	#[serde(rename = "org.matrix.msc2965.authentication", skip_serializing_if = "Option::is_none")]
	authentication: Option<AuthenticationInfo>,
}

/// # `GET /.well-known/matrix/client`
///
/// Returns the .well-known URL if it is configured, otherwise returns 404.
/// MSC3861: Includes OAuth authentication information if OAuth is enabled
pub(crate) async fn well_known_client(
	State(services): State<crate::State>,
	_body: Ruma<discover_homeserver::Request>,
) -> Result<impl IntoResponse> {
	let client_url = match services.server.config.well_known.client.as_ref() {
		| Some(url) => url.to_string(),
		| None => return Err!(Request(NotFound("Not found."))),
	};

	// MSC3861: Include OAuth authentication information if enabled
	let authentication = if services.config.oauth.enable && services.config.oauth.experimental_msc3861 {
		Some(AuthenticationInfo {
			issuer: services.config.oauth.issuer.clone(),
			account: Some(services.server.name.to_string()),
		})
	} else {
		None
	};

	let response = ExtendedClientDiscovery {
		homeserver: HomeserverInfo { base_url: client_url },
		identity_server: None,
		tile_server: None,
		authentication,
	};

	Ok(Json(response))
}

/// # `GET /.well-known/matrix/support`
///
/// Server support contact and support page of a homeserver's domain.
pub(crate) async fn well_known_support(
	State(services): State<crate::State>,
	_body: Ruma<discover_support::Request>,
) -> Result<discover_support::Response> {
	let support_page = services
		.server
		.config
		.well_known
		.support_page
		.as_ref()
		.map(ToString::to_string);

	let role = services
		.server
		.config
		.well_known
		.support_role
		.clone();

	// support page or role must be either defined for this to be valid
	if support_page.is_none() && role.is_none() {
		return Err!(Request(NotFound("Not found.")));
	}

	let email_address = services
		.server
		.config
		.well_known
		.support_email
		.clone();

	let matrix_id = services
		.server
		.config
		.well_known
		.support_mxid
		.clone();

	// if a role is specified, an email address or matrix id is required
	if role.is_some() && (email_address.is_none() && matrix_id.is_none()) {
		return Err!(Request(NotFound("Not found.")));
	}

	// TODO: support defining multiple contacts in the config
	let mut contacts: Vec<Contact> = vec![];

	if let Some(role) = role {
		let contact = Contact { role, email_address, matrix_id };

		contacts.push(contact);
	}

	// support page or role+contacts must be either defined for this to be valid
	if contacts.is_empty() && support_page.is_none() {
		return Err!(Request(NotFound("Not found.")));
	}

	Ok(discover_support::Response { contacts, support_page })
}

/// # `GET /client/server.json`
///
/// Endpoint provided by sliding sync proxy used by some clients such as Element
/// Web as a non-standard health check.
pub(crate) async fn syncv3_client_server_json(
	State(services): State<crate::State>,
) -> Result<impl IntoResponse> {
	let server_url = match services.server.config.well_known.client.as_ref() {
		| Some(url) => url.to_string(),
		| None => match services.server.config.well_known.server.as_ref() {
			| Some(url) => url.to_string(),
			| None => return Err!(Request(NotFound("Not found."))),
		},
	};

	Ok(Json(serde_json::json!({
		"server": server_url,
		"version": tuwunel_core::version(),
	})))
}
