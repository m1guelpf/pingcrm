use anyhow::Context;
use app::config::Config;
use tracing::subscriber::set_global_default;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

/// Setup telemetry for the application. Should only be called once!
///
/// # Errors
///
/// This function will return an error if the telemetry cannot be set up.
pub fn setup(config: &Config) -> Result<(), anyhow::Error> {
	let subscriber = Registry::default()
		.with(EnvFilter::from(config.app.log.clone()))
		.with(tracing_error::ErrorLayer::default())
		.with(tracing_subscriber::fmt::layer());

	std::panic::set_hook(Box::new(tracing_panic::panic_hook));
	set_global_default(subscriber).context("Failed to set a `tracing` global subscriber")
}
