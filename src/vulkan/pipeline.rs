use crate::vulkan::shader::Program;
use ash::{Device, vk};
use std::rc::Rc;

const MAX_VERTEX_BUFFER_COUNT: usize = 4;
const MAX_VERTEX_ATTRIBUTE_COUNT: usize = 16;

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
		//
		// depth  stencil state

		// vertex input
		let mut vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default();
		if let Some(vert_shader) = self.program.get_vertex_shader() {
			let input_location_mask = vert_shader.input_location_mask;

			let mut vertex_attrib_desc_count = 0;
			let mut vertex_attri_descs =
				[vk::VertexInputAttributeDescription::default(); MAX_VERTEX_ATTRIBUTE_COUNT];

			let mut vertex_binding_mask = 0;

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
			let mut vertex_binding_descs =
				[vk::VertexInputBindingDescription::default(); MAX_VERTEX_BUFFER_COUNT];
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



		vk::Pipeline::default()
	}
}
