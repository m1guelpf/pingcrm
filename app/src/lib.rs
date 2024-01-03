#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::future_not_send)]

#[derive(RustEmbed)]
#[folder = "../frontend/dist/"]
pub struct FrontendDist;

mod application;
pub mod config;
pub mod frontend;
pub mod http;
pub mod models;

pub use application::booststrap;

use rust_embed::RustEmbed;
