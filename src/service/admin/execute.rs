use tokio::time::{Duration, sleep};
use tuwunel_core::{Err, Result, debug, debug_info, error, implement, info};

/// Execute admin commands after startup
#[implement(super::Service)]
pub(super) async fn startup_execute(&self) -> Result {
	// List of commands to execute
	let commands = &self.services.server.config.admin_execute;

	// Determine if we're running in smoketest-mode which will change some behaviors
	let smoketest = self.services.server.config.test.contains("smoke");

	// When true, errors are ignored and startup continues.
	let errors = !smoketest
		&& self
			.services
			.server
			.config
			.admin_execute_errors_ignore;

	//TODO: remove this after run-states are broadcast
	sleep(Duration::from_millis(500)).await;

	for (i, command) in commands.iter().enumerate() {
		if !self.execute_command(i, command.clone()).await {
			if !errors {
				return Err!("failed to execute command");
			}
		}

		tokio::task::yield_now().await;
	}

	// The smoketest functionality is placed here for now and simply initiates
	// shutdown after all commands have executed.
	if smoketest {
		debug_info!("Smoketest mode. All commands complete. Shutting down now...");
		self.services
			.server
			.shutdown()
			.inspect_err(error::inspect_log)
			.expect("Error shutting down from smoketest");
	}

	Ok(())
}

/// Execute one admin command after startup or signal
#[implement(super::Service)]
async fn execute_command(&self, i: usize, command: String) -> bool {
	debug!("Execute command #{i}: executing {command:?}");

	let result = self.run_command(&command, "").await;
	if !result.err {
		info!("Execute command #{i} completed:\n{:#}", result.output);
		true
	} else {
		error!("Execute command #{i} failed:\n{:#}", result.output);
		false
	}
}
