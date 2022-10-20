use std::{collections::HashMap, sync::Arc};

use crate::repo::{ProgramInfo, Repo};
use egui::DragValue;
use egui_node_graph::{
	DataTypeTrait, NodeDataTrait, NodeId, NodeTemplateIter, NodeTemplateTrait, UserResponseTrait,
	WidgetValueTrait,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NodeData<R: Repo> {
	pub program_id: String,
	#[serde(skip)]
	pub repo: Option<Arc<R>>,
	pub consts: HashMap<String, NodeValueType>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeDataType {
	Tick,
	Json,
	ConstNumber,
	ConstString,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NodeValueType {
	ConstNumber { value: f64 },
	ConstString { value: String },
	Opaque,
}

impl NodeValueType {
	/// Tries to downcast this value type to a number
	pub fn try_to_number(self) -> anyhow::Result<f64> {
		if let NodeValueType::ConstNumber { value } = self {
			Ok(value)
		} else {
			anyhow::bail!("Invalid cast from {:?} to vec2", self)
		}
	}

	/// Tries to downcast this value type to a string
	pub fn try_to_string(self) -> anyhow::Result<String> {
		if let NodeValueType::ConstString { value } = self {
			Ok(value)
		} else {
			anyhow::bail!("Invalid cast from {:?} to scalar", self)
		}
	}
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NodeTemplate<R: Repo + Clone> {
	id: String,
	inner: ProgramInfo,
	#[serde(skip)]
	repo: Option<Arc<R>>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct GraphState<R: Repo + Clone> {
	#[serde(skip)]
	pub active_node: Option<NodeId>,
	#[serde(skip)]
	pub repo: Option<Arc<R>>,
}

impl<R: Repo + Clone> DataTypeTrait<GraphState<R>> for NodeDataType {
	fn data_type_color(&self, _user_state: &mut GraphState<R>) -> egui::Color32 {
		match self {
			NodeDataType::Tick => egui::Color32::from_rgb(255, 255, 255),
			NodeDataType::Json => egui::Color32::from_rgb(64, 255, 64),
			NodeDataType::ConstNumber => egui::Color32::from_rgb(128, 0, 255),
			NodeDataType::ConstString => egui::Color32::from_rgb(255, 0, 128),
		}
	}

	fn name(&self) -> std::borrow::Cow<str> {
		match self {
			NodeDataType::Tick => std::borrow::Cow::Borrowed("Tick"),
			NodeDataType::Json => std::borrow::Cow::Borrowed("Json"),
			NodeDataType::ConstNumber => std::borrow::Cow::Borrowed("ConstNumber"),
			NodeDataType::ConstString => std::borrow::Cow::Borrowed("ConstString"),
		}
	}
}

impl<R: Repo + Clone> NodeTemplateTrait for NodeTemplate<R> {
	type NodeData = NodeData<R>;
	type DataType = NodeDataType;
	type ValueType = NodeValueType;
	type UserState = GraphState<R>;

	fn node_finder_label(&self) -> &str {
		&self.inner.human_name
	}

	fn node_graph_label(&self) -> String {
		self.inner.human_name.clone()
	}

	fn user_data(&self) -> Self::NodeData {
		NodeData {
			program_id: self.id.clone(),
			consts: HashMap::new(),
			repo: self.repo.clone(),
		}
	}

	fn build_node(
		&self,
		graph: &mut egui_node_graph::Graph<Self::NodeData, Self::DataType, Self::ValueType>,
		user_state: &mut Self::UserState,
		node_id: NodeId,
	) {
		for (name, interface) in &self.inner.inputs {
			graph.add_input_param(
				node_id,
				name.clone(),
				match interface.data_format {
					crate::repo::DataFormat::Tick => NodeDataType::Tick,
					crate::repo::DataFormat::Json(_) => NodeDataType::Json,
					crate::repo::DataFormat::ConstNumber => NodeDataType::ConstNumber,
					crate::repo::DataFormat::ConstString => NodeDataType::ConstString,
				},
				match interface.data_format {
					crate::repo::DataFormat::ConstString => NodeValueType::ConstString {
						value: String::new(),
					},
					crate::repo::DataFormat::ConstNumber => {
						NodeValueType::ConstNumber { value: 0.0 }
					}
					_ => NodeValueType::Opaque,
				},
				match interface.data_format {
					crate::repo::DataFormat::ConstString => {
						egui_node_graph::InputParamKind::ConnectionOrConstant
					}
					crate::repo::DataFormat::ConstNumber => {
						egui_node_graph::InputParamKind::ConnectionOrConstant
					}
					_ => egui_node_graph::InputParamKind::ConnectionOnly,
				},
				true,
			);
		}

		for (name, interface) in &self.inner.outputs {
			graph.add_output_param(
				node_id,
				name.clone(),
				match interface.data_format {
					crate::repo::DataFormat::Tick => NodeDataType::Tick,
					crate::repo::DataFormat::Json(_) => NodeDataType::Json,
					crate::repo::DataFormat::ConstNumber => NodeDataType::ConstNumber,
					crate::repo::DataFormat::ConstString => NodeDataType::ConstString,
				},
			);
		}
	}
}

pub struct AllNodeTemplates<R: Repo + Clone> {
	pub repo: Arc<R>,
}

impl<R: Repo + Clone> NodeTemplateIter for AllNodeTemplates<R> {
	type Item = NodeTemplate<R>;

	fn all_kinds(&self) -> Vec<Self::Item> {
		self.repo
			.ls_programs()
			.iter()
			.flat_map(|program_id| {
				self.repo
					.get_program(program_id.to_string())
					.map(|v| (program_id, v))
			})
			.map(|(id, program)| NodeTemplate {
				id: id.to_string(),
				inner: program.0,
				repo: Some(self.repo.clone()),
			})
			.collect()
	}
}

#[derive(Debug, Clone)]
pub struct NodeResponse(HashMap<String, NodeValueType>);
impl UserResponseTrait for NodeResponse {}

impl WidgetValueTrait for NodeValueType {
	type Response = NodeResponse;

	fn value_widget(&mut self, param_name: &str, ui: &mut egui::Ui) -> Vec<Self::Response> {
		ui.label(param_name);
		match self {
			NodeValueType::ConstNumber { value } => {
				ui.add(DragValue::new(value));
			}
			NodeValueType::ConstString { value } => {
				ui.text_edit_singleline(value);
			}
			NodeValueType::Opaque => {}
		};
		let mut map = NodeResponse(HashMap::new());
		map.0.insert(param_name.to_string(), self.clone());
		vec![map]
	}
}

impl<R: Repo + Clone> NodeDataTrait for NodeData<R> {
	type Response = NodeResponse;
	type UserState = GraphState<R>;
	type DataType = NodeDataType;
	type ValueType = NodeValueType;

	fn bottom_ui(
		&self,
		ui: &mut egui::Ui,
		node_id: NodeId,
		graph: &egui_node_graph::Graph<Self, Self::DataType, Self::ValueType>,
		user_state: &mut Self::UserState,
	) -> Vec<egui_node_graph::NodeResponse<Self::Response, Self>>
	where
		Self::Response: egui_node_graph::UserResponseTrait,
	{
		vec![]
	}
}

pub type Graph<R> = egui_node_graph::Graph<NodeData<R>, NodeDataType, NodeValueType>;
pub type EditorState<R> = egui_node_graph::GraphEditorState<
	NodeData<R>,
	NodeDataType,
	NodeValueType,
	NodeTemplate<R>,
	GraphState<R>,
>;
