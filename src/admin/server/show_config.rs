use tuwunel_core::Result;

use crate::admin_command;

#[admin_command]
pub(super) async fn show_config(&self) -> Result {
	self.write_str(&format!("{}", *self.services.server.config))
		.await
}
