use futures::StreamExt;
use tuwunel_core::Result;

use crate::admin_command;

#[admin_command]
pub(super) async fn list(&self) -> Result {
	let tokens: Vec<_> = self
		.services
		.registration_tokens
		.iterate_tokens()
		.collect()
		.await;

	self.write_str(&format!("Found {} registration tokens:\n", tokens.len()))
		.await?;

	for token in tokens {
		self.write_str(&format!("- {token}\n")).await?;
	}

	Ok(())
}
