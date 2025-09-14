use futures::StreamExt;
use ruma::{OwnedRoomId, OwnedServerName, OwnedUserId};
use tuwunel_core::{Err, Result};

use crate::{command, get_room_info};

#[command]
pub(super) async fn disable_room(&self, room_id: OwnedRoomId) -> Result<String> {
	self.services.metadata.disable_room(&room_id);
	Ok("Room disabled.".to_owned())
}

#[command]
pub(super) async fn enable_room(&self, room_id: OwnedRoomId) -> Result<String> {
	self.services.metadata.enable_room(&room_id);
	Ok("Room enabled.".to_owned())
}

#[command]
pub(super) async fn incoming_federation(&self) -> Result<String> {
	Err!("This command is temporarily disabled")
}

#[command]
pub(super) async fn fetch_support_well_known(
	&self,
	server_name: OwnedServerName,
) -> Result<String> {
	let response = self
		.services
		.client
		.default
		.get(format!("https://{server_name}/.well-known/matrix/support"))
		.send()
		.await?;

	let text = response.text().await?;

	if text.is_empty() {
		return Err!("Response text/body is empty.");
	}

	if text.len() > 1500 {
		return Err!(
			"Response text/body is over 1500 characters, assuming no support well-known.",
		);
	}

	let json: serde_json::Value = match serde_json::from_str(&text) {
		| Ok(json) => json,
		| Err(_) => {
			return Err!("Response text/body is not valid JSON.");
		},
	};

	let pretty_json: String = match serde_json::to_string_pretty(&json) {
		| Ok(json) => json,
		| Err(_) => {
			return Err!("Response text/body is not valid JSON.");
		},
	};

	Ok(format!("Got JSON response:\n\n```json\n{pretty_json}\n```"))
}

#[command]
pub(super) async fn remote_user_in_rooms(&self, user_id: OwnedUserId) -> Result<String> {
	if user_id.server_name() == self.services.server.name {
		return Err!(
			"User belongs to our server, please use `list-joined-rooms` user admin command \
			 instead.",
		);
	}

	if !self.services.users.exists(&user_id).await {
		return Err!("Remote user does not exist in our database.");
	}

	let mut rooms: Vec<(OwnedRoomId, u64, String)> = self
		.services
		.state_cache
		.rooms_joined(&user_id)
		.then(|room_id| get_room_info(self.services, room_id))
		.collect()
		.await;

	if rooms.is_empty() {
		return Err!("User is not in any rooms.");
	}

	rooms.sort_by_key(|r| r.1);
	rooms.reverse();

	let num = rooms.len();
	let body = rooms
		.iter()
		.map(|(id, members, name)| format!("{id} | Members: {members} | Name: {name}"))
		.collect::<Vec<_>>()
		.join("\n");

	Ok(format!("Rooms {user_id} shares with us ({num}):\n```\n{body}\n```"))
}
