use std::collections::{HashMap, hash_map};

use futures::FutureExt;
use ruma::{
	EventId, OwnedEventId, RoomId, RoomVersionId, ServerName,
	api::federation::event::get_room_state_ids, events::StateEventType,
};
use tuwunel_core::{Err, Result, debug, debug_warn, err, implement, matrix::Event};

use crate::rooms::short::ShortStateKey;

/// Call /state_ids to find out what the state at this pdu is. We trust the
/// server's response to some extend (sic), but we still do a lot of checks
/// on the events
#[implement(super::Service)]
#[tracing::instrument(
	level = "debug",
	skip_all,
	fields(%origin),
)]
pub(super) async fn fetch_state(
	&self,
	origin: &ServerName,
	room_id: &RoomId,
	event_id: &EventId,
	room_version: &RoomVersionId,
	create_event_id: &EventId,
) -> Result<Option<HashMap<u64, OwnedEventId>>> {
	let res = self
		.services
		.sending
		.send_federation_request(origin, get_room_state_ids::v1::Request {
			room_id: room_id.to_owned(),
			event_id: event_id.to_owned(),
		})
		.await
		.inspect_err(|e| debug_warn!("Fetching state for event failed: {e}"))?;

	debug!("Fetching state events");
	let state_ids = res.pdu_ids.iter().map(AsRef::as_ref);
	let state_vec = self
		.fetch_auth(origin, room_id, state_ids, room_version)
		.boxed()
		.await;

	let mut state: HashMap<ShortStateKey, OwnedEventId> = HashMap::with_capacity(state_vec.len());
	for (pdu, _) in state_vec {
		let state_key = pdu
			.state_key()
			.ok_or_else(|| err!(Database("Found non-state pdu in state events.")))?;

		let shortstatekey = self
			.services
			.short
			.get_or_create_shortstatekey(&pdu.kind().to_string().into(), state_key)
			.await;

		match state.entry(shortstatekey) {
			| hash_map::Entry::Vacant(v) => {
				v.insert(pdu.event_id().to_owned());
			},
			| hash_map::Entry::Occupied(_) => {
				return Err!(Request(InvalidParam(
					"State event's type and state_key ({:?},{:?}) exists multiple times.",
					pdu.event_type(),
					pdu.state_key()
						.expect("all state events have state_key"),
				)));
			},
		}
	}

	// The original create event must still be in the state
	let create_shortstatekey = self
		.services
		.short
		.get_shortstatekey(&StateEventType::RoomCreate, "")
		.await?;

	if state
		.get(&create_shortstatekey)
		.map(AsRef::as_ref)
		!= Some(create_event_id)
	{
		return Err!(Database("Incoming event refers to wrong create event."));
	}

	Ok(Some(state))
}
