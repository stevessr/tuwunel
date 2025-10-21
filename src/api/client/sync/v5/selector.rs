use std::cmp::Ordering;

use futures::{
	FutureExt, StreamExt, TryFutureExt,
	future::{OptionFuture, join3},
};
use ruma::{OwnedRoomId, UInt, events::room::member::MembershipState, uint};
use tuwunel_core::{
	apply, is_true,
	matrix::PduCount,
	trace,
	utils::{
		BoolExt,
		future::TryExtExt,
		math::usize_from_ruma,
		stream::{BroadbandExt, IterStream},
	},
};
use tuwunel_service::sync::Connection;

use super::{
	ListIds, ResponseLists, SyncInfo, Window, WindowRoom, filter_room, filter_room_meta,
};

#[tracing::instrument(level = "debug", skip_all)]
pub(super) async fn selector(
	conn: &mut Connection,
	sync_info: SyncInfo<'_>,
) -> (Window, ResponseLists) {
	use MembershipState::*;

	let SyncInfo { services, sender_user, request, .. } = sync_info;

	trace!(?request);
	let mut rooms = services
		.state_cache
		.user_memberships(sender_user, Some(&[Join, Invite, Knock]))
		.map(|(membership, room_id)| (room_id.to_owned(), Some(membership)))
		.broad_filter_map(|(room_id, membership)| {
			match_lists_for_room(sync_info, conn, room_id, membership)
		})
		.collect::<Vec<_>>()
		.await;

	rooms.sort_by(room_sort);
	rooms
		.iter_mut()
		.enumerate()
		.for_each(|(i, room)| {
			room.ranked = i;
		});

	trace!(?rooms);
	let lists = response_lists(rooms.iter());

	trace!(?lists);
	let window = select_window(sync_info, conn, rooms.iter(), &lists).await;

	trace!(?window);
	for room in &rooms {
		conn.rooms
			.entry(room.room_id.clone())
			.or_default();
	}

	(window, lists)
}

async fn select_window<'a, Rooms>(
	sync_info: SyncInfo<'_>,
	conn: &Connection,
	rooms: Rooms,
	lists: &ResponseLists,
) -> Window
where
	Rooms: Iterator<Item = &'a WindowRoom> + Clone + Send + Sync,
{
	static FULL_RANGE: (UInt, UInt) = (UInt::MIN, UInt::MAX);

	let selections = lists
		.keys()
		.cloned()
		.filter_map(|id| conn.lists.get(&id).map(|list| (id, list)))
		.flat_map(|(id, list)| {
			let full_range = list
				.ranges
				.is_empty()
				.then_some(&FULL_RANGE)
				.into_iter();

			list.ranges
				.iter()
				.chain(full_range)
				.map(apply!(2, usize_from_ruma))
				.map(move |range| (id.clone(), range))
		})
		.flat_map(|(id, (start, end))| {
			rooms
				.clone()
				.filter(move |&room| room.lists.contains(&id))
				.enumerate()
				.skip_while(move |&(i, room)| {
					i < start
						|| conn
							.rooms
							.get(&room.room_id)
							.is_some_and(|conn_room| conn_room.roomsince >= room.last_count)
				})
				.take(end.saturating_add(1).saturating_sub(start))
				.map(|(_, room)| (room.room_id.clone(), room.clone()))
		});

	conn.subscriptions
		.iter()
		.stream()
		.broad_filter_map(async |(room_id, _)| {
			filter_room_meta(sync_info, room_id)
				.await
				.then(|| WindowRoom {
					room_id: room_id.clone(),
					membership: None,
					lists: Default::default(),
					ranked: usize::MAX,
					last_count: 0,
				})
		})
		.map(|room| (room.room_id.clone(), room))
		.chain(selections.stream())
		.collect()
		.await
}

#[tracing::instrument(
	name = "matcher",
	level = "trace",
	skip_all,
	fields(?room_id, ?membership)
)]
async fn match_lists_for_room(
	sync_info: SyncInfo<'_>,
	conn: &Connection,
	room_id: OwnedRoomId,
	membership: Option<MembershipState>,
) -> Option<WindowRoom> {
	let SyncInfo { services, sender_user, .. } = sync_info;

	let lists = conn
		.lists
		.iter()
		.stream()
		.filter_map(async |(id, list)| {
			let filter: OptionFuture<_> = list
				.filters
				.clone()
				.map(async |filters| {
					filter_room(sync_info, &filters, &room_id, membership.as_ref()).await
				})
				.into();

			filter
				.await
				.is_none_or(is_true!())
				.then(|| id.clone())
		})
		.collect::<ListIds>()
		.await;

	let last_timeline_count: OptionFuture<_> = lists
		.is_empty()
		.is_false()
		.then(|| {
			services
				.timeline
				.last_timeline_count(None, &room_id, None)
				.map_ok(PduCount::into_unsigned)
				.ok()
		})
		.into();

	let last_account_count: OptionFuture<_> = lists
		.is_empty()
		.is_false()
		.then(|| {
			services
				.account_data
				.last_count(Some(room_id.as_ref()), sender_user, None)
				.ok()
		})
		.into();

	let last_receipt_count: OptionFuture<_> = lists
		.is_empty()
		.is_false()
		.then(|| {
			services
				.read_receipt
				.last_receipt_count(&room_id, sender_user.into(), None)
				.map(Result::ok)
		})
		.into();

	let (last_timeline_count, last_account_count, last_receipt_count) =
		join3(last_timeline_count, last_account_count, last_receipt_count).await;

	Some(WindowRoom {
		room_id: room_id.clone(),
		membership,
		lists,
		ranked: 0,
		last_count: [last_timeline_count, last_account_count, last_receipt_count]
			.into_iter()
			.map(Option::flatten)
			.map(Option::unwrap_or_default)
			.max()
			.unwrap_or_default(),
	})
}

fn response_lists<'a, Rooms>(rooms: Rooms) -> ResponseLists
where
	Rooms: Iterator<Item = &'a WindowRoom>,
{
	rooms
		.flat_map(|room| room.lists.iter())
		.fold(ResponseLists::default(), |mut lists, id| {
			let list = lists.entry(id.clone()).or_default();
			list.count = list
				.count
				.checked_add(uint!(1))
				.expect("list count must not overflow JsInt");

			lists
		})
}

fn room_sort(a: &WindowRoom, b: &WindowRoom) -> Ordering {
	if a.membership != b.membership {
		if a.membership == Some(MembershipState::Invite) {
			return Ordering::Less;
		}
		if b.membership == Some(MembershipState::Invite) {
			return Ordering::Greater;
		}
	}

	b.last_count.cmp(&a.last_count)
}
