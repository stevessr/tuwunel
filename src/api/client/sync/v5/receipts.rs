use futures::{FutureExt, StreamExt};
use ruma::{
	OwnedRoomId, RoomId,
	api::client::sync::sync_events::v5::response,
	events::{AnySyncEphemeralRoomEvent, receipt::SyncReceiptEvent},
	serde::Raw,
};
use tuwunel_core::{
	Result,
	utils::{BoolExt, IterStream, stream::BroadbandExt},
};
use tuwunel_service::{Services, rooms::read_receipt::pack_receipts};

use super::{KnownRooms, SyncInfo, TodoRoom, TodoRooms, extension_rooms_todo};

#[tracing::instrument(level = "trace", skip_all)]
pub(super) async fn collect(
	services: &Services,
	sync_info: SyncInfo<'_>,
	next_batch: u64,
	known_rooms: &KnownRooms,
	todo_rooms: &TodoRooms,
) -> Result<response::Receipts> {
	let SyncInfo { request, .. } = sync_info;

	let lists = request
		.extensions
		.receipts
		.lists
		.as_deref()
		.map(<[_]>::iter);

	let rooms = request
		.extensions
		.receipts
		.rooms
		.as_deref()
		.map(<[_]>::iter);

	let rooms = extension_rooms_todo(sync_info, known_rooms, todo_rooms, lists, rooms)
		.stream()
		.broad_filter_map(async |room_id| {
			collect_room(services, sync_info, next_batch, todo_rooms, room_id).await
		})
		.collect()
		.await;

	Ok(response::Receipts { rooms })
}

async fn collect_room(
	services: &Services,
	SyncInfo { sender_user, .. }: SyncInfo<'_>,
	next_batch: u64,
	todo_rooms: &TodoRooms,
	room_id: &RoomId,
) -> Option<(OwnedRoomId, Raw<SyncReceiptEvent>)> {
	let &TodoRoom { roomsince, .. } = todo_rooms.get(room_id)?;
	let private_receipt = services
		.read_receipt
		.last_privateread_update(sender_user, room_id)
		.then(async |last_private_update| {
			if last_private_update <= roomsince || last_private_update > next_batch {
				return None;
			}

			services
				.read_receipt
				.private_read_get(room_id, sender_user)
				.map(Some)
				.await
		})
		.map(Option::into_iter)
		.map(Iterator::flatten)
		.map(IterStream::stream)
		.flatten_stream();

	let receipts: Vec<Raw<AnySyncEphemeralRoomEvent>> = services
		.read_receipt
		.readreceipts_since(room_id, roomsince, Some(next_batch))
		.filter_map(async |(read_user, _ts, v)| {
			services
				.users
				.user_is_ignored(read_user, sender_user)
				.await
				.or_some(v)
		})
		.chain(private_receipt)
		.collect()
		.boxed()
		.await;

	receipts
		.is_empty()
		.eq(&false)
		.then(|| (room_id.to_owned(), pack_receipts(receipts.into_iter())))
}
