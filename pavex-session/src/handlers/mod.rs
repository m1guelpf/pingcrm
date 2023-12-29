use std::{
	collections::HashMap,
	error::Error,
	fmt::Debug,
	future::{self, Future},
	time::Duration,
};

pub mod file;

pub use file::FileHandler;

pub trait Handler {
	type Error: Debug + Error;

	fn read(
		&mut self,
		id: &str,
	) -> impl Future<Output = Result<HashMap<String, serde_json::Value>, Self::Error>>;

	fn write<T: serde::Serialize>(
		&mut self,
		id: &str,
		attributes: T,
	) -> impl Future<Output = Result<(), Self::Error>>;

	fn destroy(&mut self, id: &str) -> impl Future<Output = Result<(), Self::Error>>;

	fn collect_garbage(
		&mut self,
		_max_lifetime: &Duration,
	) -> impl Future<Output = Result<u64, Self::Error>> {
		future::ready(Ok(0))
	}
}
