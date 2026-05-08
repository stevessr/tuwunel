use std::sync::Arc;

use futures::{Stream, StreamExt};
use ruma::{
	CanonicalJsonObject, RoomId, UserId,
	events::{AnySyncEphemeralRoomEvent, receipt::ReceiptEvent},
	serde::Raw,
};
use tuwunel_core::{
	Result, err, is_equal_to, trace,
	utils::{ReadyExt, stream::TryIgnore},
};
use tuwunel_database::{Deserialized, Interfix, Json, Map, serialize_key};

pub(super) struct Data {
	roomuserid_privateread: Arc<Map>,
	roomuserid_lastprivatereadupdate: Arc<Map>,
	services: Arc<crate::services::OnceServices>,
	readreceiptid_readreceipt: Arc<Map>,
}

pub(super) type ReceiptItem<'a> = (&'a UserId, u64, Raw<AnySyncEphemeralRoomEvent>);

/// Tag string used in the storage key to discriminate receipts per thread.
/// Empty for `Unthreaded`, `"main"` for `Main`, the event-id string for
/// `Thread(...)` (event ids start with `$`, so the values are mutually
/// exclusive). Custom variants reuse their string form. Falls back to
/// unthreaded for empty or multi-entry events.
fn event_thread_kind(event: &ReceiptEvent) -> &str {
	event
		.content
		.0
		.values()
		.next()
		.and_then(|by_type| by_type.values().next())
		.and_then(|by_user| by_user.values().next())
		.and_then(|receipt| receipt.thread.as_str())
		.unwrap_or_default()
}

impl Data {
	pub(super) fn new(args: &crate::Args<'_>) -> Self {
		let db = &args.db;
		Self {
			roomuserid_privateread: db["roomuserid_privateread"].clone(),
			roomuserid_lastprivatereadupdate: db["roomuserid_lastprivatereadupdate"].clone(),
			readreceiptid_readreceipt: db["readreceiptid_readreceipt"].clone(),
			services: args.services.clone(),
		}
	}

	#[inline]
	pub(super) async fn readreceipt_update(
		&self,
		user_id: &UserId,
		room_id: &RoomId,
		event: &ReceiptEvent,
	) {
		let thread_kind = event_thread_kind(event);
		// MSC3771: storage key suffix is `user_id || 0xFF || thread_kind` so
		// each (user, thread-context) tuple lives in its own row. Pre-MSC3771
		// rows have no kind tail; on an Unthreaded sweep also match the
		// bare-user-id ending so legacy rows are superseded rather than
		// orphaned. Kind tails ("main", `$root`) never end in `@user:host`,
		// so the legacy match cannot collide with thread-aware rows.
		let suffix = serialize_key((user_id, thread_kind))
			.expect("failed to serialize receipt key suffix");

		let user_id_bytes = user_id.as_bytes();
		let legacy_match = thread_kind.is_empty();

		let last_possible_key = (room_id, u64::MAX);
		self.readreceiptid_readreceipt
			.rev_keys_from_raw(&last_possible_key)
			.ignore_err()
			.ready_take_while(|key| key.starts_with(room_id.as_bytes()))
			.ready_filter_map(|key| {
				(key.ends_with(suffix.as_slice())
					|| (legacy_match && key.ends_with(user_id_bytes)))
				.then_some(key)
			})
			.ready_for_each(|key| self.readreceiptid_readreceipt.del(key))
			.await;

		let count = self.services.globals.next_count();
		let latest_id = (room_id, *count, user_id, thread_kind);
		self.readreceiptid_readreceipt
			.put(latest_id, Json(event));
	}

	#[inline]
	pub(super) fn readreceipts_since<'a>(
		&'a self,
		room_id: &'a RoomId,
		since: u64,
		to: Option<u64>,
	) -> impl Stream<Item = ReceiptItem<'_>> + Send + 'a {
		type Key<'a> = (&'a RoomId, u64, &'a UserId, &'a str);
		type KeyVal<'a> = (Key<'a>, CanonicalJsonObject);

		let after_since = since.saturating_add(1); // +1 so we don't send the event at since
		let first_possible_edu = (room_id, after_since);

		self.readreceiptid_readreceipt
			.stream_from(&first_possible_edu)
			.ignore_err()
			.ready_take_while(move |((r, c, ..), _): &KeyVal<'_>| {
				*r == room_id && to.is_none_or(|to| *c <= to)
			})
			.map(move |((_, count, user_id, _), mut json): KeyVal<'_>| {
				json.remove("room_id");

				let event = serde_json::value::to_raw_value(&json)?;

				Ok((user_id, count, Raw::from_json(event)))
			})
			.ignore_err()
	}

	#[inline]
	pub(super) async fn last_receipt_count<'a>(
		&'a self,
		room_id: &'a RoomId,
		since: Option<u64>,
		user_id: Option<&'a UserId>,
	) -> Result<u64> {
		type Key<'a> = (&'a RoomId, u64, &'a UserId, &'a str);

		let key = (room_id, u64::MAX);
		self.readreceiptid_readreceipt
			.rev_keys_prefix(&key)
			.ignore_err()
			.ready_take_while(|(_, c, u, _): &Key<'_>| {
				since.is_none_or(|since| since > *c) && user_id.is_none_or(is_equal_to!(*u))
			})
			.map(|(_, c, ..): Key<'_>| c)
			.boxed()
			.next()
			.await
			.ok_or_else(|| err!(Request(NotFound("No receipts found in room"))))
	}

	#[inline]
	pub(super) fn private_read_set(&self, room_id: &RoomId, user_id: &UserId, pdu_count: u64) {
		let key = (room_id, user_id);
		let next_count = self.services.globals.next_count();

		self.roomuserid_privateread.put(key, pdu_count);
		self.roomuserid_lastprivatereadupdate
			.put(key, *next_count);
	}

	#[inline]
	pub(super) async fn private_read_get_count(
		&self,
		room_id: &RoomId,
		user_id: &UserId,
	) -> Result<u64> {
		let key = (room_id, user_id);
		self.roomuserid_privateread
			.qry(&key)
			.await
			.deserialized()
	}

	#[inline]
	pub(super) async fn last_privateread_update(
		&self,
		user_id: &UserId,
		room_id: &RoomId,
	) -> u64 {
		let key = (room_id, user_id);
		self.roomuserid_lastprivatereadupdate
			.qry(&key)
			.await
			.deserialized()
			.unwrap_or(0)
	}

	#[inline]
	pub(super) async fn delete_all_read_receipts(&self, room_id: &RoomId) -> Result {
		let prefix = (room_id, Interfix);

		self.roomuserid_privateread
			.keys_prefix_raw(&prefix)
			.ignore_err()
			.ready_for_each(|key| {
				trace!("Removing key: {key:?}");
				self.roomuserid_privateread.remove(key);
			})
			.await;

		self.roomuserid_lastprivatereadupdate
			.keys_prefix_raw(&prefix)
			.ignore_err()
			.ready_for_each(|key| {
				trace!("Removing key: {key:?}");
				self.roomuserid_lastprivatereadupdate.remove(key);
			})
			.await;

		self.readreceiptid_readreceipt
			.keys_prefix_raw(&prefix)
			.ignore_err()
			.ready_for_each(|key| {
				trace!("Removing key: {key:?}");
				self.readreceiptid_readreceipt.remove(key);
			})
			.await;

		Ok(())
	}
}
