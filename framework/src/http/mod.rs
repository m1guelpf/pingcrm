use pavex::response::Response;

pub use pavex::{http::*, middleware, request, response};

pub struct Redirect {}

impl Redirect {
	pub fn to(url: &str) -> Response {
		Response::temporary_redirect()
			.insert_header(header::LOCATION, HeaderValue::from_str(url).unwrap())
	}
}
