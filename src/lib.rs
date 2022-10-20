use std::collections::HashMap;
use uuid::Uuid;

/// A node on the wpipe graph.
pub struct Node {
	pub program: String,
	pub checksum: String,
	/// A map from the interface ID to the UUID of the attached node.
	pub input_connections: HashMap<String, Uuid>,
	/// A map from the output interface ID to the UUID of the attached node.
	pub output_connections: HashMap<String, Uuid>,
}

pub mod fsrepo;
pub mod repo;

#[cfg(feature = "gui")]
pub mod gui;
#[cfg(feature = "gui")]
pub mod node;
