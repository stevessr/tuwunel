use futures::{FutureExt, StreamExt, TryStreamExt};
use tuwunel_core::{
	Result,
	utils::{result::LogErr, stream::IterStream},
};
use tuwunel_service::storage::CopyMode;

use crate::{admin_command, admin_command_dispatch};

#[admin_command_dispatch(handler_prefix = "query_storage")]
#[derive(Debug, clap::Subcommand)]
pub(crate) enum StorageCommand {
	/// List provider configurations.
	ShowConfigs,

	/// List provider instances.
	ShowProviders,

	Debug {
		/// Use configured provider by name.
		#[arg(short, long)]
		provider: Option<String>,
	},

	/// List metadata for all objects.
	List {
		/// Use configured provider by name.
		#[arg(short, long)]
		provider: Option<String>,

		/// Optionally filter by matching prefix.
		prefix: Option<String>,
	},

	/// Show metadata for an object.
	Show {
		/// Use configured provider by name.
		#[arg(short, long)]
		provider: Option<String>,

		/// Path to the object.
		location: String,
	},

	/// Copy an object from a source to a destination.
	Copy {
		/// Use configured provider by name.
		#[arg(short, long)]
		provider: Option<String>,

		/// Overwrite existing destination.
		#[arg(short, long)]
		force: bool,

		/// Path to the source.
		source: String,

		/// Path to the destination.
		destination: String,
	},

	/// Move an object from a source to a destination.
	Move {
		/// Use configured provider by name.
		#[arg(short, long)]
		provider: Option<String>,

		/// Overwrite existing destination.
		#[arg(short, long)]
		force: bool,

		/// Path to the source.
		source: String,

		/// Path to the destination.
		destination: String,
	},

	/// Delete an object at the specified location.
	Delete {
		/// Use configured provider by name.
		#[arg(short, long)]
		provider: Option<String>,

		/// Path to the location to delete. Multiple arguments allowed.
		location: Vec<String>,

		/// Report successful results in addition to failures.
		#[arg(short, long)]
		verbose: bool,
	},
}

#[admin_command]
async fn query_storage_show_configs(&self) -> Result {
	self.services
		.storage
		.configs(None)
		.try_stream()
		.try_for_each(|(id, conf)| self.write_string(format!("`{id:?}` {conf:#?}\n")))
		.await
}

#[admin_command]
async fn query_storage_show_providers(&self) -> Result {
	self.services
		.storage
		.providers()
		.try_stream()
		.try_for_each(|conf| self.write_string(format!("`{:?}` {conf:#?}\n", conf.name)))
		.await
}

#[admin_command]
async fn query_storage_debug(&self, provider: Option<String>) -> Result {
	let id = provider.as_deref().unwrap_or_default();
	let provider = self.services.storage.provider(id)?;

	self.write_string(format!("{provider:#?}")).await
}

#[admin_command]
async fn query_storage_list(&self, provider: Option<String>, prefix: Option<String>) -> Result {
	let id = provider.as_deref().unwrap_or_default();
	let provider = self.services.storage.provider(id)?;

	provider
		.list(prefix.as_deref())
		.try_for_each(|meta| writeln!(&self, "{meta:?}"))
		.boxed()
		.await
}

#[admin_command]
async fn query_storage_show(&self, provider: Option<String>, location: String) -> Result {
	let id = provider.as_deref().unwrap_or_default();
	let provider = self.services.storage.provider(id)?;
	let meta = provider.head(&location).await?;

	self.write_string(format!("{meta:#?}")).await
}

#[admin_command]
async fn query_storage_copy(
	&self,
	provider: Option<String>,
	force: bool,
	source: String,
	destination: String,
) -> Result {
	let id = provider.as_deref().unwrap_or_default();
	let provider = self.services.storage.provider(id)?;
	let overwrite = force
		.then_some(CopyMode::Overwrite)
		.unwrap_or(CopyMode::Create);

	let result = provider
		.copy(&source, &destination, overwrite)
		.await;

	self.write_string(format!("{result:?}")).await
}

#[admin_command]
async fn query_storage_move(
	&self,
	provider: Option<String>,
	force: bool,
	source: String,
	destination: String,
) -> Result {
	let id = provider.as_deref().unwrap_or_default();
	let provider = self.services.storage.provider(id)?;
	let overwrite = force
		.then_some(CopyMode::Overwrite)
		.unwrap_or(CopyMode::Create);

	let result = provider
		.rename(&source, &destination, overwrite)
		.await;

	self.write_string(format!("{result:?}")).await
}

#[admin_command]
async fn query_storage_delete(
	&self,
	provider: Option<String>,
	location: Vec<String>,
	verbose: bool,
) -> Result {
	let id = provider.as_deref().unwrap_or_default();
	let provider = self.services.storage.provider(id)?;

	provider
		.delete(location.into_iter().stream())
		.for_each(async |result| {
			match result {
				| Ok(_) if !verbose => None,

				| Ok(path) => self
					.write_string(format!("deleted: {path:?}\n"))
					.await
					.log_err()
					.ok(),

				| Err(e) => self
					.write_string(format!("failed: {e:?}"))
					.await
					.log_err()
					.ok(),
			};
		})
		.map(Ok)
		.await
}
