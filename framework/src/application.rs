use std::collections::HashMap;

use pavex::{
	blueprint::{
		constructor::Lifecycle,
		internals::RegisteredCallable,
		reflection::{RawCallable, RawCallableIdentifiers},
		Blueprint,
	},
	f,
	request::{
		body::{BodySizeLimit, BufferedBody, JsonBody},
		path::PathParams,
		query::QueryParams,
	},
};

pub use pavex::blueprint::constructor::CloningStrategy;

use self::CallbackType::{GlobalMiddleware, RegisterRoutes};
use crate::router::Router;

#[derive(Default)]
pub struct Application {
	blueprint: Blueprint,
	callbacks: Vec<Callback>,
	middleware: HashMap<String, RegisteredCallable>,
}

impl Application {
	pub fn new() -> Self {
		Self::default().default_extractors()
	}

	pub fn bind(mut self, constructor: RawCallable, error_handler: Option<RawCallable>) -> Self {
		let constructor = self
			.blueprint
			.constructor(constructor, Lifecycle::Transient);

		if let Some(error_handler) = error_handler {
			constructor.error_handler(error_handler);
		}

		self
	}

	pub fn request_scoped(
		mut self,
		constructor: RawCallable,
		cloning_strategy: CloningStrategy,
		error_handler: Option<RawCallable>,
	) -> Self {
		let constructor = self
			.blueprint
			.constructor(constructor, Lifecycle::RequestScoped)
			.cloning(cloning_strategy);

		if let Some(error_handler) = error_handler {
			constructor.error_handler(error_handler);
		}

		self
	}

	pub fn singleton(
		mut self,
		constructor: RawCallable,
		cloning_strategy: CloningStrategy,
		error_handler: Option<RawCallable>,
	) -> Self {
		let constructor = self
			.blueprint
			.constructor(constructor, Lifecycle::Singleton)
			.cloning(cloning_strategy);

		if let Some(error_handler) = error_handler {
			constructor.error_handler(error_handler);
		}

		self
	}

	pub fn global_middleware(mut self, middleware: RawCallable) -> Self {
		self.callbacks.push(Callback::new(GlobalMiddleware, |app| {
			app.blueprint.wrap(middleware);
		}));

		self
	}

	#[track_caller]
	pub fn middleware<'a, I: IntoIterator<Item = (&'a &'static str, &'a RawCallable)>>(
		mut self,
		middleware: I,
	) -> Self {
		for (name, callable) in middleware.into_iter() {
			self.middleware.insert(
				name.to_string(),
				RegisteredCallable {
					location: std::panic::Location::caller().into(),
					callable: RawCallableIdentifiers::from_raw_callable(callable.clone()),
				},
			);
		}

		self
	}

	pub fn with_telemetry(self) -> Self {
		self.request_scoped(
			f!(crate::telemetry::RootSpan::new),
			CloningStrategy::CloneIfNecessary,
			None,
		)
		.global_middleware(f!(crate::telemetry::logger))
	}

	pub fn register(mut self, mut integration: impl FnMut(&mut Blueprint)) -> Self {
		integration(&mut self.blueprint);

		self
	}

	pub fn routes(mut self, routes: impl FnOnce() -> Router) -> Self {
		let router = routes();

		self.callbacks.push(Callback::new(RegisterRoutes, |app| {
			router.register(&mut app.blueprint, &app.middleware);
		}));

		self
	}

	#[cfg(feature = "build")]
	pub fn build(mut self, output_directory: std::path::PathBuf) -> anyhow::Result<()> {
		self.run_callbacks();

		pavex_cli_client::Client::new()
			.generate(self.blueprint, output_directory)
			.execute()
	}

	fn run_callbacks(&mut self) {
		let mut callbacks = std::mem::take(&mut self.callbacks);

		callbacks.sort_by_key(|callback| match callback.r#type {
			RegisterRoutes => 0,
			GlobalMiddleware => 1,
		});

		for callback in callbacks {
			(callback.callback)(self);
		}
	}

	fn default_extractors(mut self) -> Self {
		JsonBody::register(&mut self.blueprint);
		PathParams::register(&mut self.blueprint);
		QueryParams::register(&mut self.blueprint);
		BufferedBody::register(&mut self.blueprint);
		BodySizeLimit::register(&mut self.blueprint);

		self
	}
}

struct Callback {
	r#type: CallbackType,
	callback: Box<dyn FnOnce(&mut Application)>,
}

impl Callback {
	fn new(r#type: CallbackType, callback: impl FnOnce(&mut Application) + 'static) -> Self {
		Self {
			r#type,
			callback: Box::new(callback),
		}
	}
}

enum CallbackType {
	RegisterRoutes,
	GlobalMiddleware,
}
