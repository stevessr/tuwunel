mod account_data;
mod e2ee;
mod receipts;
mod room;
mod to_device;
mod typing;

use std::{
	collections::{BTreeMap, BTreeSet},
	mem::take,
	ops::Deref,
	time::Duration,
};

use axum::extract::State;
use futures::{
	FutureExt, Stream, StreamExt, TryFutureExt, TryStreamExt,
	future::{OptionFuture, join, join3, join5, try_join},
	pin_mut,
};
use ruma::{
	DeviceId, OwnedRoomId, RoomId, UInt, UserId,
	api::client::sync::sync_events::v5::{
		Request, Response, request::ExtensionRoomConfig, response,
	},
	directory::RoomTypeFilter,
	events::room::member::MembershipState,
	uint,
};
use tokio::time::{Instant, timeout_at};
use tuwunel_core::{
	Err, Result, apply,
	error::inspect_log,
	extract_variant, is_equal_to,
	matrix::TypeStateKey,
	trace,
	utils::{
		FutureBoolExt, IterStream, ReadyExt, TryFutureExtExt,
		future::ReadyEqExt,
		math::{ruma_from_usize, usize_from_ruma},
		result::FlatOk,
		stream::{BroadbandExt, TryBroadbandExt, TryReadyExt, WidebandExt},
	},
	warn,
};
use tuwunel_service::{
	Services,
	sync::{KnownRooms, ListId, into_connection_key},
};

use super::share_encrypted_room;
use crate::{Ruma, client::DEFAULT_BUMP_TYPES};

#[derive(Copy, Clone)]
struct SyncInfo<'a> {
	sender_user: &'a UserId,
	sender_device: &'a DeviceId,
	request: &'a Request,
	globalsince: u64,
}

struct TodoRoom {
	membership: MembershipState,
	requested_state: BTreeSet<TypeStateKey>,
	timeline_limit: usize,
	roomsince: u64,
}

type TodoRooms = BTreeMap<OwnedRoomId, TodoRoom>;
type ResponseLists = BTreeMap<ListId, response::List>;

/// `POST /_matrix/client/unstable/org.matrix.simplified_msc3575/sync`
/// ([MSC4186])
///
/// A simplified version of sliding sync ([MSC3575]).
///
/// Get all new events in a sliding window of rooms since the last sync or a
/// given point in time.
///
/// [MSC3575]: https://github.com/matrix-org/matrix-spec-proposals/pull/3575
/// [MSC4186]: https://github.com/matrix-org/matrix-spec-proposals/pull/4186
#[tracing::instrument(
	name = "sync",
	level = "debug",
	skip_all,
	fields(
		user_id = %body.sender_user(),
		device_id = %body.sender_device(),
	)
)]
pub(crate) async fn sync_events_v5_route(
	State(ref services): State<crate::State>,
	mut body: Ruma<Request>,
) -> Result<Response> {
	debug_assert!(DEFAULT_BUMP_TYPES.is_sorted(), "DEFAULT_BUMP_TYPES is not sorted");

	let mut request = take(&mut body.body);
	let mut globalsince = request
		.pos
		.as_ref()
		.and_then(|string| string.parse().ok())
		.unwrap_or(0);

	let (sender_user, sender_device) = body.sender();
	let conn_key = into_connection_key(sender_user, sender_device, request.conn_id.as_deref());

	if globalsince != 0 && !services.sync.is_connection_cached(&conn_key) {
		return Err!(Request(UnknownPos(
			"Connection data unknown to server; restarting sync stream."
		)));
	}

	// Client / User requested an initial sync
	if globalsince == 0 {
		services.sync.forget_connection(&conn_key);
	}

	// Get sticky parameters from cache
	let known_rooms = services
		.sync
		.update_cache(&conn_key, &mut request);

	let sync_info = SyncInfo {
		sender_user,
		sender_device,
		globalsince,
		request: &request,
	};

	let lists = handle_lists(services, sync_info, known_rooms);

	let ping_presence = services
		.presence
		.maybe_ping_presence(sender_user, &request.set_presence)
		.inspect_err(inspect_log)
		.ok();

	let ((known_rooms, todo_rooms, lists), _) = join(lists, ping_presence).await;

	let timeout = request
		.timeout
		.as_ref()
		.map(Duration::as_millis)
		.map(TryInto::try_into)
		.flat_ok()
		.unwrap_or(services.config.client_sync_timeout_default)
		.max(services.config.client_sync_timeout_min)
		.min(services.config.client_sync_timeout_max);

	let stop_at = Instant::now()
		.checked_add(Duration::from_millis(timeout))
		.expect("configuration must limit maximum timeout");

	let mut response = Response {
		txn_id: request.txn_id.clone(),
		lists,
		pos: Default::default(),
		rooms: Default::default(),
		extensions: Default::default(),
	};

	loop {
		let watchers = services.sync.watch(sender_user, sender_device);
		let next_batch = services.globals.wait_pending().await?;

		debug_assert!(globalsince <= next_batch, "next_batch is monotonic");
		if globalsince < next_batch {
			let rooms = handle_rooms(services, sync_info, next_batch, &todo_rooms)
				.map_ok(|rooms| response.rooms = rooms);

			let extensions =
				handle_extensions(services, sync_info, next_batch, &known_rooms, &todo_rooms)
					.map_ok(|extensions| response.extensions = extensions);

			try_join(rooms, extensions).boxed().await?;

			if !is_empty_response(&response) {
				trace!(globalsince, next_batch, "response {response:?}");
				response.pos = next_batch.to_string().into();
				return Ok(response);
			}
		}

		if timeout_at(stop_at, watchers).await.is_err() {
			trace!(globalsince, next_batch, "timeout; empty response");
			response.pos = next_batch.to_string().into();
			return Ok(response);
		}

		trace!(
			globalsince,
			last_batch = ?next_batch,
			count = ?services.globals.pending_count(),
			stop_at = ?stop_at,
			"notified by watcher"
		);

		globalsince = next_batch;
	}
}

