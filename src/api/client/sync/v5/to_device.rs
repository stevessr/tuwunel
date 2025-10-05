use futures::StreamExt;
use ruma::api::client::sync::sync_events::v5::response;
use tuwunel_core::{self, Result};
use tuwunel_service::Services;

use super::SyncInfo;

#[tracing::instrument(level = "trace", skip_all, fields(globalsince, next_batch))]
pub(super) async fn collect(
	services: &Services,
	(sender_user, sender_device, globalsince, _request): SyncInfo<'_>,
	next_batch: u64,
) -> Result<Option<response::ToDevice>> {
	services
		.users
		.remove_to_device_events(sender_user, sender_device, globalsince)
		.await;

	let events: Vec<_> = services
		.users
		.get_to_device_events(sender_user, sender_device, None, Some(next_batch))
		.collect()
		.await;

	let to_device = events
		.is_empty()
		.eq(&false)
		.then(|| response::ToDevice {
			next_batch: next_batch.to_string(),
			events,
		});

	Ok(to_device)
}
