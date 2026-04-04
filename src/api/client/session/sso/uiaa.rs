use axum::extract::State;
use ruma::api::client::uiaa::{AuthType, get_uiaa_fallback_page};
use tuwunel_core::Result;

use crate::Ruma;

/// # `GET /_matrix/client/v3/auth/m.login.sso/fallback/web?session={session_id}`
///
/// Get UIAA fallback web page for SSO authentication.
#[tracing::instrument(
	name = "sso_fallback",
	level = "debug",
	skip_all,
	fields(session = body.body.session),
)]
pub(crate) async fn sso_fallback_route(
	State(services): State<crate::State>,
	body: Ruma<get_uiaa_fallback_page::v3::Request>,
) -> Result<get_uiaa_fallback_page::v3::Response> {
	let session = &body.body.session;

	// Check if this UIAA session has already been completed via SSO
	if let Some((_, _, uiaainfo)) = services
		.uiaa
		.get_uiaa_session_by_session_id(session)
		.await && uiaainfo.completed.contains(&AuthType::Sso)
	{
		let html = include_str!("complete.html");

		return Ok(get_uiaa_fallback_page::v3::Response::html(html.as_bytes().to_vec()));
	}

	// Session is not completed yet, show the prompt to continue
	let html = include_str!("required.html");
	let url_str = format!("/_matrix/client/v3/login/sso/redirect?redirectUrl=uiaa:{session}");
	let output = html.replace("{{url_str}}", &url_str);

	Ok(get_uiaa_fallback_page::v3::Response::html(output.into_bytes()))
}
