use std::collections::HashMap;

use rand::{distributions::Alphanumeric, Rng};
use serde_json::Value;

use crate::Handler;

pub struct Store<H: Handler> {
	/// The session ID.
	id: String,
	/// The session handler implementation.
	handler: H,
	/// The session name.
	name: String,
	/// Session store started status.
	started: bool,
	/// The session attributes.
	attributes: HashMap<String, serde_json::Value>,
}

impl<H: Handler> Store<H> {
	/// Create a new session instance.
	pub fn new(name: String, handler: H, id: Option<String>) -> Self {
		let mut store = Self {
			name,
			handler,
			started: false,
			id: String::new(),
			attributes: HashMap::new(),
		};

		store.set_id(id);

		store
	}

	/// Start the session, reading the data from a handler.
	pub async fn start(&mut self) -> Result<(), H::Error> {
		self.attributes.extend(self.handler.read(&self.id).await?);

		if !self.has("_token") {
			self.regenerate_token();
		}

		self.started = true;

		Ok(())
	}

	/// Save the session data to storage.
	pub async fn save(&mut self) -> Result<(), H::Error> {
		self.age_flash_data();

		self.handler.write(&self.id, &self.attributes).await?;

		self.started = false;

		Ok(())
	}

	/// Age the flash data for the session.
	fn age_flash_data(&mut self) {
		self.forget(self.get::<Vec<String>>("_flash.old").unwrap_or_default());

		self.put(
			"_flash.old",
			self.get::<Vec<String>>("_flash.new").unwrap_or_default(),
		);

		self.put::<Vec<&str>>("_flash.new", vec![]);
	}

	/// Get all of the session data.
	pub fn all(&self) -> HashMap<String, serde_json::Value> {
		self.attributes.clone()
	}

	/// Get a subset of the session data.
	pub fn only<K: IntoKey>(&self, keys: K) -> HashMap<String, serde_json::Value> {
		let keys = keys.get_keys();

		self.attributes
			.iter()
			.fold(HashMap::new(), |mut acc, (key, value)| {
				if keys.contains(key) {
					acc.insert(key.clone(), value.clone());
				}

				acc
			})
	}

	/// Checks if a key exists.
	pub fn exists<K: IntoKey>(&self, key: K) -> bool {
		key.get_keys()
			.iter()
			.all(|key| self.attributes.contains_key(key))
	}

	/// Determine if the given key is missing from the session data.
	pub fn missing<K: IntoKey>(&self, key: K) -> bool {
		!self.exists(key)
	}

	/// Checks if a key is present and not null.
	pub fn has<K: IntoKey>(&self, key: K) -> bool {
		let keys = key.get_keys();

		keys.iter().all(|key| {
			let Some(value) = self.attributes.get(key) else {
				return false;
			};

			!value.is_null()
		})
	}

