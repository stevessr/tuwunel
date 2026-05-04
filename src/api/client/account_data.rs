use axum::extract::State;
use ruma::{
	RoomId, UserId,
	api::client::config::{
		delete_global_account_data, delete_room_account_data, get_global_account_data,
		get_room_account_data, set_global_account_data, set_room_account_data,
	},
	events::{
		AnyGlobalAccountDataEventContent, AnyRoomAccountDataEventContent,
		GlobalAccountDataEventType, RoomAccountDataEventType,
	},
	serde::Raw,
};
use serde::Deserialize;
use serde_json::{Value as JsonValue, json, value::RawValue as RawJsonValue};
use tuwunel_core::{Err, Result, err};
use tuwunel_service::Services;

use crate::Ruma;

/// # `PUT /_matrix/client/r0/user/{userId}/account_data/{type}`
///
/// Sets some account data for the sender user.
pub(crate) async fn set_global_account_data_route(
	State(services): State<crate::State>,
	body: Ruma<set_global_account_data::v3::Request>,
) -> Result<set_global_account_data::v3::Response> {
	let sender_user = body.sender_user();

	if sender_user != body.user_id && body.appservice_info.is_none() {
		return Err!(Request(Forbidden("You cannot set account data for other users.")));
	}

	set_account_data(
		&services,
		None,
		&body.user_id,
		&body.event_type.to_string(),
		body.data.json(),
	)
	.await?;

	Ok(set_global_account_data::v3::Response {})
}

/// # `PUT /_matrix/client/r0/user/{userId}/rooms/{roomId}/account_data/{type}`
///
/// Sets some room account data for the sender user.
pub(crate) async fn set_room_account_data_route(
	State(services): State<crate::State>,
	body: Ruma<set_room_account_data::v3::Request>,
) -> Result<set_room_account_data::v3::Response> {
	let sender_user = body.sender_user();

	if sender_user != body.user_id && body.appservice_info.is_none() {
		return Err!(Request(Forbidden("You cannot set account data for other users.")));
	}

	set_account_data(
		&services,
		Some(&body.room_id),
		&body.user_id,
		&body.event_type.to_string(),
		body.data.json(),
	)
	.await?;

	Ok(set_room_account_data::v3::Response {})
}

/// # `GET /_matrix/client/r0/user/{userId}/account_data/{type}`
///
/// Gets some account data for the sender user.
pub(crate) async fn get_global_account_data_route(
	State(services): State<crate::State>,
	body: Ruma<get_global_account_data::v3::Request>,
) -> Result<get_global_account_data::v3::Response> {
	let sender_user = body.sender_user();

	if sender_user != body.user_id && body.appservice_info.is_none() {
		return Err!(Request(Forbidden("You cannot get account data of other users.")));
	}

	let account_data: ExtractGlobalEventContent = services
		.account_data
		.get_global(&body.user_id, body.event_type.clone())
		.await
		.map_err(|_| err!(Request(NotFound("Data not found."))))?;

	if is_empty_content(&account_data.content) {
		return Err!(Request(NotFound("Data not found.")));
	}

	Ok(get_global_account_data::v3::Response { account_data: account_data.content })
}

/// # `GET /_matrix/client/r0/user/{userId}/rooms/{roomId}/account_data/{type}`
///
/// Gets some room account data for the sender user.
pub(crate) async fn get_room_account_data_route(
	State(services): State<crate::State>,
	body: Ruma<get_room_account_data::v3::Request>,
) -> Result<get_room_account_data::v3::Response> {
	let sender_user = body.sender_user();

	if sender_user != body.user_id && body.appservice_info.is_none() {
		return Err!(Request(Forbidden("You cannot get account data of other users.")));
	}

	let account_data: ExtractRoomEventContent = services
		.account_data
		.get_room(&body.room_id, &body.user_id, body.event_type.clone())
		.await
		.map_err(|_| err!(Request(NotFound("Data not found."))))?;

	if is_empty_content(&account_data.content) {
		return Err!(Request(NotFound("Data not found.")));
	}

	Ok(get_room_account_data::v3::Response { account_data: account_data.content })
}

