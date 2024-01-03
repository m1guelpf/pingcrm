use std::{collections::HashMap, fs, io, path::PathBuf, rc::Rc, time::Duration};

pub struct FileHandler {
	path: PathBuf,
	valid_for: Duration,
}

impl FileHandler {
	pub fn new(path: PathBuf, valid_for: Duration) -> Self {
		fs::create_dir_all(&path).unwrap();

		Self { path, valid_for }
	}
}

impl super::Handler for FileHandler {
	type Error = io::Error;

	async fn read(&mut self, id: &str) -> Result<HashMap<String, serde_json::Value>, Self::Error> {
		let path = self.path.join(id);

		let file_contents = match fs::read(&path) {
			Ok(contents) => contents,
			Err(e) => {
				if e.kind() == io::ErrorKind::NotFound {
					return Ok(HashMap::new());
				} else {
					return Err(e);
				}
			},
		};

		if fs::metadata(&path)?
			.modified()?
			.elapsed()
			.unwrap_or_default()
			> self.valid_for
		{
			fs::remove_file(&path)?;
			return Ok(HashMap::new());
		}

		Ok(serde_json::from_slice(file_contents.as_ref()).unwrap_or_default())
	}

	async fn write<T: serde::Serialize>(
		&mut self,
		id: &str,
		attributes: T,
	) -> Result<(), Self::Error> {
		let path = self.path.join(id);

		fs::write(
			path,
			serde_json::to_vec(&attributes).map_err(io::Error::other)?,
		)
	}

	async fn destroy(&mut self, id: &str) -> Result<(), Self::Error> {
		let path = self.path.join(id);

		fs::remove_file(path)
	}

	async fn collect_garbage(&mut self, max_lifetime: &Duration) -> Result<u64, Self::Error> {
		let expired = fs::read_dir(&self.path)?
			.flatten()
			.map(|entry| {
				let last_modified = entry.metadata()?.modified().map_err(io::Error::other)?;

				if &last_modified.elapsed().unwrap_or_default() > max_lifetime {
					fs::remove_file(entry.path())?;
				}

				Ok::<_, io::Error>(())
			})
			.collect::<Result<Rc<_>, _>>()?;

		Ok(expired.len() as u64)
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

	#[error("failed to list session files")]
	List(String),
}
