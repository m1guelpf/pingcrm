use cookie::{Cookie, CookieJar};
use pavex::{
	http::{header, HeaderName, HeaderValue, Method},
	middleware::Next,
	request::RequestHead,
	response::Response,
};
use rand::Rng;
use std::future::IntoFuture;

use crate::{handlers, Handler, Manager, Session, SessionConfig};

pub struct StartSession {}

impl StartSession {
	pub async fn handle<C: IntoFuture<Output = Response>>(
		req: &RequestHead,
		mut session: Session,
		config: SessionConfig,
		manager: &Manager,
		next: Next<C>,
	) -> Response {
		let session_id = req
			.cookies()
			.get(&config.cookie_name)
			.map(|cookie| cookie.value().to_string());

		let mut session_backend = manager.get_backend().await;

		let session_contents = match session_id.as_ref() {
			None => Default::default(),
			Some(session_id) => session_backend.read(session_id).await.unwrap_or_default(),
		};

		session.start(session_id, session_contents).unwrap();

		Self::collect_garbage(&mut session_backend, &config)
			.await
			.unwrap();

		let mut response = next.into_future().await;

		Self::store_current_url(&mut session, req).unwrap();
		Self::add_cookie_to_response(&session, &mut response, &config);

		#[allow(clippy::unnecessary_to_owned)]
		session_backend
			.write(&session.id().to_string(), session.end().unwrap())
			.await
			.unwrap();

		response
	}

	async fn collect_garbage(
		backend: &mut impl Handler,
		config: &SessionConfig,
	) -> Result<(), handlers::file::Error> {
		let mut rng = rand::thread_rng();

		if rng.gen_range(1..config.lottery[1]) <= config.lottery[0] {
			backend.collect_garbage(&config.lifetime).await.unwrap();
		}

		Ok(())
	}

	fn store_current_url(
		session: &mut Session,
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

	fn add_cookie_to_response(session: &Session, response: &mut Response, config: &SessionConfig) {
		let cookie = Cookie::build((config.cookie_name.clone(), session.id().to_string()))
			.secure(config.secure)
			.path(config.path.clone())
			.http_only(config.http_only)
			.build();

		response.headers_mut().append(
			header::SET_COOKIE,
			HeaderValue::from_str(&cookie.to_string()).unwrap(),
		);
	}
}

pub trait RequestHeadCookiesExt {
	fn cookies(&self) -> CookieJar;
}

impl RequestHeadCookiesExt for RequestHead {
	fn cookies(&self) -> CookieJar {
		let mut jar = CookieJar::new();

		for cookies in self
			.headers
			.get_all(header::COOKIE)
			.into_iter()
			.filter_map(|value| value.to_str().ok())
			.flat_map(|value| value.split(';'))
			.filter_map(|cookie| Cookie::parse_encoded(cookie.to_owned()).ok())
		{
			jar.add(cookies);
		}

		jar
	}
}
