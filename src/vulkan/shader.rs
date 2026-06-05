use crate::vulkan::spv;
use ash::{Device, util::read_spv, vk};
use std::{fs::File, path::Path, rc::Rc};

const MAX_SET_LAYOUT_COUNT: usize = 6;

pub struct Shader {
	device: Rc<Device>,
	pub stage: vk::ShaderStageFlags,
	pub module: vk::ShaderModule,
	variable_bindings: Vec<spv::VariableBindingInfo>,
	pub input_location_mask: u32,
	pub output_location_mask: u32,
}

pub struct Program {
	device: Rc<Device>,
	pub shaders: Vec<Rc<Shader>>,
	bind_point: vk::PipelineBindPoint,
	desc_set_layouts: [vk::DescriptorSetLayout; MAX_SET_LAYOUT_COUNT],
	pub pipeline_layout: vk::PipelineLayout,
}

pub struct ShaderManager {
	device: Rc<Device>,
	shaders: Vec<Rc<Shader>>,
	programs: Vec<Rc<Program>>,
}

impl Program {
	fn new(
		device: Rc<Device>,
		bind_point: vk::PipelineBindPoint,
		shaders: Vec<Rc<Shader>>,
	) -> Self {
		let desc_set_layouts = {
			let mut set_bindings: [Option<Vec<vk::DescriptorSetLayoutBinding>>;
				MAX_SET_LAYOUT_COUNT] = std::array::from_fn(|_| None);

			for shader in &shaders {
				for var in &shader.variable_bindings {
					let opt_bindings = &mut set_bindings[var.set as usize];
					match opt_bindings {
						Some(bindings) => {
							if let Some(binding) =
								bindings.iter_mut().find(|b| b.binding == var.binding)
							{
								binding.stage_flags |= shader.stage;
							}
						}
						None => {
							*opt_bindings = Some(vec![
								vk::DescriptorSetLayoutBinding::default()
									.binding(var.binding)
									.descriptor_type(var.desc_type)
									.descriptor_count(1)
									.stage_flags(shader.stage),
							]);
						}
					}
				}
			}
			let mut desc_set_layouts = [vk::DescriptorSetLayout::null(); MAX_SET_LAYOUT_COUNT];
			for (i, bindings) in set_bindings.iter().enumerate() {
				if let Some(b) = bindings {
					let set_layout_createinfo =
						vk::DescriptorSetLayoutCreateInfo::default().bindings(&b);
					desc_set_layouts[i] = unsafe {
						device
							.create_descriptor_set_layout(&set_layout_createinfo, None)
							.unwrap()
					};
				}
			}
			desc_set_layouts
		};

		let pipeline_layout = {
			let pipeline_layout_createinfo =
				vk::PipelineLayoutCreateInfo::default().set_layouts(&desc_set_layouts);
			unsafe {
				device
					.create_pipeline_layout(&pipeline_layout_createinfo, None)
					.unwrap()
			}
		};

		Self {
			device,
			shaders,
			bind_point,
			desc_set_layouts,
			pipeline_layout,
		}
	}

	pub fn get_vertex_shader(&self) -> Option<Rc<Shader>> {
		self.shaders.iter().find(|s| s.stage == vk::ShaderStageFlags::VERTEX).cloned()
	}

	pub fn get_fragment_shader(&self) -> Option<Rc<Shader>> {
		self.shaders.iter().find(|s| s.stage == vk::ShaderStageFlags::FRAGMENT).cloned()
	}
}

impl Drop for Program {
	fn drop(&mut self) {
		for layout in self.desc_set_layouts.iter_mut() {
			if *layout != vk::DescriptorSetLayout::null() {
				unsafe { self.device.destroy_descriptor_set_layout(*layout, None); }
			}
		}
		unsafe { self.device.destroy_pipeline_layout(self.pipeline_layout, None); }
	}
}

impl Drop for Shader {
	fn drop(&mut self) {
		unsafe { self.device.destroy_shader_module(self.module, None); }
	}
}

impl ShaderManager {
	pub fn new(device: Rc<Device>) -> Self {
		Self {
			device,
			shaders: Vec::new(),
			programs: Vec::new(),
		}
	}

	fn load_shader_spv<P: AsRef<Path>>(&mut self, spv_path: P) {
		let mut spv_file = File::open(spv_path).unwrap();
		let spv_code = read_spv(&mut spv_file).unwrap();
		let shader_info = vk::ShaderModuleCreateInfo::default().code(&spv_code);
		let shader_module = unsafe {
			self.device
				.create_shader_module(&shader_info, None)
				.unwrap()
		};

		let parsed = spv::parse_code(&spv_code);
		self.shaders.push(Rc::new(Shader {
			device: Rc::clone(&self.device),
			stage: parsed.shader_stage,
			module: shader_module,
			variable_bindings: parsed.variable_binding_infos,
			input_location_mask: parsed.input_location_mask,
			output_location_mask: parsed.output_location_mask,
		}));
	}
}
