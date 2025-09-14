use ruma::{RoomId, UserId};
use tuwunel_core::{Err, Result, warn};
use tuwunel_service::Services;

pub(crate) async fn invite_check(
	services: &Services,
	sender_user: &UserId,
	room_id: Option<&RoomId>,
) -> Result {
	if !services.admin.user_is_admin(sender_user).await && services.config.block_non_admin_invites
	{
		let to_room = if let Some(room_id) = room_id {
			&format!(" to {room_id}")
		} else {
			""
		};
		warn!("{sender_user} is not an admin and attempted to send an invite{to_room}");
		return Err!(Request(Forbidden("Invites are not allowed on this server.")));
	}

	Ok(())
}
