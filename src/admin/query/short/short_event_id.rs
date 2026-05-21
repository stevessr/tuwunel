use ruma::OwnedEventId;
use tuwunel_core::Result;

use crate::admin_command;

#[admin_command]
pub(super) async fn short_event_id(&self, event_id: OwnedEventId) -> Result {
	let shortid = self
		.services
		.short
		.get_shorteventid(&event_id)
		.await?;

	self.write_str(&format!("{shortid:#?}")).await
}
