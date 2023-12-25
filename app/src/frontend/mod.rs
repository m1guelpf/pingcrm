use pavex::{
	http::{header, HeaderValue},
	request::route::RouteParams,
	response::Response,
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../frontend/dist/"]
struct StaticFiles;

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
pub fn serve_assets(RouteParams(StaticParams { path }): RouteParams<StaticParams>) -> Response {
	match StaticFiles::get(&format!("assets/{path}")) {
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
