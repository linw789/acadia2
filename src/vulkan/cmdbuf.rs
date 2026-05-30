use std::{path::Path, rc::Rc};
use crate::vulkan::shader::Program;
use ash::{Device, vk};

struct CmdBuf {
	device: Rc<Device>,
	program: Rc<Program>,
	pipeline: vk::Pipeline,
}

impl CmdBuf {
	pub fn set_graphics_program<P: AsRef<Path>>(vert_spv_path: P, frag_spv_path: P) {

	}

	pub fn render_begin() {
	}

	pub fn render_end() {
	}

	fn set_program(&mut self, program: Rc<Program>) {
		// TODO: check if self alreayd has a program.
		self.program = program;
	}
}
