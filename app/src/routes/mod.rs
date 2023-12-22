use pavex::{
	blueprint::{router::GET, Blueprint},
	f,
};

pub mod system;

pub fn handler(bp: &mut Blueprint) {
	bp.route(GET, "/healthz", f!(crate::routes::system::health_check));
}
