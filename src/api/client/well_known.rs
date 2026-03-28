use axum::extract::State;
use ruma::api::client::discovery::{
	discover_homeserver::{self, HomeserverInfo},
	discover_support::{self, Contact},
};
use tuwunel_core::{Err, Result};

use crate::{Ruma, client::rtc};

/// # `GET /.well-known/matrix/client`
///
/// Returns the .well-known URL if it is configured, otherwise returns 404.
/// Also includes RTC transport configuration for Element Call (MSC4143).
pub(crate) async fn well_known_client(
	State(services): State<crate::State>,
	_body: Ruma<discover_homeserver::Request>,
) -> Result<discover_homeserver::Response> {
	let homeserver = HomeserverInfo {
		base_url: match services.server.config.well_known.client.as_ref() {
			| Some(url) => url.to_string(),
			| None => return Err!(Request(NotFound("Not found."))),
		},
	};

	let transports = rtc::get_transports(&services)?;

	Ok(discover_homeserver::Response {
		rtc_foci: transports,
		..discover_homeserver::Response::new(homeserver)
	})
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
