use pavex::{blueprint::reflection::RawCallable, f};
use phf::phf_map;

pub mod auth;

pub const MIDDLEWARE: phf::Map<&str, RawCallable> = phf_map! {
	"auth" => f!(crate::http::middleware::auth::EnsureLoggedIn::handle),
	"guest" => f!(crate::http::middleware::auth::RedirectToDashboard::handle),
};
