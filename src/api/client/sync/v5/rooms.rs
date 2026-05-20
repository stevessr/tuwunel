mod bump_stamp;
mod heroes;

use std::collections::{BTreeMap, HashSet};

use futures::{
	FutureExt, StreamExt, TryFutureExt,
	future::{join, join3, join4},
};
use ruma::{
	JsOption, OwnedRoomId, RoomId, UserId,
	api::client::sync::sync_events::{UnreadNotificationsCount, v5::response},
	events::{
		AnySyncStateEvent, StateEventType, TimelineEventType, room::member::MembershipState,
	},
	serde::Raw,
};
use tuwunel_core::{
	Result, at, err, error, is_equal_to,
	itertools::Itertools,
	matrix::{
		Event, StateKey,
		pdu::{PduCount, PduEvent},
	},
	ref_at,
	utils::{
		BoolExt, IterStream, ReadyExt, TryFutureExtExt, math::usize_from_ruma, result::FlatOk,
		stream::BroadbandExt,
	},
};
use tuwunel_service::{Services, sync::Room};

use self::{bump_stamp::room_bump_stamp, heroes::calculate_heroes};
use super::{super::load_timeline, Connection, SyncInfo, Window, WindowRoom};
use crate::client::{annotate_membership, ignored_filter, with_membership};

#[tracing::instrument(
    name = "rooms",
    level = "debug",
    skip_all,
    fields(
        next_batch = conn.next_batch,
        window = window.len(),
    )
)]
pub(super) async fn handle(
	sync_info: SyncInfo<'_>,
	conn: &Connection,
	window: &Window,
) -> Result<BTreeMap<OwnedRoomId, response::Room>> {
	window
		.iter()
		.stream()
		.broad_filter_map(async |(room_id, room)| {
			handle_room(sync_info, conn, room)
				.map_ok(move |room| (room_id.clone(), room))
				.inspect_err(|e| error!(?room_id, "sync handler: {e:?}"))
				.await
				.ok()
		})
		.collect()
		.map(Ok)
		.await
}

#[tracing::instrument(
	name = "room",
	level = "debug",
	skip_all,
	fields(room_id, roomsince)
)]
async fn handle_room(
	SyncInfo { services, sender_user, .. }: SyncInfo<'_>,
	conn: &Connection,
	WindowRoom {
		lists, membership, room_id, last_count, ..
	}: &WindowRoom,
) -> Result<response::Room> {
	let &Room { roomsince, .. } = conn
		.rooms
		.get(room_id)
		.ok_or_else(|| err!("Missing connection state for {room_id}"))?;

	debug_assert!(
		*last_count > roomsince || *last_count == 0 || roomsince == 0,
		"Stale room shouldn't be in the window"
	);

	if matches!(*membership, Some(MembershipState::Leave | MembershipState::Ban)) {
		return Ok(response::Room {
			initial: roomsince.eq(&0).then_some(true),
			lists: lists.clone(),
			membership: membership.clone(),
			prev_batch: Some(conn.next_batch.to_string().into()),
			limited: true,
			required_state: vec![
				services
					.state_accessor
					.room_state_get(room_id, &StateEventType::RoomMember, sender_user.as_str())
					.map_ok(Event::into_format)
					.await?,
			],

			..Default::default()
		});
	}

	let is_invite = *membership == Some(MembershipState::Invite);

	let encrypted = services
		.state_accessor
		.is_encrypted_room(room_id)
		.await;

	let default_details = (0_usize, HashSet::new());
	let (timeline_limit, required_state) = lists
		.iter()
		.filter_map(|list_id| conn.lists.get(list_id))
		.map(|list| &list.room_details)
		.chain(conn.subscriptions.get(room_id).into_iter())
		.fold(default_details, |(mut timeline_limit, mut required_state), config| {
			let limit = usize_from_ruma(config.timeline_limit);

			timeline_limit = timeline_limit.max(limit);
			required_state.extend(config.required_state.clone());

			(timeline_limit, required_state)
		});

	let timeline = is_invite.is_false().then_async(|| {
		load_timeline(
			services,
			sender_user,
			room_id,
			PduCount::Normal(roomsince),
			Some(PduCount::from(conn.next_batch)),
			timeline_limit,
		)
	});

	let (timeline_pdus, limited, last_timeline_count) = timeline
		.await
		.flat_ok()
		.unwrap_or_else(|| (Vec::new(), true, PduCount::default()));

	let required_state = required_state
		.into_iter()
		.filter(|_| !timeline_pdus.is_empty())
		.collect::<Vec<_>>();

	let prev_batch = timeline_pdus
		.first()
		.map(at!(0))
		.map(PduCount::into_unsigned)
		.as_ref()
		.map(ToString::to_string);

	let bump_stamp = room_bump_stamp(
		services,
		sender_user,
		room_id,
		PduCount::Normal(roomsince),
		PduCount::from(conn.next_batch),
		last_timeline_count,
	)
	.await;

	let num_live = roomsince
		.ne(&0)
		.and_is(limited || timeline_pdus.len() >= timeline_limit)
		.then_async(|| {
			services
				.timeline
				.pdus(None, room_id, Some(roomsince.into()))
				.count()
				.map(TryInto::try_into)
				.map(Result::ok)
		});

	let required_state = collect_required_state(
		services,
		sender_user,
		room_id,
		&required_state,
		&timeline_pdus,
		encrypted,
	);

	// TODO: figure out a timestamp we can use for remote invites
	let invite_state = is_invite.then_async(|| {
		services
			.state_cache
			.invite_state(sender_user, room_id)
			.ok()
	});

	let room_name = services
		.state_accessor
		.get_name(room_id)
		.map_ok(Into::into)
		.map(Result::ok);

	let room_avatar = services
		.state_accessor
		.get_avatar(room_id)
		.map_ok(|content| content.url)
		.ok()
		.map(Option::flatten);

	let highlight_count = services
		.pusher
		.highlight_count(sender_user, room_id)
		.map(TryInto::try_into)
		.map(Result::ok);

	let notification_count = services
		.pusher
		.notification_count(sender_user, room_id)
		.map(TryInto::try_into)
		.map(Result::ok);

	let joined_count = services
		.state_cache
		.room_joined_count(room_id)
		.map_ok(TryInto::try_into)
		.map_ok(Result::ok)
		.map(FlatOk::flat_ok);

	let invited_count = services
		.state_cache
		.room_invited_count(room_id)
		.map_ok(TryInto::try_into)
		.map_ok(Result::ok)
		.map(FlatOk::flat_ok);

	let is_dm = services
		.state_accessor
		.is_direct(room_id, sender_user)
		.map(|is_dm| is_dm.then_some(is_dm));

	let last_read_count = services
		.pusher
		.last_notification_read(sender_user, room_id);

	let timeline = timeline_pdus
		.iter()
		.stream()
		.filter_map(|item| ignored_filter(services, item.clone(), sender_user))
		.map(at!(1))
		.broad_then(|pdu| with_membership(services, pdu, sender_user, encrypted))
		.map(Event::into_format)
		.collect();

	let meta = join3(room_name, room_avatar, is_dm);
	let events = join4(timeline, num_live, required_state, invite_state);
	let member_counts = join(joined_count, invited_count);
	let notification_counts = join3(highlight_count, notification_count, last_read_count);
	let (
		(room_name, room_avatar, is_dm),
		(timeline, num_live, required_state, invite_state),
		(joined_count, invited_count),
		(highlight_count, notification_count, _last_notification_read),
	) = join4(meta, events, member_counts, notification_counts)
		.boxed()
		.await;

	let heroes = services
		.config
		.calculate_heroes
		.then_async(|| {
			calculate_heroes(
				services,
				sender_user,
				room_id,
				room_name.as_ref(),
				room_avatar.as_deref(),
			)
		})
		.await
		.unwrap_or_default();

	let (heroes, heroes_name, heroes_avatar) = heroes;

	Ok(response::Room {
		initial: roomsince.eq(&0).then_some(true),
		lists: lists.clone(),
		membership: membership.clone(),
		name: room_name.or(heroes_name),
		avatar: JsOption::from_option(room_avatar.or(heroes_avatar)),
		is_dm,
		heroes,
		required_state,
		invite_state: invite_state.flatten(),
		prev_batch: prev_batch.as_deref().map(Into::into),
		num_live: num_live.flatten(),
		limited,
		timeline,
		bump_stamp,
		joined_count,
		invited_count,
		unread_notifications: UnreadNotificationsCount { highlight_count, notification_count },
	})
}

