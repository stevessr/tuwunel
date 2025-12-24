use std::{path::PathBuf, sync::Arc};

use tuwunel_core::{
	Server, debug,
	debug::INFO_SPAN_LEVEL,
	debug_info, debug_warn, expected, info, is_equal_to,
	utils::{
		BoolExt,
		math::usize_from_f64,
		result::LogDebugErr,
		stream,
		stream::{AMPLIFICATION_LIMIT, WIDTH_LIMIT},
		sys::{
			compute::{CORES_MAX, available_parallelism, is_core_available},
			storage,
		},
	},
};

use super::{QUEUE_LIMIT, WORKER_LIMIT};

#[tracing::instrument(
	level = INFO_SPAN_LEVEL,
	skip_all,
	ret(level = "trace"),
)]
pub(super) fn configure(server: &Arc<Server>) -> (usize, Vec<usize>, Vec<usize>) {
	let config = &server.config;
	let num_cores = available_parallelism();

	// This finds the block device and gathers all the properties we need.
	let path: PathBuf = config.database_path.clone();
	let device_name = storage::name_from_path(&path)
		.log_debug_err()
		.ok();

	let devices = storage::md_discover(&path);
	let topology_detected = devices.md.is_empty().is_false();
	debug!(?topology_detected, ?device_name, ?devices);

	// The default worker count is masked-on if we didn't find better information.
	let default_worker_count = topology_detected
		.is_false()
		.then_some(config.db_pool_workers);

	// Sum the total number of possible tags. When no hardware detected this will
	// default to the default_worker_count
	let total_tags = devices
		.md
		.iter()
		.flat_map(|md| md.mq.iter())
		.filter(|mq| mq.cpu_list.iter().copied().any(is_core_available))
		.filter_map(|mq| mq.nr_tags)
		.chain(default_worker_count)
		.fold(0_usize, usize::saturating_add);

	// Determine the worker groupings. Each indice represents a hardware queue and
	// contains the number of workers which will service it. This vector is
	// truncated to the number of cores on systems which have multiple hardware
	// queues per core. When no hardware is detected this defaults to one queue with
	// a default count of workers.
	let worker_counts: Vec<_> = devices
		.md
		.iter()
		.inspect(|md| debug!(?md))
		.flat_map(|md| md.mq.iter())
		.filter(|mq| mq.cpu_list.iter().copied().any(is_core_available))
		.map(|mq| {
			let shares = mq
				.cpu_list
				.iter()
				.filter(|&&id| is_core_available(id))
				.count()
				.max(1);

			let limit = config
				.db_pool_workers_limit
				.saturating_mul(shares);

			let limit = devices.md.iter().fold(0_usize, |acc, mq| {
				mq.nr_requests
					.map(|nr| nr.min(limit))
					.or(Some(limit))
					.map(|nr| acc.saturating_add(nr))
					.unwrap_or(acc)
			});

			debug!(?mq, ?shares, ?limit);

			mq.nr_tags.unwrap_or(WORKER_LIMIT.0).min(limit)
		})
		.chain(default_worker_count)
		.take(num_cores)
		.collect();

	// Determine our software queue size for each hardware queue. This is the mpmc
	// between the tokio worker and the pool worker.
	let queue_sizes: Vec<_> = worker_counts
		.iter()
		.map(|worker_count| {
			worker_count
				.saturating_mul(config.db_pool_queue_mult)
				.clamp(QUEUE_LIMIT.0, QUEUE_LIMIT.1)
		})
		.collect();

	// Determine the CPU affinities of each hardware queue. Each indice is a core
	// and each value is the associated hardware queue. On systems which share
	// queues between cores some values will be repeated; on systems with multiple
	// queues per core the affinities are assumed to match and we don't require a
	// vector of vectors. There is a little hiftiness going on because cpu's which
	// are not available to the process are filtered out, similar to the
	// worker_counts.
	let topology = devices
		.md
		.iter()
		.flat_map(|md| md.mq.iter())
		.fold(vec![0; CORES_MAX], |mut topology, mq| {
			mq.cpu_list
				.iter()
				.filter(|&&id| is_core_available(id))
				.for_each(|&id| {
					topology[id] = mq.id;
				});

			topology
		})
		.into_iter()
		.take(num_cores)
		.collect();

	// Regardless of the capacity of all queues we establish some limit on the total
	// number of workers; this is hopefully hinted by nr_requests.
	let max_workers = devices
		.md
		.iter()
		.flat_map(|md| md.mq.iter())
		.filter_map(|mq| mq.nr_tags)
		.chain(default_worker_count)
		.fold(0_usize, usize::saturating_add)
		.clamp(WORKER_LIMIT.0, WORKER_LIMIT.1);

	// Determine the final worker count which we'll be spawning.
	let total_workers = worker_counts
		.iter()
		.sum::<usize>()
		.clamp(WORKER_LIMIT.0, max_workers);

	// After computing all of the above we can update the global automatic stream
	// width, hopefully with a better value tailored to this system.
	let num_queues = queue_sizes.len();
	if config.stream_width_scale > 0.0 {
		update_stream_width(server, num_queues, total_workers, total_tags);
	}

	if topology_detected {
		debug_info!(?topology, ?worker_counts, ?queue_sizes, "Frontend topology",);
		info!(
			device_name = ?device_name.as_deref().unwrap_or("None"),
			?num_cores,
			?num_queues,
			?total_workers,
			?total_tags,
			stream_width = ?stream::automatic_width(),
			amplification = ?stream::automatic_amplification(),
			"Frontend topology",
		);
	} else {
		debug_warn!(
			device_name = ?device_name.as_deref().unwrap_or("None"),
			?total_workers,
			stream_width = ?stream::automatic_width(),
			amplification = ?stream::automatic_amplification(),
			"Storage hardware not detected for database directory; assuming defaults.",
		);
	}

	assert!(total_workers > 0, "some workers expected");
	assert!(!queue_sizes.is_empty(), "some queues expected");
	assert!(
		!queue_sizes.iter().copied().any(is_equal_to!(0)),
		"positive queue sizes expected"
	);

	(total_workers, queue_sizes, topology)
}

