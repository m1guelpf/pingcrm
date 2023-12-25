use pavex::{
	http::{header, HeaderValue},
	request::RequestHead,
	response::Response,
};

pub use cookie::{Cookie, CookieBuilder, CookieJar};

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

pub trait ResponseCookiesExt {
	fn add_cookie(&mut self, cookie: Cookie<'static>);
}

impl ResponseCookiesExt for Response {
	fn add_cookie(&mut self, cookie: Cookie<'_>) {
		self.headers_mut().append(
			header::SET_COOKIE,
			HeaderValue::from_str(&cookie.to_string()).unwrap(),
		);
	}
}
