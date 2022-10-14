use std::collections::HashMap;
use uuid::Uuid;

/// A node on the wpipe graph.
pub struct Node {
	pub program: String,
	pub checksum: String,
	/// A map from the interface ID to the
	pub input_connections: HashMap<String, Uuid>,
}

pub mod fsrepo;
pub mod repo;

#[cfg(feature = "gui")]
pub mod gui;
