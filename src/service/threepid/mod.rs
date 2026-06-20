mod binding;
mod canonical;
mod pending;

use std::sync::Arc;

use ruma::{MilliSecondsSinceUnixEpoch, thirdparty::Medium};
use serde::{Deserialize, Serialize};
use tuwunel_core::Result;
use tuwunel_database::Map;

pub use self::{canonical::canonicalize_email, pending::PendingOutcome};

/// Third-party identifier (email) storage. Holds the forward `(user, email)`
/// bindings, the reverse `email -> user` lookup, and the pending email
/// verification sessions. Storage only; no SMTP or HTTP surface.
pub struct Service {
	db: Data,
}

struct Data {
	userid_email: Arc<Map>,
	email_userid: Arc<Map>,
	threepidsid_pending: Arc<Map>,
}

/// CBOR value of a `userid_email` row: the per-binding metadata, with the
/// address carried in the composite key.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Binding {
	medium: Medium,
	validated_at: MilliSecondsSinceUnixEpoch,
	added_at: MilliSecondsSinceUnixEpoch,
}

/// Validated `(medium, address)` pair handed back when a pending verification
/// is consumed by the add flow.
#[derive(Clone, Debug)]
pub struct Association {
	pub medium: Medium,
	pub address: String,
}

impl crate::Service for Service {
	fn build(args: &crate::Args<'_>) -> Result<Arc<Self>> {
		Ok(Arc::new(Self {
			db: Data {
				userid_email: args.db["userid_email"].clone(),
				email_userid: args.db["email_userid"].clone(),
				threepidsid_pending: args.db["threepidsid_pending"].clone(),
			},
		}))
	}

	fn name(&self) -> &str { crate::service::make_name(std::module_path!()) }
}
