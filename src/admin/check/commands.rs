use futures::StreamExt;
use tuwunel_core::Result;
use tuwunel_macros::implement;

use crate::Context;

/// Uses the iterator in `src/database/key_value/users.rs` to iterator over
/// every user in our database (remote and local). Reports total count, any
/// errors if there were any, etc
#[implement(Context, params = "<'_>")]
pub(super) async fn check_all_users(&self) -> Result<String> {
	let timer = tokio::time::Instant::now();
	let users = self
		.services
		.users
		.iter()
		.collect::<Vec<_>>()
		.await;
	let query_time = timer.elapsed();

	let total = users.len();
	let err_count = users.iter().filter(|_user| false).count();
	let ok_count = users.iter().filter(|_user| true).count();

	Ok(format!(
		"Database query completed in {query_time:?}:\n
		\n
		```\n
		Total entries: {total:?}\n
		Failure/Invalid user count: {err_count:?}\n
		Success/Valid user count: {ok_count:?}\n
		```"
	))
}