#[allow(clippy::as_conversions, clippy::cast_precision_loss)]
fn update_stream_width(
	server: &Arc<Server>,
	num_queues: usize,
	total_workers: usize,
	total_tags: usize,
) {
	let config = &server.config;
	let scale: f64 = config.stream_width_scale.min(100.0).into();
	let auto_scale = total_tags as f64 / total_workers as f64;
	let auto_scale_width = auto_scale / num_queues as f64;

	let req_width = expected!(total_workers / num_queues).next_multiple_of(8);
	let req_width = req_width as f64 * auto_scale_width.clamp(1.0, 4.0);
	let req_width = usize_from_f64(req_width * scale)
		.expect("failed to convert f64 to usize")
		.next_multiple_of(4)
		.clamp(WIDTH_LIMIT.0, WIDTH_LIMIT.1);

	let req_amp = config.stream_amplification as f64 * auto_scale.clamp(1.0, 4.0);
	let req_amp = usize_from_f64(req_amp * scale)
		.expect("failed to convert f64 to usize")
		.next_multiple_of(64)
		.clamp(AMPLIFICATION_LIMIT.0, AMPLIFICATION_LIMIT.1);

	let (old_width, new_width) = stream::set_width(req_width);
	let (old_amp, new_amp) = stream::set_amplification(req_amp);
	debug!(
		config_scale = ?config.stream_width_scale,
		?auto_scale,
		?auto_scale_width,
		?req_width,
		?old_width,
		?new_width,
		?old_amp,
		?new_amp,
		"Updated global stream width"
	);
}
