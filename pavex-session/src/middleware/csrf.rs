use std::{env, future::IntoFuture};

use pavex::{
	http::{HeaderName, StatusCode},
	middleware::Next,
	request::RequestHead,
	response::{IntoResponse, Response},
};

use crate::{CsrfConfig, Session, SessionConfig};

pub struct VerifyCsrfToken {}

impl VerifyCsrfToken {
	pub async fn handle<C: IntoFuture<Output = Response>>(
		req: &RequestHead,
		session: &Session,
		config: &SessionConfig,
		next: Next<C>,
	) -> Result<Response, TokenMismatchError> {
		if Self::is_reading(req)
			|| Self::running_tests()
			|| Self::is_excluded(req, &config.csrf)
			|| Self::tokens_match(req, session)
		{
			let response = next.into_future().await;

			return Ok(response);
		}

		Err(TokenMismatchError {})
	}

	fn is_reading(req: &RequestHead) -> bool {
		req.method == "GET" || req.method == "HEAD" || req.method == "OPTIONS"
	}

	fn running_tests() -> bool {
		let env = env::var("APP_ENV").unwrap_or_default();

		env == "test" || env == "testing"
	}

	fn is_excluded(req: &RequestHead, config: &CsrfConfig) -> bool {
		config
			.exclude_paths
			.iter()
			.any(|route| req.target.path() == route)
	}

	fn tokens_match(req: &RequestHead, session: &Session) -> bool {
		let token = Self::get_token(req);

		token.is_some() && token == session.token()
	}

	fn get_token(req: &RequestHead) -> Option<String> {
		let Some(token) = req.headers.get(HeaderName::from_static("x-csrf-token")) else {
			return None;
		};

		token.to_str().map(ToString::to_string).ok()
	}
}

#[derive(Debug, thiserror::Error)]
#[error("CSRF token mismatch.")]
pub struct TokenMismatchError {}

impl IntoResponse for TokenMismatchError {
	fn into_response(self) -> Response {
		Response::new(StatusCode::from_u16(419).unwrap()).set_typed_body(self.to_string())
	}
}