fn is_empty_response(response: &Response) -> bool {
	response.extensions.is_empty()
		&& response
			.rooms
			.iter()
			.all(|(_, room)| room.timeline.is_empty() && room.invite_state.is_none())
}

#[tracing::instrument(
    level = "debug",
    skip_all,
    fields(
        known_rooms = known_rooms.len(),
    )
)]
#[allow(clippy::too_many_arguments)]
async fn handle_lists(
	services: &Services,
	sync_info: SyncInfo<'_>,
	known_rooms: KnownRooms,
) -> (KnownRooms, TodoRooms, ResponseLists) {
	let &SyncInfo {
		sender_user,
		sender_device,
		request,
		globalsince,
	} = &sync_info;

	let all_joined_rooms = services
		.state_cache
		.rooms_joined(sender_user)
		.map(ToOwned::to_owned)
		.collect::<Vec<OwnedRoomId>>();

	let all_invited_rooms = services
		.state_cache
		.rooms_invited(sender_user)
		.map(ToOwned::to_owned)
		.collect::<Vec<OwnedRoomId>>();

	let all_knocked_rooms = services
		.state_cache
		.rooms_knocked(sender_user)
		.map(ToOwned::to_owned)
		.collect::<Vec<OwnedRoomId>>();

	let (all_joined_rooms, all_invited_rooms, all_knocked_rooms) =
		join3(all_joined_rooms, all_invited_rooms, all_knocked_rooms).await;

	let all_invited_rooms = all_invited_rooms.iter().map(AsRef::as_ref);
	let all_knocked_rooms = all_knocked_rooms.iter().map(AsRef::as_ref);
	let all_joined_rooms = all_joined_rooms.iter().map(AsRef::as_ref);
	let all_rooms = all_joined_rooms
		.clone()
		.chain(all_invited_rooms.clone())
		.chain(all_knocked_rooms.clone());

	let mut todo_rooms: TodoRooms = BTreeMap::new();
	let mut response_lists = ResponseLists::new();
	for (list_id, list) in &request.lists {
		let active_rooms: Vec<_> = match list.filters.as_ref().and_then(|f| f.is_invite) {
			| None => all_rooms.clone().collect(),
			| Some(true) => all_invited_rooms.clone().collect(),
			| Some(false) => all_joined_rooms.clone().collect(),
		};

		let active_rooms = match list.filters.as_ref().map(|f| &f.not_room_types) {
			| None => active_rooms,
			| Some(filter) if filter.is_empty() => active_rooms,
			| Some(value) =>
				filter_rooms(
					services,
					value,
					&true,
					active_rooms.iter().stream().map(Deref::deref),
				)
				.collect()
				.await,
		};

		let mut new_known_rooms: BTreeSet<OwnedRoomId> = BTreeSet::new();
		let ranges = list.ranges.clone();
		for mut range in ranges {
			range.0 = uint!(0);
			range.1 = range.1.checked_add(uint!(1)).unwrap_or(range.1);
			range.1 = range
				.1
				.clamp(range.0, UInt::try_from(active_rooms.len()).unwrap_or(UInt::MAX));

			let room_ids =
				active_rooms[usize_from_ruma(range.0)..usize_from_ruma(range.1)].to_vec();

			let new_rooms: BTreeSet<OwnedRoomId> = room_ids
				.clone()
				.into_iter()
				.map(From::from)
				.collect();

			new_known_rooms.extend(new_rooms);
			for room_id in room_ids {
				let todo_room = todo_rooms
					.entry(room_id.to_owned())
					.or_insert(TodoRoom {
						membership: MembershipState::Join,
						requested_state: BTreeSet::new(),
						timeline_limit: 0_usize,
						roomsince: u64::MAX,
					});

				todo_room.membership = if all_invited_rooms
					.clone()
					.any(is_equal_to!(room_id))
				{
					MembershipState::Invite
				} else {
					MembershipState::Join
				};

				todo_room.requested_state.extend(
					list.room_details
						.required_state
						.iter()
						.map(|(ty, sk)| (ty.clone(), sk.as_str().into())),
				);

				let limit: usize = usize_from_ruma(list.room_details.timeline_limit).min(100);
				todo_room.timeline_limit = todo_room.timeline_limit.max(limit);

				// 0 means unknown because it got out of date
				todo_room.roomsince = todo_room.roomsince.min(
					known_rooms
						.get(list_id.as_str())
						.and_then(|k| k.get(room_id))
						.copied()
						.unwrap_or(0),
				);
			}
		}

		if let Some(conn_id) = request.conn_id.as_deref() {
			let conn_key = into_connection_key(sender_user, sender_device, conn_id.into());
			let list_id = list_id.as_str().into();
			services
				.sync
				.update_known_rooms(&conn_key, list_id, new_known_rooms, globalsince);
		}

		response_lists.insert(list_id.clone(), response::List {
			count: ruma_from_usize(active_rooms.len()),
		});
	}

	let (known_rooms, todo_rooms) =
		fetch_subscriptions(services, sync_info, known_rooms, todo_rooms).await;

	(known_rooms, todo_rooms, response_lists)
}

