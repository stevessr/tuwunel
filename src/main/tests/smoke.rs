#![cfg(test)]

use tuwunel::Server;
use tuwunel_core::{Result, runtime};

#[test]
fn dummy() {}

#[test]
#[should_panic = "dummy"]
fn panic_dummy() { panic!("dummy") }

#[test]
fn smoke() -> Result {
	let runtime = runtime::new(None)?;
	let server = Server::new(None, Some(runtime.handle()))?;

	tuwunel::exec(&server, runtime)
}
