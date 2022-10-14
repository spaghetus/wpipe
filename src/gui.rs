//! Implements an EGUI application for WPipe.

use crate::repo::Repo;
use eframe::App;
use egui_notify::Toasts;

pub struct WPipeState<R: Repo> {
	pub repo: R,
	pub toasts: Toasts,
}

impl<REPO: Repo> App for WPipeState<REPO> {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		egui::TopBottomPanel::top("menu").show(ctx, |ui| {
			ui.horizontal(|ui| {
				ui.label("WPipe");
				ui.separator();
				egui::global_dark_light_mode_switch(ui);
				egui::menu::bar(ui, |ui| {
					egui::menu::menu_button(ui, "File", |ui| {
						if ui.button("Load from path").clicked() {
							self.toasts.info("Loading isn't implemented yet...");
						}
						ui.menu_button("Load Recent...", |ui| {
							ui.label("Unimplemented");
						});
					});
					egui::menu::menu_button(ui, "View", |_ui| {})
				})
			})
		});
		egui::SidePanel::left("programs").show(ctx, |ui| {
			ui.heading("Program Palette");
			ui.separator();
			let programs: Vec<_> = self
				.repo
				.ls_programs()
				.into_iter()
				.flat_map(|name| {
					self.repo
						.get_program(name.clone())
						.map(|program| (name, program))
				})
				.collect();
			for (name, (program_info, checksum)) in programs {
				ui.heading(program_info.human_name);
				ui.label(format!("{} - {}", name, &checksum[..8]));
				ui.label(program_info.description);
				ui.collapsing("Takes:", |ui| {
					let mut inputs: Vec<_> = program_info.inputs.iter().collect();
					inputs.sort_by(|(a, _), (b, _)| a.cmp(b));
					for (_name, interface) in inputs {
						ui.label(interface.name.clone());
						ui.code(serde_json::to_string_pretty(&interface.data_format).unwrap());
					}
				});
				ui.collapsing("Puts:", |ui| {
					let mut outputs: Vec<_> = program_info.outputs.iter().collect();
					outputs.sort_by(|(a, _), (b, _)| a.cmp(b));
					for (_name, interface) in outputs {
						ui.label(interface.name.clone());
						ui.code(serde_json::to_string_pretty(&interface.data_format).unwrap());
					}
				});
				if ui.button("Add").clicked() {
					self.toasts.info("Adding programs is unimplemented...");
				}
				ui.separator();
			}
		});
		egui::CentralPanel::default().show(ctx, |ui| {
			ui.label("TODO");
		});
		self.toasts.show(ctx);
	}
}
