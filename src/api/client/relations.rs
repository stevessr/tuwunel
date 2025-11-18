use axum::extract::State;
use futures::{FutureExt, Stream, StreamExt, TryFutureExt, future::try_join3};
use ruma::{
	EventId, RoomId, UInt, UserId,
	api::{
		Direction,
		client::relations::{
			get_relating_events, get_relating_events_with_rel_type,
			get_relating_events_with_rel_type_and_event_type,
		},
	},
	events::{TimelineEventType, relation::RelationType},
};
use tuwunel_core::{
	Err, Error, Result, at, err,
	matrix::{
		ShortRoomId,
		event::{Event, RelationTypeEqual},
		pdu::{Pdu, PduCount, PduId},
	},
	utils::{
		BoolExt,
		result::FlatOk,
		stream::{IterStream, ReadyExt, WidebandExt},
	},
};
use tuwunel_service::Services;

use crate::Ruma;

/// # `GET /_matrix/client/r0/rooms/{roomId}/relations/{eventId}/{relType}/{eventType}`
pub(crate) async fn get_relating_events_with_rel_type_and_event_type_route(
	State(services): State<crate::State>,
	body: Ruma<get_relating_events_with_rel_type_and_event_type::v1::Request>,
) -> Result<get_relating_events_with_rel_type_and_event_type::v1::Response> {
	paginate_relations_with_filter(
		&services,
		body.sender_user(),
		&body.room_id,
		&body.event_id,
		body.event_type.clone().into(),
		body.rel_type.clone().into(),
		body.from.as_deref(),
		body.to.as_deref(),
		body.limit,
		body.recurse,
		body.dir,
	)
	.await
	.map(|res| get_relating_events_with_rel_type_and_event_type::v1::Response {
		chunk: res.chunk,
		next_batch: res.next_batch,
		prev_batch: res.prev_batch,
		recursion_depth: res.recursion_depth,
	})
}

/// # `GET /_matrix/client/r0/rooms/{roomId}/relations/{eventId}/{relType}`
pub(crate) async fn get_relating_events_with_rel_type_route(
	State(services): State<crate::State>,
	body: Ruma<get_relating_events_with_rel_type::v1::Request>,
) -> Result<get_relating_events_with_rel_type::v1::Response> {
	paginate_relations_with_filter(
		&services,
		body.sender_user(),
		&body.room_id,
		&body.event_id,
		None,
		body.rel_type.clone().into(),
		body.from.as_deref(),
		body.to.as_deref(),
		body.limit,
		body.recurse,
		body.dir,
	)
	.await
	.map(|res| get_relating_events_with_rel_type::v1::Response {
		chunk: res.chunk,
		next_batch: res.next_batch,
		prev_batch: res.prev_batch,
		recursion_depth: res.recursion_depth,
	})
}

