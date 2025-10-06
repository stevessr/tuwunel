use std::collections::BTreeMap;

use futures::{FutureExt, StreamExt, TryFutureExt};
use ruma::{
	api::client::sync::sync_events::v5::response,
	events::typing::{SyncTypingEvent, TypingEventContent},
	serde::Raw,
};
use tuwunel_core::{
	Result, debug_error,
	utils::{IterStream, ReadyExt},
};
use tuwunel_service::Services;

use super::{KnownRooms, SyncInfo, TodoRooms, extension_rooms_todo};

#[tracing::instrument(level = "trace", skip_all, fields(globalsince))]
pub(super) async fn collect(
	services: &Services,
	sync_info: SyncInfo<'_>,
	_next_batch: u64,
	known_rooms: &KnownRooms,
	todo_rooms: &TodoRooms,
) -> Result<response::Typing> {
	use response::Typing;

	let SyncInfo { sender_user, request, .. } = sync_info;

	let lists = request
		.extensions
		.typing
		.lists
		.as_deref()
		.map(<[_]>::iter);

	let rooms = request
		.extensions
		.typing
		.rooms
		.as_deref()
		.map(<[_]>::iter);

	extension_rooms_todo(sync_info, known_rooms, todo_rooms, lists, rooms)
		.stream()
		.filter_map(async |room_id| {
			services
				.typing
				.typing_users_for_user(room_id, sender_user)
				.inspect_err(|e| debug_error!(%room_id, "Failed to get typing events: {e}"))
				.await
				.ok()
				.filter(|users| !users.is_empty())
				.map(|users| (room_id, users))
		})
		.ready_filter_map(|(room_id, users)| {
			let content = TypingEventContent::new(users);
			let event = SyncTypingEvent { content };
			let event = Raw::new(&event);

			Some((room_id.to_owned(), event.ok()?))
		})
		.collect::<BTreeMap<_, _>>()
		.map(|rooms| Typing { rooms })
		.map(Ok)
		.await
}
