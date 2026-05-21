use tuwunel_core::Result;

use crate::admin_command;

#[admin_command]
pub(super) async fn list_connections(&self) -> Result {
	let connections = self.services.sync.list_loaded_connections().await;

	for connection_key in connections {
		self.write_str(&format!("{connection_key:?}\n"))
			.await?;
	}

	Ok(())
}
