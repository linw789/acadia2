use std::{rc::Rc, vec::Vec};
use ash::{Device as VkDevice, vk};
use crate::vulkan::{context::Context, frame::Frame, shader::ShaderManager};

pub struct Device {
	device: Rc<VkDevice>,
	shader_manager: ShaderManager,
	pipeline_layouts: Vec<vk::Pipeline>,
	frames: Vec<Frame>,
}

impl Device {
	pub fn new(context: &Context, max_frames_in_flight: u64) -> Self {
		
	}
	pub fn request_command_buffer() {
	}
}
