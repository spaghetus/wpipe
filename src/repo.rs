use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tokio::io::{AsyncRead, AsyncWrite};

/// Information about a program that may become a node.
#[derive(Serialize, Deserialize, Clone)]
pub struct ProgramInfo {
	pub human_name: String,
	pub description: String,
	pub category: String,
	pub inputs: HashMap<String, Interface>,
	pub outputs: HashMap<String, Interface>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Interface {
	pub name: String,
	pub data_format: DataFormat,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum DataFormat {
	/// The data is only a timing signal, and contains no extra information.
	/// Programs can assume that "tick" inputs send only ASCII LF, and should send only ASCII LF on "tick" outputs.
	Tick,
	/// The data is JSON, matching the associated schema. This is checked only at runtime inside WPipe.
	Json(Value),
	/// The data is a constant number, and will only be sent once.
	ConstNumber,
	/// The data is a constant string, and will only be sent once.
	ConstString,
}

pub trait CheckCompletion {
	fn is_program_finished(&mut self) -> bool;
}

pub trait Repo {
	/// The type of the pipe where data will be written.
	type InPipe: AsyncWrite + Send + Sync + Sized;

	/// The type which will be used to check whether a program has finished execution.
	type Checker: CheckCompletion + Send + Sync + Sized;

	/// The type which will be used to read output from a program.
	type OutPipe: AsyncRead + Send + Sync + Sized;

	/// Load a program's information from the program repository.
	fn get_program(&self, program: String) -> Option<(ProgramInfo, String)>;

	/// Execute a program.
	#[allow(clippy::type_complexity)]
	fn exec_program(
		&self,
		program: String,
		info: ProgramInfo,
	) -> Option<(
		HashMap<String, Self::InPipe>,
		Self::Checker,
		HashMap<String, Self::OutPipe>,
	)>;

	fn ls_programs(&self) -> Vec<String>;
}
