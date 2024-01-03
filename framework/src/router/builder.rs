use pavex::{
	blueprint::{
		internals::RegisteredCallable,
		reflection::{RawCallable, RawCallableIdentifiers},
	},
	http::Method,
};

use super::{
	group::{GroupRouter, RouteGroup},
	route::{Route, SerializedRoute},
	Router,
};

pub struct Builder<'r> {
	pub(crate) router: &'r mut Router,
	pub(crate) path: Option<String>,
	pub(crate) name: Option<String>,
	pub(crate) middleware: Vec<String>,
}

impl<'r> Builder<'r> {
	pub fn name(&mut self, name: &str) -> &mut Self {
		self.name = Some(name.to_string());
		self
	}

	pub fn path(&mut self, path: &str) -> &mut Self {
		self.path = Some(path.to_string());
		self
	}

	pub fn middleware<M: OneOrMultiple>(&mut self, middleware: M) -> &mut Self {
		self.middleware = middleware.get();

		self
	}

	pub fn prefix(&mut self, prefix: &str) -> &mut Self {
		self.path = Some(prefix.to_string());

		self
	}

	pub fn group(&mut self, group: impl FnOnce(GroupRouter<'_>)) -> RouteGroup<'_> {
		let mut group = self.router.group(group);

		if let Some(name) = &self.name {
			group.name(name);
		}

		if let Some(path) = &self.path {
			group.prefix(path);
		}

		group
	}
}

macro_rules! impl_method {
	($fn_name:ident, $method:ident) => {
		impl<'r> Builder<'r> {
			#[track_caller]
			pub fn $fn_name(&mut self, path: &str, callable: RawCallable) -> Route<'_> {
				Route {
					router: self.router,
					route: SerializedRoute {
						name: self.name.clone(),
						method: Method::$method,
						middleware: self.middleware.clone(),
						path: self
							.path
							.as_ref()
							.map_or_else(|| path.to_string(), |prefix| format!("{prefix}{path}")),
						request_handler: RegisteredCallable {
							location: std::panic::Location::caller().into(),
							callable: RawCallableIdentifiers::from_raw_callable(callable),
						},
						..Default::default()
					},
				}
			}
		}
	};
}

impl_method!(get, GET);
impl_method!(put, PUT);
impl_method!(post, POST);
impl_method!(head, HEAD);
impl_method!(patch, PATCH);
impl_method!(delete, DELETE);
impl_method!(options, OPTIONS);

pub trait OneOrMultiple {
	fn get(self) -> Vec<String>;
}

impl OneOrMultiple for Vec<&str> {
	fn get(self) -> Vec<String> {
		self.into_iter().map(|s| s.to_string()).collect()
	}
}

impl OneOrMultiple for &str {
	fn get(self) -> Vec<String> {
		vec![self.to_string()]
	}
}
