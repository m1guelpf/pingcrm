use std::collections::HashMap;

use pavex::{
	blueprint::{
		internals::{RegisteredCallable, RegisteredRoute, RegisteredWrappingMiddleware},
		reflection::{RawCallable, RawCallableIdentifiers},
		Blueprint,
	},
	http::Method,
};

use self::{
	builder::{Builder, OneOrMultiple},
	group::{GroupRouter, RouteGroup, SerializedRouteGroup},
	resource::Resource,
	route::{Route, SerializedRoute},
};

mod builder;
mod group;
mod resource;
mod route;

#[derive(Default)]
pub struct Router {
	routes: Vec<SerializedRoute>,
}

impl Router {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn name(&mut self, name: &str) -> Builder<'_> {
		Builder {
			path: None,
			router: self,
			middleware: Vec::new(),
			name: Some(name.to_string()),
		}
	}

	pub fn path(&mut self, path: &str) -> Builder<'_> {
		Builder {
			name: None,
			router: self,
			middleware: Vec::new(),
			path: Some(path.to_string()),
		}
	}

	pub fn prefix(&mut self, prefix: &str) -> Builder<'_> {
		Builder {
			name: None,
			router: self,
			middleware: Vec::new(),
			path: Some(prefix.to_string()),
		}
	}

	pub fn middleware<M: OneOrMultiple>(&mut self, middleware: M) -> Builder<'_> {
		Builder {
			path: None,
			name: None,
			router: self,
			middleware: middleware.get(),
		}
	}

	pub fn group(&mut self, group: impl FnOnce(GroupRouter<'_>)) -> RouteGroup<'_> {
		let mut router = Router::new();

		group(GroupRouter {
			router: &mut router,
		});

		RouteGroup {
			router: self,
			group: SerializedRouteGroup {
				routes: router.routes,
				..Default::default()
			},
		}
	}

	#[track_caller]
	pub fn resource(&mut self, path: &str, callable: RawCallable) -> Resource<'_> {
		Resource::new(
			self,
			path,
			RegisteredCallable {
				location: std::panic::Location::caller().into(),
				callable: RawCallableIdentifiers::from_raw_callable(callable.clone()),
			},
		)
	}

	#[track_caller]
	pub fn api_resource(&mut self, path: &str, callable: RawCallable) -> Resource<'_> {
		Resource::new(
			self,
			path,
			RegisteredCallable {
				location: std::panic::Location::caller().into(),
				callable: RawCallableIdentifiers::from_raw_callable(callable.clone()),
			},
		)
		.api()
	}

	pub(crate) fn register(
		mut self,
		blueprint: &mut Blueprint,
		middleware: &HashMap<String, RegisteredCallable>,
	) {
		for route in self.routes.drain(..) {
			let registered_route = RegisteredRoute {
				path: route.path.clone(),
				method_guard: route.method.into(),
				request_handler: route.request_handler,
				error_handler: route.error_handler,
			};

			if route.middleware.is_empty() {
				blueprint.routes.push(registered_route);
				continue;
			}

			let mut middleware = route
				.middleware
				.into_iter()
				.map(|name| {
					middleware.get(&name).unwrap_or_else(|| {
						panic!(
							"Route [{}] expects middleware `{name}`, but it is not registered.",
							route.path
						)
					})
				})
				.collect::<Vec<_>>();

			let mut nested_bp = Blueprint::new();
			nested_bp.routes.push(registered_route);

			for middleware in middleware.drain(..) {
				nested_bp.middlewares.push(RegisteredWrappingMiddleware {
					middleware: RegisteredCallable {
						callable: middleware.callable.clone(),
						location: middleware.location.clone(),
					},
					error_handler: None,
				})
			}

			blueprint.nest(nested_bp);
		}
	}

	fn store_route(&mut self, route: SerializedRoute) {
		self.routes.push(route);
	}
}

macro_rules! impl_method {
	($fn_name:ident, $method:ident) => {
		impl Router {
			#[track_caller]
			pub fn $fn_name(&mut self, path: &str, callable: RawCallable) -> Route<'_> {
				Route {
					router: self,
					route: SerializedRoute {
						path: path.to_string(),
						method: Method::$method,
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
