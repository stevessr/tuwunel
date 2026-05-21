use std::fmt::Write;

use tuwunel_core::{Result, info};

use crate::admin_command;

#[admin_command]
pub(super) async fn list_dependencies(&self, names: bool) -> Result {
	if names {
		let out = info::cargo::dependencies_names().join(" ");
		return self.write_str(&out).await;
	}

	let mut out = String::new();
	let deps = info::cargo::dependencies();
	writeln!(out, "| name | version | features |")?;
	writeln!(out, "| ---- | ------- | -------- |")?;
	for (name, dep) in deps {
		let version = dep.try_req().unwrap_or("*");
		let feats = dep.req_features();
		let feats = if !feats.is_empty() {
			feats.join(" ")
		} else {
			String::new()
		};

		writeln!(out, "| {name} | {version} | {feats} |")?;
	}

	self.write_str(&out).await
}
