//! Implements an EGUI application for WPipe.

use std::sync::Arc;

use crate::{node::AllNodeTemplates, repo::Repo};
use eframe::App;
use egui::{CollapsingHeader, Key};
use egui_node_graph::NodeFinder;
use egui_notify::Toasts;
use epaint::{Pos2, Vec2};

pub struct WPipeState<R: Repo + Clone> {
	pub repo: R,
	pub toasts: Toasts,
	pub graph_state: crate::node::EditorState<R>,
	pub user_state: crate::node::GraphState<R>,
}

impl<REPO: Repo + Clone> App for WPipeState<REPO> {
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
				CollapsingHeader::new("Takes:")
					.id_source(checksum.clone() + "takes")
					.show(ui, |ui| {
						let mut inputs: Vec<_> = program_info.inputs.iter().collect();
						inputs.sort_by(|(a, _), (b, _)| a.cmp(b));
						for (_name, interface) in inputs {
							ui.label(interface.name.clone());
							ui.code(serde_json::to_string_pretty(&interface.data_format).unwrap());
						}
					});
				CollapsingHeader::new("Puts:")
					.id_source(checksum.clone() + "puts")
					.show(ui, |ui| {
						let mut outputs: Vec<_> = program_info.outputs.iter().collect();
						outputs.sort_by(|(a, _), (b, _)| a.cmp(b));
						for (_name, interface) in outputs {
							ui.label(interface.name.clone());
							ui.code(serde_json::to_string_pretty(&interface.data_format).unwrap());
						}
					});
				ui.separator();
			}
		});
		egui::CentralPanel::default().show(ctx, |ui| {
			let all_kinds = AllNodeTemplates {
				repo: Arc::new(self.repo.clone()),
			};
			let res = self
				.graph_state
				.draw_graph_editor(ui, all_kinds, &mut self.user_state);
			if !res.node_responses.is_empty() {
				// Save
			}
			if ctx.input_mut().consume_key(
				egui::Modifiers {
					alt: false,
					ctrl: false,
					shift: true,
					mac_cmd: false,
					command: false,
				},
				Key::A,
			) {
				self.graph_state.node_finder = Some(NodeFinder::new_at(
					ctx.input()
						.pointer
						.interact_pos()
						.unwrap_or(Pos2::new(0.0, 0.0)),
				))
			}
		});
		self.toasts.show(ctx);
	}
}
