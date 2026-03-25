use axum::extract::State;
use ruma::api::client::rtc::{RtcTransport, transports};
use serde_json::Value;
use tuwunel_core::{Result, err, error::inspect_log};
use tuwunel_service::Services;

use crate::Ruma;

/// # `GET /_matrix/client/unstable/org.matrix.msc4143/rtc/transports`
///
/// Get MatrixRTC transports for MSC4143
pub(crate) async fn get_transports_route(
	State(services): State<crate::State>,
	_body: Ruma<transports::v1::Request>,
) -> Result<transports::v1::Response> {
	let transports = get_transports(&services)?;

	Ok(transports::v1::Response { rtc_transports: transports })
}

pub(crate) fn get_transports(services: &Services) -> Result<Vec<RtcTransport>> {
	// Add RTC transport configuration if available (MSC4143 / Element Call)
	// Element Call has evolved through several versions with different field
	// expectations
	services
		.server
		.config
		.well_known
		.rtc_transports
		.iter()
		.map(|transport| {
			let focus_type = transport
				.get("type")
				.and_then(Value::as_str)
				.ok_or_else(|| err!("`type` is not a valid string"))?;

			let transport = transport
				.as_object()
				.cloned()
				.ok_or_else(|| err!("`rtc_transport` is not a valid object"))?;

			RtcTransport::new(focus_type, transport).map_err(Into::into)
		})
		.map(|transport: Result<_>| {
			transport.map_err(|e| {
				err!(Config("global.well_known.rtc_transports", "Malformed value(s): {e:?}"))
			})
		})
		.chain(
			services
				.config
				.well_known
				.livekit_url
				.iter()
				.map(|livekit_url| Ok(RtcTransport::livekit(livekit_url.clone()))),
		)
		.collect::<Result<_>>()
		.inspect_err(inspect_log)
}
