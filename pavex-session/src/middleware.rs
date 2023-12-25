use std::future::IntoFuture;

use pavex::{
	http::{HeaderName, Method},
	middleware::Next,
	request::RequestHead,
	response::Response,
};
use pavex_cookies::{Cookie, CookieJar, RequestHeadCookiesExt, ResponseCookiesExt};
use rand::Rng;

use crate::{
	handlers::{self, FileHandler},
	Manager, SessionConfig, Store,
};

pub struct Middleware {}

impl Middleware {
	pub async fn handle<C>(
		req: &RequestHead,
		config: SessionConfig,
		manager: &Manager,
		next: Next<C>,
	) -> Response
	where
		C: IntoFuture<Output = Response>,
	{
		if config.driver.is_none() {
			return next.into_future().await;
		}

		let mut session = Self::get_session(&config.cookie_name, &req.cookies(), manager).await;
		Self::start_session(&mut session, req).await.unwrap();

		Self::collect_garbage(&mut session, config.lottery)
			.await
			.unwrap();

		let mut response = next.into_future().await;

		Self::store_current_url(&mut session, req).unwrap();
		Self::add_cookie_to_response(&session, &mut response, &config);

		Self::save_session(&mut session).await.unwrap();

		response
	}

	async fn get_session(
		cookie_name: &str,
		cookies: &CookieJar,
		manager: &Manager,
	) -> Store<FileHandler> {
		let mut session = manager.create_file_driver().await;
		session.set_id(
			cookies
				.get(cookie_name)
				.map(|cookie| cookie.value().to_string()),
		);

		session
	}

	async fn collect_garbage(
		session: &mut Store<FileHandler>,
		lottery: [u8; 2],
	) -> Result<(), handlers::file::Error> {
		let mut rng = rand::thread_rng();

		if rng.gen_range(1..lottery[1]) <= lottery[0] {
			session.collect_garbage().await?;
		}

		Ok(())
	}

	async fn start_session(
		session: &mut Store<FileHandler>,
		req: &RequestHead,
	) -> Result<(), handlers::file::Error> {
		session.set_request_on_handler(req).await.unwrap();
		session.start().await.unwrap();

		Ok(())
	}

	fn store_current_url(
		session: &mut Store<FileHandler>,
		req: &RequestHead,
	) -> Result<(), handlers::file::Error> {
		let is_xhr = req
			.headers
			.get(HeaderName::from_static("x-requested-with"))
			.map(|v| v.as_bytes() == b"XMLHttpRequest")
			.unwrap_or_default();

		if req.method == Method::GET && !is_xhr {
			session.set_previous_url(req.uri.to_string());
		}

		Ok(())
	}

	fn add_cookie_to_response(
		session: &Store<FileHandler>,
		response: &mut Response,
		config: &SessionConfig,
	) {
		let cookie = Cookie::build((config.cookie_name.clone(), session.id().to_string()))
			.secure(config.secure)
			.path(config.path.clone())
			.http_only(config.http_only)
			.build();

		response.add_cookie(cookie);
	}

	async fn save_session(session: &mut Store<FileHandler>) -> Result<(), handlers::file::Error> {
		session.save().await?;

		Ok(())
	}
}
