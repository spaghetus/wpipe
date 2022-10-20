use std::{path::PathBuf, str::FromStr};

use wpipe::fsrepo::DirectoryRepo;

#[cfg(feature = "gui")]
fn main() {
	use std::sync::Arc;

	use eframe::NativeOptions;
	use egui_notify::Toasts;
	use wpipe::node::GraphState;

	let repo = DirectoryRepo {
		path: PathBuf::from_str("./repo").unwrap(),
		cache: Default::default(),
	};

	let app = wpipe::gui::WPipeState {
		toasts: Toasts::default(),
		graph_state: Default::default(),
		user_state: GraphState {
			active_node: None,
			repo: Some(Arc::new(repo.clone())),
		},
		repo,
	};

	eframe::run_native(
		"WPipe",
		NativeOptions::default(),
		Box::new(|_| Box::new(app)),
	)
}
