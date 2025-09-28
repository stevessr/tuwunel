#![cfg(test)]

use insta::{assert_debug_snapshot, with_settings};
use tuwunel::Server;
use tuwunel_core::{Args, Result, runtime};

#[test]
fn smoke_async() -> Result {
	with_settings!({
		description => "Smoke Async",
		snapshot_suffix => "smoke_async",
	}, {
		let args = Args::default_test(&["smoke", "fresh"]);
		let runtime = runtime::new(Some(&args))?;
		let server = Server::new(Some(&args), Some(runtime.handle()))?;
		let result = runtime.block_on(async {
			let _services = tuwunel::async_start(&server).await?;
			tuwunel::async_run(&server).await?;
			tuwunel::async_stop(&server).await
		});

		assert_debug_snapshot!(result);
		result
	})
}
