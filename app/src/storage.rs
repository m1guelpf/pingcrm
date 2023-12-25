use flysystem::{adapters::LocalAdapter, Filesystem};
use std::path::PathBuf;

/// Register the local storage adapter.
///
/// # Panics
///
/// This function will panic if the adapter fails to initialize.
pub async fn register() -> Filesystem<LocalAdapter> {
	Filesystem::new(flysystem::adapters::local::Config {
		lazy_root_creation: true,
		location: PathBuf::from("./storage/"),
	})
	.await
	.unwrap()
}
