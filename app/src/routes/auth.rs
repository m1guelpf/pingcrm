use ensemble::{types::Hashed, Model};
use pavex::{
	http::StatusCode,
	request::body::JsonBody,
	response::{IntoResponse, Response},
};
use serde_json::json;

use crate::{
	frontend::{inertia::InertiaResponse, Inertia},
	models::User,
};

pub struct AuthenticatedSessionController;

#[derive(Debug, serde::Deserialize)]
pub struct LoginRequest {
	email: String,
	password: Hashed<String>,
}

impl AuthenticatedSessionController {
	pub fn create(inertia: &Inertia) -> InertiaResponse {
		inertia.render("Auth/Login", ())
	}

	/// Attempt to authenticate a user with the given credentials.
	///
	/// # Panics
	///
	/// This function will panic if the database query fails.
	pub async fn store(inertia: &Inertia, JsonBody(req): JsonBody<LoginRequest>) -> Response {
		let user = User::query()
			.r#where("email", '=', req.email)
			.r#where("password", '=', req.password)
			.first::<User>()
			.await
			.unwrap();

		let Some(user) = user else {
			return inertia
				.render(
					"Auth/Login",
					json!({
						"errors": {
							"email": ["Invalid email or password"]
						}
					}),
				)
				.into_response();
		};

		dbg!("logged in as {}", user.name);

		StatusCode::OK.into_response()
	}
}
