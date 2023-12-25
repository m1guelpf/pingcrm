use dotenvy::dotenv;
use server::{config, telemetry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	dotenv().ok();

	let config = config::load(None)?;

	telemetry::setup(&config)?;

	if let Err(e) = server::run(config).await {
		tracing::error!(
			error.msg = %e,
			error.error_chain = ?e,
			"The application is exiting due to an error"
		)
	}

	Ok(())
}
