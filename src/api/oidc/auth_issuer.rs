use axum::{Json, extract::State, response::IntoResponse};
use serde::Serialize;
use tuwunel_core::Result;

use super::oidc_issuer_url;

#[derive(Serialize)]
struct AuthIssuerResponse {
	issuer: String,
}

pub(crate) async fn auth_issuer_route(
	State(services): State<crate::State>,
) -> Result<impl IntoResponse> {
	let issuer = oidc_issuer_url(&services)?;

	Ok(Json(AuthIssuerResponse { issuer }))
}
