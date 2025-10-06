use futures::{StreamExt, future::join};
use ruma::{api::client::sync::sync_events::v5::response, events::AnyRawAccountDataEvent};
use tuwunel_core::{
	Result, extract_variant,
	utils::{IterStream, ReadyExt, stream::BroadbandExt},
};
use tuwunel_service::Services;

use super::{KnownRooms, SyncInfo, TodoRoom, TodoRooms, extension_rooms_todo};

#[tracing::instrument(level = "trace", skip_all, fields(globalsince, next_batch))]
pub(super) async fn collect(
	services: &Services,
	sync_info: SyncInfo<'_>,
	next_batch: u64,
	known_rooms: &KnownRooms,
	todo_rooms: &TodoRooms,
) -> Result<response::AccountData> {
	let SyncInfo { sender_user, globalsince, request, .. } = sync_info;

	let lists = request
		.extensions
		.account_data
		.lists
		.as_deref()
		.map(<[_]>::iter);

	let rooms = request
		.extensions
		.account_data
		.rooms
		.as_deref()
		.map(<[_]>::iter);

	let rooms = extension_rooms_todo(sync_info, known_rooms, todo_rooms, lists, rooms)
		.stream()
		.broad_filter_map(async |room_id| {
			let &TodoRoom { roomsince, .. } = todo_rooms.get(room_id)?;
			let changes: Vec<_> = services
				.account_data
				.changes_since(Some(room_id), sender_user, roomsince, Some(next_batch))
				.ready_filter_map(|e| extract_variant!(e, AnyRawAccountDataEvent::Room))
				.collect()
				.await;

			changes
				.is_empty()
				.eq(&false)
				.then(move || (room_id.to_owned(), changes))
		})
		.collect();

	let global = services
		.account_data
		.changes_since(None, sender_user, globalsince, Some(next_batch))
		.ready_filter_map(|e| extract_variant!(e, AnyRawAccountDataEvent::Global))
		.collect();

	let (global, rooms) = join(global, rooms).await;

	Ok(response::AccountData { global, rooms })
}
