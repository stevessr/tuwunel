//! Tuwunel's client-IP extractor.
//!
//! Wraps `axum_client_ip::SecureClientIp` with a two-mode fallback:
//!
//! * If the operator configured `ip_source`, a [`ConfiguredIpSource`] marker is
//!   installed in request extensions and we delegate to
//!   [`axum_client_ip::SecureClientIp`] with that source. Exception: if the
//!   peer shown by `ConnectInfo` is on a loopback interface, the insecure
//!   header-scan + `ConnectInfo` fallback runs instead, since a loopback peer
//!   (e.g. a locally-connected appservice bridge) cannot have spoofed the
//!   address.
//! * Otherwise the insecure header-scan + `ConnectInfo` fallback runs directly,
//!   preserving the prior default behaviour, including the socket-address
//!   fallback that matters for Unix-socket deployments.
//!
//! The plain `SecureClientIpSource::ConnectInfo` extension that
//! `src/router/layers.rs` installs by default is intentionally ignored here;
//! only the [`ConfiguredIpSource`] marker participates in the secure path.
//! This avoids flipping behaviour for deployments that never opted in.
//!
//! The header-scan chain mirrors the leftmost-IP behaviour that
//! `axum_client_ip::InsecureClientIp` provided in 0.7; the 1.x crate
//! removed that extractor, and inlining the small chain here is what
//! lets the loopback short-circuit reuse the same fallback for the
//! configured path.

use std::{
	fmt,
	marker::Sync,
	net::{IpAddr, SocketAddr},
};

use axum::extract::{ConnectInfo, FromRequestParts};
use axum_client_ip::{SecureClientIp, SecureClientIpSource};
use http::{Extensions, HeaderMap, StatusCode, request::Parts};

/// Tuwunel client-IP extractor. See module docs.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ClientIp(pub(crate) IpAddr);

/// Marker wrapper around [`SecureClientIpSource`] placed into request
/// extensions only when an operator has explicitly configured
/// `ip_source`.
#[derive(Clone, Debug)]
pub struct ConfiguredIpSource(pub SecureClientIpSource);

impl<S> FromRequestParts<S> for ClientIp
where
	S: Sync,
{
	type Rejection = (StatusCode, &'static str);

	async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
		const ERROR: StatusCode = StatusCode::INTERNAL_SERVER_ERROR;

		if let Some(ConfiguredIpSource(source)) = parts.extensions.get::<ConfiguredIpSource>()
			&& !peer_is_loopback(&parts.extensions)
		{
			return SecureClientIp::from(source, &parts.headers, &parts.extensions)
				.map(|SecureClientIp(ip)| Self(ip))
				.map_err(|_| (ERROR, "Can't extract client IP from configured ip_source"));
		}

		insecure_fallback(&parts.headers, &parts.extensions)
			.map(Self)
			.ok_or((ERROR, "Can't extract `ClientIp`, provide `axum::extract::ConnectInfo`"))
	}
}

impl fmt::Display for ClientIp {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { fmt::Display::fmt(&self.0, f) }
}

fn peer_is_loopback(extensions: &Extensions) -> bool {
	extensions
		.get::<ConnectInfo<SocketAddr>>()
		.is_some_and(|ConnectInfo(addr)| addr.ip().to_canonical().is_loopback())
}

/// Leftmost header scan with `ConnectInfo` fallback.
fn insecure_fallback(headers: &HeaderMap, extensions: &Extensions) -> Option<IpAddr> {
	leftmost_x_forwarded_for(headers)
		.or_else(|| leftmost_forwarded(headers))
		.or_else(|| single_ip_header(headers, "x-real-ip"))
		.or_else(|| single_ip_header(headers, "fly-client-ip"))
		.or_else(|| single_ip_header(headers, "true-client-ip"))
		.or_else(|| single_ip_header(headers, "cf-connecting-ip"))
		.or_else(|| cloudfront_viewer_address(headers))
		.or_else(|| {
			extensions
				.get::<ConnectInfo<SocketAddr>>()
				.map(|ConnectInfo(addr)| addr.ip())
		})
}

fn leftmost_x_forwarded_for(headers: &HeaderMap) -> Option<IpAddr> {
	headers
		.get_all("x-forwarded-for")
		.iter()
		.filter_map(|v| v.to_str().ok())
		.flat_map(|s| s.split(','))
		.find_map(|s| s.trim().parse::<IpAddr>().ok())
}

/// Parse `for=` from the leftmost RFC 7239 stanza. Tolerates quoted
/// values, bracketed IPv6, and an optional `:port` suffix.
fn leftmost_forwarded(headers: &HeaderMap) -> Option<IpAddr> {
	headers
		.get_all("forwarded")
		.iter()
		.filter_map(|v| v.to_str().ok())
		.flat_map(|s| s.split(','))
		.find_map(parse_forwarded_for)
}