/// # `GET /_matrix/client/r0/rooms/{roomId}/relations/{eventId}`
pub(crate) async fn get_relating_events_route(
	State(services): State<crate::State>,
	body: Ruma<get_relating_events::v1::Request>,
) -> Result<get_relating_events::v1::Response> {
	paginate_relations_with_filter(
		&services,
		body.sender_user(),
		&body.room_id,
		&body.event_id,
		None,
		None,
		body.from.as_deref(),
		body.to.as_deref(),
		body.limit,
		body.recurse,
		body.dir,
	)
	.await
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(
	name = "relations",
	level = "debug",
	skip_all,
	fields(room_id, target, from, to, dir, limit, recurse),
	ret(level = "trace")
)]
async fn paginate_relations_with_filter(
	services: &Services,
	sender_user: &UserId,
	room_id: &RoomId,
	target: &EventId,
	filter_event_type: Option<TimelineEventType>,
	filter_rel_type: Option<RelationType>,
	from: Option<&str>,
	to: Option<&str>,
	limit: Option<UInt>,
	recurse: bool,
	dir: Direction,
) -> Result<get_relating_events::v1::Response> {
	let from: Option<PduCount> = from.map(str::parse).transpose()?;

	let to: Option<PduCount> = to.map(str::parse).flat_ok();

	// Spec (v1.10) recommends depth of at least 3
	let depth: u8 = if recurse { 3 } else { 1 };

	let limit: usize = limit
		.map(TryInto::try_into)
		.flat_ok()
		.unwrap_or(30)
		.min(100);

	let shortroomid = services.short.get_shortroomid(room_id);

	let target = services
		.timeline
		.get_pdu_id(target)
		.map_ok(PduId::from)
		.map_ok(Ok::<_, Error>);

	let visible = services
		.state_accessor
		.user_can_see_state_events(sender_user, room_id)
		.map(|visible| {
			visible.ok_or_else(|| err!(Request(Forbidden("You cannot view this room."))))
		});

	let (shortroomid, target, ()) = try_join3(shortroomid, target, visible).await?;

	let Ok(target) = target else {
		return Ok(get_relating_events::v1::Response::new(Vec::new()));
	};

	if shortroomid != target.shortroomid {
		return Err!(Request(NotFound("Event not found in room.")));
	}

	//TODO: support backfilled relations
	if let PduCount::Backfilled(_) = target.count {
		return Ok(get_relating_events::v1::Response::new(Vec::new()));
	}

	let events: Vec<_> = get_relations(
		services,
		sender_user,
		target.shortroomid,
		target.count,
		from,
		limit,
		depth,
		dir,
	)
	.await //TODO: XXX
	.ready_take_while(|(count, _)| Some(*count) != to)
	.ready_filter(|(_, pdu)| {
		filter_event_type
			.as_ref()
			.is_none_or(|kind| kind == pdu.kind())
	})
	.ready_filter(|(_, pdu)| {
		filter_rel_type
			.as_ref()
			.is_none_or(|rel_type| rel_type.relation_type_equal(pdu))
	})
	.wide_filter_map(|item| visibility_filter(services, sender_user, item))
	.take(limit)
	.collect()
	.await;

	Ok(get_relating_events::v1::Response {
		recursion_depth: recurse.then_some(depth.into()),

		next_batch: events
			.last()
			.map(at!(0))
			.as_ref()
			.map(ToString::to_string),

		prev_batch: events
			.first()
			.map(at!(0))
			.or(from)
			.as_ref()
			.map(ToString::to_string),

		chunk: events
			.into_iter()
			.map(at!(1))
			.map(Event::into_format)
			.collect(),
	})
}

#[allow(clippy::too_many_arguments)]
async fn get_relations(
	services: &Services,
	sender_user: &UserId,
	shortroomid: ShortRoomId,
	target: PduCount,
	from: Option<PduCount>,
	limit: usize,
	max_depth: u8,
	dir: Direction,
) -> impl Stream<Item = (PduCount, Pdu)> + Send {
	let mut pdus: Vec<_> = services
		.pdu_metadata
		.get_relations(shortroomid, target, from, dir, Some(sender_user))
		.take(limit)
		.collect()
		.await;

	let mut stack: Vec<_> = pdus
		.iter()
		.filter(|_| max_depth > 0)
		.map(|(count, _)| (*count, 1))
		.collect();

	'limit: while let Some((target, depth)) = stack.pop() {
		let PduCount::Normal(target) = target else {
			continue;
		};

		let relations: Vec<_> = services
			.pdu_metadata
			.get_relations(shortroomid, target.into(), from, dir, Some(sender_user))
			.take(limit.saturating_sub(pdus.len()))
			.collect()
			.await;

		for (target, pdu) in relations {
			if depth < max_depth {
				stack.push((target, depth.saturating_add(1)));
			}

			if pdus.len() < limit {
				pdus.push((target, pdu));
			} else {
				break 'limit;
			}
		}
	}

	pdus.into_iter().stream()
}

async fn visibility_filter<Pdu: Event>(
	services: &Services,
	sender_user: &UserId,
	item: (PduCount, Pdu),
) -> Option<(PduCount, Pdu)> {
	let (_, pdu) = &item;

	services
		.state_accessor
		.user_can_see_event(sender_user, pdu.room_id(), pdu.event_id())
		.await
		.then_some(item)
}
