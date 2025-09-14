use clap::Subcommand;
use ruma::{OwnedEventId, OwnedRoomOrAliasId};
use tuwunel_core::Result;

use crate::{command, command_dispatch};

#[command_dispatch]
#[derive(Debug, Subcommand)]
/// Query tables from database
pub(crate) enum ShortCommand {
	ShortEventId {
		event_id: OwnedEventId,
	},

	ShortRoomId {
		room_id: OwnedRoomOrAliasId,
	},
}

#[command]
pub(super) async fn short_event_id(&self, event_id: OwnedEventId) -> Result<String> {
	let shortid = self
		.services
		.short
		.get_shorteventid(&event_id)
		.await?;

	Ok(format!("{shortid:#?}"))
}

#[command]
pub(super) async fn short_room_id(&self, room_id: OwnedRoomOrAliasId) -> Result<String> {
	let room_id = self
		.services
		.alias
		.maybe_resolve(&room_id)
		.await?;

	let shortid = self
		.services
		.short
		.get_shortroomid(&room_id)
		.await?;

	Ok(format!("{shortid:#?}"))
}
