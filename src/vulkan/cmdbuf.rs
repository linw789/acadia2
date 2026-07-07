use crate::vulkan::{buffer::Buffer, device::Device, pipeline::PipelineBuilder, shader::Program};
use ash::vk;
use std::{path::Path, rc::Rc};

pub struct CmdBuf {
	pub cmd_buf: vk::CommandBuffer,

	pipeline_builder: PipelineBuilder,
	pipeline: vk::Pipeline,

	present_image: vk::Image,

	vertex_buffer: Option<Buffer>,

	device: Rc<Device>,
}

impl CmdBuf {
	pub fn new(device: Rc<Device>, cmd_buf: vk::CommandBuffer) -> Self {
		Self {
			cmd_buf,
			pipeline_builder: PipelineBuilder::default(),
			pipeline: vk::Pipeline::null(),
			present_image: vk::Image::null(),
			vertex_buffer: None, // Buffer::new(Rc::clone(&device), 4096, vk::BufferUsageFlags::VERTEX_BUFFER),
			device,
		}
	}

	pub fn destruct(&mut self) {}

	pub fn set_present_image(&mut self, present_image: vk::Image) {
		self.present_image = present_image;
	}

	pub fn begin_rendering(&self) {
		// Re-start command buffer recording.
		unsafe {
			self.device
				.api
				.reset_command_buffer(self.cmd_buf, vk::CommandBufferResetFlags::RELEASE_RESOURCES)
				.unwrap();
			let cmdbuf_begin_info =
				vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
			self.device
				.api
				.begin_command_buffer(self.cmd_buf, &cmdbuf_begin_info)
				.unwrap();
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
				.image(self.present_image)
				.subresource_range(
					vk::ImageSubresourceRange::default()
						.aspect_mask(vk::ImageAspectFlags::COLOR)
						.base_mip_level(0)
						.level_count(1)
						.base_array_layer(0)
						.layer_count(1),
				);

			let dependency_info = vk::DependencyInfo::default().image_memory_barriers(std::slice::from_ref(&barrier));
			self.device.api.cmd_pipeline_barrier2(self.cmd_buf, &dependency_info);
		}
	}

	pub fn end_rendering(&self) {
		// After rendering, transition the present image to the layout PRESENT_SRC_KHR.
		{
			let barrier = vk::ImageMemoryBarrier2::default()
				.src_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
				.src_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
				.dst_stage_mask(vk::PipelineStageFlags2::BOTTOM_OF_PIPE)
				.dst_access_mask(vk::AccessFlags2::NONE)
				.old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
				.new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
				.src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
				.dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
				.image(self.present_image)
				.subresource_range(vk::ImageSubresourceRange {
					aspect_mask: vk::ImageAspectFlags::COLOR,
					base_mip_level: 0,
					level_count: 1,
					base_array_layer: 0,
					layer_count: 1,
				});
			let dependency_info = vk::DependencyInfo::default().image_memory_barriers(std::slice::from_ref(&barrier));
			unsafe {
				self.device.api.cmd_pipeline_barrier2(self.cmd_buf, &dependency_info);
			}
		}

		// End command buffer recording.
		unsafe {
			self.device.api.end_command_buffer(self.cmd_buf).unwrap();
		}
	}

	pub fn set_program(&mut self, program: Rc<Program>) {
		self.pipeline_builder.program = Some(program);
	}

	pub fn alloc_vertex_data(&mut self, binding: u32, size: u64, stride: u64, input_rate: vk::VertexInputRate) {
		
	}

	pub fn set_vertex_attrib(&mut self, attrib_index: usize, binding: u32, format: vk::Format, offset: u32) {
		self.pipeline_builder.set_vertex_attributes(attrib_index, binding, format, offset);
	}
}
