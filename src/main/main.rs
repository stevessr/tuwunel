use std::sync::atomic::Ordering;

use tuwunel::{Server, restart};
use tuwunel_core::{Result, args, debug_info, runtime};

fn main() -> Result {
	let args = args::parse();
	let runtime = runtime::new(Some(&args))?;
	let server = Server::new(Some(&args), Some(runtime.handle()))?;

	tuwunel::exec(&server, runtime)?;

	#[cfg(unix)]
	if server.server.restarting.load(Ordering::Acquire) {
		restart::restart();
	}

	debug_info!("Exit");
	Ok(())
}
