use pavex::request::RequestHead;
use std::{
	collections::HashMap,
	error::Error,
	fmt::Debug,
	future::{self, Future},
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

	fn set_exists(&mut self, _: bool) -> impl Future<Output = Result<(), Self::Error>> {
		future::ready(Ok(()))
	}

	fn set_request(&mut self, _: &RequestHead) -> impl Future<Output = Result<(), Self::Error>> {
		future::ready(Ok(()))
	}

	fn garbage_collect(&mut self) -> impl Future<Output = Result<(), Self::Error>> {
		future::ready(Ok(()))
	}
}
