//! System utilities related to compute/processing

use std::{cell::Cell, fmt::Debug, sync::LazyLock};

use crate::is_equal_to;

type Id = usize;

type Mask = u128;

const MASK_BITS: usize = 128;

/// The mask of logical cores available to the process (at startup).
static CORES_AVAILABLE: LazyLock<Mask> = LazyLock::new(|| into_mask(query_cores_available()));

thread_local! {
	/// Tracks the affinity for this thread. This is updated when affinities
	/// are set via our set_affinity() interface.
	static CORE_AFFINITY: Cell<Mask> = const { Cell::new(0) };
}

/// Set the core affinity for this thread. The ID should be listed in
/// CORES_AVAILABLE. Empty input is a no-op; prior affinity unchanged.
#[tracing::instrument(
	level = "debug",
	skip_all,
	fields(
		id = ?std::thread::current().id(),
		name = %std::thread::current().name().unwrap_or("None"),
		set = ?ids.clone().collect::<Vec<_>>(),
		CURRENT = %format!("[b{:b}]", CORE_AFFINITY.get()),
		AVAILABLE = %format!("[b{:b}]", *CORES_AVAILABLE),
	),
)]
pub fn set_affinity<I>(mut ids: I)
where
	I: Iterator<Item = Id> + Clone + Debug,
{
	use core_affinity::{CoreId, set_each_for_current, set_for_current};

	let n = ids.clone().count();
	let mask: Mask = ids.clone().fold(0, |mask, id| {
		debug_assert!(is_core_available(id), "setting affinity to unavailable core");
		mask | (1 << id)
	});

	if n > 1 {
		set_each_for_current(ids.map(|id| CoreId { id }));
	} else if n > 0 {
		set_for_current(CoreId { id: ids.next().expect("n > 0") });
	}

	if mask.count_ones() > 0 {
		CORE_AFFINITY.replace(mask);
	}
}

/// Get the core affinity for this thread.
pub fn get_affinity() -> impl Iterator<Item = Id> { from_mask(CORE_AFFINITY.get()) }

/// Get the number of threads which could execute in parallel based on hardware
/// constraints of this system.
#[inline]
#[must_use]
pub fn available_parallelism() -> usize { cores_available().count() }

/// Gets the ID of the nth core available. This bijects our sequence of cores to
/// actual ID's which may have gaps for cores which are not available.
#[inline]
#[must_use]
pub fn nth_core_available(i: usize) -> Option<Id> { cores_available().nth(i) }

/// Determine if core (by id) is available to the process.
#[inline]
#[must_use]
pub fn is_core_available(id: Id) -> bool { cores_available().any(is_equal_to!(id)) }

/// Get the list of cores available. The values were recorded at program start.
#[inline]
pub fn cores_available() -> impl Iterator<Item = Id> { from_mask(*CORES_AVAILABLE) }

fn query_cores_available() -> impl Iterator<Item = Id> {
	core_affinity::get_core_ids()
		.unwrap_or_default()
		.into_iter()
		.map(|core_id| core_id.id)
}

fn into_mask<I>(ids: I) -> Mask
where
	I: Iterator<Item = Id>,
{
	ids.inspect(|&id| {
		debug_assert!(id < MASK_BITS, "Core ID must be < Mask::BITS at least for now");
	})
	.fold(Mask::default(), |mask, id| mask | (1 << id))
}

fn from_mask(v: Mask) -> impl Iterator<Item = Id> {
	(0..MASK_BITS).filter(move |&i| (v & (1 << i)) != 0)
}
