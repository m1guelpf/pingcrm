use crate::{handlers::FileHandler, Handler, SessionConfig};

#[derive(Debug, Clone)]
pub struct Manager {
	config: SessionConfig,
}

impl Manager {
	pub fn new(config: SessionConfig) -> Self {
		Self { config }
	}

	pub(crate) async fn get_backend(&self) -> impl Handler {
		FileHandler::new(self.config.file_location.clone(), self.config.lifetime)
	}
}
