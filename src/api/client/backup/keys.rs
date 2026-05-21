use axum::extract::State;
use ruma::api::client::backup::{add_backup_keys, delete_backup_keys, get_backup_keys};
use tuwunel_core::{Err, Result};

use super::get_count_etag;
use crate::Ruma;

/// # `PUT /_matrix/client/r0/room_keys/keys`
///
/// Add the received backup keys to the database.
///
/// - Only manipulating the most recently created version of the backup is
///   allowed
/// - Adds the keys to the backup
/// - Returns the new number of keys in this backup and the etag
pub(crate) async fn add_backup_keys_route(
	State(services): State<crate::State>,
	body: Ruma<add_backup_keys::v3::Request>,
) -> Result<add_backup_keys::v3::Response> {
	if services
		.key_backups
		.get_latest_backup_version(body.sender_user())
		.await
		.is_ok_and(|version| version != body.version)
	{
		return Err!(Request(InvalidParam(
			"You may only manipulate the most recently created version of the backup."
		)));
	}

	for (room_id, room) in &body.rooms {
		for (session_id, key_data) in &room.sessions {
			services
				.key_backups
				.add_key(body.sender_user(), &body.version, room_id, session_id, key_data)
				.await?;
		}
	}

	let (count, etag) = get_count_etag(&services, body.sender_user(), &body.version).await?;

	Ok(add_backup_keys::v3::Response { count, etag })
}

/// # `GET /_matrix/client/r0/room_keys/keys`
///
/// Retrieves all keys from the backup.
pub(crate) async fn get_backup_keys_route(
	State(services): State<crate::State>,
	body: Ruma<get_backup_keys::v3::Request>,
) -> Result<get_backup_keys::v3::Response> {
	let rooms = services
		.key_backups
		.get_all(body.sender_user(), &body.version)
		.await;

	Ok(get_backup_keys::v3::Response { rooms })
}

/// # `DELETE /_matrix/client/r0/room_keys/keys`
///
/// Delete the keys from the backup.
pub(crate) async fn delete_backup_keys_route(
	State(services): State<crate::State>,
	body: Ruma<delete_backup_keys::v3::Request>,
) -> Result<delete_backup_keys::v3::Response> {
	services
		.key_backups
		.delete_all_keys(body.sender_user(), &body.version)
		.await;

	let (count, etag) = get_count_etag(&services, body.sender_user(), &body.version).await?;

	Ok(delete_backup_keys::v3::Response { count, etag })
}
