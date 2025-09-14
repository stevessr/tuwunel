use std::collections::BTreeMap;

use axum::{Json, extract::State};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use tuwunel_core::info;

#[derive(Deserialize)]
pub(crate) struct AdminCommandRequest {
	command: String,
	input: String,
	token: String,
}

#[derive(Serialize)]
pub(crate) struct EventEntry {
	span: String,
	level: String,
	values: BTreeMap<String, String>,
}

#[derive(Serialize)]
pub(crate) struct AdminCommandResponse {
	output: String,
	error: bool,
	logs: Vec<EventEntry>,
}

/// # `POST /_tuwunel/admin/command`
///
/// Run an admin command
pub(crate) async fn admin_command(
	State(services): State<crate::State>,
	body: Json<AdminCommandRequest>,
) -> Result<Json<AdminCommandResponse>, (StatusCode, String)> {
	let token = &services.server.config.admin_api_token;
	if token.is_empty() || &body.token != token {
		return Err((StatusCode::FORBIDDEN, "wrong token".to_owned()));
	}

	info!("Running command {}", &body.command);

	let result = services
		.admin
		.run_command(&body.command, &body.input)
		.await;
	let response = AdminCommandResponse {
		output: result.output,
		error: result.err,
		logs: result
			.logs
			.iter()
			.map(|event| EventEntry {
				level: event.level.as_str().to_owned(),
				span: event.span_name.to_owned(),
				values: event
					.values
					.iter()
					.map(|value| ((*value.0).to_owned(), value.1.to_owned()))
					.collect::<BTreeMap<String, String>>(),
			})
			.collect::<Vec<EventEntry>>(),
	};
	Ok(Json::from(response))
}
