mod auth;
mod client;
mod jwk;
mod signing_key;
mod token;

use std::sync::Arc;

use serde_json::Value as JsonValue;
use tuwunel_core::{Result, debug_info, warn};
use tuwunel_database::Map;

pub use self::{
	auth::{AUTH_REQUEST_LIFETIME, AuthCodeSession, AuthRequest},
	client::{ClientRegistration, DcrRequest},
	token::IdTokenClaims,
};
use self::{
	jwk::init_jwk,
	signing_key::{SigningKey, init_signing_key},
};

pub struct Server {
	db: Data,
	jwk: JsonValue,
	key: SigningKey,
}

struct Data {
	oidc_signingkey: Arc<Map>,
	oidcclientid_registration: Arc<Map>,
	oidccode_authsession: Arc<Map>,
	oidcreqid_authrequest: Arc<Map>,
}

impl Server {
	pub(super) fn build(args: &crate::Args<'_>) -> Result<Option<Self>> {
		if args.server.config.identity_provider.is_empty()
			|| args.server.config.well_known.client.is_none()
		{
			warn!(
				"OIDC server (next-gen auth) requires `well_known.client` and one or more \
				 `identity_provider` to be configured"
			);

			return Ok(None);
		}

		debug_info!("Initializing OIDC server for next-gen auth (MSC2965)");

		let db = Data {
			oidc_signingkey: args.db["oidc_signingkey"].clone(),
			oidcclientid_registration: args.db["oidcclientid_registration"].clone(),
			oidccode_authsession: args.db["oidccode_authsession"].clone(),
			oidcreqid_authrequest: args.db["oidcreqid_authrequest"].clone(),
		};

		let key = init_signing_key(&db)?;

		Ok(Some(Self {
			db,
			jwk: init_jwk(&key.key_der, &key.key_id)?,
			key,
		}))
	}
}
