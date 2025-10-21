use std::{cmp::Ordering, collections::HashSet};

use futures::{
	FutureExt, StreamExt, TryFutureExt, TryStreamExt,
	future::{OptionFuture, join, join3, join4},
};
use ruma::{
	JsOption, MxcUri, OwnedMxcUri, RoomId, UInt, UserId,
	api::client::sync::sync_events::{
		UnreadNotificationsCount,
		v5::{DisplayName, response},
	},
	events::{
		StateEventType,
		TimelineEventType::{
			self, Beacon, CallInvite, PollStart, RoomEncrypted, RoomMessage, Sticker,
		},
		room::member::MembershipState,
	},
};
use tuwunel_core::{
	Result, at, debug_error, err, is_equal_to,
	matrix::{Event, StateKey, pdu::PduCount},
	ref_at,
	utils::{
		BoolExt, IterStream, ReadyExt, TryFutureExtExt, math::usize_from_ruma, result::FlatOk,
		stream::BroadbandExt,
	},
};
use tuwunel_service::{Services, sync::Room};

use super::{super::load_timeline, Connection, SyncInfo, WindowRoom};
use crate::client::ignored_filter;

static DEFAULT_BUMP_TYPES: [TimelineEventType; 6] =
	[CallInvite, PollStart, Beacon, RoomEncrypted, RoomMessage, Sticker];

#[tracing::instrument(
	name = "room",
	level = "debug",
	skip_all,
	fields(room_id, roomsince)
)]
#[allow(clippy::too_many_arguments)]
pub(super) async fn handle(
	SyncInfo { services, sender_user, .. }: SyncInfo<'_>,
	conn: &Connection,
	WindowRoom { lists, membership, room_id, .. }: &WindowRoom,
) -> Result<Option<response::Room>> {
	debug_assert!(DEFAULT_BUMP_TYPES.is_sorted(), "DEFAULT_BUMP_TYPES is not sorted");

	let &Room { roomsince, .. } = conn
		.rooms
		.get(room_id)
		.ok_or_else(|| err!("Missing connection state for {room_id}"))?;

	let is_invite = *membership == Some(MembershipState::Invite);
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

	let timeline: OptionFuture<_> = is_invite
		.is_false()
		.then(|| {
			load_timeline(
				services,
				sender_user,
				room_id,
				PduCount::Normal(roomsince),
				Some(PduCount::from(conn.next_batch)),
				timeline_limit,
			)
		})
		.into();

	let Ok(timeline) = timeline.await.transpose() else {
		debug_error!(?room_id, "Missing timeline.");
		return Ok(None);
	};

	let (timeline_pdus, limited, _lastcount) =
		timeline.unwrap_or_else(|| (Vec::new(), true, PduCount::default()));

	if roomsince != 0 && timeline_pdus.is_empty() && !is_invite {
		return Ok(None);
	}

	let prev_batch = timeline_pdus
		.first()
		.map(at!(0))
		.map(PduCount::into_unsigned)
		.or_else(|| roomsince.ne(&0).then_some(roomsince))
		.as_ref()
		.map(ToString::to_string);

	let bump_stamp = timeline_pdus
		.iter()
		.filter(|(_, pdu)| {
			DEFAULT_BUMP_TYPES
				.binary_search(pdu.event_type())
				.is_ok()
		})
		.fold(Option::<UInt>::None, |mut bump_stamp, (_, pdu)| {
			let ts = pdu.origin_server_ts().0;
			if bump_stamp.is_none_or(|bump_stamp| bump_stamp < ts) {
				bump_stamp.replace(ts);
			}

			bump_stamp
		});

	let lazy = required_state
		.iter()
		.any(is_equal_to!(&(StateEventType::RoomMember, "$LAZY".into())));

	let mut timeline_senders: Vec<_> = timeline_pdus
		.iter()
		.filter(|_| lazy)
		.map(ref_at!(1))
		.map(Event::sender)
		.collect();

	timeline_senders.sort();
	timeline_senders.dedup();
	let timeline_senders = timeline_senders
		.iter()
		.map(|sender| (StateEventType::RoomMember, StateKey::from_str(sender.as_str())))
		.stream();

	let wildcard_state = required_state
		.iter()
		.filter(|(_, state_key)| state_key == "*")
		.map(|(event_type, _)| {
			services
				.state_accessor
				.room_state_keys(room_id, event_type)
				.map_ok(|state_key| (event_type.clone(), state_key))
				.ready_filter_map(Result::ok)
		})
		.stream()
		.flatten();

	let required_state = required_state
		.iter()
		.cloned()
		.stream()
		.chain(wildcard_state)
		.chain(timeline_senders)
		.broad_filter_map(async |state| {
			let state_key: StateKey = match state.1.as_str() {
				| "$LAZY" | "*" => return None,
				| "$ME" => sender_user.as_str().into(),
				| _ => state.1.clone(),
			};

			services
				.state_accessor
				.room_state_get(room_id, &state.0, &state_key)
				.map_ok(Event::into_format)
				.ok()
				.await
		})
		.collect();

	// TODO: figure out a timestamp we can use for remote invites
	let invite_state: OptionFuture<_> = is_invite
		.then(|| {
			services
				.state_cache
				.invite_state(sender_user, room_id)
				.ok()
		})
		.into();

	let timeline = timeline_pdus
		.iter()
		.stream()
		.filter_map(|item| ignored_filter(services, item.clone(), sender_user))
		.map(at!(1))
		.map(Event::into_format)
		.collect();

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
		.user
		.highlight_count(sender_user, room_id)
		.map(TryInto::try_into)
		.map(Result::ok);

	let notification_count = services
		.user
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

	let meta = join3(room_name, room_avatar, is_dm);
	let events = join3(timeline, required_state, invite_state);
	let member_counts = join(joined_count, invited_count);
	let notification_counts = join(highlight_count, notification_count);
	let (
		(room_name, room_avatar, is_dm),
		(timeline, required_state, invite_state),
		(joined_count, invited_count),
		(highlight_count, notification_count),
	) = join4(meta, events, member_counts, notification_counts)
		.boxed()
		.await;

	let (heroes, hero_name, heroes_avatar) = calculate_heroes(
		services,
		sender_user,
		room_id,
		room_name.as_ref(),
		room_avatar.as_deref(),
	)
	.await?;

	let num_live = None; // Count events in timeline greater than global sync counter

	Ok(Some(response::Room {
		initial: Some(roomsince == 0),
		lists: lists.clone(),
		membership: membership.clone(),
		name: room_name.or(hero_name),
		avatar: JsOption::from_option(room_avatar.or(heroes_avatar)),
		is_dm,
		required_state,
		invite_state: invite_state.flatten(),
		prev_batch: prev_batch.as_deref().map(Into::into),
		limited,
		timeline,
		bump_stamp,
		heroes,
		num_live,
		joined_count,
		invited_count,
		unread_notifications: UnreadNotificationsCount { highlight_count, notification_count },
	}))
}

