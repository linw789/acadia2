use crate::vulkan::shader::Program;
use arrayvec::ArrayVec;
use ash::{Device, vk};
use std::rc::Rc;

const MAX_VERTEX_BUFFER_COUNT: usize = 4;
const MAX_VERTEX_ATTRIBUTE_COUNT: usize = 16;
const MAX_ATTACHMENT_COUNT: usize = 8;

struct PipelineState {
	program: Rc<Program>,
	vertex_attributes: [VertexAttribute; MAX_VERTEX_ATTRIBUTE_COUNT],
	vertex_bindings: [VertexBinding; MAX_VERTEX_BUFFER_COUNT],
}

#[derive(Default, Clone, Copy)]
struct VertexAttribute {
	binding: u32,
	offset: u32,
	format: vk::Format,
}

struct VertexBinding {
	stride: u32,
	input_rate: vk::VertexInputRate,
}

impl PipelineState {
	pub fn set_vertex_attributes(
		&mut self,
		attrib_index: u32,
		binding: u32,
		format: vk::Format,
		offset: u32,
	) {
		self.vertex_attributes[attrib_index as usize] = VertexAttribute {
			binding,
			offset,
			format,
		}
	}

	pub fn set_vertex_binding(
		&mut self,
		binding_index: u32,
		stride: u32,
		input_rate: vk::VertexInputRate,
	) {
		self.vertex_bindings[binding_index as usize] = VertexBinding { stride, input_rate };
	}

	pub fn build_graphics_pipeline(&self, device: Rc<Device>) -> vk::Pipeline {
		// stages
		let shader_entry_name = c"name";
		const MAX_SHADER_STAGE_COUNT: usize = 8;
		let mut stages = ArrayVec::<_, MAX_SHADER_STAGE_COUNT>::new();
		for shader in &self.program.shaders {
			stages.push(
				vk::PipelineShaderStageCreateInfo::default()
					.module(shader.module)
					.name(shader_entry_name)
					.stage(shader.stage),
			);
		}

		// viewport and scissor
		// Because we use dynamic viewport, we can pass a dummy viewport and scissor to create-info to
		// make Vulkan validation layer happy.
		let viewports = [vk::Viewport::default()];
		let scissor = [vk::Rect2D::default()];
		let viewport_state = vk::PipelineViewportStateCreateInfo::default()
			.viewports(&viewports)
			.scissors(&scissor);

		// dynamic states
		const MAX_DYNAMIC_STATE_COUNT: usize = 16;
		let mut dynamic_states = ArrayVec::<_, MAX_DYNAMIC_STATE_COUNT>::new();
		dynamic_states.push(vk::DynamicState::VIEWPORT);
		dynamic_states.push(vk::DynamicState::SCISSOR);
		let dynamic_states_createinfo = vk::PipelineDynamicStateCreateInfo::default()
			.dynamic_states(&dynamic_states);

		// rasterization state
		let line_width = 0.8;
		let raster_state = vk::PipelineRasterizationStateCreateInfo::default()
			.front_face(vk::FrontFace::COUNTER_CLOCKWISE)
			.cull_mode(vk::CullModeFlags::BACK)
			.line_width(line_width)
			.polygon_mode(vk::PolygonMode::FILL)
			.depth_bias_enable(true);

		// multisample state
		let multisample_state = vk::PipelineMultisampleStateCreateInfo::default()
			.rasterization_samples(vk::SampleCountFlags::TYPE_1);

		// depth and stencil state
		let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::default()
			.depth_test_enable(true)
			.depth_write_enable(true)
			.depth_compare_op(vk::CompareOp::GREATER);

		// vertex input
		let mut vertex_attri_descs =
			[vk::VertexInputAttributeDescription::default(); MAX_VERTEX_ATTRIBUTE_COUNT];
		let mut vertex_binding_descs =
			[vk::VertexInputBindingDescription::default(); MAX_VERTEX_BUFFER_COUNT];
		let mut vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default();
		if let Some(vert_shader) = self.program.get_vertex_shader() {
			let input_location_mask = vert_shader.input_location_mask;

			let mut vertex_binding_mask = 0;
			let mut vertex_attrib_desc_count = 0;
			for attrib_index in 0..MAX_VERTEX_ATTRIBUTE_COUNT {
				if (input_location_mask & (1 << attrib_index)) != 0 {
					let attrib = &self.vertex_attributes[attrib_index as usize];
					vertex_attri_descs[vertex_attrib_desc_count].location = attrib_index as u32;
					vertex_attri_descs[vertex_attrib_desc_count].binding = attrib.binding;
					vertex_attri_descs[vertex_attrib_desc_count].format = attrib.format;
					vertex_attri_descs[vertex_attrib_desc_count].offset = attrib.offset;
					vertex_attrib_desc_count += 1;

					vertex_binding_mask |= 1 << attrib.binding;
				}
			}

			let mut vertex_binding_desc_count = 0;
			for buf_index in 0..MAX_VERTEX_BUFFER_COUNT {
				if (vertex_binding_mask & (1 << buf_index)) != 0 {
					let binding_desc = &mut vertex_binding_descs[vertex_binding_desc_count];
					binding_desc.binding = buf_index as u32;
					binding_desc.stride = self.vertex_bindings[buf_index].stride;
					binding_desc.input_rate = self.vertex_bindings[buf_index].input_rate;
					vertex_binding_desc_count += 1;
				}
			}

			vertex_input_state = vertex_input_state
				.vertex_binding_descriptions(&vertex_binding_descs[..vertex_binding_desc_count])
				.vertex_attribute_descriptions(&vertex_attri_descs[..vertex_attrib_desc_count]);
		}

		// blend state
		let mut color_attachment_states =
			[vk::PipelineColorBlendAttachmentState::default(); MAX_ATTACHMENT_COUNT];
		let mut color_blend_state = vk::PipelineColorBlendStateCreateInfo::default();
		if let Some(frag_shader) = self.program.get_fragment_shader() {
			let output_location_mask = frag_shader.output_location_mask;
			for attachment_index in 0..MAX_ATTACHMENT_COUNT {
				if output_location_mask & (1 << attachment_index) != 0 {
					color_attachment_states[attachment_index] =
						vk::PipelineColorBlendAttachmentState::default()
							.blend_enable(false)
							.color_write_mask(vk::ColorComponentFlags::RGBA);
				}
			}
		}

		let mut rendering_createinfo = vk::PipelineRenderingCreateInfo::default()
			// TODO: dynamically set these formats
			.color_attachment_formats(&[vk::Format::R8G8B8A8_UNORM])
			.depth_attachment_format(vk::Format::D32_SFLOAT);

		let pipeline_createinfo = vk::GraphicsPipelineCreateInfo::default()
			.stages(&stages)
			.viewport_state(&viewport_state)
			.dynamic_state(&dynamic_states_createinfo)
			.rasterization_state(&raster_state)
			.multisample_state(&multisample_state)
			.depth_stencil_state(&depth_stencil_state)
			.vertex_input_state(&vertex_input_state)
			.color_blend_state(&color_blend_state)
			.layout(self.program.pipeline_layout)
			.push_next(&mut rendering_createinfo);

		let pipelines = unsafe {
			device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_createinfo], None).unwrap()
		};

		pipelines[0]
	}
}
