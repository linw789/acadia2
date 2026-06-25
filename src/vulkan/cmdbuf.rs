use ash::{Device, vk::{self, PhysicalDeviceMemoryProperties}};
use crate::vulkan::{buffer::{Buffer}, shader::Program};
use std::rc::Rc;

struct CmdBuf {
	device: Rc<Device>,
	program: Rc<Program>,
	pipeline: vk::Pipeline,
	vertex_buffer: Buffer,
	index_buffer: Buffer,
	uniform_buffer: Buffer,
}

impl CmdBuf {
	pub fn new(device: &Rc<Device>, program: Rc<Program>, phy_mem_props: &PhysicalDeviceMemoryProperties) -> Self {
		Self {
			device: Rc::clone(&device),
			program,
			pipeline: vk::Pipeline::null(),
			vertex_buffer: Buffer::new(Rc::clone(&device), 4096, vk::BufferUsageFlags::VERTEX_BUFFER, phy_mem_props),
			index_buffer: Buffer::new(Rc::clone(&device), 4096, vk::BufferUsageFlags::INDEX_BUFFER, phy_mem_props),
			uniform_buffer: Buffer::new(Rc::clone(&device), 4096, vk::BufferUsageFlags::UNIFORM_BUFFER, phy_mem_props),
		}
	}

	pub fn allocate_vertex_data(binding: u32, size: u64, stride: u64, input_rate: vk::VertexInputRate, data: &[u8]) {
	}

	pub fn begin_rendering() {
	}

	pub fn end_rendering() {
	}

	fn set_program(&mut self, program: Rc<Program>) {
		// TODO: check if self alreayd has a program.
		self.program = program;
	}

	fn set_vertex_binding() {}
}
