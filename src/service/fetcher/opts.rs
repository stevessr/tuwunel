//! Caller contract and result types for a fetch: [`Opts`] in, [`Outcome`] out.
//!
//! [`Op`] selects the federation endpoint and folds into the single-flight
//! [`Key`]; [`Failure`] is the internal error converted at the public edge.

use std::{fmt, num::NonZeroUsize};

use bytes::Bytes;
use ruma::{OwnedEventId, OwnedRoomId, OwnedServerName, RoomVersionId};
use tuwunel_core::err;

/// Federation endpoint a fetch targets. The dedup key folds this in, so two
/// callers asking for the same event over different endpoints do not coalesce.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Op {
	/// `GET /_matrix/federation/v1/event/{eventId}`
	Event,

	/// `GET /_matrix/federation/v1/event/{eventId}` for an event fetched while
	/// reconstructing an auth chain; routed like [`Op::Event`] but pins the
	/// room's authority server ahead of the popularity ranking.
	AuthEvent,

	/// `GET /_matrix/federation/v1/event_auth/{roomId}/{eventId}`
	AuthChain,

	/// `GET /_matrix/federation/v1/backfill/{roomId}`
	Backfill,

	/// `GET /_matrix/federation/v1/state_ids/{roomId}?event_id=`
	StateIds,

	/// `POST /_matrix/federation/v1/get_missing_events/{roomId}`
	MissingEvents,
}

/// Per-round width schedule for staged fan-out: how many candidate servers a
/// fetch races concurrently in each escalation round, before the worker's
/// per-round ceiling and remaining-budget clamps. `Fixed(1)` (the `Opts::new`
/// default) reproduces strictly-sequential attempts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FanoutGrowth {
	/// Every round races the same width.
	Fixed(NonZeroUsize),

	/// `base`, `base + step`, `base + 2*step`, ...
	Linear {
		base: NonZeroUsize,
		step: NonZeroUsize,
	},

	/// `base`, `base * factor`, `base * factor^2`, ...  Base 1, factor 2 is the
	/// 1 -> 2 -> 4 -> 8 hedging ramp.
	Geometric {
		base: NonZeroUsize,
		factor: NonZeroUsize,
	},
}

impl FanoutGrowth {
	/// Width for round `round` (0-based). Always >= 1; saturating, so a runaway
	/// exponent cannot overflow (the candidate pool and `attempt_limit` clamp
	/// the value to something small regardless).
	#[must_use]
	pub fn round_width(self, round: usize) -> usize {
		match self {
			| Self::Fixed(width) => width.get(),
			| Self::Linear { base, step } => base
				.get()
				.saturating_add(step.get().saturating_mul(round)),
			| Self::Geometric { base, factor } => {
				let exp = u32::try_from(round).unwrap_or(u32::MAX);

				base.get()
					.saturating_mul(factor.get().saturating_pow(exp))
			},
		}
	}
}

/// Caller contract. `event_id` is the sought datum for [`Op::Event`] /
/// [`Op::AuthEvent`] / [`Op::AuthChain`] / [`Op::StateIds`] and a reference
/// point for the others.
#[derive(Clone, Debug)]
pub struct Opts {
	pub op: Op,
	pub room_id: OwnedRoomId,
	pub event_id: Option<OwnedEventId>,
	pub hint: Option<OwnedServerName>,
	pub room_version: Option<RoomVersionId>,
	pub attempt_limit: Option<NonZeroUsize>,
	pub backfill_limit: Option<NonZeroUsize>,

	/// Per-round width curve for staged fan-out. `Fixed(1)` is sequential.
	pub fanout_growth: FanoutGrowth,

	/// Per-round concurrency ceiling. `None` lets the curve run free, clamped
	/// only by the candidate pool and `attempt_limit`; `Some(n)` caps each
	/// round at `n`.
	pub fanout_max_width: Option<NonZeroUsize>,

	/// Cap on escalation rounds before giving up. `None` runs until exhaustion.
	pub fanout_rounds: Option<NonZeroUsize>,

	pub check_event_id: bool,
	pub check_conforms: bool,
	pub check_hashes: bool,
	// Accepted but not yet consulted; redaction-aware hash verification is unimplemented.
	pub authoritative_redaction: bool,
	pub check_signature: bool,
}

