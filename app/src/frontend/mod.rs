use pavex::{
	blueprint::{constructor::Lifecycle, router::GET, Blueprint},
	f,
	http::{header, HeaderValue},
	request::route::RouteParams,
	response::Response,
};
use rust_embed::RustEmbed;

pub mod inertia;
pub mod vite;

pub use inertia::Inertia;
pub use vite::Vite;

#[RouteParams]
pub struct StaticParams {
	path: String,
}

/// Serve static assets from the `assets` build directory.
///
/// # Errors
///
/// Returns a 404 response if the file is not found.
pub fn serve_assets<E: RustEmbed>(
	RouteParams(StaticParams { path }): RouteParams<StaticParams>,
) -> Response {
	match E::get(&format!("assets/{path}")) {
		Some(content) => {
			let mime = mime_guess::from_path(path).first_or_octet_stream();

			Response::ok()
				.append_header(
					header::CONTENT_TYPE,
					HeaderValue::from_str(mime.as_ref()).unwrap_or_else(|_| unreachable!()),
				)
				.set_typed_body(content.data)
				.box_body()
		},
		None => Response::not_found().box_body(),
	}
}

/// Register all required constructors and routes for Vite and Inertia.
/// Note that the type parameter `E` should be a fully qualified import path.
pub fn register(bp: &mut Blueprint) {
	// Vite
	bp.constructor(
		f!(crate::frontend::Vite::shared::<crate::FrontendDist>),
		Lifecycle::Singleton,
	);
	bp.route(
		GET,
		"/assets/*path",
		f!(crate::frontend::serve_assets::<crate::FrontendDist>),
	);

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
