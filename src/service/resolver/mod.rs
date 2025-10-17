pub mod actual;
pub mod cache;
mod dns;
pub mod fed;
#[cfg(test)]
mod tests;
mod well_known;

use std::sync::Arc;

use async_trait::async_trait;
use tuwunel_core::{Result, arrayvec::ArrayString, utils::MutexMap};

use self::{cache::Cache, dns::Resolver};

pub struct Service {
	pub cache: Arc<Cache>,
	pub resolver: Arc<Resolver>,
	resolving: Resolving,
	services: Arc<crate::services::OnceServices>,
}

type Resolving = MutexMap<NameBuf, ()>;
type NameBuf = ArrayString<256>;

#[async_trait]
impl crate::Service for Service {
	#[allow(
		clippy::as_conversions,
		clippy::cast_sign_loss,
		clippy::cast_possible_truncation
	)]
	fn build(args: &crate::Args<'_>) -> Result<Arc<Self>> {
		let cache = Cache::new(args);
		Ok(Arc::new(Self {
			cache: cache.clone(),
			resolver: Resolver::build(args.server, cache)?,
			resolving: MutexMap::new(),
			services: args.services.clone(),
		}))
	}

	async fn clear_cache(&self) {
		self.resolver.clear_cache();
		self.cache.clear().await;
	}

	fn name(&self) -> &str { crate::service::make_name(std::module_path!()) }
}
