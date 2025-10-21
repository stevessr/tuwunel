mod account_data;
mod e2ee;
mod filter;
mod receipts;
mod room;
mod selector;
mod to_device;
mod typing;

use std::{collections::BTreeMap, fmt::Debug, time::Duration};

use axum::extract::State;
use futures::{
	FutureExt, TryFutureExt, TryStreamExt,
	future::{OptionFuture, join, join5, try_join},
};
use ruma::{
	DeviceId, OwnedRoomId, RoomId, UserId,
	api::client::sync::sync_events::v5::{
		ListId, Request, Response, request::ExtensionRoomConfig, response,
	},
	events::room::member::MembershipState,
};
use tokio::time::{Instant, timeout_at};
use tuwunel_core::{
	Err, Result, apply, at,
	debug::INFO_SPAN_LEVEL,
	err,
	error::inspect_log,
	extract_variant,
	smallvec::SmallVec,
	trace,
	utils::{
		BoolExt, IterStream, TryFutureExtExt,
		result::FlatOk,
		stream::{TryBroadbandExt, TryReadyExt},
	},
};
use tuwunel_service::{
	Services,
	sync::{Connection, into_connection_key},
};

use self::{
	filter::{filter_room, filter_room_meta},
	selector::selector,
};
use super::share_encrypted_room;
use crate::Ruma;

#[derive(Copy, Clone)]
struct SyncInfo<'a> {
	services: &'a Services,
	sender_user: &'a UserId,
	sender_device: &'a DeviceId,
	request: &'a Request,
}

#[derive(Clone, Debug)]
struct WindowRoom {
	room_id: OwnedRoomId,
	membership: Option<MembershipState>,
	lists: ListIds,
	ranked: usize,
	last_count: u64,
}

type Window = BTreeMap<OwnedRoomId, WindowRoom>;
type ResponseLists = BTreeMap<ListId, response::List>;
type ListIds = SmallVec<[ListId; 1]>;

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
	level = INFO_SPAN_LEVEL,
	skip_all,
	fields(
		user_id = %body.sender_user().localpart(),
		device_id = %body.sender_device(),
		conn_id = ?body.body.conn_id.clone().unwrap_or_default(),
		since = ?body.body.pos.clone().or_else(|| body.body.pos_qrs_.clone()).unwrap_or_default(),
	)
)]
pub(crate) async fn sync_events_v5_route(
	State(ref services): State<crate::State>,
	body: Ruma<Request>,
) -> Result<Response> {
	let (sender_user, sender_device) = body.sender();
	let request = &body.body;
	let since = request
		.pos
		.as_ref()
		.or(request.pos_qrs_.as_ref())
		.and_then(|string| string.parse().ok())
		.unwrap_or(0);

	let timeout = request
		.timeout
		.as_ref()
		.or(request.timeout_qrs_.as_ref())
		.map(Duration::as_millis)
		.map(TryInto::try_into)
		.flat_ok()
		.unwrap_or(services.config.client_sync_timeout_default)
		.max(services.config.client_sync_timeout_min)
		.min(services.config.client_sync_timeout_max);

	let stop_at = Instant::now()
		.checked_add(Duration::from_millis(timeout))
		.expect("configuration must limit maximum timeout");

	let conn_key = into_connection_key(sender_user, sender_device, request.conn_id.as_deref());
	let conn_val = since
		.ne(&0)
		.then(|| services.sync.find_connection(&conn_key))
		.unwrap_or_else(|| Ok(services.sync.init_connection(&conn_key)))
		.map_err(|_| err!(Request(UnknownPos("Connection lost; restarting sync stream."))))?;

	let conn = conn_val.lock();
	let ping_presence = services
		.presence
		.maybe_ping_presence(sender_user, &request.set_presence)
		.inspect_err(inspect_log)
		.ok();

	let (mut conn, _) = join(conn, ping_presence).await;

	// The client must either use the last returned next_batch or replay the
	// next_batch from the penultimate request: it's either up-to-date or
	// one-behind. If we receive anything else we can boot them.
	let advancing = since == conn.next_batch;
	let replaying = since == conn.globalsince;
	if !advancing && !replaying {
		return Err!(Request(UnknownPos("Requesting unknown or stale stream position.")));
	}

	debug_assert!(
		advancing || replaying,
		"Request should either be advancing or replaying the last request."
	);

	// Update parameters regardless of replay or advance
	conn.update_cache(request);
	conn.update_rooms_prologue(advancing);
	conn.globalsince = since;
	conn.next_batch = services.globals.current_count();

	let sync_info = SyncInfo {
		services,
		sender_user,
		sender_device,
		request,
	};

	let mut response = Response {
		txn_id: request.txn_id.clone(),
		lists: Default::default(),
		pos: Default::default(),
		rooms: Default::default(),
		extensions: Default::default(),
	};

	loop {
		debug_assert!(
			conn.globalsince <= conn.next_batch,
			"next_batch should not be greater than since."
		);

		let window;
		(window, response.lists) = selector(&mut conn, sync_info).boxed().await;

		let watch_rooms = window.keys().map(AsRef::as_ref).stream();
		let watchers = services
			.sync
			.watch(sender_user, sender_device, watch_rooms);

		conn.next_batch = services.globals.wait_pending().await?;
		if conn.globalsince < conn.next_batch {
			let rooms =
				handle_rooms(sync_info, &conn, &window).map_ok(|rooms| response.rooms = rooms);

			let extensions = handle_extensions(sync_info, &conn, &window)
				.map_ok(|extensions| response.extensions = extensions);

			try_join(rooms, extensions).boxed().await?;

			conn.update_rooms_epilogue(window.keys().map(AsRef::as_ref));

			if !is_empty_response(&response) {
				response.pos = conn.next_batch.to_string().into();
				trace!(conn.globalsince, conn.next_batch, "response {response:?}");
				return Ok(response);
			}
		}

		if timeout_at(stop_at, watchers).await.is_err() || services.server.is_stopping() {
			response.pos = conn.next_batch.to_string().into();
			trace!(conn.globalsince, conn.next_batch, "timeout; empty response");
			return Ok(response);
		}

		trace!(
			conn.globalsince,
			last_batch = ?conn.next_batch,
			count = ?services.globals.pending_count(),
			stop_at = ?stop_at,
			"notified by watcher"
		);

		conn.globalsince = conn.next_batch;
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
	name = "rooms",
	level = "debug",
	skip_all,
	fields(
		next_batch = conn.next_batch,
		window = window.len(),
	)
)]
async fn handle_rooms(
	sync_info: SyncInfo<'_>,
	conn: &Connection,
	window: &Window,
) -> Result<BTreeMap<OwnedRoomId, response::Room>> {
	window
		.iter()
		.try_stream()
		.broad_and_then(async |(room_id, room)| {
			room::handle(sync_info, conn, room)
				.map_ok(|room| (room_id, room))
				.await
		})
		.ready_try_filter_map(|(room_id, room)| Ok(room.map(|room| (room_id, room))))
		.map_ok(|(room_id, room)| (room_id.to_owned(), room))
		.try_collect()
		.await
}

