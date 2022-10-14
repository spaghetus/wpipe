use std::{path::PathBuf, str::FromStr};

use wpipe::fsrepo::DirectoryRepo;

#[cfg(feature = "gui")]
fn main() {
	use eframe::NativeOptions;
	use egui_notify::Toasts;

	let repo = DirectoryRepo {
		path: PathBuf::from_str("./repo").unwrap(),
		cache: Default::default(),
	};

	let app = wpipe::gui::WPipeState {
		repo,
		toasts: Toasts::default(),
	};

	eframe::run_native(
		"WPipe",
		NativeOptions::default(),
		Box::new(|_| Box::new(app)),
	)
}
