use std::sync::Arc;

use object_store::local::LocalFileSystem;
use tuwunel_core::{
	Result,
	config::{StorageProvider, StorageProviderLocal},
	debug, debug_info, error, trace,
};

use super::Provider;

#[tracing::instrument(name = "new", level = "debug", skip_all, err)]
pub(in super::super) fn new(
	args: &crate::Args<'_>,
	name: &str,
	config: &StorageProviderLocal,
) -> Result<Option<(String, Arc<Provider>)>> {
	// Fail successfully if this provider is disabled by the configuration..
	if config.base_path.is_empty() {
		debug!(?name, "s3_provider.bucket not set. This configuration will be skipped");
		return Ok(None);
	}

	trace!(?name, ?config, "Initializing LocalFS...");

	let provider = LocalFileSystem::new_with_prefix(config.base_path.clone())
		.inspect_err(|e| error!("Failed to configure S3 storage client: {e}"))?
		.with_automatic_cleanup(config.delete_empty_directories);

	debug_info!(
		name = %name,
		path = ?config.base_path,
		"Started Local FS storage client.",
	);

	let provider = Provider {
		name: name.to_owned(),
		base_path: Some(config.base_path.clone().into()),
		config: StorageProvider::local(config.clone()),
		services: args.services.clone(),
		provider: Box::new(provider),
	};

	Ok(Some((name.to_owned(), Arc::new(provider))))
}
