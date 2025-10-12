use std::collections::BTreeMap;

use futures::{FutureExt, StreamExt, TryFutureExt};
use ruma::{
	api::client::sync::sync_events::v5::response,
	events::typing::{SyncTypingEvent, TypingEventContent},
	serde::Raw,
};
use tuwunel_core::{
	Result, debug_error,
	utils::{IterStream, ReadyExt},
};

use super::{Connection, SyncInfo, Window, extension_rooms_selector};

#[tracing::instrument(level = "trace", skip_all)]
pub(super) async fn collect(
	sync_info: SyncInfo<'_>,
	conn: &Connection,
	window: &Window,
) -> Result<response::Typing> {
	use response::Typing;

	let SyncInfo { services, sender_user, .. } = sync_info;

	let implicit = conn
		.extensions
		.typing
		.lists
		.as_deref()
		.map(<[_]>::iter);

	let explicit = conn
		.extensions
		.typing
		.rooms
		.as_deref()
		.map(<[_]>::iter);

	extension_rooms_selector(sync_info, conn, window, implicit, explicit)
		.stream()
		.filter_map(async |room_id| {
			services
				.typing
				.typing_users_for_user(room_id, sender_user)
				.inspect_err(|e| debug_error!(%room_id, "Failed to get typing events: {e}"))
				.await
				.ok()
				.filter(|users| !users.is_empty())
				.map(|users| (room_id, users))
		})
		.ready_filter_map(|(room_id, users)| {
			let content = TypingEventContent::new(users);
			let event = SyncTypingEvent { content };
			let event = Raw::new(&event);

			Some((room_id.to_owned(), event.ok()?))
		})
		.collect::<BTreeMap<_, _>>()
		.map(|rooms| Typing { rooms })
		.map(Ok)
		.await
}
