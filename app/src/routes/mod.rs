#![allow(clippy::must_use_candidate)]

use pavex::{
	blueprint::{
		router::{GET, POST},
		Blueprint,
	},
	f,
};

pub mod auth;
pub mod system;

pub fn handler(bp: &mut Blueprint) {
	bp.route(GET, "/healthz", f!(crate::routes::system::health_check));
	bp.route(
		GET,
		"/login",
		f!(crate::routes::auth::AuthenticatedSessionController::create),
	);
	bp.route(
		POST,
		"/login",
		f!(crate::routes::auth::AuthenticatedSessionController::store),
	);

	bp.route(GET, "/assets/*path", f!(crate::frontend::serve_assets));
}
