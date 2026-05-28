use ash::vk;
use spirv::{Decoration, ExecutionModel, Op, StorageClass};
use std::{
	fs::File,
	io::{Error, ErrorKind, Read, Result},
	path::Path,
	slice,
};

struct OpTypeBool {}

struct OpTypeInt {
	width: u32,
	signedness: u32,
}

struct OpTypeFloat {
	width: u32,
}

struct OpTypePointer {
	storage_class: StorageClass,
	type_id: u32,
}

struct OpTypeStruct {
	member_ids: Vec<u32>,
}

struct OpTypeImage {}

struct OpTypeSampler {}

struct OpTypeSampledImage {}

struct OpTypeArray {
	element_type_id: u32,
	length: u32,
}

enum OpType {
	Bool(OpTypeBool),
	Int(OpTypeInt),
	Float(OpTypeFloat),
	Pointer(OpTypePointer),
	Struct(OpTypeStruct),
	Image(OpTypeImage),
	Sampler(OpTypeSampler),
	SampledImage(OpTypeSampledImage),
	Array(OpTypeArray),
}

struct TypeInfo {
	id: u32,
	op_type: OpType,
}

struct ConstantInfo {
	id: u32,
	type_id: u32,
}

struct VariableInfo {
	id: u32,
	type_id: u32,
	storage_class: StorageClass,
	set: u32,
	binding: u32,
}

// Currenty only support descriptor set and binding decorations.
struct DecorationInfo {
	target_id: u32,
	decoration: Decoration,
	value: u32,
}

/// Data about a shader variable's binding, set, and type.
pub struct VariableBindingInfo {
	pub set: u32,
	pub binding: u32,
	pub desc_type: vk::DescriptorType,
}

pub struct Parsed {
	pub shader_stage: vk::ShaderStageFlags,
	pub variable_binding_infos: Vec<VariableBindingInfo>,
}

