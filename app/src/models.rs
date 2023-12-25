use ensemble::{
	types::{DateTime, Hashed},
	Model,
};

#[derive(Debug, Model)]
pub struct User {
	pub id: u64,
	pub name: String,
	pub email: String,
	pub password: Hashed<String>,

	pub created_at: DateTime,
	pub updated_at: DateTime,
}
