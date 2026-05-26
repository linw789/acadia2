use ash::{Device, util::read_spv, vk};
use std::{
	fs::File,
	io::{Error, ErrorKind, Read, Result},
	path::Path,
	rc::Rc,
	slice,
};

#[repr(u8)]
pub enum ShaderStage {
	Vertex = 0,
	Fragment = 1,
}
const SHADER_STAGE_COUNT: usize = 2;

pub struct Shader {
	module: vk::ShaderModule,
}

pub struct Program {
	shaders: [Rc<Shader>; SHADER_STAGE_COUNT],
}

pub struct ShaderManager {
	device: Rc<Device>,
	shaders: Vec<Rc<Shader>>,
	programs: Vec<Rc<Program>>,
}

impl Shader {
	fn new(shader_module: vk::ShaderModule) -> Self {
		Self {
			module: shader_module,
		}
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

	fn load_shaders_spv<P: AsRef<Path>>(&mut self, vert_spv_path: P, frag_spv_path: P) {
		{
			let mut vert_spv_file = File::open(vert_spv_path).unwrap();
			let vert_spv_code = read_spv(&mut vert_spv_file).unwrap();
			let vert_shader_info = vk::ShaderModuleCreateInfo::default().code(&vert_spv_code);
			let vert_shader_module = unsafe {
				self.device
					.create_shader_module(&vert_shader_info, None)
					.unwrap()
			};
			self.shaders.push(Rc::new(Shader::new(vert_shader_module)));
		};

		{
			let mut frag_spv_file = File::open(frag_spv_path).unwrap();
			let frag_spv_code = read_spv(&mut frag_spv_file).unwrap();
			let frag_shader_info = vk::ShaderModuleCreateInfo::default().code(&frag_spv_code);
			let frag_shader_module = unsafe {
				self.device
					.create_shader_module(&frag_shader_info, None)
					.unwrap()
			};
			self.shaders.push(Rc::new(Shader::new(frag_shader_module)));
		};
	}

	fn parse_spv_file<P: AsRef<Path>>(&mut self, spv_file_path: P) -> Result<()> {
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
			return Err(Error::new(ErrorKind::InvalidData, "Spv file has invalid magic number."));
		}

		self.parse_spv_code(&spv_words)
	}

	fn parse_spv_code(&mut self, spv_code: &[u32]) -> Result<()> {
		let id_bound = spv_code[3];
		let mut ids = vec![];


		Ok(())
	}
}