pub fn parse_code(spv_code: &[u32]) -> Parsed {
	let id_bound = spv_code[3];

	let mut shader_stage: Option<vk::ShaderStageFlags> = None;

	let mut parsed_decorations: Vec<DecorationInfo> = Vec::new();
	let mut parsed_types: Vec<TypeInfo> = Vec::new();
	let mut parsed_constants: Vec<ConstantInfo> = Vec::new();
	let mut parsed_variables: Vec<VariableInfo> = Vec::new();

	let mut word_pos = 5;
	while word_pos < spv_code.len() {
		let word_0 = spv_code[word_pos];

		let opcode = word_0 & 0x0000_ffff;
		let word_count = (word_0 >> 16) as usize;

		let instruction = &spv_code[word_pos..(word_pos + word_count)];

		let opcode = Op::from_u32(opcode).unwrap();
		match opcode {
			Op::EntryPoint => {
				debug_assert!(word_count >= 2);
				let model = ExecutionModel::from_u32(instruction[1]).unwrap();
				shader_stage = shader_stage_from_execution_model(model);
			}
			Op::Decorate => {
				debug_assert!(word_count >= 3);

				let target_id = instruction[1];
				debug_assert!(target_id < id_bound);

				let decoration = Decoration::from_u32(instruction[2]).unwrap();
				match decoration {
					Decoration::DescriptorSet => {
						debug_assert!(word_count == 4);
						parsed_decorations.push(DecorationInfo {
							target_id,
							decoration,
							value: instruction[3],
						});
					}
					Decoration::Binding => {
						debug_assert!(word_count == 4);
						parsed_decorations.push(DecorationInfo {
							target_id,
							decoration,
							value: instruction[3],
						});
					}
					_ => (),
				}
			}
			Op::TypeBool => {
				debug_assert!(word_count == 2);

				let result_id = instruction[1];
				debug_assert!(result_id < id_bound);

				let spv_type = TypeInfo {
					id: result_id,
					op_type: OpType::Bool(OpTypeBool {}),
				};
				parsed_types.push(spv_type);
			}
			Op::TypeInt => {
				debug_assert!(word_count == 4);

				let result_id = instruction[1];
				debug_assert!(result_id < id_bound);

				let spv_type = TypeInfo {
					id: result_id,
					op_type: OpType::Int(OpTypeInt {
						width: instruction[2],
						signedness: instruction[3],
					}),
				};
				parsed_types.push(spv_type);
			}
			Op::TypeFloat => {
				debug_assert!(word_count >= 3);

				let result_id = instruction[1];
				debug_assert!(result_id < id_bound);

				let spv_type = TypeInfo {
					id: result_id,
					op_type: OpType::Float(OpTypeFloat {
						width: instruction[2],
					}),
				};
				parsed_types.push(spv_type);
			}
			Op::TypePointer => {
				debug_assert!(word_count == 4);

				let result_id = instruction[1];
				debug_assert!(result_id < id_bound);

				let spv_type = TypeInfo {
					id: result_id,
					op_type: OpType::Pointer(OpTypePointer {
						storage_class: StorageClass::from_u32(instruction[2]).unwrap(),
						type_id: instruction[3],
					}),
				};
				parsed_types.push(spv_type);
			}
			Op::TypeStruct => {
				debug_assert!(word_count >= 2);

				let result_id = instruction[1];
				debug_assert!(result_id < id_bound);

				let member_ids = instruction[2..word_count].to_vec();
				let op_struct = OpTypeStruct { member_ids };
				let spv_type = TypeInfo {
					id: result_id,
					op_type: OpType::Struct(op_struct),
				};
				parsed_types.push(spv_type);
			}
			Op::TypeImage => {
				debug_assert!(word_count >= 9);

				let result_id = instruction[1];
				debug_assert!(result_id < id_bound);

				let op_image = OpTypeImage {};
				let spv_type = TypeInfo {
					id: result_id,
					op_type: OpType::Image(op_image),
				};
				parsed_types.push(spv_type);
			}
			Op::TypeSampler => {
				debug_assert!(word_count >= 9);

				let result_id = instruction[1];
				debug_assert!(result_id < id_bound);

				let op_sampler = OpTypeSampler {};
				let spv_type = TypeInfo {
					id: result_id,
					op_type: OpType::Sampler(op_sampler),
				};
				parsed_types.push(spv_type);
			}
			Op::TypeSampledImage => {
				debug_assert!(word_count >= 9);

				let result_id = instruction[1];
				debug_assert!(result_id < id_bound);

				let op_sampled_image = OpTypeSampledImage {};
				let spv_type = TypeInfo {
					id: result_id,
					op_type: OpType::SampledImage(op_sampled_image),
				};
				parsed_types.push(spv_type);
			}
			Op::TypeArray => {
				debug_assert!(word_count == 4);

				let result_id = instruction[1];
				debug_assert!(result_id < id_bound);

				let spv_type = TypeInfo {
					id: result_id,
					op_type: OpType::Array(OpTypeArray {
						element_type_id: instruction[2],
						length: instruction[3],
					}),
				};
				parsed_types.push(spv_type);
			}
			Op::Constant => {
				debug_assert!(word_count >= 4);

				let type_id = instruction[1];
				debug_assert!(type_id < id_bound);

				let result_id = instruction[2];
				debug_assert!(result_id < id_bound);

				let constant = ConstantInfo {
					id: result_id,
					type_id,
				};
				parsed_constants.push(constant);
			}
			Op::Variable => {
				debug_assert!(word_count >= 4);

				let type_id = instruction[1];
				debug_assert!(type_id < id_bound);

				let result_id = instruction[2];
				debug_assert!(result_id < id_bound);

				let storage_class = StorageClass::from_u32(instruction[3]).unwrap();

				let variable = VariableInfo {
					id: result_id,
					type_id,
					storage_class,
					..Default::default()
				};
				parsed_variables.push(variable);
			}
			_ => (),
		}

		word_pos += word_count;
	}

	let mut binding_infos: Vec<VariableBindingInfo> = Vec::new();

	for dec in &parsed_decorations {
		if let Some(var) = parsed_variables.iter_mut().find(|v| v.id == dec.target_id) {
			match dec.decoration {
				Decoration::DescriptorSet => {
					var.set = dec.value;
				}
				Decoration::Binding => {
					var.binding = dec.value;
				}
				_ => (),
			}
		}
	}

	for var in &parsed_variables {
		if var.binding != u32::MAX {
			assert!(var.set != u32::MAX);

			let spv_type = parsed_types.iter().find(|&t| t.id == var.type_id).unwrap();
			let info = VariableBindingInfo {
				set: var.set,
				binding: var.binding,
				desc_type: vk_descriptor_type_from(&spv_type.op_type),
			};
			binding_infos.push(info);
		}
	}

	Parsed {
		shader_stage: shader_stage.unwrap(),
		variable_binding_infos: binding_infos,
	}
}

