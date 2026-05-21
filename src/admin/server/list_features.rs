use std::fmt::Write;

use tuwunel_core::{Result, info};

use crate::admin_command;

#[admin_command]
pub(super) async fn list_features(&self, available: bool, enabled: bool, comma: bool) -> Result {
	let delim = if comma { "," } else { " " };
	if enabled && !available {
		let features = info::rustc::features().join(delim);
		let out = format!("`\n{features}\n`");
		return self.write_str(&out).await;
	}

	if available && !enabled {
		let features = info::cargo::features().join(delim);
		let out = format!("`\n{features}\n`");
		return self.write_str(&out).await;
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

	self.write_str(&features).await
}
