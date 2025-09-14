use clap::Subcommand;
use futures::TryStreamExt;
use ruma::OwnedRoomOrAliasId;
use tuwunel_core::{PduCount, Result, utils::stream::TryTools};

use crate::{command, command_dispatch};

#[command_dispatch]
#[derive(Debug, Subcommand)]
/// Query tables from database
pub(crate) enum RoomTimelineCommand {
	Pdus {
		room_id: OwnedRoomOrAliasId,

		from: Option<String>,

		#[arg(short, long)]
		limit: Option<usize>,
	},

	Last {
		room_id: OwnedRoomOrAliasId,
	},
}

#[command]
pub(super) async fn last(&self, room_id: OwnedRoomOrAliasId) -> Result<String> {
	let room_id = self
		.services
		.alias
		.maybe_resolve(&room_id)
		.await?;

	let result = self
		.services
		.timeline
		.last_timeline_count(None, &room_id, None)
		.await?;

	Ok(format!("{result:#?}"))
}

#[command]
pub(super) async fn pdus(
	&self,
	room_id: OwnedRoomOrAliasId,
	from: Option<String>,
	limit: Option<usize>,
) -> Result<String> {
	let room_id = self
		.services
		.alias
		.maybe_resolve(&room_id)
		.await?;

	let from: Option<PduCount> = from.as_deref().map(str::parse).transpose()?;

	let result: Vec<_> = self
		.services
		.timeline
		.pdus_rev(None, &room_id, from)
		.try_take(limit.unwrap_or(3))
		.try_collect()
		.await?;

	Ok(format!("{result:#?}"))
}
