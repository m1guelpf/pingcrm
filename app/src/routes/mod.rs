#![allow(clippy::must_use_candidate)]

use pavex::{
	blueprint::{router::GET, Blueprint},
	f,
};

pub mod auth;
pub mod system;

pub fn handler(bp: &mut Blueprint) {
	bp.route(GET, "/healthz", f!(crate::routes::system::health_check));

	bp.nest_at("/auth", auth::routes());
}