#[tracing::instrument(
	level = "debug",
	skip_all,
	fields(
		global_since,
		known_rooms = known_rooms.len(),
		todo_rooms = todo_rooms.len(),
	)
)]
async fn fetch_subscriptions(
	services: &Services,
	SyncInfo {
		sender_user,
		sender_device,
		globalsince,
		request,
	}: SyncInfo<'_>,
	known_rooms: KnownRooms,
	todo_rooms: TodoRooms,
) -> (KnownRooms, TodoRooms) {
	let subs = (todo_rooms, BTreeSet::new());
	let (todo_rooms, known_subs) = request
		.room_subscriptions
		.iter()
		.stream()
		.broad_filter_map(async |(room_id, room)| {
			let not_exists = services.metadata.exists(room_id).eq(&false);
			let is_disabled = services.metadata.is_disabled(room_id);
			let is_banned = services.metadata.is_banned(room_id);

			pin_mut!(not_exists, is_disabled, is_banned);
			not_exists
				.or(is_disabled)
				.or(is_banned)
				.await
				.eq(&false)
				.then_some((room_id, room))
		})
		.ready_fold(subs, |(mut todo_rooms, mut known_subs), (room_id, room)| {
			let todo_room = todo_rooms
				.entry(room_id.clone())
				.or_insert(TodoRoom {
					membership: MembershipState::Join,
					requested_state: BTreeSet::new(),
					timeline_limit: 0_usize,
					roomsince: u64::MAX,
				});

			todo_room.requested_state.extend(
				room.required_state
					.iter()
					.map(|(ty, sk)| (ty.clone(), sk.as_str().into())),
			);

			let limit: UInt = room.timeline_limit;
			todo_room.timeline_limit = todo_room
				.timeline_limit
				.max(usize_from_ruma(limit));

			// 0 means unknown because it got out of date
			todo_room.roomsince = todo_room.roomsince.min(
				known_rooms
					.get("subscriptions")
					.and_then(|k| k.get(room_id))
					.copied()
					.unwrap_or(0),
			);

			known_subs.insert(room_id.clone());
			(todo_rooms, known_subs)
		})
		.await;

	if let Some(conn_id) = request.conn_id.as_deref() {
		let conn_key = into_connection_key(sender_user, sender_device, conn_id.into());
		let list_id = "subscriptions".into();
		services
			.sync
			.update_known_rooms(&conn_key, list_id, known_subs, globalsince);
	}

	(known_rooms, todo_rooms)
}

#[tracing::instrument(
	level = "debug",
	skip_all,
	fields(?filters, negate)
)]
fn filter_rooms<'a, Rooms>(
	services: &'a Services,
	filters: &'a [RoomTypeFilter],
	negate: &'a bool,
	rooms: Rooms,
) -> impl Stream<Item = &'a RoomId> + Send + 'a
where
	Rooms: Stream<Item = &'a RoomId> + Send + 'a,
{
	rooms
		.wide_filter_map(async |room_id| {
			match services
				.state_accessor
				.get_room_type(room_id)
				.await
			{
				| Ok(room_type) => Some((room_id, Some(room_type))),
				| Err(e) if e.is_not_found() => Some((room_id, None)),
				| Err(_) => None,
			}
		})
		.map(|(room_id, room_type)| (room_id, RoomTypeFilter::from(room_type)))
		.ready_filter_map(|(room_id, room_type_filter)| {
			let contains = filters.contains(&room_type_filter);
			let pos = !*negate && (filters.is_empty() || contains);
			let neg = *negate && !contains;

			(pos || neg).then_some(room_id)
		})
}

