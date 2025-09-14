use axum::extract::State;
use ruma::{
	OwnedRoomAliasId, OwnedRoomId, OwnedUserId,
	api::client::room::{
		Visibility,
		create_room::{self, v3::RoomPreset},
	},
	events::{
		TimelineEventType,
		room::{guest_access::GuestAccess, join_rules::JoinRule},
	},
};
use tuwunel_core::{
	Err, Result, debug_info, debug_warn, err,
	matrix::{StateKey, pdu::PduBuilder},
	warn,
};
use tuwunel_service::{Services, appservice::RegistrationInfo};

use crate::{Ruma, client::utils::invite_check};

/// # `POST /_matrix/client/v3/createRoom`
///
/// Creates a new room.
///
/// - Room ID is randomly generated
/// - Create alias if `room_alias_name` is set
/// - Send create event
/// - Join sender user
/// - Send power levels event
/// - Send canonical room alias
/// - Send join rules
/// - Send history visibility
/// - Send guest access
/// - Send events listed in initial state
/// - Send events implied by `name` and `topic`
/// - Send invite events
#[allow(clippy::large_stack_frames)]
pub(crate) async fn create_room_route(
	State(services): State<crate::State>,
	body: Ruma<create_room::v3::Request>,
) -> Result<create_room::v3::Response> {
	can_create_room_check(&services, &body).await?;
	can_publish_directory_check(&services, &body).await?;

	let sender_user = body.sender_user();

	// Figure out preset. We need it for preset specific events
	let preset = body
		.preset
		.clone()
		.unwrap_or(match &body.visibility {
			| Visibility::Public => RoomPreset::PublicChat,
			| Visibility::Private => RoomPreset::PrivateChat,
			| _ => return Err!(Request(InvalidParam("Room visibility is not supported"))),
		});

	let (join_rule, guest_access, equal_pl) = match preset {
		| RoomPreset::PrivateChat => (JoinRule::Invite, GuestAccess::CanJoin, false),
		| RoomPreset::TrustedPrivateChat => (JoinRule::Invite, GuestAccess::CanJoin, true),
		| RoomPreset::PublicChat => (JoinRule::Public, GuestAccess::Forbidden, false),
		| _ => return Err!(Request(InvalidParam("Room preset is not supported"))),
	};

	let room_id = match &body.room_id {
		| Some(room_id) => Some(custom_room_id_check(&services, room_id).await?),
		| None => None,
	};

	// Determine room version
	let room_version = body.room_version.as_ref();

	// Error on existing alias before committing to creation.
	#[rustfmt::skip]
	let alias = match body.room_alias_name.as_ref() {
		| Some(alias) => Some(
			room_alias_check(
				&services,
				alias,
				body.appservice_info.as_ref()
			).await?),
		| None => None,
	};

	let invites = if (!body.invite.is_empty() || !body.invite_3pid.is_empty())
		&& invite_check(&services, sender_user, room_id.as_deref())
			.await
			.is_ok()
	{
		let mut invites: Vec<OwnedUserId> = Vec::new();
		for user_id in &body.invite {
			if services
				.users
				.user_is_ignored(sender_user, user_id)
				.await
			{
				continue;
			}

			if services
				.users
				.user_is_ignored(user_id, sender_user)
				.await
			{
				continue;
			}

			invites.push(user_id.to_owned());
		}

		invites
	} else {
		Vec::new()
	};

	let additional_creators = if equal_pl { invites.as_slice() } else { &[] };

	let mut initial_state: Vec<PduBuilder> = Vec::new();
	for event in &body.initial_state {
		let mut pdu_builder = event
			.deserialize_as_unchecked::<PduBuilder>()
			.map_err(|e| {
				err!(Request(InvalidParam(warn!("Invalid initial state event: {e:?}"))))
			})?;

		debug_info!("Room creation initial state event: {event:?}");

		// client/appservice workaround: if a user sends an initial_state event with a
		// state event in there with the content of literally `{}` (not null or empty
		// string), let's just skip it over and warn.
		if pdu_builder.content.get().eq("{}") {
			debug_warn!("skipping empty initial state event with content of `{{}}`: {event:?}");
			debug_warn!("content: {}", pdu_builder.content.get());
			continue;
		}

		// Implicit state key defaults to ""
		pdu_builder
			.state_key
			.get_or_insert_with(StateKey::new);

		// Silently skip encryption events if they are not allowed
		if pdu_builder.event_type == TimelineEventType::RoomEncryption
			&& !services.config.allow_encryption
		{
			continue;
		}

		initial_state.push(pdu_builder);
	}

	let (room_id, state_lock) = services
		.create
		.create_room(
			sender_user,
			room_id.as_deref(),
			room_version,
			alias.as_deref(),
			additional_creators,
			body.is_direct,
			initial_state,
			join_rule,
			guest_access,
			matches!(body.visibility, Visibility::Public),
			body.name.as_deref(),
			body.topic.as_deref(),
			body.power_level_content_override.as_ref(),
			body.creation_content.as_ref(),
		)
		.await?;

	drop(state_lock);

	for user_id in &invites {
		services
			.membership
			.invite(sender_user, user_id, &room_id, None, body.is_direct)
			.await?;
	}

	Ok(create_room::v3::Response::new(room_id))
}

