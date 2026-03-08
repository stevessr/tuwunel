mod plain;
#[cfg(feature = "direct_tls")]
mod tls;
mod unix;

#[cfg(all(feature = "systemd", target_os = "linux"))]
use std::os::fd::FromRawFd;
use std::{
	net::{SocketAddr, TcpListener},
	sync::{Arc, atomic::Ordering},
};

use tokio::task::JoinSet;
use tuwunel_core::{Result, debug_info};
use tuwunel_service::Services;

use super::layers;
use crate::handle::ServerHandle;

/// Serve clients
pub(super) async fn serve(services: Arc<Services>, handle: ServerHandle) -> Result {
	let server = &services.server;
	let config = &server.config;

	let (app, _guard) = layers::build(&services)?;

	let mut join_set = JoinSet::new();

	#[cfg(unix)]
	if let Some(unix_socket) = &config.unix_socket_path {
		let socket_perms = config.get_unix_socket_perms()?;

		unix::serve(server, &app, &handle.handle_unix, &mut join_set, unix_socket, socket_perms)
			.await?;
	}

	let mut listen_addrs: Vec<SocketAddr> = vec![];

	#[cfg(all(feature = "systemd", target_os = "linux"))]
	let mut listeners: Vec<TcpListener> = if let Ok(fds) = sd_notify::listen_fds() {
		fds.filter_map(|fd| {
			// SAFETY: systemd should already take care of providing
			// the correct TCP socket, so we just use it via raw fd
			let listener = unsafe { TcpListener::from_raw_fd(fd) };
			listener
				.set_nonblocking(true)
				.expect("Failed to set non-blocking");
			if let Ok(addr) = listener.local_addr() {
				listen_addrs.push(addr);
				Some(listener)
			} else {
				None
			}
		})
		.collect()
	} else {
		vec![]
	};

	#[cfg(not(all(feature = "systemd", target_os = "linux")))]
	let mut listeners: Vec<TcpListener> = vec![];

	let addrs = config.get_bind_addrs();

	if !addrs.is_empty() && listeners.is_empty() {
		listeners = addrs
			.clone()
			.into_iter()
			.map(|addr| {
				let listener = TcpListener::bind(addr).expect("Failed to bind {addr:?}");
				listener
					.set_nonblocking(true)
					.expect("Failed to set non-blocking");
				listen_addrs.push(addr);
				listener
			})
			.collect();
	}

	#[cfg_attr(not(feature = "direct_tls"), expect(clippy::redundant_else))]
	if config.tls.certs.is_some() {
		#[cfg(feature = "direct_tls")]
		{
			services.globals.init_rustls_provider()?;
			tls::serve(server, &app, &handle.handle_ip, &mut join_set, &listen_addrs, &listeners)
				.await?;
		}

		#[cfg(not(feature = "direct_tls"))]
		return tuwunel_core::Err!(Config(
			"tls",
			"tuwunel was not built with direct TLS support (\"direct_tls\")"
		));
	} else {
		plain::serve(server, &app, &handle.handle_ip, &mut join_set, &listen_addrs, &listeners);
	}

	assert!(!join_set.is_empty(), "at least one listener should be installed");

	join_set.join_all().await;

	let handle_active = server
		.metrics
		.requests_handle_active
		.load(Ordering::Acquire);

	debug_info!(
		handle_finished = server
			.metrics
			.requests_handle_finished
			.load(Ordering::Acquire),
		panics = server
			.metrics
			.requests_panic
			.load(Ordering::Acquire),
		handle_active,
		"Stopped listening on {listen_addrs:?}",
	);

	debug_assert_eq!(0, handle_active, "active request handles still pending");

	Ok(())
}
