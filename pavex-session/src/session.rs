use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

use rand::{distributions::Alphanumeric, Rng};
use serde_json::Value;

#[derive(Clone, Default)]
pub struct Session {
	/// The session ID.
	id: Rc<RefCell<String>>,
	/// Whether the session has been started.
	started: Rc<RefCell<bool>>,
	/// The session attributes.
	attributes: Rc<RefCell<HashMap<String, serde_json::Value>>>,
}

impl Debug for Session {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Session")
			.field("id", &self.id.borrow())
			.field("started", &*self.started.borrow())
			.field("attributes", &self.attributes.borrow())
			.finish()
	}
}

impl Session {
	/// Create a new session instance.
	pub fn new() -> Self {
		Self::default()
	}

	/// Start the session, reading the data from a handler.
	pub fn start(
		&mut self,
		id: Option<String>,
		attributes: HashMap<String, serde_json::Value>,
	) -> Result<(), Error> {
		if self.has_started() {
			return Err(Error::AlreadyStarted);
		}

		self.set_id(id);
		self.attributes.replace(attributes);

		if !self.has("_token") {
			self.regenerate_token();
		}

		*self.started.borrow_mut() = true;

		Ok(())
	}

	/// Save the session data to storage.
	pub fn end(&mut self) -> Result<HashMap<String, serde_json::Value>, Error> {
		self.age_flash_data();
		*self.started.borrow_mut() = false;

		Ok(self.attributes.take())
	}

	/// Age the flash data for the session.
	fn age_flash_data(&mut self) {
		self.forget(self.get::<Vec<String>>("_flash.old").unwrap_or_default());

		self.set(
			"_flash.old",
			self.get::<Vec<String>>("_flash.new").unwrap_or_default(),
		);

		self.set::<Vec<&str>>("_flash.new", vec![]);
	}

	/// Get all of the session data.
	pub fn all(&self) -> HashMap<String, serde_json::Value> {
		if !self.has_started() {
			panic!("Tried to read from session before it was started");
		}

		self.attributes.borrow().clone()
	}

	/// Get a subset of the session data.
	pub fn only<K: IntoKey>(&self, keys: K) -> HashMap<String, serde_json::Value> {
		if !self.has_started() {
			panic!("Tried to read from session before it was started");
		}

		let keys = keys.get_keys();

		self.attributes
			.borrow()
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
		let attributes = self.attributes.borrow();

		key.get_keys()
			.iter()
			.all(|key| attributes.contains_key(key))
	}

	/// Determine if the given key is missing from the session data.
	pub fn missing<K: IntoKey>(&self, key: K) -> bool {
		!self.exists(key)
	}

	/// Checks if a key is present and not null.
	pub fn has<K: IntoKey>(&self, key: K) -> bool {
		let attributes = self.attributes.borrow();
		let keys = key.get_keys();

		keys.iter().all(|key| {
			let Some(value) = attributes.get(key) else {
				return false;
			};

			!value.is_null()
		})
	}

	/// Get an item from the session.
	pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
		if !self.has_started() {
			panic!("Tried to read from session before it was started");
		}

		self.attributes
			.borrow()
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
		self.attributes.replace(attributes);
	}

	/// Put a key / value pair in the session.
	pub fn set<T: serde::Serialize>(&mut self, key: &str, value: T) {
		self.attributes
			.borrow_mut()
			.insert(key.to_string(), serde_json::to_value(value).unwrap());
	}

	/// Get an item from the session, or store the default value.
	pub fn remember<T: for<'de> serde::Deserialize<'de> + serde::Serialize>(
		&mut self,
		key: &str,
		value: impl FnOnce() -> T,
	) -> T {
		if let Some(value) = self.attributes.borrow().get(key) {
			if !value.is_null() {
				return serde_json::from_value(value.clone()).unwrap();
			}
		};

		let value = value();
		self.set(key, &value);

		value
	}

	/// Push a value onto a session array.
	pub fn push<T: serde::Serialize>(&mut self, key: &str, value: T) {
		let mut default_values = Vec::new();
		let mut attributes = self.attributes.borrow_mut();

		let values = match attributes.get_mut(key) {
			Some(Value::Array(values)) => values,
			Some(_) => panic!("Key {key} is not an array"),
			None => &mut default_values,
		};

		values.push(serde_json::to_value(value).unwrap());
	}

	/// Increment the value of an item in the session.
	pub fn increment(&mut self, key: &str, amount: i64) -> i64 {
		let value = match self.attributes.borrow_mut().get_mut(key) {
			Some(Value::Number(value)) => value.as_i64().unwrap(),
			Some(_) => panic!("Key {key} is not a number"),
			None => 0,
		};

		self.set(key, value + amount);

		value + amount
	}

	/// Decrement the value of an item in the session.
	pub fn decrement(&mut self, key: &str, amount: i64) -> i64 {
		self.increment(key, -amount)
	}

	/// Flash a key / value pair to the session.
	pub fn flash<T: serde::Serialize>(&mut self, key: &str, value: T) {
		self.set(key, value);
		self.push("_flash.new", key);
		self.remove_from_old_flash_data(key);
	}

	/// Flash a key / value pair to the session for immediate use.
	pub fn now<T: serde::Serialize>(&mut self, key: &str, value: T) {
		self.set(key, value);
		self.push("_flash.old", key);
	}

	/// Reflash all of the session flash data.
	pub fn reflash(&mut self) {
		self.merge_new_flashes(
			self.get::<Vec<_>>("_flash.old")
				.unwrap_or_default()
				.into_iter(),
		);

		self.set::<Vec<&str>>("_flash.old", vec![]);
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
		let mut attributes = self.attributes.borrow_mut();
		let keys = keys.get_keys();

		for key in keys {
			attributes.remove(&key);
		}
	}

	/// Remove all of the items from the session.
	pub fn flush(&mut self) {
		self.attributes.borrow_mut().clear();
	}

	/// Determine if the session has been started.
	pub fn has_started(&self) -> bool {
		*self.started.borrow()
	}

	/// Get the current session ID.
	pub fn id(&self) -> String {
		self.id.borrow().clone()
	}

	/// Get the CSRF token value.
	pub fn token(&self) -> Option<String> {
		self.get("_token")
	}

	/// Regenerate the CSRF token value.
	pub fn regenerate_token(&mut self) {
		self.set("_token", random_str(40));
	}

	/// Get the previous URL from the session.
	pub fn previous_url(&self) -> Option<String> {
		self.get("_previous.url")
	}

	/// Set the "previous" URL in the session.
	pub fn set_previous_url(&mut self, url: String) {
		self.set("_previous.url", url);
	}

	pub fn flashed(&self) -> HashMap<String, serde_json::Value> {
		let flashed = self.get::<Vec<String>>("_flash.new").unwrap_or_default();

		self.only(flashed)
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

		self.set("_flash.new", values);
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

		self.set("_flash.old", values);
	}

	/// Set the session ID.
	pub fn set_id(&mut self, id: Option<String>) {
		let mut self_id = self.id.borrow_mut();

		let Some(id) = id else {
			*self_id = random_str(40);
			return;
		};

		*self_id = if id.chars().all(|c| c.is_ascii_alphanumeric()) && id.len() == 40 {
			id
		} else {
			random_str(40)
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("This session has already been initialized")]
	AlreadyStarted,
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
