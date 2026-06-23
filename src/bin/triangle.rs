use glam::{vec3, vec4};
use acadia::vulkan::wsi::Wsi;

struct Triangle {

}

impl Triangle {
	fn draw_frame() {
		let vertices = [
			vec3(-1.0, -1.0, 0.0),
			vec3(1.0, -1.0, 0.0),
			vec3(-1.0, 1.0, 0.0),
		];
		let colors = [
			vec4(0.0, 1.0, 0.0, 1.0),
			vec4(0.0, 0.0, 1.0, 1.0),
			vec4(1.0, 0.0, 0.0, 1.0),
		];

		/*
		cmd.begine_rendering();
		cmd.set_program();
		cmd.alloc_vertex_data();
		cmd.alloc_vertex_data();
		cmd.set_vertex_attribute();
		cmd.set_vertex_attribute();
		cmd.draw();
		cmd.end_rendering();
		device.submit(cmd);
		*/
	}
}

fn main() {
	println!("hello triangle!");

}

