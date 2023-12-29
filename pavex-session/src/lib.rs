use pavex::{
	blueprint::{
		constructor::{CloningStrategy, Lifecycle},
		Blueprint,
	},
	f,
};

pub use config::{CsrfConfig, SessionConfig};
pub use handlers::Handler;
pub use manager::Manager;
pub use middleware::{StartSession, TokenMismatchError, VerifyCsrfToken};
pub use session::Session;

mod config;
pub mod handlers;
mod manager;
mod middleware;
mod session;

pub fn register(bp: &mut Blueprint) {
	bp.constructor(f!(crate::Session::new), Lifecycle::RequestScoped)
		.cloning(CloningStrategy::CloneIfNecessary);

	bp.constructor(f!(crate::Manager::new), Lifecycle::Singleton);
	bp.wrap(f!(crate::StartSession::handle));
}
