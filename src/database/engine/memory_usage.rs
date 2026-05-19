use std::{ffi::CStr, fmt::Write};

use rocksdb::perf::get_memory_usage_stats;
use tuwunel_core::{Result, implement};

use super::{
	Engine,
	context::{ColCache, SHARED_POOL},
};
use crate::or_else;

fn mib(input: u64) -> f64 { f64::from(u32::try_from(input / 1024).unwrap_or(0)) / 1024.0 }

/// Multi-line cache-entry stats blob. Live variant forces a fresh
/// collection per call; the "fast" variant returns a cached snapshot and is
/// unsuitable for admin diagnostics since it never refreshes within a
/// session. The string surface is what `property_value_cf` returns; the
/// structured map form is C++-only.
const ENTRY_STATS_PROPERTY: &str = "rocksdb.block-cache-entry-stats";

/// Per-CF byte count of the block cache the CF is using. Returns the same
/// value for every participant of a shared pool.
const CACHE_CAPACITY_PROPERTY: &CStr = c"rocksdb.block-cache-capacity";

#[implement(Engine)]
pub fn memory_usage(&self) -> Result<String> {
	let mut res = String::new();
	let row_cache = self.ctx.row_cache.lock()?;
	let row_usage = u64::try_from(row_cache.get_usage())?;
	let row_capacity = u64::try_from(self.ctx.row_cache_capacity)?;
	let stats =
		get_memory_usage_stats(Some(&[&self.db]), Some(&[&*row_cache])).or_else(or_else)?;

	writeln!(res, "- Memory buffers: {:.2} MiB", mib(stats.mem_table_total))?;
	writeln!(res, "- Pending write: {:.2} MiB", mib(stats.mem_table_unflushed))?;
	writeln!(res, "- Table readers: {:.2} MiB", mib(stats.mem_table_readers_total))?;
	writeln!(
		res,
		"- Row cache: {:.2} / {:.2} MiB ({:.1}%)",
		mib(row_usage),
		mib(row_capacity),
		utilization_percent(row_usage, row_capacity),
	)?;

	drop(row_cache);

	let pools = self.ctx.col_cache.lock()?;
	if pools.is_empty() {
		return Ok(res);
	}

	writeln!(res, "- Block cache pools:")?;
	for (name, pool) in &*pools {
		self.write_pool(&mut res, name, pool)?;
	}

	Ok(res)
}

#[implement(Engine)]
fn write_pool(&self, out: &mut String, name: &str, pool: &ColCache) -> Result {
	let label = if name == SHARED_POOL { "Shared" } else { name };
	let pinned = u64::try_from(pool.cache.get_pinned_usage())?;
	let usage = u64::try_from(pool.cache.get_usage())?;
	let capacity = pool
		.participants
		.first()
		.copied()
		.map(|cf_name| self.cf(cf_name))
		.and_then(|cf| {
			self.property_integer(&cf, CACHE_CAPACITY_PROPERTY)
				.ok()
		})
		.unwrap_or(0);

	writeln!(out, "  - {label}")?;
	writeln!(
		out,
		"    - Usage: {:.2} / {:.2} MiB ({:.1}%)",
		mib(usage),
		mib(capacity),
		utilization_percent(usage, capacity),
	)?;
	writeln!(out, "    - Pinned: {:.2} MiB", mib(pinned))?;

	let multi_cf = pool.participants.len() != 1 || pool.participants[0] != name;
	if multi_cf {
		writeln!(out, "    - Participants: {}", pool.participants.join(", "))?;
	}

	if let Some(repr) = pool.participants.first() {
		self.write_entry_stats(out, repr)?;
	}

	Ok(())
}

#[implement(Engine)]
fn write_entry_stats(&self, out: &mut String, cf_name: &str) -> Result {
	let cf = self.cf(cf_name);
	let raw = self.property(&cf, ENTRY_STATS_PROPERTY)?;
	let Some(by_role) = raw
		.lines()
		.find(|line| line.starts_with("Block cache entry stats"))
	else {
		return Ok(());
	};

	let Some((_, roles)) = by_role.split_once(": ") else {
		return Ok(());
	};

	writeln!(out, "    - Entries:")?;
	for role in roles.trim_end_matches(')').split(") ") {
		let pretty = role.replacen('(', ": ", 1).replace(',', " ");
		writeln!(out, "      - {pretty}")?;
	}

	Ok(())
}

#[expect(clippy::as_conversions, clippy::cast_precision_loss)]
fn utilization_percent(usage: u64, capacity: u64) -> f64 {
	if capacity == 0 {
		return 0.0;
	}

	(usage as f64 / capacity as f64) * 100.0
}