pub fn parse_file<P: AsRef<Path>>(spv_file_path: P) -> Result<Parsed> {
	let mut spv_file = File::open(&spv_file_path)?;
	let file_byte_size = spv_file.metadata()?.len();

	if file_byte_size % 4 != 0 {
		return Err(Error::new(
			ErrorKind::InvalidData,
			"Spv file size isn't multple of 4.",
		));
	}
	if file_byte_size > (isize::MAX as u64) {
		return Err(Error::new(ErrorKind::InvalidData, "Spv file size too big."));
	}

	const WORD_SIZE: usize = 4;
	let file_word_size = (file_byte_size as usize) / WORD_SIZE;
	let mut spv_words = vec![0u32; file_word_size];

	spv_file.read_exact(unsafe {
		// Soundness:
		// - spv_words was allocated just above, so it's definitely not null.
		// - The array pointed to by the casted pointer has length that's equal to the file size.
		// - The casted pointer is only accessed within this function.
		// - We checked above that the file size is smaller than isize::MAX.
		slice::from_raw_parts_mut(
			spv_words.as_mut_ptr() as *mut u8,
			file_word_size as usize * WORD_SIZE,
		)
	})?;

	// We only support little-endian CPUs.
	{
		let x: u32 = 1;
		let bytes = x.to_ne_bytes();
		assert!(bytes[0] == 1);
	}

	const SPV_MAGIC_NUMBER_LITTLE_ENDIAN: u32 = 0x07230203;
	const SPV_MAGIC_NUMBER_BIG_ENDIAN: u32 = 0x03022307;

	let magic_num = spv_words[0];
	if magic_num == SPV_MAGIC_NUMBER_LITTLE_ENDIAN {
		// do nothing
	} else if magic_num == SPV_MAGIC_NUMBER_BIG_ENDIAN {
		for w in &mut spv_words {
			*w = w.swap_bytes();
		}
	} else {
		return Err(Error::new(
			ErrorKind::InvalidData,
			"Spv file has invalid magic number.",
		));
	}

	Ok(parse_code(&spv_words))
}

fn shader_stage_from_execution_model(model: ExecutionModel) -> Option<vk::ShaderStageFlags> {
	match model {
		ExecutionModel::Vertex => Some(vk::ShaderStageFlags::VERTEX),
		ExecutionModel::Fragment => Some(vk::ShaderStageFlags::FRAGMENT),
		ExecutionModel::GLCompute => Some(vk::ShaderStageFlags::COMPUTE),
		_ => {
			println!("Unhandled execution model ({:?}) for shader stage.", model);
			None
		}
	}
}

fn vk_descriptor_type_from(op_type: &OpType) -> vk::DescriptorType {
	match op_type {
		OpType::Bool(_) | OpType::Int(_) | OpType::Float(_) | OpType::Struct(_) => {
			vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC
		}
		OpType::Image(_) => vk::DescriptorType::STORAGE_IMAGE,
		OpType::Sampler(_) => vk::DescriptorType::SAMPLER,
		OpType::SampledImage(_) => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
		_ => panic!("Unsupported OpType to VkDescriptorType conversion."),
	}
}

impl Default for VariableInfo {
	fn default() -> Self {
		Self {
			id: 0,
			type_id: 0,
			storage_class: StorageClass::UniformConstant,
			set: u32::MAX,
			binding: u32::MAX,
		}
	}
}
