pub mod local;
pub mod s3;

use std::sync::Arc;

use futures::{FutureExt, Stream, TryFutureExt, TryStreamExt};
use object_store::{
	CopyMode, DynObjectStore, GetResult, ObjectMeta, ObjectStore, ObjectStoreExt, PutPayload,
	PutResult,
	path::{Path, PathPart},
};
use tuwunel_core::{
	Error, Result, config::StorageProvider, debug, derivative::Derivative, err, error, implement,
	info,
};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Provider {
	pub name: String,
	pub config: StorageProvider,
	pub provider: Box<DynObjectStore>,
	pub(crate) path: Option<String>,
	#[expect(unused)]
	#[derivative(Debug = "ignore")]
	services: Arc<crate::services::OnceServices>,
}

#[implement(Provider)]
#[tracing::instrument(skip_all, err)]
pub(super) async fn start(&self) -> Result {
	debug!(
		name = ?self.name,
		"Checking storage provider client connection..."
	);

	self.ping().await?;

	info!(
		name = %self.name,
		"Connected to storage provider"
	);

	Ok(())
}

#[implement(Provider)]
pub async fn put(&self, path: &str, payload: PutPayload) -> Result<PutResult> {
	let path = self.path(path)?;

	self.provider
		.put(&path, payload)
		.map_err(Error::from)
		.await
}

#[implement(Provider)]
pub async fn get(&self, path: &str) -> Result<GetResult> {
	let path = self.path(path)?;

	self.provider
		.get(&path)
		.map_err(Error::from)
		.await
}

#[implement(Provider)]
pub async fn delete(&self, path: &str) -> Result {
	self.provider
		.delete(&self.path(path)?)
		.map_err(Error::from)
		.await
}

#[implement(Provider)]
pub async fn rename(&self, src: &str, dst: &str, overwrite: CopyMode) -> Result {
	let src = self.path(src)?;
	let dst = self.path(dst)?;

	match overwrite {
		| CopyMode::Overwrite => self.provider.rename(&src, &dst).left_future(),
		| CopyMode::Create => self
			.provider
			.rename_if_not_exists(&src, &dst)
			.right_future(),
	}
	.map_err(Error::from)
	.await
}

#[implement(Provider)]
pub async fn copy(&self, src: &str, dst: &str, overwrite: CopyMode) -> Result {
	let src = self.path(src)?;
	let dst = self.path(dst)?;

	match overwrite {
		| CopyMode::Overwrite => self.provider.copy(&src, &dst).left_future(),
		| CopyMode::Create => self
			.provider
			.copy_if_not_exists(&src, &dst)
			.right_future(),
	}
	.map_err(Error::from)
	.await
}

#[implement(Provider)]
pub fn list(&self, prefix: Option<&str>) -> impl Stream<Item = Result<ObjectMeta>> + Send {
	self.provider
		.list(prefix.map(Into::into).as_ref())
		.map_err(Error::from)
}

#[implement(Provider)]
pub async fn head(&self, path: &str) -> Result<ObjectMeta> {
	self.provider
		.head(&self.path(path)?)
		.map_err(Error::from)
		.await
}

#[implement(Provider)]
pub async fn ping(&self) -> Result {
	self.list(None)
		.try_next()
		.inspect_err(|e| error!("Failed to connect to storage provider: {e:?}"))
		.boxed()
		.await
		.map(|_| ())
}

#[implement(Provider)]
#[expect(clippy::iter_on_single_items)]
pub fn path<'a>(&'a self, location: &'a str) -> Result<Path> {
	let location = PathPart::parse(location)
		.map_err(|e| err!("Failed to parse location into canonical PathPart: {e}"))?;

	let base: Option<PathPart<'a>> = self
		.path
		.as_deref()
		.map(TryInto::try_into)
		.transpose()
		.map_err(Error::from)?;

	Ok([base.into_iter(), Some(location).into_iter()]
		.into_iter()
		.flatten()
		.collect())
}
