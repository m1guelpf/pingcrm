use pavex::{
	blueprint::{
		constructor::{CloningStrategy, Lifecycle},
		Blueprint,
	},
	f,
};

use crate::routes;

#[must_use]
/// The main blueprint, containing all the routes, constructors and error handlers required by our API.
pub fn blueprint() -> Blueprint {
	let mut bp = Blueprint::new();

	register_common_constructors(&mut bp);
	register_frontend_constructors(&mut bp);
	add_telemetry_middleware(&mut bp);
	routes::handler(&mut bp);

	bp
}

/// Common constructors used by all routes.
fn register_common_constructors(bp: &mut Blueprint) {
	// Query parameters
	bp.constructor(
		f!(pavex::request::query::QueryParams::extract),
		Lifecycle::RequestScoped,
	)
	.error_handler(f!(
		pavex::request::query::errors::ExtractQueryParamsError::into_response
	));

	// Route parameters
	bp.constructor(
		f!(pavex::request::route::RouteParams::extract),
		Lifecycle::RequestScoped,
	)
	.error_handler(f!(
		pavex::request::route::errors::ExtractRouteParamsError::into_response
	));

	// Json body
	bp.constructor(
		f!(pavex::request::body::JsonBody::extract),
		Lifecycle::RequestScoped,
	)
	.error_handler(f!(
		pavex::request::body::errors::ExtractJsonBodyError::into_response
	));
	bp.constructor(
		f!(pavex::request::body::BufferedBody::extract),
		Lifecycle::RequestScoped,
	)
	.error_handler(f!(
		pavex::request::body::errors::ExtractBufferedBodyError::into_response
	));
	bp.constructor(
		f!(<pavex::request::body::BodySizeLimit as std::default::Default>::default),
		Lifecycle::RequestScoped,
	);
}

fn register_frontend_constructors(bp: &mut Blueprint) {
	// Vite
	bp.constructor(f!(crate::frontend::vite::Vite::new), Lifecycle::Singleton);

	// Inertia
	bp.constructor(
		f!(crate::frontend::inertia::InertiaRequest::new),
		Lifecycle::RequestScoped,
	);
	bp.constructor(
		f!(crate::frontend::inertia::Inertia::new),
		Lifecycle::RequestScoped,
	);
	bp.wrap(f!(crate::frontend::inertia::middleware));
}

/// Add the telemetry middleware, as well as the constructors of its dependencies.
fn add_telemetry_middleware(bp: &mut Blueprint) {
	bp.constructor(
		f!(crate::telemetry::RootSpan::new),
		Lifecycle::RequestScoped,
	)
	.cloning(CloningStrategy::CloneIfNecessary);

	bp.wrap(f!(crate::telemetry::logger));
}
