#![cfg(test)]

use tuwunel_api::router::ConfiguredIpSource;
use tuwunel_core::config::IpSource;

use super::ip_source_layer;

#[test]
fn ip_source_layer_none_returns_identity_branch() {
	let layer = ip_source_layer(None);

	assert!(matches!(layer, tower::util::Either::Right(_)));
}

#[test]
fn ip_source_layer_connect_info_returns_extension_branch() {
	let layer = ip_source_layer(Some(IpSource::ConnectInfo));

	assert!(matches!(
		layer,
		tower::util::Either::Left(axum::Extension(ConfiguredIpSource(_)))
	));
}
