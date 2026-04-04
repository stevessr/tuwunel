pub(super) mod account;
pub(super) mod auth_issuer;
pub(super) mod auth_metadata;
pub(super) mod authorize;
pub(super) mod complete;
pub(super) mod jwks;
pub(super) mod registration;
pub(super) mod revoke;
pub(super) mod token;
pub(super) mod userinfo;

use axum::{Json, response::IntoResponse};
use http::StatusCode;
use tuwunel_core::{Result, err};

pub(super) use self::{
	account::*, auth_issuer::*, auth_metadata::*, authorize::*, complete::*, jwks::*,
	registration::*, revoke::*, token::*, userinfo::*,
};

const OIDC_REQ_ID_LENGTH: usize = 32;

fn oauth_error(
	status: StatusCode,
	error: &str,
	description: &str,
) -> http::Response<axum::body::Body> {
	(
		status,
		Json(serde_json::json!({
			"error": error,
			"error_description": description,
		})),
	)
		.into_response()
}

fn oidc_issuer_url(services: &tuwunel_service::Services) -> Result<String> {
	services
		.config
		.well_known
		.client
		.as_ref()
		.map(|url| {
			let s = url.to_string();
			if s.ends_with('/') { s } else { s + "/" }
		})
		.ok_or_else(|| {
			err!(Config("well_known.client", "well_known.client must be set for OIDC server"))
		})
}

fn extract_device_id(scope: &str) -> Option<String> {
	scope
		.split_whitespace()
		.find_map(|s| s.strip_prefix("urn:matrix:org.matrix.msc2967.client:device:"))
		.map(ToOwned::to_owned)
}