fn parse_forwarded_for(stanza: &str) -> Option<IpAddr> {
	let for_value = stanza
		.split(';')
		.find_map(|part| {
			let (k, v) = part.split_once('=')?;
			k.trim()
				.eq_ignore_ascii_case("for")
				.then_some(v.trim())
		})?
		.trim_matches('"');

	let bracketed = for_value
		.strip_prefix('[')
		.and_then(|s| s.split_once(']'))
		.map(|(ip, _rest)| ip);

	let candidate = bracketed
		.or_else(|| for_value.rsplit_once(':').map(|(ip, _port)| ip))
		.unwrap_or(for_value);

	candidate.trim().parse::<IpAddr>().ok()
}

fn single_ip_header(headers: &HeaderMap, name: &'static str) -> Option<IpAddr> {
	headers
		.get(name)
		.and_then(|v| v.to_str().ok())
		.and_then(|s| s.trim().parse::<IpAddr>().ok())
}

fn cloudfront_viewer_address(headers: &HeaderMap) -> Option<IpAddr> {
	headers
		.get("cloudfront-viewer-address")
		.and_then(|v| v.to_str().ok())
		.and_then(|s| s.rsplit_once(':').map(|(ip, _port)| ip))
		.and_then(|s| s.trim().parse::<IpAddr>().ok())
}

#[cfg(test)]
mod tests {
	use std::{iter, net::SocketAddr};

	use axum::{
		extract::{ConnectInfo, FromRequestParts},
		http::{Request, StatusCode, request::Parts},
	};
	use axum_client_ip::SecureClientIpSource;

	use super::{ClientIp, ConfiguredIpSource};

	fn parts(headers: impl IntoIterator<Item = (&'static str, &'static str)>) -> Parts {
		let mut request = Request::builder().uri("/");
		for (name, value) in headers {
			request = request.header(name, value);
		}
		let (parts, ()) = request.body(()).unwrap().into_parts();
		parts
	}

	async fn extract_client_ip(
		parts: &mut Parts,
	) -> Result<ClientIp, (StatusCode, &'static str)> {
		ClientIp::from_request_parts(parts, &()).await
	}

	#[tokio::test]
	async fn x_forwarded_for_uses_leftmost_ip() {
		let mut parts = parts([("X-Forwarded-For", "1.1.1.1, 2.2.2.2")]);
		let ClientIp(ip) = extract_client_ip(&mut parts).await.unwrap();
		assert_eq!(ip.to_string(), "1.1.1.1");
	}

	#[tokio::test]
	async fn x_forwarded_for_takes_priority_over_x_real_ip() {
		let mut parts =
			parts([("X-Forwarded-For", "1.1.1.1, 2.2.2.2"), ("X-Real-Ip", "3.3.3.3")]);
		let ClientIp(ip) = extract_client_ip(&mut parts).await.unwrap();
		assert_eq!(ip.to_string(), "1.1.1.1");
	}

	#[tokio::test]
	async fn x_forwarded_for_accepts_ipv6() {
		let mut parts = parts([("X-Forwarded-For", "2001:db8::1, 2001:db8::2")]);
		let ClientIp(ip) = extract_client_ip(&mut parts).await.unwrap();
		assert_eq!(ip.to_string(), "2001:db8::1");
	}

	#[tokio::test]
	async fn x_real_ip_works() {
		let mut parts = parts([("X-Real-Ip", "1.2.3.4")]);
		let ClientIp(ip) = extract_client_ip(&mut parts).await.unwrap();
		assert_eq!(ip.to_string(), "1.2.3.4");
	}

	#[tokio::test]
	async fn malformed_headers_fall_through_to_next_valid_source() {
		let mut parts = parts([
			("X-Forwarded-For", "foo"),
			("X-Real-Ip", "foo"),
			("Forwarded", "foo"),
			("Forwarded", "for=1.1.1.1;proto=https;by=2.2.2.2"),
		]);
		let ClientIp(ip) = extract_client_ip(&mut parts).await.unwrap();
		assert_eq!(ip.to_string(), "1.1.1.1");
	}

	#[tokio::test]
	async fn no_headers_or_connect_info_rejects() {
		let mut parts = parts(iter::empty());
		let err = extract_client_ip(&mut parts).await.unwrap_err();
		assert_eq!(err.0, StatusCode::INTERNAL_SERVER_ERROR);
		assert!(err.1.contains("ConnectInfo"), "{err:?}");
	}

	#[tokio::test]
	async fn configured_source_uses_secure_extraction() {
		let mut parts = parts([("X-Forwarded-For", "1.1.1.1, 2.2.2.2")]);
		parts
			.extensions
			.insert(ConfiguredIpSource(SecureClientIpSource::RightmostXForwardedFor));
		let ClientIp(ip) = extract_client_ip(&mut parts).await.unwrap();
		assert_eq!(ip.to_string(), "2.2.2.2");
	}

