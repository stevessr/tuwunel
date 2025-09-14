use clap::Parser;
use tuwunel_core::Result;
use tuwunel_macros::{command, command_dispatch};

use crate::user::debug::Cmd;

#[derive(Debug, Parser)]
#[command(name = "tuwunel", version = tuwunel_core::version())]
#[command_dispatch]
pub(super) enum UserCommand {
	#[command(subcommand)]
	Debug(Cmd),
}

mod debug {
	use clap::Subcommand;
	use tuwunel_core::Result;
	use tuwunel_macros::{command, command_dispatch};

	#[command_dispatch]
	#[derive(Debug, Subcommand)]
	pub(crate) enum Cmd {
		Echo {},
	}

	#[command]
	pub(super) async fn echo(&self) -> Result<String> {
		let sender = self.sender;
		Ok(format!("Running echo command from {sender}"))
	}
}
