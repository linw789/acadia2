use crate::vulkan::{
	buffer::{Buffer, BufferWriter},
	device::Device,
	pipeline::PipelineBuilder,
	shader::Program,
};
use ash::vk;
use std::rc::Rc;

pub struct CmdBuf {
	pub handle: vk::CommandBuffer,

	viewport: vk::Viewport,
	scissor: vk::Rect2D,

	pipeline_builder: PipelineBuilder,
	pipeline: vk::Pipeline,

	present_image: vk::Image,
	present_image_view: vk::ImageView,

	vertex_buffer: Option<Buffer>,

	device: Rc<Device>,
}

pub struct RenderingInfo {
	pub render_area: vk::Rect2D,
}

impl<'a> CmdBuf {
	pub fn new(device: Rc<Device>, handle: vk::CommandBuffer) -> Self {
		Self {
			handle,
			viewport: vk::Viewport::default(),
			scissor: vk::Rect2D::default(),
			pipeline_builder: PipelineBuilder::default(),
			pipeline: vk::Pipeline::null(),
			present_image: vk::Image::null(),
			present_image_view: vk::ImageView::null(),
			vertex_buffer: None, // Buffer::new(Rc::clone(&device), 4096, vk::BufferUsageFlags::VERTEX_BUFFER),
			device,
		}
	}

	pub fn destruct(&mut self) {
		if let Some(buf) = self.vertex_buffer.as_mut() {
			buf.destruct();
		}
		unsafe { self.device.api.destroy_pipeline(self.pipeline, None); }
	}

	pub fn set_present_image_and_view(&mut self, image: vk::Image, view: vk::ImageView) {
		self.present_image = image;
		self.present_image_view = view;
	}

	pub fn begin_rendering(&mut self, info: RenderingInfo) {
		self.viewport = vk::Viewport {
			x: info.render_area.offset.x as f32,
			y: info.render_area.offset.y as f32,
			width: info.render_area.extent.width as f32,
			height: info.render_area.extent.height as f32,
			min_depth: 0.0,
			max_depth: 1.0,
		};
		self.scissor = info.render_area;

		let color_attachment_infos = [vk::RenderingAttachmentInfo::default()
			.image_view(self.present_image_view)
			.image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
			.load_op(vk::AttachmentLoadOp::CLEAR)
			.store_op(vk::AttachmentStoreOp::STORE)
			.clear_value(vk::ClearValue {
				color: vk::ClearColorValue {
					float32: [135.0 / 255.0, 206.0 / 255.0, 250.0 / 255.0, 15.0 / 255.0],
				},
		})];

		let rendering_info = vk::RenderingInfo::default()
			.render_area(info.render_area)
			.layer_count(1)
			.color_attachments(&color_attachment_infos);

		// Re-start command buffer recording.
		unsafe {
			self.device
				.api
				.reset_command_buffer(self.handle, vk::CommandBufferResetFlags::RELEASE_RESOURCES)
				.unwrap();
			let cmdbuf_begin_info =
				vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
			self.device
				.api
				.begin_command_buffer(self.handle, &cmdbuf_begin_info)
				.unwrap();

			// Transition the present image to the layout COLOR_ATTACHMENT_OPTIMAL.
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
			self.device.api.cmd_pipeline_barrier2(self.handle, &dependency_info);

			self.device.api.cmd_begin_rendering(self.handle, &rendering_info);
		}
	}

	pub fn end_rendering(&self) {
		unsafe { self.device.api.cmd_end_rendering(self.handle); }

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
				self.device.api.cmd_pipeline_barrier2(self.handle, &dependency_info);
			}
		}

		// End command buffer recording.
		unsafe {
			self.device.api.end_command_buffer(self.handle).unwrap();
		}
	}

	pub fn set_program(&mut self, program: Rc<Program>) {
		self.pipeline_builder.program = Some(program);
	}

	pub fn is_vertex_data_allocated(&self) -> bool {
		self.vertex_buffer.is_some()
	}

	pub fn alloc_vertex_data(
		&'a mut self,
		binding: u32,
		size: u64,
		stride: u32,
		input_rate: vk::VertexInputRate,
	) -> BufferWriter<'a> {
		assert!(self.vertex_buffer.is_none());

		self.vertex_buffer = Some(Buffer::new(
			Rc::clone(&self.device),
			size,
			vk::BufferUsageFlags::VERTEX_BUFFER,
		));
		self.pipeline_builder.set_vertex_binding(binding, stride, input_rate);
		self.vertex_buffer.as_ref().unwrap().buffer_writer()
	}

	pub fn set_vertex_attrib(&mut self, attrib_index: usize, binding: u32, format: vk::Format, offset: u32) {
		self.pipeline_builder
			.set_vertex_attributes(attrib_index, binding, format, offset);
	}

	pub fn set_color_format(&mut self, format: vk::Format) {
		self.pipeline_builder.state.color_format = format;
	}

	pub fn draw(&mut self, vertex_count: u32) {
		if self.pipeline == vk::Pipeline::null() {
			self.pipeline = self.pipeline_builder.build_graphics_pipeline(&self.device);
		}

		unsafe {
			self.device
				.api
				.cmd_bind_pipeline(self.handle, vk::PipelineBindPoint::GRAPHICS, self.pipeline);

			self.device.api.cmd_set_viewport(self.handle, 0, &[self.viewport]);
			self.device.api.cmd_set_scissor(self.handle, 0, &[self.scissor]);

			self.device
				.api
				.cmd_bind_vertex_buffers(self.handle, 0, &[self.vertex_buffer.as_ref().unwrap().buf], &[0]);

			self.device.api.cmd_draw(self.handle, vertex_count, 1, 0, 0);
		}
	}
}
