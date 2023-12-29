#![allow(clippy::module_name_repetitions)]

use std::{path::PathBuf, time::Duration};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SessionConfig {
	/// The amount of time that you wish the session to be allowed to remain idle before it expires.
	#[serde(with = "humantime_serde")]
	pub lifetime: Duration,

	/// The location where session files should be stored.
	pub file_location: PathBuf,

	/// The name of the cookie used to identify a session instance by ID.
	pub cookie_name: String,

	/// The path for which the session cookie will be regarded as available.
	pub path: String,

	/// The domain of the cookie used to identify a session in your application.
	pub domain: Option<String>,

	/// If true, session cookies will only be sent back to the server if your browser has a HTTPS connection.
	pub secure: bool,

	/// Setting this value to true will prevent JavaScript from accessing the value of the cookie.
	pub http_only: bool,

	/// For drivers who need to clean their storage manually, the chance that it will happen on a given request.
	pub lottery: [u8; 2],

	/// This option determines how your cookies behave when cross-site requests take place, and can be used to mitigate CSRF attacks.
	pub same_site: String,

	/// The CSRF configuration.
	pub csrf: CsrfConfig,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct CsrfConfig {
	/// A list of paths that should be excluded from CSRF protection.
	pub exclude_paths: Vec<String>,
}