#[tracing::instrument(name = "heroes", level = "trace", skip_all)]
#[allow(clippy::type_complexity)]
async fn calculate_heroes(
	services: &Services,
	sender_user: &UserId,
	room_id: &RoomId,
	room_name: Option<&DisplayName>,
	room_avatar: Option<&MxcUri>,
) -> Result<(Option<Vec<response::Hero>>, Option<DisplayName>, Option<OwnedMxcUri>)> {
	const MAX_HEROES: usize = 5;
	let heroes: Vec<_> = services
		.state_cache
		.room_members(room_id)
		.ready_filter(|&member| member != sender_user)
		.ready_filter_map(|member| room_name.is_none().then_some(member))
		.map(ToOwned::to_owned)
		.broadn_filter_map(MAX_HEROES, async |user_id| {
			let content = services
				.state_accessor
				.get_member(room_id, &user_id)
				.await
				.ok()?;

			let name: OptionFuture<_> = content
				.displayname
				.is_none()
				.then(|| services.users.displayname(&user_id).ok())
				.into();

			let avatar: OptionFuture<_> = content
				.avatar_url
				.is_none()
				.then(|| services.users.avatar_url(&user_id).ok())
				.into();

			let (name, avatar) = join(name, avatar).await;
			let hero = response::Hero {
				user_id,
				avatar: avatar.unwrap_or(content.avatar_url),
				name: name
					.unwrap_or(content.displayname)
					.map(Into::into),
			};

			Some(hero)
		})
		.take(MAX_HEROES)
		.collect()
		.await;

	let hero_name = match heroes.len().cmp(&(1_usize)) {
		| Ordering::Less => None,
		| Ordering::Equal => Some(
			heroes[0]
				.name
				.clone()
				.unwrap_or_else(|| heroes[0].user_id.as_str().into()),
		),
		| Ordering::Greater => {
			let firsts = heroes[1..]
				.iter()
				.map(|h| {
					h.name
						.clone()
						.unwrap_or_else(|| h.user_id.as_str().into())
				})
				.collect::<Vec<_>>()
				.join(", ");

			let last = heroes[0]
				.name
				.clone()
				.unwrap_or_else(|| heroes[0].user_id.as_str().into());

			Some(format!("{firsts} and {last}")).map(Into::into)
		},
	};

	let heroes_avatar = (room_avatar.is_none() && room_name.is_none())
		.then(|| {
			heroes
				.first()
				.and_then(|hero| hero.avatar.clone())
		})
		.flatten();

	Ok((Some(heroes), hero_name, heroes_avatar))
}