#[tracing::instrument(
    level = "debug",
    skip_all,
    fields(
        next_batch,
        todo_rooms = todo_rooms.len(),
    )
)]
async fn handle_rooms(
	services: &Services,
	sync_info: SyncInfo<'_>,
	next_batch: u64,
	todo_rooms: &TodoRooms,
) -> Result<BTreeMap<OwnedRoomId, response::Room>> {
	let rooms: BTreeMap<_, _> = todo_rooms
		.iter()
		.try_stream()
		.broad_and_then(async |(room_id, todo_room)| {
			let room = room::handle(services, next_batch, sync_info, room_id, todo_room).await?;

			Ok((room_id, room))
		})
		.ready_try_filter_map(|(room_id, room)| Ok(room.map(|room| (room_id, room))))
		.map_ok(|(room_id, room)| (room_id.to_owned(), room))
		.try_collect()
		.await?;

	Ok(rooms)
}

#[tracing::instrument(
	level = "debug",
	skip_all,
	fields(
		global_since,
		known_rooms = known_rooms.len(),
	)
)]
async fn handle_extensions(
	services: &Services,
	sync_info: SyncInfo<'_>,
	next_batch: u64,
	known_rooms: &KnownRooms,
	todo_rooms: &TodoRooms,
) -> Result<response::Extensions> {
	let SyncInfo { request, .. } = sync_info;

	let account_data: OptionFuture<_> = request
		.extensions
		.account_data
		.enabled
		.unwrap_or(false)
		.then(|| account_data::collect(services, sync_info, next_batch, known_rooms, todo_rooms))
		.into();

	let receipts: OptionFuture<_> = request
		.extensions
		.receipts
		.enabled
		.unwrap_or(false)
		.then(|| receipts::collect(services, sync_info, next_batch, known_rooms, todo_rooms))
		.into();

	let typing: OptionFuture<_> = request
		.extensions
		.typing
		.enabled
		.unwrap_or(false)
		.then(|| typing::collect(services, sync_info, next_batch, known_rooms, todo_rooms))
		.into();

	let to_device: OptionFuture<_> = request
		.extensions
		.to_device
		.enabled
		.unwrap_or(false)
		.then(|| to_device::collect(services, sync_info, next_batch))
		.into();

	let e2ee: OptionFuture<_> = request
		.extensions
		.e2ee
		.enabled
		.unwrap_or(false)
		.then(|| e2ee::collect(services, sync_info, next_batch))
		.into();

	let (account_data, receipts, typing, to_device, e2ee) =
		join5(account_data, receipts, typing, to_device, e2ee)
			.map(apply!(5, |t: Option<_>| t.unwrap_or(Ok(Default::default()))))
			.await;

	Ok(response::Extensions {
		account_data: account_data?,
		receipts: receipts?,
		typing: typing?,
		to_device: to_device?,
		e2ee: e2ee?,
	})
}

fn extension_rooms_todo<'a, ListIter, ConfigIter>(
	SyncInfo { request, .. }: SyncInfo<'a>,
	known_rooms: &'a KnownRooms,
	todo_rooms: &'a TodoRooms,
	lists: Option<ListIter>,
	rooms: Option<ConfigIter>,
) -> impl Iterator<Item = &'a RoomId> + Send + Sync + 'a
where
	ListIter: Iterator<Item = &'a ListId> + Clone + Send + Sync + 'a,
	ConfigIter: Iterator<Item = &'a ExtensionRoomConfig> + Clone + Send + Sync + 'a,
{
	let lists_explicit = lists.clone().into_iter().flatten();

	let rooms_explicit = rooms
		.clone()
		.into_iter()
		.flatten()
		.filter_map(|erc| extract_variant!(erc, ExtensionRoomConfig::Room))
		.map(AsRef::<RoomId>::as_ref);

	let lists_requested = request
		.lists
		.keys()
		.filter(move |_| lists.is_none());

	let rooms_implicit = todo_rooms
		.keys()
		.map(AsRef::as_ref)
		.filter(move |_| rooms.is_none());

	lists_explicit
		.chain(lists_requested)
		.flat_map(|list_id| {
			known_rooms
				.get(list_id.as_str())
				.into_iter()
				.flat_map(BTreeMap::keys)
		})
		.map(AsRef::as_ref)
		.chain(rooms_explicit)
		.chain(rooms_implicit)
}
