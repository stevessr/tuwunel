mod commands;

use clap::Subcommand;
use tuwunel_core::Result;

use crate::command_dispatch;

#[command_dispatch]
#[derive(Debug, Subcommand)]
pub(super) enum CheckCommand {
	CheckAllUsers,
}
