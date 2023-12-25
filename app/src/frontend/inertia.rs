#![allow(clippy::module_name_repetitions)]

use indoc::formatdoc;
use pavex::{
	http::{header, HeaderName, HeaderValue, Method, StatusCode},
	middleware::Next,
	request::RequestHead,
	response::{
		body::{raw::RawBody, Html, Json},
		IntoResponse, Response,
	},
};
use sha256::digest as sha256;
use std::{future::IntoFuture, sync::Arc};

use crate::frontend::vite::Vite;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Page<T = serde_json::Value> {
	pub props: T,
	pub url: String,
	pub version: Option<String>,
	pub component: &'static str,
}

#[derive(Debug, Clone)]
pub struct Inertia {
	vite: Arc<Vite>,
	version: Option<String>,
	request: Option<InertiaRequest>,
}

impl Inertia {
	/// Creates a new Inertia instance with the details of the current request.
	///
	/// # Panics
	///
	/// This function will panic if it fails to serialize the frontend manifest.
	#[must_use]
	pub fn new(vite: &Vite, request: InertiaRequest) -> Self {
		let version = if let Vite::Production { manifest } = &vite {
			Some(sha256(serde_json::to_string(&manifest).unwrap()))
		} else {
			None
		};

		Self {
			version,
			request: Some(request),
			vite: Arc::new(vite.clone()),
		}
	}

	#[must_use]
	#[allow(clippy::unused_self)]
	/// Returns a frontend redirect to the given URL.
	///
	/// # Panics
	///
	/// This function will panic if the given URL is not valid UTF-8.
	pub fn redirect(&self, url: &str) -> Response {
		Response::conflict()
			.append_header(
				HeaderName::from_static("x-inertia-location"),
				HeaderValue::from_str(url).unwrap(),
			)
			.box_body()
	}

	/// Returns an Inertia response.
	///
	/// # Panics
	///
	/// Panics if called outside of a request context or if it fails to serialize the given props.
	pub fn render<T: serde::Serialize>(
		&self,
		component: &'static str,
		props: T,
	) -> InertiaResponse {
		let request = self.request.clone().unwrap();

		let page = Page {
			component,
			url: request.path.clone(),
			version: self.version.clone(),
			props: serde_json::to_value(props).unwrap(),
		};

		InertiaResponse {
			page,
			request,
			vite: self.vite.clone(),
		}
	}
}

/// Middleware that handles Inertia requests.
pub async fn middleware<C>(inertia: &Inertia, request: &RequestHead, next: Next<C>) -> Response
where
	C: IntoFuture<Output = Response> + Send,
	C::IntoFuture: Send,
{
	if let Some(inertia_req) = &inertia.request {
		if matches!(request.method, Method::GET) && inertia.version != inertia_req.version {
			return inertia.redirect(&inertia_req.path);
		}
	}

	let mut response = next
		.into_future()
		.await
		.append_header(header::VARY, HeaderValue::from_static("x-inertia"));

	let has_empty_body = response.body().size_hint().exact() == Some(0);
	let referer = request
		.headers
		.get(header::REFERER)
		.and_then(|header| header.to_str().ok());
	if response.status() == StatusCode::OK && has_empty_body && referer.is_some() {
		return inertia.redirect(referer.unwrap_or_else(|| unreachable!()));
	}

	if response.status() == StatusCode::FOUND
		&& matches!(request.method, Method::PUT | Method::PATCH | Method::DELETE)
	{
		response = response.set_status(StatusCode::SEE_OTHER);
	}

	response
}

#[derive(Debug, Clone)]
pub struct InertiaRequest {
	path: String,
	is_xhr: bool,
	version: Option<String>,
}

impl InertiaRequest {
	pub fn new(request: &RequestHead) -> Self {
		Self {
			path: request.uri.path().to_string(),
			is_xhr: request
				.headers
				.get("X-Inertia")
				.is_some_and(|header| header == "true"),
			version: request
				.headers
				.get("X-Inertia-Version")
				.and_then(|header| header.to_str().map(ToString::to_string).ok()),
		}
	}
}

pub struct InertiaResponse {
	page: Page,
	vite: Arc<Vite>,
	request: InertiaRequest,
}

impl InertiaResponse {
	fn html_page(&self) -> String {
		formatdoc! {r#"
            <!doctype html>
            <html lang="en">
                <head>
                    <title>PingCRM</title>
                    <meta charset="UTF-8" />
                    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                    {}{}
                </head>
                <body>
                    <div id="app" data-page='{}'></div>
                </body>
            </html>
        "#, self.vite.dev_scripts().unwrap_or_default(), self.vite.asset("src/index.tsx").unwrap(), serde_json::to_string(&self.page).unwrap()}
	}
}

impl IntoResponse for InertiaResponse {
	fn into_response(self) -> Response {
		let response = Response::ok().append_header(
			HeaderName::from_static("x-inertia"),
			HeaderValue::from_str("true").unwrap(),
		);

		if self.request.is_xhr {
			response
				.set_typed_body(Json::new(self.page).unwrap())
				.box_body()
		} else {
			response
				.set_typed_body(Html::from(Self::html_page(&self)))
				.box_body()
		}
	}
}
