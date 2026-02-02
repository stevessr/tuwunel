#![cfg(unix)]

use std::{net::SocketAddr, os::unix::net::UnixListener, path::PathBuf, sync::Arc};

use axum::{Extension, Router, extract::ConnectInfo};
use axum_server::Handle;
use tokio::task::JoinSet;
use tuwunel_core::{Result, Server};

#[tracing::instrument(skip_all, level = "debug")]
pub(super) async fn serve(
	server: &Arc<Server>,
	router: &Router,
	handle: &Handle<std::os::unix::net::SocketAddr>,
	join_set: &mut JoinSet<core::result::Result<(), std::io::Error>>,
	unix_socket: &PathBuf,
) -> Result {
	let unix_listener = UnixListener::bind(unix_socket)?;
	unix_listener.set_nonblocking(true)?;

	let router = router
		.clone()
		.layer(Extension(ConnectInfo("0.0.0.0".parse::<SocketAddr>())))
		.into_make_service();
	let acceptor = axum_server::from_unix(unix_listener)?
		.handle(handle.clone())
		.serve(router);
	join_set.spawn_on(acceptor, server.runtime());

	Ok(())
}
