use tuwunel_core::{Err, Result};

use crate::{command, command_dispatch};

#[command_dispatch]
#[derive(Debug, clap::Subcommand)]
pub(crate) enum TesterCommand {
	Panic,
	Failure,
	Tester,
	Timer,
}

#[rustfmt::skip]
#[command]
async fn panic(&self) -> Result<String> {

	panic!("panicked")
}

#[rustfmt::skip]
#[command]
async fn failure(&self) -> Result<String> {

	Err!("failed")
}

#[inline(never)]
#[rustfmt::skip]
#[command]
async fn tester(&self) -> Result<String> {
	Ok("Ok".to_owned())
}

#[inline(never)]
#[rustfmt::skip]
#[command]
async fn timer(&self) -> Result<String> {
	let started = std::time::Instant::now();
	timed(self.input);

	let elapsed = started.elapsed();
	Ok(format!("completed in {elapsed:#?}"))
}

#[inline(never)]
#[rustfmt::skip]
#[allow(unused_variables)]
fn timed(body: &str) {

}
