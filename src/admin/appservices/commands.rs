use futures::StreamExt;
use tuwunel_core::{Err, Result};

use crate::command;

#[command]
pub(super) async fn register(&self) -> Result<String> {
	let parsed_config = serde_yaml::from_str(self.input);
	match parsed_config {
		| Err(e) => Err!("Could not parse appservice config as YAML: {e}"),
		| Ok(registration) => match self
			.services
			.appservice
			.register_appservice(&registration, self.input)
			.await
			.map(|()| registration.id)
		{
			| Err(e) => Err!("Failed to register appservice: {e}"),
			| Ok(id) => Ok(format!("Appservice registered with ID: {id}")),
		},
	}
}

#[command]
pub(super) async fn unregister(&self, appservice_identifier: String) -> Result<String> {
	match self
		.services
		.appservice
		.unregister_appservice(&appservice_identifier)
		.await
	{
		| Err(e) => Err!("Failed to unregister appservice: {e}"),
		| Ok(()) => Ok("Appservice unregistered.".to_owned()),
	}
}

#[command]
pub(super) async fn show_appservice_config(
	&self,
	appservice_identifier: String,
) -> Result<String> {
	match self
		.services
		.appservice
		.get_registration(&appservice_identifier)
		.await
	{
		| None => Err!("Appservice does not exist."),
		| Some(config) => {
			let config_str = serde_yaml::to_string(&config)?;
			Ok(format!("Config for {appservice_identifier}:\n\n```yaml\n{config_str}\n```"))
		},
	}
}

#[command]
pub(super) async fn list_registered(&self) -> Result<String> {
	let appservices = self
		.services
		.appservice
		.iter_ids()
		.collect::<Vec<String>>()
		.await;
	let len = appservices.len();
	let list = appservices.join(", ");
	Ok(format!("Appservices ({len}): {list}"))
}
