use ensemble::{types::Hashed, Model};
use pavex::{
	blueprint::{
		router::{GET, POST},
		Blueprint,
	},
	f,
	http::StatusCode,
	request::body::JsonBody,
	response::{IntoResponse, Response},
};
use pavex_session::Session;
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
	pub async fn store(
		inertia: &Inertia,
		mut session: Session,
		JsonBody(req): JsonBody<LoginRequest>,
	) -> Response {
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

		session.set("auth.user", user.id);

		StatusCode::OK.into_response()
	}
}

pub fn routes() -> Blueprint {
	let mut bp = Blueprint::new();

	bp.route(
		GET,
		"/login",
		f!(crate::routes::auth::AuthenticatedSessionController::create),
	);
	bp.route(
		POST,
		"/login",
		f!(crate::routes::auth::AuthenticatedSessionController::store),
	);

	bp
}
