use std::{collections::HashMap, io, path::PathBuf, time::Duration};

use flysystem::{adapters::LocalAdapter, Filesystem};

pub struct FileHandler {
	path: PathBuf,
	valid_for: Duration,
	files: Filesystem<LocalAdapter>,
}

impl FileHandler {
	pub async fn new(
		path: PathBuf,
		valid_for: Duration,
		files: Filesystem<LocalAdapter>,
	) -> Result<Self, Error> {
		Ok(Self {
			path,
			files,
			valid_for,
		})
	}
}

impl super::Handler for FileHandler {
	type Error = Error;

	async fn read(&mut self, id: &str) -> Result<HashMap<String, serde_json::Value>, Self::Error> {
		let path = self.path.join(id);

		let file_exists = self
			.files
			.file_exists(&path)
			.await
			.map_err(|e| Error::FileExists(e.to_string()))?;

		let has_expired = match self.files.last_modified(&path).await {
			Ok(last_modified) => last_modified.elapsed()? > self.valid_for,
			Err(e) => {
				if e.kind() == io::ErrorKind::NotFound {
					true
				} else {
					return Err(Error::LastModified(e.to_string()));
				}
			},
		};

		if file_exists && !has_expired {
			let data = self
				.files
				.read::<Vec<u8>>(&path)
				.await
				.map_err(|e| Error::Read(e.to_string()))?;
			serde_json::from_slice(&data).unwrap_or_default()
		}

		Ok(HashMap::new())
	}

	async fn write<T: serde::Serialize>(
		&mut self,
		id: &str,
		attributes: T,
	) -> Result<(), Self::Error> {
		let path = self.path.join(id);

		self.files
			.write(&path, serde_json::to_vec(&attributes).unwrap_or_default())
			.await
			.map_err(|e| Error::Write(e.to_string()))
	}

	async fn destroy(&mut self, id: &str) -> Result<(), Self::Error> {
		let path = self.path.join(id);

		self.files
			.delete(&path)
			.await
			.map_err(|e| Error::Delete(e.to_string()))
	}
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("failed to initialize Flysystem adapter")]
	Config(String),

	#[error("failed to check if session file exists")]
	FileExists(String),

	#[error("failed to get last modified time of session file")]
	LastModified(String),

	#[error("failed to get elapsed time since last modified")]
	SystemTime(#[from] std::time::SystemTimeError),

	#[error("failed to read session file")]
	Read(String),

	#[error("failed to write to session file")]
	Write(String),

	#[error("failed to delete session file")]
	Delete(String),
}
