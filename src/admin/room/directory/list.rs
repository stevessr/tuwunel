use futures::StreamExt;
use tuwunel_core::Result;

use crate::{PAGE_SIZE, admin_command, get_room_info};

#[admin_command]
pub(super) async fn directory_list(&self, page: Option<usize>) -> Result {
	let page = page.unwrap_or(1);
	let mut rooms: Vec<_> = self
		.services
		.directory
		.public_rooms()
		.then(|room_id| get_room_info(self.services, room_id))
		.collect()
		.await;

	rooms.sort_by_key(|r| r.1);
	rooms.reverse();

	let rooms: Vec<_> = rooms
		.into_iter()
		.skip(page.saturating_sub(1).saturating_mul(PAGE_SIZE))
		.take(PAGE_SIZE)
		.collect();

	if rooms.is_empty() {
		self.write_str("No rooms are published.").await?;

		return Ok(());
	}

	let body = rooms
		.iter()
		.map(|(id, members, name)| format!("{id} | Members: {members} | Name: {name}"))
		.collect::<Vec<_>>()
		.join("\n");

	self.write_str(&format!("Rooms (page {page}):\n```\n{body}\n```"))
		.await
}
