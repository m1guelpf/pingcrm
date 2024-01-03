use std::{collections::HashMap, ops::Deref};

use pavex::{
	blueprint::{
		internals::RegisteredCallable,
		reflection::{Location, RawCallable, RawCallableIdentifiers},
	},
	f,
	http::Method,
};

use super::{builder::OneOrMultiple, Router};

pub struct Route<'r> {
	pub(crate) router: &'r mut Router,
	pub(crate) route: SerializedRoute,
}

impl<'r> Route<'r> {
	pub fn name(&mut self, name: &str) -> &mut Self {
		self.route.name = Some(name.to_string());
		self
	}

	pub fn middleware<M: OneOrMultiple>(&mut self, middleware: M) -> &mut Self {
		self.route.middleware = middleware.get();

		self
	}

	#[track_caller]
	pub fn error_handler(&mut self, callable: RawCallable) -> &mut Self {
		self.route.error_handler = Some(RegisteredCallable {
			callable: RawCallableIdentifiers::from_raw_callable(callable),
			location: std::panic::Location::caller().into(),
		});

		self
	}
}

impl<'r> Drop for Route<'r> {
	fn drop(&mut self) {
		self.router.store_route(self.route.clone());
	}
}

pub struct SerializedRoute {
	pub(crate) path: String,
	pub(crate) method: Method,
	pub(crate) name: Option<String>,
	pub(crate) middleware: Vec<String>,
	pub(crate) request_handler: RegisteredCallable,
	pub(crate) error_handler: Option<RegisteredCallable>,
}

impl Clone for SerializedRoute {
	fn clone(&self) -> Self {
		Self {
			path: self.path.clone(),
			name: self.name.clone(),
			method: self.method.clone(),
			middleware: self.middleware.clone(),
			request_handler: RegisteredCallable {
				callable: self.request_handler.callable.clone(),
				location: self.request_handler.location.clone(),
			},
			error_handler: self
				.error_handler
				.as_ref()
				.map(|handler| RegisteredCallable {
					callable: handler.callable.clone(),
					location: handler.location.clone(),
				}),
		}
	}
}

impl Default for SerializedRoute {
	fn default() -> Self {
		Self {
			name: None,
			path: String::new(),
			method: Method::GET,
			error_handler: None,
			middleware: Vec::new(),
			request_handler: RegisteredCallable {
				location: Location {
					line: line!(),
					column: column!(),
					file: file!().to_string(),
				},
				callable: RawCallableIdentifiers::from_raw_callable(f!(core::unreachable)),
			},
		}
	}
}

pub struct NamedRoutes(HashMap<String, String>);

impl From<&[SerializedRoute]> for NamedRoutes {
	fn from(routes: &[SerializedRoute]) -> Self {
		let mut named_routes = HashMap::new();

		for route in routes {
			if let Some(name) = &route.name {
				named_routes.insert(name.clone(), route.path.clone());
			}
		}

		Self(named_routes)
	}
}

impl Deref for NamedRoutes {
	type Target = HashMap<String, String>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
