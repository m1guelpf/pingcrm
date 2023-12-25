#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

mod blueprint;
pub mod config;
pub mod frontend;
pub mod routes;
pub mod telemetry;

pub use blueprint::blueprint;
