use anyhow::Context;
use app::config::Config;
use dotenvy::dotenv;
use pavex::server::Server;
use server::{config, telemetry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	dotenv().ok();

	let config = config::load(None)?;

	telemetry::setup(&config)?;

	if let Err(e) = run(config).await {
		tracing::error!(
			error.msg = %e,
			error.error_chain = ?e,
			"The application is exiting due to an error"
		)
	}

	Ok(())
}

async fn run(config: Config) -> anyhow::Result<()> {
	let state = sdk::build_application_state().await;

	let tcp_listener = config
		.server
		.listener()
		.await
		.context("Failed to bind the server TCP listener")?;

	let address = tcp_listener
		.local_addr()
		.context("The server TCP listener doesn't have a local socket address")?;

	tracing::info!("⚡ Starting server at http://{address}");

	sdk::run(Server::new().listen(tcp_listener), state).await;

	Ok(())
}