	/// Get an item from the session.
	pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
		self.attributes
			.get(key)
			.map(|value| serde_json::from_value(value.clone()).unwrap())
	}

	/// Get the value of a given key and then forget it.
	pub fn pull<T: serde::de::DeserializeOwned>(&mut self, key: &str) -> Option<T> {
		let value = self.get(key);
		self.forget(key);

		value
	}

	/// Replace the given session attributes entirely.
	pub fn replace(&mut self, attributes: HashMap<String, serde_json::Value>) {
		self.attributes = attributes;
	}

	/// Put a key / value pair in the session.
	pub fn put<T: serde::Serialize>(&mut self, key: &str, value: T) {
		self.attributes
			.insert(key.to_string(), serde_json::to_value(value).unwrap());
	}

	/// Get an item from the session, or store the default value.
	pub fn remember<T: for<'de> serde::Deserialize<'de> + serde::Serialize>(
		&mut self,
		key: &str,
		value: impl FnOnce() -> T,
	) -> T {
		if let Some(value) = self.attributes.get(key) {
			if !value.is_null() {
				return serde_json::from_value(value.clone()).unwrap();
			}
		};

		let value = value();
		self.put(key, &value);

		value
	}

	/// Push a value onto a session array.
	pub fn push<T: serde::Serialize>(&mut self, key: &str, value: T) {
		let mut default_values = Vec::new();

		let values = match self.attributes.get_mut(key) {
			Some(Value::Array(values)) => values,
			Some(_) => panic!("Key {key} is not an array"),
			None => &mut default_values,
		};

		values.push(serde_json::to_value(value).unwrap());
	}

	/// Increment the value of an item in the session.
	pub fn increment(&mut self, key: &str, amount: i64) -> i64 {
		let value = match self.attributes.get_mut(key) {
			Some(Value::Number(value)) => value.as_i64().unwrap(),
			Some(_) => panic!("Key {key} is not a number"),
			None => 0,
		};

		self.put(key, value + amount);

		value + amount
	}

	/// Decrement the value of an item in the session.
	pub fn decrement(&mut self, key: &str, amount: i64) -> i64 {
		self.increment(key, -amount)
	}

	/// Flash a key / value pair to the session.
	pub fn flash<T: serde::Serialize>(&mut self, key: &str, value: T) {
		self.put(key, value);
		self.push("_flash.new", key);
		self.remove_from_old_flash_data(key);
	}

	/// Flash a key / value pair to the session for immediate use.
	pub fn now<T: serde::Serialize>(&mut self, key: &str, value: T) {
		self.put(key, value);
		self.push("_flash.old", key);
	}

	/// Reflash all of the session flash data.
	pub fn reflash(&mut self) {
		self.merge_new_flashes(
			self.get::<Vec<_>>("_flash.old")
				.unwrap_or_default()
				.into_iter(),
		);

		self.put::<Vec<&str>>("_flash.old", vec![]);
	}

	/// Reflash a subset of the current flash data.
	pub fn keep<K: IntoKey>(&mut self, keys: K) {
		let keys = keys.get_keys();

		self.merge_new_flashes(keys.iter().cloned());
		self.remove_from_old_flash_data(keys);
	}

	/// Remove an item from the session, returning its value.
	pub fn remove<T: for<'de> serde::Deserialize<'de>>(&mut self, key: &str) -> Option<T> {
		let value = self.get(key);
		self.forget(key);

		value
	}

	/// Remove one or many items from the session.
	pub fn forget<K: IntoKey>(&mut self, keys: K) {
		let keys = keys.get_keys();

		for key in keys {
			self.attributes.remove(&key);
		}
	}

	/// Remove all of the items from the session.
	pub fn flush(&mut self) {
		self.attributes = HashMap::new();
	}

	/// Flush the session data and regenerate the ID.
	pub async fn invalidate(&mut self) -> Result<(), H::Error> {
		self.flush();

		self.migrate(true).await
	}

	/// Generate a new session identifier.
	pub async fn regenerate(&mut self, destroy: bool) -> Result<(), H::Error> {
		self.migrate(destroy).await?;
		self.regenerate_token();

		Ok(())
	}

	/// Generate a new session ID for the session.
	pub async fn migrate(&mut self, destroy: bool) -> Result<(), H::Error> {
		if destroy {
			self.handler.destroy(&self.id).await?;
		}

		self.handler.set_exists(false).await?;
		self.set_id(None);

		Ok(())
	}

	/// Determine if the session has been started.
	pub fn is_started(&self) -> bool {
		self.started
	}

	/// Get the name of the session.
	pub fn get_name(&self) -> &str {
		&self.name
	}

	/// Set the name of the session.
	pub fn set_name(&mut self, name: String) {
		self.name = name;
	}

	/// Get the current session ID.
	pub fn id(&self) -> &str {
		&self.id
	}

	/// Get the CSRF token value.
	pub fn token(&self) -> Option<String> {
		self.get("_token")
	}

	/// Regenerate the CSRF token value.
	pub fn regenerate_token(&mut self) {
		self.put("_token", random_str(40));
	}

	/// Get the previous URL from the session.
	pub fn previous_url(&self) -> Option<String> {
		self.get("_previous.url")
	}

	/// Set the "previous" URL in the session.
	pub fn set_previous_url(&mut self, url: String) {
		self.put("_previous.url", url);
	}

	/// Set the request on the handler instance.
	pub async fn set_request_on_handler(
		&mut self,
		request: &pavex::request::RequestHead,
	) -> Result<(), H::Error> {
		self.handler.set_request(request).await
	}

	/// Merge new flash keys into the new flash array.
	fn merge_new_flashes(&mut self, keys: impl Iterator<Item = String>) {
		let mut values = self
			.get::<Vec<String>>("_flash.new")
			.unwrap_or_default()
			.iter()
			.map(|key| key.to_string())
			.chain(keys)
			.collect::<Vec<_>>();

		values.sort_unstable();
		values.dedup();

		self.put("_flash.new", values);
	}

	/// Remove the given keys from the old flash data.
	fn remove_from_old_flash_data<K: IntoKey>(&mut self, keys: K) {
		let keys = keys.get_keys();

		let values = self
			.get::<Vec<String>>("_flash.old")
			.unwrap_or_default()
			.iter()
			.map(|key| key.to_string())
			.filter(|key| !keys.contains(key))
			.collect::<Vec<_>>();

		self.put("_flash.old", values);
	}

	/// Set the session ID.
	pub fn set_id(&mut self, id: Option<String>) {
		let Some(id) = id else {
			self.id = random_str(40);
			return;
		};

		self.id = if id.chars().all(|c| c.is_ascii_alphanumeric()) && id.len() == 40 {
			id
		} else {
			random_str(40)
		}
	}

	/// Collect garbage for the session handler.
	pub async fn collect_garbage(&mut self) -> Result<(), H::Error> {
		self.handler.garbage_collect().await
	}
}

pub trait IntoKey {
	fn get_keys(&self) -> Vec<String>;
}
impl IntoKey for &str {
	fn get_keys(&self) -> Vec<String> {
		vec![self.to_string()]
	}
}
impl IntoKey for String {
	fn get_keys(&self) -> Vec<String> {
		vec![self.clone()]
	}
}
impl IntoKey for Vec<String> {
	fn get_keys(&self) -> Vec<String> {
		self.clone()
	}
}
impl IntoKey for [&str] {
	fn get_keys(&self) -> Vec<String> {
		self.iter().map(|s| s.to_string()).collect()
	}
}

/// Generate a random alpha-numeric string of the given length.
fn random_str(len: usize) -> String {
	let rng = rand::thread_rng();

	rng.sample_iter(Alphanumeric)
		.map(char::from)
		.take(len)
		.collect()
}
