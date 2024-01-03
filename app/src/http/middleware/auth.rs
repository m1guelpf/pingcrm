use std::future::IntoFuture;

use framework::http::{middleware::Next, response::Response, Redirect};
use pavex_session::Session;

pub struct EnsureLoggedIn {}

impl EnsureLoggedIn {
	pub async fn handle<C: IntoFuture<Output = Response>>(
		session: Session,
		next: Next<C>,
	) -> Response {
		dbg!("called auth middleware");

		if session.get::<u64>("auth.user").is_none() {
			return Redirect::to("/auth/login");
		}

		next.into_future().await
	}
}

pub struct RedirectToDashboard {}

impl RedirectToDashboard {
	pub async fn handle<C: IntoFuture<Output = Response>>(
		session: Session,
		next: Next<C>,
	) -> Response {
		dbg!("called guest middleware", &session);
		if session.get::<u64>("auth.user").is_some() {
			return Redirect::to("/dashboard");
		}

		next.into_future().await
	}
}
