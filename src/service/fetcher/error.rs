//! Internal failure type for a fetch.

use std::fmt;

use ruma::OwnedServerName;
use tuwunel_core::err;

/// Internal failure shape. Kept `Clone` so it can ride the shared-result
/// channel to every coalesced caller; converted to [`tuwunel_core::Error`] at
/// the public boundary. Carries the servers tried for the operator-facing
/// message.
#[derive(Clone, Debug)]
pub(super) enum Failure {
	/// Every candidate was tried and none returned a valid response.
	NotFound {
		attempted: Vec<OwnedServerName>,
	},

	/// No candidate servers were available to try.
	NoCandidates,

	/// All callers dropped the future before a server answered.
	Cancelled,
}

impl fmt::Display for Failure {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			| Self::NotFound { attempted } => {
				write!(f, "event not found on any of {} servers", attempted.len())
			},
			| Self::NoCandidates => write!(f, "no candidate servers available"),
			| Self::Cancelled => write!(f, "fetch cancelled"),
		}
	}
}

impl From<Failure> for tuwunel_core::Error {
	fn from(failure: Failure) -> Self { err!(Request(NotFound("{failure}"))) }
}
