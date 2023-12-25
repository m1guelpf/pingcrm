use flysystem::{adapters::LocalAdapter, Filesystem};
use handlers::FileHandler;
use pavex::{
	blueprint::{constructor::Lifecycle, Blueprint},
	f,
};
use std::{path::PathBuf, time::Duration};

pub use handlers::Handler;
pub use middleware::Middleware;
pub use store::Store;

pub mod handlers;
mod middleware;
mod store;

#[derive(Debug, Clone)]
pub struct Manager {
	config: SessionConfig,
	files: Filesystem<LocalAdapter>,
}

impl Manager {
	pub fn new(config: SessionConfig, files: Filesystem<LocalAdapter>) -> Self {
		Self { config, files }
	}

	async fn create_file_driver(&self) -> Store<FileHandler> {
		let adapter = FileHandler::new(
			self.config.file_location.clone(),
			self.config.lifetime,
			self.files.clone(),
		)
		.await
		.unwrap();

		Store::new(self.config.cookie_name.clone(), adapter, None)
	}
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SessionConfig {
	/// The default session driver that will be used on requests.
	pub driver: Option<String>,

	/// The amount of time that you wish the session to be allowed to remain idle before it expires.
	#[serde(with = "humantime_serde")]
	pub lifetime: Duration,

	/// When using the native session driver, the location where session files should be stored.
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
}

pub fn register(bp: &mut Blueprint) {
	bp.wrap(f!(crate::Middleware::handle));
	bp.constructor(f!(crate::Manager::new), Lifecycle::Singleton);
}