	#[tokio::test]
	async fn configured_source_without_matching_header_rejects() {
		let mut parts = parts(iter::empty());
		parts
			.extensions
			.insert(ConfiguredIpSource(SecureClientIpSource::RightmostXForwardedFor));
		let err = extract_client_ip(&mut parts).await.unwrap_err();
		assert_eq!(err.0, StatusCode::INTERNAL_SERVER_ERROR);
		assert_eq!(err.1, "Can't extract client IP from configured ip_source");
	}

	#[tokio::test]
	async fn secure_client_ip_source_extension_does_not_hijack() {
		let mut parts = parts([("X-Forwarded-For", "1.1.1.1, 2.2.2.2")]);
		parts
			.extensions
			.insert(SecureClientIpSource::ConnectInfo);
		let ClientIp(ip) = extract_client_ip(&mut parts).await.unwrap();
		assert_eq!(ip.to_string(), "1.1.1.1");
	}

	#[tokio::test]
	async fn connect_info_fallback_uses_real_socket_addr_without_config() {
		let socket_addr = SocketAddr::from(([203, 0, 113, 9], 4567));
		let mut parts = parts(iter::empty());
		parts.extensions.insert(ConnectInfo(socket_addr));

		let ClientIp(ip) = extract_client_ip(&mut parts).await.unwrap();
		assert_eq!(ip, socket_addr.ip());
	}

	#[tokio::test]
	async fn bare_secure_client_ip_source_connect_info_does_not_hijack() {
		let socket_addr = SocketAddr::from(([203, 0, 113, 10], 4567));
		let mut parts = parts([("X-Forwarded-For", "1.1.1.1, 2.2.2.2")]);
		parts.extensions.insert(ConnectInfo(socket_addr));
		parts
			.extensions
			.insert(SecureClientIpSource::ConnectInfo);

		let ClientIp(ip) = extract_client_ip(&mut parts).await.unwrap();
		assert_eq!(ip.to_string(), "1.1.1.1");
	}

	#[tokio::test]
	async fn loopback_peer_bypasses_configured_source_for_locally_connected_bridges() {
		let socket_addr = SocketAddr::from(([127, 0, 0, 1], 38000));
		let mut parts = parts(iter::empty());
		parts.extensions.insert(ConnectInfo(socket_addr));
		parts
			.extensions
			.insert(ConfiguredIpSource(SecureClientIpSource::RightmostXForwardedFor));

		let ClientIp(ip) = extract_client_ip(&mut parts).await.unwrap();
		assert_eq!(ip, socket_addr.ip());
	}

	#[tokio::test]
	async fn loopback_peer_with_proxy_header_still_uses_insecure_fallback() {
		// When a loopback peer also forwards a proxy header (e.g. a local
		// reverse proxy in a sidecar), the insecure leftmost-XFF behaviour wins
		// over the loopback ConnectInfo fallback, matching how the unconfigured
		// path already behaves.
		let socket_addr = SocketAddr::from(([127, 0, 0, 1], 38000));
		let mut parts = parts([("X-Forwarded-For", "9.9.9.9")]);
		parts.extensions.insert(ConnectInfo(socket_addr));
		parts
			.extensions
			.insert(ConfiguredIpSource(SecureClientIpSource::RightmostXForwardedFor));

		let ClientIp(ip) = extract_client_ip(&mut parts).await.unwrap();
		assert_eq!(ip.to_string(), "9.9.9.9");
	}

	#[tokio::test]
	async fn ipv6_loopback_peer_also_bypasses_configured_source() {
		let socket_addr = SocketAddr::from(([0_u16, 0, 0, 0, 0, 0, 0, 1], 38000));
		let mut parts = parts(iter::empty());
		parts.extensions.insert(ConnectInfo(socket_addr));
		parts
			.extensions
			.insert(ConfiguredIpSource(SecureClientIpSource::RightmostXForwardedFor));

		let ClientIp(ip) = extract_client_ip(&mut parts).await.unwrap();
		assert_eq!(ip, socket_addr.ip());
	}

	#[tokio::test]
	async fn non_loopback_peer_with_configured_source_still_rejects() {
		let socket_addr = SocketAddr::from(([203, 0, 113, 9], 38000));
		let mut parts = parts(iter::empty());
		parts.extensions.insert(ConnectInfo(socket_addr));
		parts
			.extensions
			.insert(ConfiguredIpSource(SecureClientIpSource::RightmostXForwardedFor));

		let err = extract_client_ip(&mut parts).await.unwrap_err();
		assert_eq!(err.0, StatusCode::INTERNAL_SERVER_ERROR);
		assert_eq!(err.1, "Can't extract client IP from configured ip_source");
	}
}