#[tracing::instrument(
	name = "extensions",
	level = "debug",
	skip_all,
	fields(
		next_batch = conn.next_batch,
		window = window.len(),
		rooms = conn.rooms.len(),
		subs = conn.subscriptions.len(),
	)
)]
async fn handle_extensions(
	sync_info: SyncInfo<'_>,
	conn: &Connection,
	window: &Window,
) -> Result<response::Extensions> {
	let SyncInfo { .. } = sync_info;

	let account_data: OptionFuture<_> = conn
		.extensions
		.account_data
		.enabled
		.unwrap_or(false)
		.then(|| account_data::collect(sync_info, conn, window))
		.into();

	let receipts: OptionFuture<_> = conn
		.extensions
		.receipts
		.enabled
		.unwrap_or(false)
		.then(|| receipts::collect(sync_info, conn, window))
		.into();

	let typing: OptionFuture<_> = conn
		.extensions
		.typing
		.enabled
		.unwrap_or(false)
		.then(|| typing::collect(sync_info, conn, window))
		.into();

	let to_device: OptionFuture<_> = conn
		.extensions
		.to_device
		.enabled
		.unwrap_or(false)
		.then(|| to_device::collect(sync_info, conn))
		.into();

	let e2ee: OptionFuture<_> = conn
		.extensions
		.e2ee
		.enabled
		.unwrap_or(false)
		.then(|| e2ee::collect(sync_info, conn))
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

#[tracing::instrument(
	name = "selector",
	level = "trace",
	skip_all,
	fields(?implicit, ?explicit),
)]
fn extension_rooms_selector<'a, ListIter, SubsIter>(
	SyncInfo { .. }: SyncInfo<'a>,
	conn: &'a Connection,
	window: &'a Window,
	implicit: Option<ListIter>,
	explicit: Option<SubsIter>,
) -> impl Iterator<Item = &'a RoomId> + Send + Sync + 'a
where
	ListIter: Iterator<Item = &'a ListId> + Clone + Debug + Send + Sync + 'a,
	SubsIter: Iterator<Item = &'a ExtensionRoomConfig> + Clone + Debug + Send + Sync + 'a,
{
	let has_all_subscribed = explicit
		.clone()
		.into_iter()
		.flatten()
		.any(|erc| matches!(erc, ExtensionRoomConfig::AllSubscribed));

	let all_subscribed = has_all_subscribed
		.then(|| conn.subscriptions.keys())
		.into_iter()
		.flatten()
		.map(AsRef::as_ref);

	let rooms_explicit = has_all_subscribed
		.is_false()
		.then(move || {
			explicit
				.into_iter()
				.flatten()
				.filter_map(|erc| extract_variant!(erc, ExtensionRoomConfig::Room))
				.map(AsRef::as_ref)
		})
		.into_iter()
		.flatten();

	let rooms_selected = window
		.iter()
		.filter(move |(_, room)| {
			implicit.as_ref().is_none_or(|lists| {
				lists
					.clone()
					.any(|list| room.lists.contains(list))
			})
		})
		.map(at!(0))
		.map(AsRef::as_ref);

	all_subscribed
		.chain(rooms_explicit)
		.chain(rooms_selected)
}
