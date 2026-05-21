use futures::StreamExt;
use ruma::OwnedRoomId;
use tuwunel_core::{Err, Result, utils::IterStream};

use crate::{admin_command, get_room_info};

#[admin_command]
pub(super) async fn list_banned_rooms(&self, no_details: bool) -> Result {
	let room_ids: Vec<OwnedRoomId> = self
		.services
		.metadata
		.list_banned_rooms()
		.map(Into::into)
		.collect()
		.await;

	if room_ids.is_empty() {
		return Err!("No rooms are banned.");
	}

	let mut rooms = room_ids
		.iter()
		.stream()
		.then(|room_id| get_room_info(self.services, room_id))
		.collect::<Vec<_>>()
		.await;

	rooms.sort_by_key(|r| r.1);
	rooms.reverse();

	let num = rooms.len();

	let body = rooms
		.iter()
		.map(|(id, members, name)| {
			if no_details {
				format!("{id}")
			} else {
				format!("{id}\tMembers: {members}\tName: {name}")
			}
		})
		.collect::<Vec<_>>()
		.join("\n");

	self.write_str(&format!("Rooms Banned ({num}):\n```\n{body}\n```"))
		.await
}
