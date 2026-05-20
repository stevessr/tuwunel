use clap::Subcommand;
use futures::StreamExt;
use ruma::{OwnedRoomId, OwnedServerName, OwnedUserId};
use tuwunel_core::Result;

use crate::Context;

#[derive(Debug, Subcommand)]
pub(crate) enum RoomStateCacheCommand {
	ServerInRoom {
		server: OwnedServerName,
		room_id: OwnedRoomId,
	},

	RoomServers {
		room_id: OwnedRoomId,
	},

	ServerRooms {
		server: OwnedServerName,
	},

	RoomMembers {
		room_id: OwnedRoomId,
	},

	LocalUsersInRoom {
		room_id: OwnedRoomId,
	},

	ActiveLocalUsersInRoom {
		room_id: OwnedRoomId,
	},

	RoomJoinedCount {
		room_id: OwnedRoomId,
	},

	RoomInvitedCount {
		room_id: OwnedRoomId,
	},

	RoomUserOnceJoined {
		room_id: OwnedRoomId,
	},

	RoomMembersInvited {
		room_id: OwnedRoomId,
	},

	GetInviteCount {
		room_id: OwnedRoomId,
		user_id: OwnedUserId,
	},

	GetLeftCount {
		room_id: OwnedRoomId,
		user_id: OwnedUserId,
	},

	RoomsJoined {
		user_id: OwnedUserId,
	},

	RoomsLeft {
		user_id: OwnedUserId,
	},

	RoomsInvited {
		user_id: OwnedUserId,
	},

	InviteState {
		user_id: OwnedUserId,
		room_id: OwnedRoomId,
	},

	UserMemberships {
		user_id: OwnedUserId,
	},
}

pub(super) async fn process(subcommand: RoomStateCacheCommand, context: &Context<'_>) -> Result {
	let cache = &context.services.state_cache;

	match subcommand {
		| RoomStateCacheCommand::ServerInRoom { server, room_id } =>
			context
				.write_timed_query(cache.server_in_room(&server, &room_id))
				.await,
		| RoomStateCacheCommand::RoomServers { room_id } =>
			context
				.write_timed_query(
					cache
						.room_servers(&room_id)
						.map(ToOwned::to_owned)
						.collect::<Vec<_>>(),
				)
				.await,
		| RoomStateCacheCommand::ServerRooms { server } =>
			context
				.write_timed_query(
					cache
						.server_rooms(&server)
						.map(ToOwned::to_owned)
						.collect::<Vec<_>>(),
				)
				.await,
		| RoomStateCacheCommand::RoomMembers { room_id } =>
			context
				.write_timed_query(
					cache
						.room_members(&room_id)
						.map(ToOwned::to_owned)
						.collect::<Vec<_>>(),
				)
				.await,
		| RoomStateCacheCommand::LocalUsersInRoom { room_id } =>
			context
				.write_timed_query(
					cache
						.local_users_in_room(&room_id)
						.map(ToOwned::to_owned)
						.collect::<Vec<_>>(),
				)
				.await,
		| RoomStateCacheCommand::ActiveLocalUsersInRoom { room_id } =>
			context
				.write_timed_query(
					cache
						.active_local_users_in_room(&room_id)
						.map(ToOwned::to_owned)
						.collect::<Vec<_>>(),
				)
				.await,
		| RoomStateCacheCommand::RoomJoinedCount { room_id } =>
			context
				.write_timed_query(cache.room_joined_count(&room_id))
				.await,
		| RoomStateCacheCommand::RoomInvitedCount { room_id } =>
			context
				.write_timed_query(cache.room_invited_count(&room_id))
				.await,
		| RoomStateCacheCommand::RoomUserOnceJoined { room_id } =>
			context
				.write_timed_query(
					cache
						.room_useroncejoined(&room_id)
						.map(ToOwned::to_owned)
						.collect::<Vec<_>>(),
				)
				.await,
		| RoomStateCacheCommand::RoomMembersInvited { room_id } =>
			context
				.write_timed_query(
					cache
						.room_members_invited(&room_id)
						.map(ToOwned::to_owned)
						.collect::<Vec<_>>(),
				)
				.await,
		| RoomStateCacheCommand::GetInviteCount { room_id, user_id } =>
			context
				.write_timed_query(cache.get_invite_count(&room_id, &user_id))
				.await,
		| RoomStateCacheCommand::GetLeftCount { room_id, user_id } =>
			context
				.write_timed_query(cache.get_left_count(&room_id, &user_id))
				.await,
		| RoomStateCacheCommand::RoomsJoined { user_id } =>
			context
				.write_timed_query(
					cache
						.rooms_joined(&user_id)
						.map(ToOwned::to_owned)
						.collect::<Vec<_>>(),
				)
				.await,
		| RoomStateCacheCommand::RoomsInvited { user_id } =>
			context
				.write_timed_query(
					cache
						.rooms_invited_state(&user_id)
						.collect::<Vec<_>>(),
				)
				.await,
		| RoomStateCacheCommand::RoomsLeft { user_id } =>
			context
				.write_timed_query(
					cache
						.rooms_left_state(&user_id)
						.collect::<Vec<_>>(),
				)
				.await,
		| RoomStateCacheCommand::InviteState { user_id, room_id } =>
			context
				.write_timed_query(cache.invite_state(&user_id, &room_id))
				.await,
		| RoomStateCacheCommand::UserMemberships { user_id } =>
			context
				.write_timed_query(
					cache
						.all_user_memberships(&user_id)
						.map(|(membership, room_id)| (membership, room_id.to_owned()))
						.collect::<Vec<_>>(),
				)
				.await,
	}
}
