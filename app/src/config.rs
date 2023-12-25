use pavex::server::IncomingStream;
use serde_aux::field_attributes::deserialize_number_from_string;
use std::net::SocketAddr;

#[derive(Debug, Clone, serde::Deserialize)]
/// The application's configuration.
pub struct Config {
	pub app: AppConfig,
	pub server: ServerConfig,
	pub database: DatabaseConfig,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct AppConfig {
	/// The name of your application.
	pub name: String,

	/// The log level to use.
	pub log: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct DatabaseConfig {
	/// The URL of the database to connect to.
	pub url: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, serde::Deserialize)]
/// Configuration for the HTTP server used to expose the application.
pub struct ServerConfig {
	/// The port that the server must listen on.
	#[serde(deserialize_with = "deserialize_number_from_string")]
	pub port: u16,
	/// The network interface that the server must be bound to.
	/// E.g. `0.0.0.0` for listening to incoming requests from all sources.
	pub ip: std::net::IpAddr,
}

impl ServerConfig {
	/// Bind a TCP listener according to the specified parameters.
	///
	/// # Errors
	///
	/// This function will return an error if the listener cannot be bound.
	pub async fn listener(&self) -> Result<IncomingStream, std::io::Error> {
		let addr = SocketAddr::new(self.ip, self.port);
		IncomingStream::bind(addr).await
	}
}
