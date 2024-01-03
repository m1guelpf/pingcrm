#![allow(clippy::must_use_candidate)]

use framework::{f, router::Router};

pub mod auth;
pub mod system;

pub fn handler() -> Router {
	let mut router = Router::new();

	router
		.name("health-check")
		.get("/healthz", f!(crate::http::routes::system::health_check));

	router.name("auth").prefix("/auth").group(|mut router| {
		router
			.resource(
				"/login",
				f!(crate::http::routes::auth::AuthenticatedSessionController),
			)
			.only(vec!["index", "store"])
			.middleware("guest");

		router
			.delete(
				"/logout",
				f!(crate::http::routes::auth::AuthenticatedSessionController::destroy),
			)
			.name(".logout")
			.middleware("auth");
	});

	router
}
