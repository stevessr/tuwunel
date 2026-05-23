use core::iter::once;
use std::collections::BTreeMap;

use axum::extract::State;
use ruma::api::{
	client::discovery::{
		discover_homeserver::{self, HomeserverInfo},
		discover_support::{self, Contact},
	},
	identity_service::tos::get_terms_of_service::v2::{LocalizedPolicy, Policies},
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

	let rtc_foci = rtc::get_transports(&services)?;

	Ok(discover_homeserver::Response {
		rtc_foci,
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
		.config
		.well_known
		.support_page
		.as_ref()
		.map(ToString::to_string);

	let single_contact = services
		.config
		.well_known
		.support_role
		.clone()
		.map(|role| Contact {
			role,
			email_address: services.config.well_known.support_email.clone(),
			matrix_id: services.config.well_known.support_mxid.clone(),
			pgp_key: services.config.well_known.support_pgp_key.clone(),
		});

	let contacts = {
		let contacts = services
			.config
			.well_known
			.support_contact
			.clone()
			.into_values()
			.map(Into::into);

		match single_contact {
			| Some(contact) => contacts.chain(once(contact)).collect(),
			| None => contacts.collect(),
		}
	};

	let policies = services
		.config
		.well_known
		.support_policy
		.clone()
		.into_values()
		.map(|policy| {
			let localized = BTreeMap::from([(
				policy.policy_translation.language.clone(),
				LocalizedPolicy::from(policy.policy_translation),
			)]);

			(policy.name, Policies { version: policy.version, localized })
		})
		.collect();

	Ok(discover_support::Response { contacts, support_page, policies })
}
