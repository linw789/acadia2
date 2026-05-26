use ash::vk;
use spirv::{ExecutionModel, Op};
use std::io::Result;

pub struct OpTypeStruct {
	member_ids: Vec<u32>,
}

pub struct OpTypeImage {
}

pub struct OpTypeSampler {
}

pub struct OpTypeSampledImage {
}

pub struct OpTypePointer {
}

pub enum OpType {
	Struct(OpTypeStruct),
	Image(OpTypeImage),
	Sampler(OpTypeSampler),
	SampledImage(OpTypeSampledImage),
}

pub enum OpConstant {
	I32(i32),
	U32(u32),
	F32(f32),
}

pub struct SpvType {
	id: u32,
	op_type: OpType,
}

pub struct SpvConstant {
	id: u32,
	constant: OpConstant,
}

pub struct SpvVariable {
	id: u32,
	type_id: u32,
}

pub struct ParsedInstructions {
	types: Vec<SpvType>,
	constants: Vec<SpvConstant>,
	variables: Vec<SpvVariable>,
}

pub fn parse(spv_code: &[u32]) {
	let id_bound = spv_code[3];

	let mut shader_stage: Option<vk::ShaderStageFlags> = None;
	let mut word_pos = 5;
	while word_pos < spv_code.len() {
		let word_0 = spv_code[word_pos];

		let opcode = word_0 & 0x0000_ffff;
		let word_count = (word_0 >> 16) as usize;

		let instruction = &spv_code[word_pos..(word_pos + word_count)];

		let mut parsed_types: Vec<SpvType> = Vec::new();
		let mut parsed_constants: Vec<SpvConstant> = Vec::new();

		let opcode = Op::from_u32(opcode).unwrap();
		match opcode {
			Op::EntryPoint => {
				debug_assert!(word_count >= 2);
				let model = ExecutionModel::from_u32(instruction[1]).unwrap();
				shader_stage = shader_stage_from_execution_model(model);
			}
			Op::TypeStruct => {
				debug_assert!(word_count >= 2);

				let result_id = instruction[1];
				debug_assert!(result_id < id_bound);

				let member_ids = instruction[2..word_count].to_vec();
				let op_struct = OpTypeStruct { member_ids };
				let spv_type = SpvType { id: result_id, op_type: OpType::Struct(op_struct), };
				parsed_types.push(spv_type);
			}
			Op::TypeImage => {
				debug_assert!(word_count >= 9);

				let result_id = instruction[1];
				debug_assert!(result_id < id_bound);

				let op_image = OpTypeImage {};
				let spv_type = SpvType { id: result_id, op_type: OpType::Image(op_image), };
				parsed_types.push(spv_type);
			}
			Op::TypeSampler => {
				debug_assert!(word_count >= 9);

				let result_id = instruction[1];
				debug_assert!(result_id < id_bound);

				let op_sampler = OpTypeSampler {};
				let spv_type = SpvType { id: result_id, op_type: OpType::Sampler(op_sampler), };
				parsed_types.push(spv_type);
			}
			Op::TypeSampledImage => {
				debug_assert!(word_count >= 9);

				let result_id = instruction[1];
				debug_assert!(result_id < id_bound);

				let op_sampled_image = OpTypeSampledImage {};
				let spv_type = SpvType { id: result_id, op_type: OpType::SampledImage(op_sampled_image), };
				parsed_types.push(spv_type);
			}
			Op::Constant => {
			}
			_ => ()
		}

		word_pos += word_count;
	}
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

fn parse_op_type(instruction: &[u32]) -> Result<SpvType> {
}
