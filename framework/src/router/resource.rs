use std::fmt::Display;

use pavex::{
	blueprint::{internals::RegisteredCallable, reflection::RawCallableIdentifiers},
	http::Method,
};

use super::{builder::OneOrMultiple, route::SerializedRoute, Router};

pub struct Resource<'r> {
	pub(crate) router: &'r mut Router,
	pub(crate) resource: SerializedResource,
}

impl<'r> Resource<'r> {
	pub(crate) fn new(router: &'r mut Router, path: &str, callable: RegisteredCallable) -> Self {
		let name = path
			.split('/')
			.filter(|s| !s.is_empty())
			.last()
			.expect("Could not find name of resource on path {path}");

		Self {
			router,
			resource: SerializedResource {
				callable,
				path: path.to_string(),
				middleware: Vec::new(),
				name: name.to_string(),
				methods: vec![
					ResourceMethod::Index,
					ResourceMethod::Create,
					ResourceMethod::Store,
					ResourceMethod::Show,
					ResourceMethod::Edit,
					ResourceMethod::Update,
					ResourceMethod::Destroy,
				],
			},
		}
	}

	pub(crate) fn api(mut self) -> Self {
		self.resource.methods = vec![
			ResourceMethod::Index,
			ResourceMethod::Store,
			ResourceMethod::Show,
			ResourceMethod::Update,
			ResourceMethod::Destroy,
		];

		self
	}

	pub fn only<M: Methods>(&mut self, methods: M) -> &mut Self {
		self.resource.methods = methods.get();

		self
	}

	pub fn except<M: Methods>(&mut self, methods: M) -> &mut Self {
		let methods = methods.get();

		self.resource
			.methods
			.retain(|method| !methods.contains(method));

		self
	}

	pub fn middleware<M: OneOrMultiple>(&mut self, middleware: M) -> &mut Self {
		self.resource.middleware = middleware.get();

		self
	}

	pub fn name(&mut self, name: &str) -> &mut Self {
		self.resource.name = name.to_string();

		self
	}
}

impl<'r> Drop for Resource<'r> {
	fn drop(&mut self) {
		for method in &self.resource.methods {
			let mut route = method.route_for(
				&self.resource.path,
				&self.resource.name,
				&self.resource.callable,
			);
			route.middleware = self.resource.middleware.clone();

			self.router.store_route(route)
		}
	}
}

pub struct SerializedResource {
	pub(crate) name: String,
	pub(crate) path: String,
	pub(crate) middleware: Vec<String>,
	pub(crate) methods: Vec<ResourceMethod>,
	pub(crate) callable: RegisteredCallable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceMethod {
	Index,
	Create,
	Store,
	Show,
	Edit,
	Update,
	Destroy,
}

impl ResourceMethod {
	fn route_for(&self, path: &str, name: &str, callable: &RegisteredCallable) -> SerializedRoute {
		let singular_resource = pluralizer::pluralize(name, 1, false);

		let path = match self {
			Self::Create => format!("{path}/create"),
			Self::Index | Self::Store => path.to_string(),
			Self::Edit => format!("{path}/:{singular_resource}/edit"),
			Self::Show | Self::Update | Self::Destroy => {
				format!("{path}/:{singular_resource}")
			},
		};

		let method = match self {
			Self::Store => Method::POST,
			Self::Update => Method::PUT,
			Self::Destroy => Method::DELETE,
			Self::Index | Self::Create | Self::Show | Self::Edit => Method::GET,
		};

		SerializedRoute {
			path,
			method,
			error_handler: None,
			middleware: Vec::new(),
			name: Some(format!("{name}.{self}")),
			request_handler: RegisteredCallable {
				location: callable.location.clone(),
				callable: RawCallableIdentifiers::from_raw_parts(
					format!("{}::{self}", callable.callable.raw_path()),
					callable.callable.registered_at().to_string(),
				),
			},
		}
	}
}

impl Display for ResourceMethod {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Index => write!(f, "index"),
			Self::Create => write!(f, "create"),
			Self::Store => write!(f, "store"),
			Self::Show => write!(f, "show"),
			Self::Edit => write!(f, "edit"),
			Self::Update => write!(f, "update"),
			Self::Destroy => write!(f, "destroy"),
		}
	}
}

impl From<&str> for ResourceMethod {
	fn from(method: &str) -> Self {
		match method {
			"edit" => Self::Edit,
			"show" => Self::Show,
			"index" => Self::Index,
			"store" => Self::Store,
			"update" => Self::Update,
			"create" => Self::Create,
			"destroy" => Self::Destroy,
			_ => panic!("Invalid resource method: {}", method),
		}
	}
}

pub trait Methods {
	fn get(self) -> Vec<ResourceMethod>;
}

impl Methods for ResourceMethod {
	fn get(self) -> Vec<ResourceMethod> {
		vec![self]
	}
}

impl Methods for Vec<ResourceMethod> {
	fn get(self) -> Vec<ResourceMethod> {
		self
	}
}

impl Methods for Vec<&str> {
	fn get(self) -> Vec<ResourceMethod> {
		self.into_iter().map(ResourceMethod::from).collect()
	}
}
