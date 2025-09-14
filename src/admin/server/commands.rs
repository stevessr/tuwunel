use std::{fmt::Write, path::PathBuf, sync::Arc};

use tuwunel_core::{Err, Result, info, utils::time, warn};

use crate::command;

#[command]
pub(super) async fn uptime(&self) -> Result<String> {
	let elapsed = self
		.services
		.server
		.started
		.elapsed()
		.expect("standard duration");

	let result = time::pretty(elapsed);
	Ok(format!("{result}."))
}

#[command]
pub(super) async fn show_config(&self) -> Result<String> {
	Ok(format!("{}", *self.services.server.config))
}

#[command]
pub(super) async fn reload_config(&self, path: Option<PathBuf>) -> Result<String> {
	let path = path.as_deref().into_iter();
	self.services.config.reload(path)?;

	Ok("Successfully reconfigured.".to_owned())
}

#[command]
pub(super) async fn list_features(
	&self,
	available: bool,
	enabled: bool,
	comma: bool,
) -> Result<String> {
	let delim = if comma { "," } else { " " };
	if enabled && !available {
		let features = info::rustc::features().join(delim);
		let out = format!("`\n{features}\n`");
		return Ok(out);
	}

	if available && !enabled {
		let features = info::cargo::features().join(delim);
		let out = format!("`\n{features}\n`");
		return Ok(out);
	}

	let mut features = String::new();
	let enabled = info::rustc::features();
	let available = info::cargo::features();
	for feature in available {
		let active = enabled.contains(&feature.as_str());
		let emoji = if active { "✅" } else { "❌" };
		let remark = if active { "[enabled]" } else { "" };
		writeln!(features, "{emoji} {feature} {remark}")?;
	}

	Ok(features)
}

#[command]
pub(super) async fn memory_usage(&self) -> Result<String> {
	let services_usage = self.services.memory_usage().await?;
	let database_usage = self.services.db.engine.memory_usage()?;
	let allocator_usage = tuwunel_core::alloc::memory_usage()
		.map_or(String::new(), |s| format!("\nAllocator:\n{s}"));

	Ok(format!(
		"Services:\n{services_usage}\nDatabase:\n{database_usage}{allocator_usage}",
	))
}

#[command]
pub(super) async fn clear_caches(&self) -> Result<String> {
	self.services.clear_cache().await;

	Ok("Done.".to_owned())
}

#[command]
pub(super) async fn list_backups(&self) -> Result<String> {
	let mut out = String::new();

	for backup in self.services.db.engine.backup_list()? {
		writeln!(out, "{backup}")?;
	}

	Ok(out)
}

#[command]
pub(super) async fn backup_database(&self) -> Result<String> {
	let db = Arc::clone(&self.services.db);
	let result = self
		.services
		.server
		.runtime()
		.spawn_blocking(move || match db.engine.backup() {
			| Ok(()) => "Done".to_owned(),
			| Err(e) => format!("Failed: {e}"),
		})
		.await?;

	let count = self.services.db.engine.backup_count()?;
	Ok(format!("{result}. Currently have {count} backups."))
}

#[command]
pub(super) async fn admin_notice(&self, message: Vec<String>) -> Result<String> {
	let message = message.join(" ");
	self.services.admin.send_text(&message).await;

	Ok("Notice was sent to #admins".to_owned())
}

#[command]
pub(super) async fn reload_mods(&self) -> Result<String> {
	self.services.server.reload()?;

	Ok("Reloading server...".to_owned())
}

#[command]
#[cfg(unix)]
pub(super) async fn restart(&self, force: bool) -> Result<String> {
	use tuwunel_core::utils::sys::current_exe_deleted;

	if !force && current_exe_deleted() {
		return Err!(
			"The server cannot be restarted because the executable changed. If this is expected \
			 use --force to override."
		);
	}

	self.services.server.restart()?;

	Ok("Restarting server...".to_owned())
}

#[command]
pub(super) async fn shutdown(&self) -> Result<String> {
	warn!("shutdown command");
	self.services.server.shutdown()?;

	Ok("Shutting down server...".to_owned())
}
