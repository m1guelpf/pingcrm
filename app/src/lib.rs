#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::future_not_send)]

mod blueprint;
pub mod config;
pub mod frontend;
pub mod models;
pub mod routes;
pub mod telemetry;

pub use blueprint::blueprint;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../frontend/dist/"]
pub struct FrontendDist;
