use app::config::Config;
use pavex::server::Server;
use server::config::{self, Environment};

pub struct TestApi {
	pub address: String,
	pub client: reqwest::Client,
}

impl TestApi {
	pub async fn spawn() -> Self {
		let config = Self::get_config();

		ensemble::setup(&config.database.url).unwrap();

		let tcp_listener = config
			.server
			.listener()
			.await
			.expect("Failed to bind the server TCP listener");
		let address = tcp_listener
			.local_addr()
			.expect("The server TCP listener doesn't have a local socket address");

		let address = format!("http://{}:{}", config.server.ip, address.port());

		let application_state = sdk::build_application_state(config).await.unwrap();
		let server_builder = Server::new().listen(tcp_listener);

		tokio::spawn(async move { sdk::run(server_builder, application_state).await });

		TestApi {
			address,
			client: reqwest::Client::new(),
		}
	}

	fn get_config() -> Config {
		config::load(Some(Environment::Test)).expect("Failed to load test configuration")
	}
}

/// Convenient methods for calling the API under test.
impl TestApi {
	pub async fn get_ping(&self) -> reqwest::Response {
		self.client
			.get(&format!("{}/api/ping", &self.address))
			.send()
			.await
			.expect("Failed to execute request.")
	}
}
