use glam::{vec3, vec4};

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


	}
}

fn main() {
	println!("hello triangle!");
}

