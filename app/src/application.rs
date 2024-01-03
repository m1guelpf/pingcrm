use framework::{application::CloningStrategy::CloneIfNecessary, f, Application};

use crate::{
	frontend,
	http::{middleware::MIDDLEWARE, routes},
};

#[must_use]
pub fn booststrap() -> Application {
	// Initialize the application...
	Application::new()
        // with request-augmenting telemetry...
		.with_telemetry()
        // register the application's middleware map...
		.middleware(&MIDDLEWARE)
        // register the application's routes...
		.routes(routes::handler)
        // initialize the application's session...
		.register(pavex_session::register)
        // register the application's frontend layer
		.register(frontend::register)
        // and register the session config as a singleton.
		.singleton(f!(crate::config::session_config), CloneIfNecessary, None)
}
