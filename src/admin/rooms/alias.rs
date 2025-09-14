use std::fmt::Write;

use clap::Subcommand;
use futures::StreamExt;
use ruma::{OwnedRoomAliasId, OwnedRoomId};
use tuwunel_core::{Err, Result};

use crate::Context;

#[derive(Debug, Subcommand)]
pub(crate) enum RoomAliasCommand {
	/// - Make an alias point to a room.
	Set {
		#[arg(short, long)]
		/// Set the alias even if a room is already using it
		force: bool,

		/// The room id to set the alias on
		room_id: OwnedRoomId,

		/// The alias localpart to use (`alias`, not `#alias:servername.tld`)
		room_alias_localpart: String,
	},

	/// - Remove a local alias
	Remove {
		/// The alias localpart to remove (`alias`, not `#alias:servername.tld`)
		room_alias_localpart: String,
	},

	/// - Show which room is using an alias
	Which {
		/// The alias localpart to look up (`alias`, not
		/// `#alias:servername.tld`)
		room_alias_localpart: String,
	},

	/// - List aliases currently being used
	List {
		/// If set, only list the aliases for this room
		room_id: Option<OwnedRoomId>,
	},
}

pub(super) async fn process(command: RoomAliasCommand, context: &Context<'_>) -> Result<String> {
	let services = context.services;
	let server_user = &services.globals.server_user;

	match command {
		| RoomAliasCommand::Set { ref room_alias_localpart, .. }
		| RoomAliasCommand::Remove { ref room_alias_localpart }
		| RoomAliasCommand::Which { ref room_alias_localpart } => {
			let room_alias_str =
				format!("#{}:{}", room_alias_localpart, services.globals.server_name());
			let room_alias = match OwnedRoomAliasId::parse(room_alias_str) {
				| Ok(alias) => alias,
				| Err(err) => {
					return Err!("Failed to parse alias: {err}");
				},
			};
			match command {
				| RoomAliasCommand::Set { force, room_id, .. } => {
					match (
						force,
						services
							.alias
							.resolve_local_alias(&room_alias)
							.await,
					) {
						| (true, Ok(id)) => {
							match services
								.alias
								.set_alias(&room_alias, &room_id, server_user)
							{
								| Err(err) => Err!("Failed to remove alias: {err}"),
								| Ok(()) =>
									Ok(format!("Successfully overwrote alias (formerly {id})")),
							}
						},
						| (false, Ok(id)) => Err!(
							"Refusing to overwrite in use alias for {id}, use -f or --force to \
							 overwrite"
						),
						| (_, Err(_)) => {
							match services
								.alias
								.set_alias(&room_alias, &room_id, server_user)
							{
								| Err(err) => Err!("Failed to remove alias: {err}"),
								| Ok(()) => Ok("Successfully set alias".to_owned()),
							}
						},
					}
				},
				| RoomAliasCommand::Remove { .. } => {
					match services
						.alias
						.resolve_local_alias(&room_alias)
						.await
					{
						| Err(_) => Err!("Alias isn't in use."),
						| Ok(id) => match services
							.alias
							.remove_alias(&room_alias, server_user)
							.await
						{
							| Err(err) => Err!("Failed to remove alias: {err}"),
							| Ok(()) => Ok(format!("Removed alias from {id}")),
						},
					}
				},
				| RoomAliasCommand::Which { .. } => {
					match services
						.alias
						.resolve_local_alias(&room_alias)
						.await
					{
						| Err(_) => Err!("Alias isn't in use."),
						| Ok(id) => Ok(format!("Alias resolves to {id}")),
					}
				},
				| RoomAliasCommand::List { .. } => unreachable!(),
			}
		},
		| RoomAliasCommand::List { room_id } =>
			if let Some(room_id) = room_id {
				let aliases: Vec<OwnedRoomAliasId> = services
					.alias
					.local_aliases_for_room(&room_id)
					.map(Into::into)
					.collect()
					.await;

				let mut plain_list = String::new();
				for alias in aliases {
					writeln!(plain_list, "- {alias}")?;
				}

				let plain = format!("Aliases for {room_id}:\n{plain_list}");
				Ok(plain)
			} else {
				let aliases = services
					.alias
					.all_local_aliases()
					.map(|(room_id, localpart)| (room_id.into(), localpart.into()))
					.collect::<Vec<(OwnedRoomId, String)>>()
					.await;

				let server_name = services.globals.server_name();
				let mut plain_list = String::new();
				for (alias, id) in aliases {
					writeln!(plain_list, "- `{alias}` -> #{id}:{server_name}")?;
				}

				let plain = format!("Aliases:\n{plain_list}");
				Ok(plain)
			},
	}
}