impl Opts {
	/// All validation toggles default on; the caller relaxes them per request.
	#[must_use]
	pub fn new(op: Op, room_id: OwnedRoomId) -> Self {
		Self {
			op,
			room_id,
			event_id: None,
			hint: None,
			room_version: None,
			attempt_limit: None,
			backfill_limit: None,
			fanout_growth: FanoutGrowth::Fixed(NonZeroUsize::MIN),
			fanout_max_width: None,
			fanout_rounds: None,
			check_event_id: true,
			check_conforms: true,
			check_hashes: true,
			authoritative_redaction: true,
			check_signature: true,
		}
	}

	#[must_use]
	pub fn event_id(self, event_id: OwnedEventId) -> Self {
		Self { event_id: Some(event_id), ..self }
	}

	#[must_use]
	pub fn hint(self, hint: OwnedServerName) -> Self { Self { hint: Some(hint), ..self } }

	/// Room version for [`Op::Event`] id and signature checks. `None` keeps the
	/// V11 default, so callers on a non-V11 room must name it to avoid a
	/// spurious rejection.
	#[must_use]
	pub fn room_version(self, room_version: RoomVersionId) -> Self {
		Self { room_version: Some(room_version), ..self }
	}

	#[must_use]
	pub fn attempt_limit(self, attempt_limit: NonZeroUsize) -> Self {
		Self {
			attempt_limit: Some(attempt_limit),
			..self
		}
	}

	#[must_use]
	pub fn fanout(self, growth: FanoutGrowth) -> Self { Self { fanout_growth: growth, ..self } }

	#[must_use]
	pub fn fanout_max_width(self, max_width: NonZeroUsize) -> Self {
		Self {
			fanout_max_width: Some(max_width),
			..self
		}
	}

	#[must_use]
	pub fn fanout_rounds(self, rounds: NonZeroUsize) -> Self {
		Self { fanout_rounds: Some(rounds), ..self }
	}

	/// Apply the op's advised staged-fan-out ramp. `Opts::new` is otherwise
	/// dark on every op, so a callsite opts in by chaining this; the generic
	/// and single-shot-batch ops keep the sequential default.
	#[must_use]
	pub fn fanout_for_op(self) -> Self {
		use FanoutGrowth::{Geometric, Linear};

		const ONE: NonZeroUsize = NonZeroUsize::new(1).unwrap();
		const TWO: NonZeroUsize = NonZeroUsize::new(2).unwrap();
		const THREE: NonZeroUsize = NonZeroUsize::new(3).unwrap();
		const FOUR: NonZeroUsize = NonZeroUsize::new(4).unwrap();
		const FIVE: NonZeroUsize = NonZeroUsize::new(5).unwrap();

		match self.op {
			| Op::AuthEvent => self
				.fanout(Geometric { base: ONE, factor: TWO })
				.fanout_max_width(FOUR)
				.fanout_rounds(FIVE),
			| Op::AuthChain => self
				.fanout(Linear { base: ONE, step: ONE })
				.fanout_max_width(TWO)
				.fanout_rounds(TWO),
			| Op::StateIds => self
				.fanout(Linear { base: ONE, step: ONE })
				.fanout_max_width(THREE)
				.fanout_rounds(THREE),
			| Op::MissingEvents => self
				.fanout(Geometric { base: ONE, factor: TWO })
				.fanout_rounds(THREE),
			| Op::Event | Op::Backfill => self,
		}
	}

	/// Toggle every validation gate at once. Callers that re-validate
	/// downstream pass `false` to fetch raw bytes without rejecting non-V11
	/// events.
	#[must_use]
	pub fn checks(self, enabled: bool) -> Self {
		Self {
			check_event_id: enabled,
			check_conforms: enabled,
			check_hashes: enabled,
			check_signature: enabled,
			..self
		}
	}
}

/// Raw response body plus the server that answered. `bytes` is ref-counted so
/// concurrent callers coalesced onto one fetch share a single buffer.
#[derive(Debug)]
pub struct Outcome {
	pub bytes: Bytes,
	pub origin: OwnedServerName,
}

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

/// Single-flight dedup key. `MissingEvents` does not coalesce on a single
/// event_id (OQ6); a body hash will fold in when that op gains a multi-id
/// request body in a later phase.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(super) struct Key {
	pub(super) op: Op,
	pub(super) room_id: OwnedRoomId,
	pub(super) event_id: Option<OwnedEventId>,
}

impl Key {
	pub(super) fn new(opts: &Opts) -> Self {
		Self {
			op: opts.op,
			room_id: opts.room_id.clone(),
			event_id: opts.event_id.clone(),
		}
	}
}
