use std::{env::VarError, fmt::Display, path::Path, str::FromStr};

use anyhow::Context;
use app::config::Config;
use figment::{
	providers::{Env, Format, Yaml},
	Figment,
};

/// Retrieve the application configuration by merging together multiple configuration sources.
///
/// # Errors
///
/// This function will return an error if the configuration cannot be loaded.
pub fn load(default_profile: Option<Environment>) -> Result<Config, anyhow::Error> {
	let env = load_env(default_profile).context("Failed to load the desired app env")?;

	let config_dir = Path::new(env!(
		"CARGO_MANIFEST_DIR",
		"`CARGO_MANIFEST_DIR` was not set. Are you using a custom build system?"
	))
	.join("../config");

	let figment = Figment::new()
		.merge(Yaml::file(config_dir.join("base.yml")))
		.merge(Yaml::file(config_dir.join(format!("{env}.yml"))))
		.merge(Env::raw().split("_"));

	figment.extract().context(format!(
		"Failed to load hierarchical configuration from {}",
		config_dir.display()
	))
}

/// Load the application profile from the `APP_PROFILE` environment variable.
fn load_env(default_profile: Option<Environment>) -> Result<Environment, anyhow::Error> {
	static CURRENT_ENV_VAR: &str = "APP_ENV";

	match std::env::var(CURRENT_ENV_VAR) {
		Ok(raw_value) => raw_value.parse().with_context(|| {
			format!("Failed to parse the `{CURRENT_ENV_VAR}` environment variable")
		}),
		Err(VarError::NotPresent) if default_profile.is_some() => Ok(default_profile.unwrap()),
		Err(e) => Err(anyhow::anyhow!(e).context(format!(
			"Failed to read the `{CURRENT_ENV_VAR}` environment variable"
		))),
	}
}

/// The application profile, i.e. the type of environment the application is running in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
	/// Test profile.
	Test,
	/// Development profile.
	Dev,
	/// Production profile.
	Prod,
}

impl Environment {
	/// Return the environment as a string.
	#[must_use]
	pub const fn as_str(&self) -> &'static str {
		match self {
			Self::Dev => "dev",
			Self::Test => "test",
			Self::Prod => "prod",
		}
	}
}

impl FromStr for Environment {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
            "test" => Ok(Self::Test),
            "dev" | "development" => Ok(Self::Dev),
            "prod" | "production" => Ok(Self::Prod),
            s => Err(anyhow::anyhow!(
                "`{s}` is not a valid application profile.\nValid options are: `test`, `dev`, `prod`.",
            )),
        }
	}
}

impl Display for Environment {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self.as_str())
	}
}
