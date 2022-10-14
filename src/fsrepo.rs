use tokio::{
	fs::File,
	io::{AsyncRead, AsyncWrite},
	process::{Child, Command},
};
use uuid::Uuid;

use crate::repo::{CheckCompletion, ProgramInfo, Repo};
use std::{
	collections::HashMap,
	ops::Deref,
	path::PathBuf,
	pin::Pin,
	str::FromStr,
	sync::{Arc, Mutex},
};

pub struct DirectoryRepo {
	pub path: PathBuf,
	pub cache: Arc<Mutex<HashMap<String, (ProgramInfo, String)>>>,
}

impl Repo for DirectoryRepo {
	type InPipe = NamedPipe;

	type Checker = Child;

	type OutPipe = NamedPipe;

	fn get_program(&self, program: String) -> Option<(crate::repo::ProgramInfo, String)> {
		let program_path = self.path.join(program.clone());
		let meta_path = program_path.with_file_name(
			program_path
				.components()
				.last()
				.unwrap()
				.as_os_str()
				.to_string_lossy()
				.to_string() + ".json",
		);
		if let Ok(contents) = std::fs::read(meta_path) {
			let hash = sha256::digest_bytes(&contents);
			if let Some((cached, c_hash)) = self.cache.lock().unwrap().get(&program) {
				if c_hash == &hash {
					return Some((cached.clone(), c_hash.clone()));
				}
			}

			match serde_json::from_slice::<ProgramInfo>(&contents) {
				Ok(v) => Some((v, hash)),
				Err(e) => {
					eprintln!("{}", e);
					None
				}
			}
		} else {
			unimplemented!()
		}
	}

	fn exec_program(
		&self,
		program: String,
		info: ProgramInfo,
	) -> Option<(
		std::collections::HashMap<String, Self::InPipe>,
		Self::Checker,
		std::collections::HashMap<String, Self::OutPipe>,
	)> {
		let program_path = self.path.join(program);
		let mut cmd = Command::new(program_path);
		let mut in_pipes: HashMap<_, _> = Default::default();
		let mut out_pipes: HashMap<_, _> = Default::default();
		for (name, _) in info.inputs {
			let pipe = NamedPipe::new();
			cmd.env(name.clone(), pipe.path.clone());
			in_pipes.insert(name, pipe);
		}
		for (name, _) in info.outputs {
			let pipe = NamedPipe::new();
			cmd.env(name.clone(), pipe.path.clone());
			out_pipes.insert(name, pipe);
		}

		cmd.spawn().ok().map(|child| (in_pipes, child, out_pipes))
	}

	fn ls_programs(&self) -> Vec<String> {
		std::fs::read_dir(&self.path)
			.expect("Failed to list programs")
			.flatten()
			.map(|read_dir| read_dir.file_name().to_string_lossy().to_string())
			.filter(|name| !name.ends_with(".json"))
			.collect()
	}
}

impl CheckCompletion for Child {
	fn is_program_finished(&mut self) -> bool {
		self.try_wait().is_ok()
	}
}

pub struct NamedPipe {
	file: File,
	path: PathBuf,
}

impl Deref for NamedPipe {
	type Target = File;

	fn deref(&self) -> &Self::Target {
		&self.file
	}
}

impl AsyncWrite for NamedPipe {
	fn poll_write(
		mut self: std::pin::Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
		buf: &[u8],
	) -> std::task::Poll<Result<usize, std::io::Error>> {
		Pin::new(&mut self.file).poll_write(cx, buf)
	}

	fn poll_flush(
		mut self: std::pin::Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Result<(), std::io::Error>> {
		Pin::new(&mut self.file).poll_flush(cx)
	}

	fn poll_shutdown(
		mut self: std::pin::Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Result<(), std::io::Error>> {
		Pin::new(&mut self.file).poll_shutdown(cx)
	}
}

impl AsyncRead for NamedPipe {
	fn poll_read(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
		buf: &mut tokio::io::ReadBuf<'_>,
	) -> std::task::Poll<std::io::Result<()>> {
		Pin::new(&mut self.file).poll_read(cx, buf)
	}
}

impl NamedPipe {
	pub fn new() -> NamedPipe {
		let pipe_uuid = Uuid::new_v4();
		let base_path: PathBuf = PathBuf::from_str("/run/wpipe").unwrap();
		std::fs::create_dir_all(base_path.clone())
			.expect("Failed to create directory for named pipes!");
		let fifo_path = base_path.join(pipe_uuid.to_string());
		nix::unistd::mkfifo(&fifo_path, nix::sys::stat::Mode::from_bits(0o700).unwrap())
			.expect("Failed to create fifo");
		let file = std::fs::OpenOptions::new()
			.read(true)
			.write(true)
			.create(false)
			.open(fifo_path.clone())
			.expect("Failed to open fifo");
		NamedPipe {
			file: tokio::fs::File::from_std(file),
			path: fifo_path,
		}
	}
}

impl Default for NamedPipe {
	fn default() -> Self {
		Self::new()
	}
}

impl Drop for NamedPipe {
	fn drop(&mut self) {
		std::fs::remove_file(self.path.clone()).expect("Failed to clean up fifo")
	}
}
