use ash::{Device, util::read_spv, vk};
use std::{fs::File, path::Path, rc::Rc};

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

    pub fn load_shaders_spv<P: AsRef<Path>>(&mut self, vert_spv_path: P, frag_spv_path: P) {
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
}