/// # `DELETE /_matrix/client/unstable/org.matrix.msc3391/user/{userId}/account_data/{type}`
///
/// MSC3391: erase the named global account data type for the user.
pub(crate) async fn delete_global_account_data_route(
	State(services): State<crate::State>,
	body: Ruma<delete_global_account_data::unstable::Request>,
) -> Result<delete_global_account_data::unstable::Response> {
	let sender_user = body.sender_user();

	if sender_user != body.user_id && body.appservice_info.is_none() {
		return Err!(Request(Forbidden("You cannot delete account data for other users.")));
	}

	services
		.account_data
		.delete(None, &body.user_id, body.event_type.to_string().into())
		.await?;

	Ok(delete_global_account_data::unstable::Response {})
}

/// # `DELETE /_matrix/client/unstable/org.matrix.msc3391/user/{userId}/rooms/{roomId}/account_data/{type}`
///
/// MSC3391: erase the named room account data type for the user.
pub(crate) async fn delete_room_account_data_route(
	State(services): State<crate::State>,
	body: Ruma<delete_room_account_data::unstable::Request>,
) -> Result<delete_room_account_data::unstable::Response> {
	let sender_user = body.sender_user();

	if sender_user != body.user_id && body.appservice_info.is_none() {
		return Err!(Request(Forbidden("You cannot delete account data for other users.")));
	}

	services
		.account_data
		.delete(Some(&body.room_id), &body.user_id, body.event_type.clone())
		.await?;

	Ok(delete_room_account_data::unstable::Response {})
}

async fn set_account_data(
	services: &Services,
	room_id: Option<&RoomId>,
	sender_user: &UserId,
	event_type_s: &str,
	data: &RawJsonValue,
) -> Result {
	if event_type_s == RoomAccountDataEventType::FullyRead.to_cow_str() {
		return Err!(Request(BadJson(
			"This endpoint cannot be used for marking a room as fully read (setting \
			 m.fully_read)"
		)));
	}

	if event_type_s == GlobalAccountDataEventType::PushRules.to_cow_str() {
		return Err!(Request(BadJson(
			"This endpoint cannot be used for setting/configuring push rules."
		)));
	}

	let data: serde_json::Value = serde_json::from_str(data.get())
		.map_err(|e| err!(Request(BadJson(warn!("Invalid JSON provided: {e}")))))?;

	services
		.account_data
		.update(
			room_id,
			sender_user,
			event_type_s.into(),
			&json!({
				"type": event_type_s,
				"content": data,
			}),
		)
		.await
}

/// MSC3391: tombstoned account data carries `content: {}`. Sync delta
/// surfaces the empty event so clients can apply the deletion; everywhere
/// else (GET, initial sync) treats it as not-present.
fn is_empty_content<T>(content: &Raw<T>) -> bool { is_empty_object_json(content.json()) }

/// Equivalent test against a stored account-data event (`{type, content}`)
/// rather than the bare `content` payload. Used by sync filters.
pub(crate) fn is_empty_account_data_event<T>(event: &Raw<T>) -> bool {
	#[derive(Deserialize)]
	struct ContentOnly<'a> {
		#[serde(borrow)]
		content: &'a RawJsonValue,
	}

	serde_json::from_str::<ContentOnly<'_>>(event.json().get())
		.ok()
		.is_some_and(|c| is_empty_object_json(c.content))
}

fn is_empty_object_json(s: &RawJsonValue) -> bool {
	serde_json::from_str::<JsonValue>(s.get())
		.ok()
		.and_then(|v| v.as_object().map(serde_json::Map::is_empty))
		.unwrap_or(false)
}

#[derive(Deserialize)]
struct ExtractRoomEventContent {
	content: Raw<AnyRoomAccountDataEventContent>,
}

#[derive(Deserialize)]
struct ExtractGlobalEventContent {
	content: Raw<AnyGlobalAccountDataEventContent>,
}
