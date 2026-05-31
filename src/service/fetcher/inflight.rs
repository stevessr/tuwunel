//! Single-flight bookkeeping for one in-flight fetch.
//!
//! [`Key`] is the dedup key the worker's in-flight map is keyed on;
//! [`Inflight`] is the worker-owned entry every coalesced caller subscribes to;
//! [`SharedResult`] is the broadcast outcome and [`Subscription`] the caller's
//! handle, whose liveness token cancels the fetch on drop.

use std::sync::{Arc, Weak};

use ruma::{OwnedEventId, OwnedRoomId};
use tokio::sync::watch::{Receiver, Sender};

use super::{Failure, Op, Opts, Outcome};

/// Single-flight dedup key. `MissingEvents` does not coalesce on a single
/// event_id (OQ6); a body hash will fold in when that op gains a multi-id
/// request body in a later phase.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(super) struct Key {
	/// Endpoint class; two ops over the same event do not coalesce.
	pub(super) op: Op,

	/// Room the event belongs to.
	pub(super) room_id: OwnedRoomId,

	/// Sought event, or `None` for ops that do not key on one.
	pub(super) event_id: Option<OwnedEventId>,
}

/// Outcome shared by every caller coalesced onto one fetch. Cheap to clone so
/// the worker can broadcast it down each subscriber's channel.
pub(super) type SharedResult = Result<Arc<Outcome>, Failure>;

/// Reply handed to a caller: the channel it awaits the outcome on, plus the
/// sole strong liveness token whose drop cancels the in-flight fetch.
pub(super) type Subscription = (Receiver<Option<SharedResult>>, Arc<()>);

/// One in-flight fetch, owned by the worker. The worker is the sole mutator, so
/// no lock guards it; coalesced callers reach it only through their channels.
pub(super) struct Inflight {
	/// Result channel. Coalesced callers subscribe to await the outcome.
	pub(super) tx: Sender<Option<SharedResult>>,

	/// Liveness signal. The strong token rides to the callers; the worker holds
	/// this weak ref and the fetch bails once it can no longer upgrade it.
	pub(super) interest: Weak<()>,

	/// Retained (shared) so a re-armed key re-dispatches without re-cloning it.
	pub(super) opts: Arc<Opts>,
}

impl Key {
	/// Derive the single-flight key from a request's [`Opts`].
	pub(super) fn new(opts: &Opts) -> Self {
		Self {
			op: opts.op,
			room_id: opts.room_id.clone(),
			event_id: opts.event_id.clone(),
		}
	}
}
