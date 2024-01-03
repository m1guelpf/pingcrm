use std::ops::{Deref, DerefMut};

use super::{builder::OneOrMultiple, route::SerializedRoute, Router};

pub struct RouteGroup<'r> {
	pub(crate) router: &'r mut Router,
	pub(crate) group: SerializedRouteGroup,
}

impl<'r> RouteGroup<'r> {
	pub fn name(&mut self, name: &str) -> &mut Self {
		self.group.name = Some(name.to_string());
		self
	}

	pub fn middleware<M: OneOrMultiple>(&mut self, middleware: M) -> &mut Self {
		self.group.middleware = middleware.get();

		self
	}

	pub fn prefix(&mut self, prefix: &str) -> &mut Self {
		self.group.prefix = Some(prefix.to_string());
		self
	}
}

impl<'r> Drop for RouteGroup<'r> {
	fn drop(&mut self) {
		for mut route in self.group.routes.drain(..) {
			if let Some(prefix) = &self.group.prefix {
				route.path = format!("{prefix}{}", route.path);
			}

			if let Some(name) = &self.group.name {
				if let Some(route_name) = route.name {
					route.name = Some(format!("{name}{route_name}"));
				} else {
					route.name = Some(name.clone());
				}
			}

			self.router.store_route(route);
		}
	}
}

#[derive(Clone, Default)]
pub struct SerializedRouteGroup {
	pub(crate) name: Option<String>,
	pub(crate) prefix: Option<String>,
	pub(crate) middleware: Vec<String>,
	pub(crate) routes: Vec<SerializedRoute>,
}

pub struct GroupRouter<'r> {
	pub(crate) router: &'r mut Router,
}

impl Deref for GroupRouter<'_> {
	type Target = Router;

	fn deref(&self) -> &Router {
		self.router
	}
}

impl DerefMut for GroupRouter<'_> {
	fn deref_mut(&mut self) -> &mut Router {
		self.router
	}
}
