use futures::StreamExt;
use tuwunel_core::Result;

use crate::admin_command;

#[admin_command]
pub(super) async fn list_users(&self) -> Result {
	let users: Vec<_> = self
		.services
		.users
		.list_local_users()
		.map(ToString::to_string)
		.collect()
		.await;

	let mut plain_msg = format!("Found {} local user account(s):\n```\n", users.len());
	plain_msg += users.join("\n").as_str();
	plain_msg += "\n```";

	self.write_str(&plain_msg).await
}
