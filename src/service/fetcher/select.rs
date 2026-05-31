//! Candidate server selection for a fetch.
//!
//! The [`Select`] seam enumerates the server pool; [`RoomCandidates`] derives
//! it from room state and orders it by population, pinning the room's authority
//! server ahead of the ranking for auth fetches.

use std::{collections::BTreeSet, sync::Arc};

use async_trait::async_trait;
use futures::{Stream, StreamExt, future::Either};
use ruma::{EventId, OwnedServerName, ServerName};
use tuwunel_core::{
	arrayvec::ArrayVec,
	implement,
	utils::{BoolExt, IterStream, ReadyExt, StreamTools, rand::index},
};

use super::{Op, Opts};
use crate::{
	federation::{Candidates, WhenAllBackedOff},
	services::OnceServices,
};

/// Population-ranked servers kept per fetch; bounds the serial federation
/// fan-out per missing event.
const ROUTE_FANOUT: usize = 5;

/// Candidate enumeration seam. The production impl derives the server pool from
/// room state; tests substitute a fixed list.
#[async_trait]
pub(super) trait Select: Send + Sync {
	async fn candidates(&self, opts: &Opts) -> Candidates;
}

pub(super) struct RoomCandidates {
	pub(super) services: Arc<OnceServices>,
}

#[async_trait]
impl Select for RoomCandidates {
	#[tracing::instrument(
		level = "trace",
		skip_all,
		fields(
			room_id = %opts.room_id,
		),
	)]
	async fn candidates(&self, opts: &Opts) -> Candidates {
		if !opts.candidates.is_empty() {
			return self.ranked_override(opts).await;
		}

		let authority = self.authority_server(opts).await;

		let mxid_hosts = [
			opts.event_id
				.as_deref()
				.and_then(EventId::server_name),
			opts.room_id.server_name(),
		]
		.into_iter()
		.flatten()
		.map(ToOwned::to_owned);

		let ordered: Candidates = opts
			.hint
			.clone()
			.into_iter()
			.chain(authority)
			.stream()
			.chain(self.route_by_popularity(opts).await)
			.chain(mxid_hosts.stream())
			.ready_filter(|server| self.is_eligible(server))
			.collect()
			.await;

		self.rank_unique(ordered).await
	}
}

/// Rank a caller-supplied candidate pool in place of the room-derived one,
/// filtering the ineligible (our own server, forbidden remotes). The hint, if
/// any, still leads.
#[implement(RoomCandidates)]
#[tracing::instrument(level = "trace", skip_all)]
async fn ranked_override(&self, opts: &Opts) -> Candidates {
	let ordered: Candidates = opts
		.hint
		.iter()
		.chain(opts.candidates.iter())
		.filter(|&server| self.is_eligible(server))
		.cloned()
		.collect();

	self.rank_unique(ordered).await
}

/// Dedup an assembled candidate list in place, then order it by peer-status
/// reachability.
#[implement(RoomCandidates)]
async fn rank_unique(&self, mut ordered: Candidates) -> Candidates {
	let mut seen = BTreeSet::new();
	ordered.retain(|server| seen.insert(server.clone()));

	self.services
		.federation
		.rank_candidates(ordered, WhenAllBackedOff::Attempt)
		.await
}

/// The room's most-powerful server, pinned ahead of the population ranking
/// for auth-event and auth-chain fetches only.
#[implement(RoomCandidates)]
#[tracing::instrument(level = "trace", skip_all)]
async fn authority_server(&self, opts: &Opts) -> Option<OwnedServerName> {
	matches!(opts.op, Op::AuthEvent | Op::AuthChain)
		.then_async(|| {
			self.services
				.state_cache
				.most_powerful_user_server(&opts.room_id)
		})
		.await
		.flatten()
}

/// Participating servers sampled in proportion to their resident member
/// count: each draw lands on a random member, so its server appears with
/// probability proportional to that server's population. Populous servers
/// therefore lead more often, without ranking the whole membership. Falls
/// back to the participating-server set when the room has no resident
/// members.
#[implement(RoomCandidates)]
#[tracing::instrument(level = "trace", skip_all)]
async fn route_by_popularity<'a>(
	&'a self,
	opts: &'a Opts,
) -> impl Stream<Item = OwnedServerName> + Send + 'a {
	let sampled: ArrayVec<OwnedServerName, ROUTE_FANOUT> = self
		.services
		.state_cache
		.room_members(&opts.room_id)
		.sample_by(|user| user.server_name().to_owned())
		.await;

	if sampled.is_empty() {
		return Either::Right(
			self.services
				.state_cache
				.room_servers(&opts.room_id)
				.map(ToOwned::to_owned),
		);
	}

	Either::Left(sampled.into_iter().stream())
}

/// Uniform-random window over the participating-server cursor: count, skip
/// a uniform offset, then take a small run. Fully lazy, with no popularity
/// aggregation.
#[implement(RoomCandidates)]
#[allow(dead_code)]
async fn route_uniformly<'a>(
	&'a self,
	opts: &'a Opts,
) -> impl Stream<Item = OwnedServerName> + Send + 'a {
	let count = self
		.services
		.state_cache
		.room_servers(&opts.room_id)
		.count()
		.await;

	let offset = index(count);

	self.services
		.state_cache
		.room_servers(&opts.room_id)
		.map(ToOwned::to_owned)
		.skip(offset)
		.take(ROUTE_FANOUT)
}

#[implement(RoomCandidates)]
fn is_eligible(&self, server: &ServerName) -> bool {
	!self.services.globals.server_is_ours(server)
		&& !self
			.services
			.server
			.config
			.is_forbidden_remote_server_name(server)
}
