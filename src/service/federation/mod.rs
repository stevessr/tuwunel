mod execute;
mod format;
mod peer;
pub mod scheme;

use std::sync::Arc;

use tuwunel_core::Result;
use tuwunel_database::Map;

pub use self::peer::{Classification, ShouldAttempt};
use crate::services::OnceServices;

pub struct Service {
	services: Arc<OnceServices>,
	statuses: Arc<Map>,
}

impl crate::Service for Service {
	fn build(args: &crate::Args<'_>) -> Result<Arc<Self>> {
		Ok(Arc::new(Self {
			services: args.services.clone(),
			statuses: args.db["servername_status"].clone(),
		}))
	}

	fn name(&self) -> &str { crate::service::make_name(std::module_path!()) }
}