async fn collect_required_state(
	services: &Services,
	sender_user: &UserId,
	room_id: &RoomId,
	required_state: &[(StateEventType, StateKey)],
	timeline_pdus: &[(PduCount, PduEvent)],
	encrypted: bool,
) -> Vec<Raw<AnySyncStateEvent>> {
	let lazy = required_state
		.iter()
		.any(is_equal_to!(&(StateEventType::RoomMember, "$LAZY".into())));

	let timeline_senders = timeline_pdus
		.iter()
		.filter(|_| lazy)
		.map(ref_at!(1))
		.map(Event::sender)
		.map(UserId::as_str);

	let timeline_member_targets = timeline_pdus
		.iter()
		.filter(|_| lazy)
		.map(ref_at!(1))
		.filter(|event| *event.event_type() == TimelineEventType::RoomMember)
		.filter_map(Event::state_key);

	let timeline_senders = timeline_senders
		.chain(timeline_member_targets)
		.sorted_unstable()
		.dedup()
		.map(|sender| (StateEventType::RoomMember, StateKey::from_str(sender)))
		.collect::<Vec<_>>();

	let wildcard_types: Vec<StateEventType> = required_state
		.iter()
		.filter(|(_, state_key)| state_key == "*")
		.map(|(event_type, _)| event_type.clone())
		.collect();

	// Sequential await: per-event-type stream → Vec resolution. Stream form
	// (.then + flatten) triggers an HRTB mismatch downstream against join4.
	let mut wildcard_state: Vec<(StateEventType, StateKey)> = Vec::new();
	for event_type in wildcard_types {
		wildcard_state.extend(wildcard_state_keys(services, room_id, event_type).await);
	}

	required_state
		.iter()
		.cloned()
		.stream()
		.chain(wildcard_state.into_iter().stream())
		.chain(timeline_senders.into_iter().stream())
		.broad_filter_map(async |state| {
			let state_key: StateKey = match state.1.as_str() {
				| "$LAZY" | "*" => return None,
				| "$ME" => sender_user.as_str().into(),
				| _ => state.1.clone(),
			};

			let mut pdu = services
				.state_accessor
				.room_state_get(room_id, &state.0, &state_key)
				.map_ok(Event::into_pdu)
				.ok()
				.await?;

			annotate_membership(services, &mut pdu, sender_user, encrypted).await;

			Some(Event::into_format(pdu))
		})
		.collect()
		.await
}

async fn wildcard_state_keys(
	services: &Services,
	room_id: &RoomId,
	event_type: StateEventType,
) -> Vec<(StateEventType, StateKey)> {
	services
		.state_accessor
		.room_state_keys(room_id, &event_type)
		.ready_filter_map(Result::ok)
		.map(|state_key| (event_type.clone(), state_key))
		.collect()
		.await
}
