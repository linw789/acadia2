use crate::vulkan::{device::Device, shader::Program};
use arrayvec::ArrayVec;
use ash::vk;
use std::rc::Rc;

const MAX_VERTEX_BUFFER_COUNT: usize = 4;
const MAX_VERTEX_ATTRIBUTE_COUNT: usize = 16;
const MAX_ATTACHMENT_COUNT: usize = 8;

#[derive(Default)]
pub struct PipelineBuilder {
	pub program: Option<Rc<Program>>,
	pub vertex_attributes: [VertexAttribute; MAX_VERTEX_ATTRIBUTE_COUNT],
	pub vertex_bindings: [VertexBinding; MAX_VERTEX_BUFFER_COUNT],
	pub state: PipelineState,
}

#[derive(Clone, Copy, Default)]
struct VertexAttribute {
	pub binding: u32,
	pub offset: u32,
	pub format: vk::Format,
}

#[derive(Clone, Copy, Default)]
struct VertexBinding {
	stride: u32,
	input_rate: vk::VertexInputRate,
}

pub struct PipelineState {
	pub primitive_topology: vk::PrimitiveTopology,
	pub color_format: vk::Format,
	pub depth_test: bool,
	pub depth_write: bool,
	pub depth_compare_op: vk::CompareOp,
	pub depth_format: vk::Format,
	pub dynamic_depth_bias_enable: bool,
	pub line_width: f32,
}

impl PipelineBuilder {
	pub fn set_vertex_attributes(&mut self, attrib_index: usize, binding: u32, format: vk::Format, offset: u32) {
		self.vertex_attributes[attrib_index as usize] = VertexAttribute {
			binding,
			offset,
			format,
		}
	}

	pub fn set_vertex_binding(&mut self, binding_index: u32, stride: u32, input_rate: vk::VertexInputRate) {
		self.vertex_bindings[binding_index as usize] = VertexBinding { stride, input_rate };
	}

	pub fn build_graphics_pipeline(&self, device: &Device) -> vk::Pipeline {
		let program = self.program.as_ref().unwrap();

		// stages
		let shader_entry_name = c"main";
		const MAX_SHADER_STAGE_COUNT: usize = 8;
		let mut stages = ArrayVec::<_, MAX_SHADER_STAGE_COUNT>::new();
		for shader in &program.shaders {
			stages.push(
				vk::PipelineShaderStageCreateInfo::default()
					.module(shader.module)
					.name(shader_entry_name)
					.stage(shader.stage),
			);
		}

		let input_assembly_state =
			vk::PipelineInputAssemblyStateCreateInfo::default().topology(self.state.primitive_topology);

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
		if self.state.dynamic_depth_bias_enable {
			dynamic_states.push(vk::DynamicState::DEPTH_BIAS);
		}
		let dynamic_states_createinfo = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

		// rasterization state
		let line_width = 0.8;
		let raster_state = vk::PipelineRasterizationStateCreateInfo::default()
			.front_face(vk::FrontFace::COUNTER_CLOCKWISE)
			.cull_mode(vk::CullModeFlags::BACK)
			.line_width(line_width)
			.polygon_mode(vk::PolygonMode::FILL)
			.depth_bias_enable(self.state.dynamic_depth_bias_enable);

		// multisample state
		let multisample_state =
			vk::PipelineMultisampleStateCreateInfo::default().rasterization_samples(vk::SampleCountFlags::TYPE_1);

		// depth and stencil state
		let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::default()
			.depth_test_enable(self.state.depth_test)
			.depth_write_enable(self.state.depth_write)
			.depth_compare_op(self.state.depth_compare_op);

		// vertex input
		let mut vertex_attri_descs = [vk::VertexInputAttributeDescription::default(); MAX_VERTEX_ATTRIBUTE_COUNT];
		let mut vertex_binding_descs = [vk::VertexInputBindingDescription::default(); MAX_VERTEX_BUFFER_COUNT];
		let mut vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default();
		if let Some(vert_shader) = program.get_vertex_shader() {
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
		let mut color_attachment_states = [vk::PipelineColorBlendAttachmentState::default(); MAX_ATTACHMENT_COUNT];
		let mut color_attachment_count = 0;
		if let Some(frag_shader) = program.get_fragment_shader() {
			for i in 0..MAX_ATTACHMENT_COUNT {
				if frag_shader.output_location_mask & (1 << i) != 0 {
					color_attachment_states[i] = vk::PipelineColorBlendAttachmentState::default()
						// TODO: How to dynamically set these?
						.blend_enable(false)
						.color_write_mask(vk::ColorComponentFlags::RGBA);
					color_attachment_count += 1;
				}
			}
		}
		let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
			.attachments(&color_attachment_states[..color_attachment_count]);

		let mut rendering_createinfo = vk::PipelineRenderingCreateInfo::default()
			.color_attachment_formats(std::slice::from_ref(&self.state.color_format));
		if self.state.depth_format == vk::Format::UNDEFINED {
			rendering_createinfo = rendering_createinfo.depth_attachment_format(self.state.depth_format);
		}

		let pipeline_createinfo = vk::GraphicsPipelineCreateInfo::default()
			.stages(&stages)
			.input_assembly_state(&input_assembly_state)
			.viewport_state(&viewport_state)
			.dynamic_state(&dynamic_states_createinfo)
			.rasterization_state(&raster_state)
			.multisample_state(&multisample_state)
			.depth_stencil_state(&depth_stencil_state)
			.vertex_input_state(&vertex_input_state)
			.color_blend_state(&color_blend_state)
			.layout(program.pipeline_layout)
			.push_next(&mut rendering_createinfo);

		let pipelines = unsafe {
			device
				.api
				.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_createinfo], None)
				.unwrap()
		};

		pipelines[0]
	}
}

impl Default for PipelineState {
	fn default() -> Self {
		Self {
			primitive_topology: vk::PrimitiveTopology::TRIANGLE_LIST,
			color_format: vk::Format::UNDEFINED,
			depth_test: true,
			depth_write: true,
			depth_compare_op: vk::CompareOp::GREATER,
			depth_format: vk::Format::UNDEFINED,
			dynamic_depth_bias_enable: false,
			line_width: 0.5,
		}
	}
}
