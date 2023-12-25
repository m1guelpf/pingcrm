#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use anyhow::Context;
use app::config::Config;
use pavex::server::Server;

pub mod config;
mod migrations;
pub mod telemetry;

/// The application's entry point.
///
/// # Errors
///
/// Errors if the application state can't be built.
pub async fn run(config: Config) -> anyhow::Result<()> {
	ensemble::setup(&config.database.url)?;
	ensemble::migrate!(migrations::CreateUsersTable).await?;

	let tcp_listener = config
		.server
		.listener()
		.await
		.context("Failed to bind the server TCP listener")?;

	let state = sdk::build_application_state(config).await?;

	let address = tcp_listener
		.local_addr()
		.context("The server TCP listener doesn't have a local socket address")?;

	tracing::info!("⚡ Starting server at http://{address}");

	sdk::run(Server::new().listen(tcp_listener), state).await;

	Ok(())
}
