use std::fmt::Write;

use clap::Subcommand;
use futures::StreamExt;
use ruma::OwnedServerName;
use tuwunel_core::{Result, utils::time};

use crate::{command, command_dispatch};

#[command_dispatch]
#[derive(Debug, Subcommand)]
/// Resolver service and caches
pub(crate) enum ResolverCommand {
	/// Query the destinations cache
	DestinationsCache {
		server_name: Option<OwnedServerName>,
	},

	/// Query the overrides cache
	OverridesCache {
		name: Option<String>,
	},
}

#[command]
async fn destinations_cache(&self, server_name: Option<OwnedServerName>) -> Result<String> {
	use tuwunel_service::resolver::cache::CachedDest;

	let mut out = String::new();

	writeln!(out, "| Server Name | Destination | Hostname | Expires |")?;
	writeln!(out, "| ----------- | ----------- | -------- | ------- |")?;

	let mut destinations = self
		.services
		.resolver
		.cache
		.destinations()
		.boxed();

	while let Some((name, CachedDest { dest, host, expire })) = destinations.next().await {
		if let Some(server_name) = server_name.as_ref() {
			if name != server_name {
				continue;
			}
		}

		let expire = time::format(expire, "%+");
		writeln!(out, "| {name} | {dest} | {host} | {expire} |")?;
	}

	Ok(out)
}

#[command]
async fn overrides_cache(&self, server_name: Option<String>) -> Result<String> {
	use tuwunel_service::resolver::cache::CachedOverride;

	let mut out = String::new();

	writeln!(out, "| Server Name | IP  | Port | Expires | Overriding |")?;
	writeln!(out, "| ----------- | --- | ---- | ------- | ---------- |")?;

	let mut overrides = self.services.resolver.cache.overrides().boxed();

	while let Some((name, CachedOverride { ips, port, expire, overriding })) =
		overrides.next().await
	{
		if let Some(server_name) = server_name.as_ref() {
			if name != server_name {
				continue;
			}
		}

		let expire = time::format(expire, "%+");
		writeln!(out, "| {name} | {ips:?} | {port} | {expire} | {overriding:?} |")?;
	}

	Ok(out)
}
