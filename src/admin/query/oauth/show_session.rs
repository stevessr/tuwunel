use tuwunel_core::Result;
use tuwunel_service::oauth::SessionId;

use crate::admin_command;

#[admin_command]
pub(super) async fn oauth_show_session(&self, id: SessionId) -> Result {
	let session = self.services.oauth.sessions.get(&id).await?;

	self.write_str(&format!("{session:#?}\n")).await
}
