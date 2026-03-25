#![cfg(test)]
#![allow(unused_features)] // 1.96.0-nightly 2026-03-07 bug

use insta::{assert_debug_snapshot, with_settings};
use tuwunel::{Args, Server, runtime};

#[test]
#[should_panic = "There was a problem with your configuration"]
fn listener_conf_err() {
	with_settings!({
		description => "Listener Configuration Err",
		snapshot_suffix => "listener_conf_err",
	}, {
		let mut args = Args::default_test(&["smoke", "fresh", "cleanup"]);
		args.option.push("address=[\"256.256.256.256\"]".into());

		let runtime = runtime::new(Some(&args)).unwrap();
		let server = Server::new(Some(&args), Some(runtime.handle())).unwrap();
		let result = tuwunel::exec(&server, runtime);

		assert_debug_snapshot!(result);
	});
}
