use std::cmp::Ordering;

use futures::{
	FutureExt, StreamExt, TryFutureExt,
	future::{OptionFuture, join, join3, join4},
};
use ruma::{
	JsOption, MxcUri, OwnedMxcUri, RoomId, UInt, UserId,
	api::client::sync::sync_events::{UnreadNotificationsCount, v5::response},
	events::{StateEventType, room::member::MembershipState},
};
use tuwunel_core::{
	Result, at, debug_error, is_equal_to,
	matrix::{Event, StateKey, pdu::PduCount},
	ref_at,
	utils::{IterStream, ReadyExt, TryFutureExtExt, result::FlatOk, stream::BroadbandExt},
};
use tuwunel_service::Services;

use super::{SyncInfo, TodoRoom};
use crate::client::{DEFAULT_BUMP_TYPES, ignored_filter, sync::load_timeline};

#[tracing::instrument(level = "debug", skip_all, fields(room_id, roomsince))]
#[allow(clippy::too_many_arguments)]
pub(super) async fn handle(
	services: &Services,
	next_batch: u64,
	SyncInfo { sender_user, .. }: SyncInfo<'_>,
	room_id: &RoomId,
	&TodoRoom {
		ref membership,
		ref requested_state,
		timeline_limit,
		roomsince,
	}: &TodoRoom,
) -> Result<Option<response::Room>> {
	let timeline: OptionFuture<_> = membership
		.ne(&MembershipState::Invite)
		.then(|| {
			load_timeline(
				services,
				sender_user,
				room_id,
				PduCount::Normal(roomsince),
				Some(PduCount::from(next_batch)),
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

	if roomsince != 0 && timeline_pdus.is_empty() && membership.ne(&MembershipState::Invite) {
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
			let ts = pdu.origin_server_ts().get();
			if bump_stamp.is_none_or(|bump_stamp| bump_stamp < ts) {
				bump_stamp.replace(ts);
			}

			bump_stamp
		});

	let lazy = requested_state
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
		.map(|sender| (StateEventType::RoomMember, StateKey::from_str(sender.as_str())));

	let required_state = requested_state
		.iter()
		.cloned()
		.chain(timeline_senders)
		.stream()
		.broad_filter_map(async |state| {
			let state_key: StateKey = match state.1.as_str() {
				| "$LAZY" => return None,
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
	let invite_state: OptionFuture<_> = membership
		.eq(&MembershipState::Invite)
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

	let meta = join(room_name, room_avatar);
	let events = join3(timeline, required_state, invite_state);
	let member_counts = join(joined_count, invited_count);
	let notification_counts = join(highlight_count, notification_count);
	let (
		(room_name, room_avatar),
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
		room_name.as_deref(),
		room_avatar.as_deref(),
	)
	.await?;

	let num_live = None; // Count events in timeline greater than global sync counter

	Ok(Some(response::Room {
		initial: Some(roomsince == 0),
		name: room_name.or(hero_name),
		avatar: JsOption::from_option(room_avatar.or(heroes_avatar)),
		invite_state: invite_state.flatten(),
		required_state,
		timeline,
		is_dm: None,
		prev_batch,
		limited,
		bump_stamp,
		heroes,
		num_live,
		joined_count,
		invited_count,
		unread_notifications: UnreadNotificationsCount { highlight_count, notification_count },
	}))
}

#[tracing::instrument(level = "debug", skip_all, fields(room_id, roomsince))]
#[allow(clippy::type_complexity)]
async fn calculate_heroes(
	services: &Services,
	sender_user: &UserId,
	room_id: &RoomId,
	room_name: Option<&str>,
	room_avatar: Option<&MxcUri>,
) -> Result<(Option<Vec<response::Hero>>, Option<String>, Option<OwnedMxcUri>)> {
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
				name: name.unwrap_or(content.displayname),
				avatar: avatar.unwrap_or(content.avatar_url),
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
				.unwrap_or_else(|| heroes[0].user_id.to_string()),
		),
		| Ordering::Greater => {
			let firsts = heroes[1..]
				.iter()
				.map(|h| {
					h.name
						.clone()
						.unwrap_or_else(|| h.user_id.to_string())
				})
				.collect::<Vec<_>>()
				.join(", ");

			let last = heroes[0]
				.name
				.clone()
				.unwrap_or_else(|| heroes[0].user_id.to_string());

			Some(format!("{firsts} and {last}"))
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
