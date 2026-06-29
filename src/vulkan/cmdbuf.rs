use crate::vulkan::{buffer::Buffer, shader::Program};
use ash::{
	Device,
	vk::{self, PhysicalDeviceMemoryProperties},
};
use std::rc::Rc;

pub struct CmdBuf {
	pipeline: vk::Pipeline,
	vertex_buffer: Buffer,
	index_buffer: Buffer,
	uniform_buffer: Buffer,
	cmd_buf: vk::CommandBuffer,
	device: Rc<Device>,
}

impl CmdBuf {
	pub fn new(device: Rc<Device>, phy_mem_props: &PhysicalDeviceMemoryProperties, cmd_buf: vk::CommandBuffer) -> Self {
		Self {
			pipeline: vk::Pipeline::null(),
			vertex_buffer: Buffer::new(
				Rc::clone(&device),
				4096,
				vk::BufferUsageFlags::VERTEX_BUFFER,
				phy_mem_props,
			),
			index_buffer: Buffer::new(
				Rc::clone(&device),
				4096,
				vk::BufferUsageFlags::INDEX_BUFFER,
				phy_mem_props,
			),
			uniform_buffer: Buffer::new(
				Rc::clone(&device),
				4096,
				vk::BufferUsageFlags::UNIFORM_BUFFER,
				phy_mem_props,
			),
			cmd_buf,
			device,
		}
	}

	pub fn begin_rendering(&mut self, present_image: vk::Image) {
		// Re-start command buffer recording.
		unsafe {
			self.device.reset_command_buffer(self.cmd_buf, vk::CommandBufferResetFlags::RELEASE_RESOURCES).unwrap();
			let cmdbuf_begin_info = vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
			self.device.begin_command_buffer(self.cmd_buf, &cmdbuf_begin_info).unwrap();
		}

		// Transition the present image to the layout COLOR_ATTACHMENT_OPTIMAL.
		unsafe {
			let barrier = vk::ImageMemoryBarrier2::default()
				.src_stage_mask(vk::PipelineStageFlags2::TOP_OF_PIPE)
				.src_access_mask(vk::AccessFlags2::NONE)
				.dst_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
				.dst_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
				.old_layout(vk::ImageLayout::UNDEFINED)
				.new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
				.src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
				.dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
				.image(present_image)
				.subresource_range(vk::ImageSubresourceRange {
					aspect_mask: vk::ImageAspectFlags::COLOR,
					base_mip_level: 0,
					level_count: 1,
					base_array_layer: 0,
					layer_count: 1,
				});

			let dependency_info = vk::DependencyInfo::default().image_memory_barriers(&[barrier]);
			self.device.cmd_pipeline_barrier2(self.cmd_buf, &dependency_info);
		}
	}

	pub fn end_rendering() {}

	pub fn allocate_vertex_data(binding: u32, size: u64, stride: u64, input_rate: vk::VertexInputRate, data: &[u8]) {}

	fn set_program(&mut self, program: Rc<Program>) {
		// TODO: check if self alreayd has a program.
		self.program = program;
	}

	fn set_vertex_binding() {}
}