async fn room_alias_check(
	services: &Services,
	room_alias_name: &str,
	appservice_info: Option<&RegistrationInfo>,
) -> Result<OwnedRoomAliasId> {
	// Basic checks on the room alias validity
	if room_alias_name.contains(':') {
		return Err!(Request(InvalidParam(
			"Room alias contained `:` which is not allowed. Please note that this expects a \
			 localpart, not the full room alias.",
		)));
	} else if room_alias_name.contains(char::is_whitespace) {
		return Err!(Request(InvalidParam(
			"Room alias contained spaces which is not a valid room alias.",
		)));
	}

	// check if room alias is forbidden
	if services
		.config
		.forbidden_alias_names
		.is_match(room_alias_name)
	{
		return Err!(Request(Unknown("Room alias name is forbidden.")));
	}

	let server_name = services.globals.server_name();
	let full_room_alias = OwnedRoomAliasId::parse(format!("#{room_alias_name}:{server_name}"))
		.map_err(|e| {
			err!(Request(InvalidParam(debug_error!(
				?e,
				?room_alias_name,
				"Failed to parse room alias.",
			))))
		})?;

	if services
		.alias
		.resolve_local_alias(&full_room_alias)
		.await
		.is_ok()
	{
		return Err!(Request(RoomInUse("Room alias already exists.")));
	}

	if let Some(info) = appservice_info {
		if !info.aliases.is_match(full_room_alias.as_str()) {
			return Err!(Request(Exclusive("Room alias is not in namespace.")));
		}
	} else if services
		.appservice
		.is_exclusive_alias(&full_room_alias)
		.await
	{
		return Err!(Request(Exclusive("Room alias reserved by appservice.",)));
	}

	debug_info!("Full room alias: {full_room_alias}");

	Ok(full_room_alias)
}

/// if a room is being created with a custom room ID, run our checks against it
async fn custom_room_id_check(services: &Services, custom_room_id: &str) -> Result<OwnedRoomId> {
	// apply forbidden room alias checks to custom room IDs too
	if services
		.config
		.forbidden_alias_names
		.is_match(custom_room_id)
	{
		return Err!(Request(Unknown("Custom room ID is forbidden.")));
	}

	if custom_room_id.contains(':') {
		return Err!(Request(InvalidParam(
			"Custom room ID contained `:` which is not allowed. Please note that this expects a \
			 localpart, not the full room ID.",
		)));
	} else if custom_room_id.contains(char::is_whitespace) {
		return Err!(Request(InvalidParam(
			"Custom room ID contained spaces which is not valid."
		)));
	}

	let server_name = services.globals.server_name();
	let full_room_id = format!("!{custom_room_id}:{server_name}");

	let room_id = OwnedRoomId::parse(full_room_id)
		.inspect(|full_room_id| debug_info!(?full_room_id, "Full custom room ID"))
		.inspect_err(|e| {
			warn!(?e, ?custom_room_id, "Failed to create room with custom room ID");
		})?;

	// check if room ID doesn't already exist instead of erroring on auth check
	if services
		.short
		.get_shortroomid(&room_id)
		.await
		.is_ok()
	{
		return Err!(Request(RoomInUse("Room with that custom room ID already exists",)));
	}

	Ok(room_id)
}

async fn can_publish_directory_check(
	services: &Services,
	body: &Ruma<create_room::v3::Request>,
) -> Result {
	if !services
		.server
		.config
		.lockdown_public_room_directory
		|| body.appservice_info.is_some()
		|| body.visibility != Visibility::Public
		|| services
			.admin
			.user_is_admin(body.sender_user())
			.await
	{
		return Ok(());
	}

	let msg = format!(
		"Non-admin user {} tried to publish new to the directory while \
		 lockdown_public_room_directory is enabled",
		body.sender_user(),
	);

	warn!("{msg}");
	if services.server.config.admin_room_notices {
		services.admin.notice(&msg).await;
	}

	Err!(Request(Forbidden("Publishing rooms to the room directory is not allowed")))
}

async fn can_create_room_check(
	services: &Services,
	body: &Ruma<create_room::v3::Request>,
) -> Result {
	if !services.config.allow_room_creation
		&& body.appservice_info.is_none()
		&& !services
			.admin
			.user_is_admin(body.sender_user())
			.await
	{
		return Err!(Request(Forbidden("Room creation has been disabled.",)));
	}

	Ok(())
}
