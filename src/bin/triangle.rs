use glam::{vec3, vec4};
use acadia::vulkan::wsi::Wsi;
use ::winit::{
	application::ApplicationHandler,
	dpi::PhysicalSize,
	event::WindowEvent,
	event_loop::{ControlFlow, EventLoop, ActiveEventLoop},
	window::{Window, WindowId},
};

struct Triangle {
	window: Option<Window>,
	window_size: PhysicalSize<u32>,
	exit_requested: bool,
}

impl Triangle {
	fn new(window_size: PhysicalSize<u32>) -> Self {
		Self {
			window: None,
			window_size: PhysicalSize { width: 640, height: 480, },
			exit_requested: false,
		}
	}
	fn draw_frame(&mut self) {
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
		cmd.begin_rendering();
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

impl ApplicationHandler for Triangle {
		fn resumed(&mut self, event_loop: &ActiveEventLoop) {
			if self.window.is_none() {
				self.window = Some(
						event_loop
								.create_window(
										Window::default_attributes()
												.with_inner_size(self.window_size)
												.with_title("Acadia"),
								)
								.unwrap(),
				);
			}
    }

		fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
			match event {
				WindowEvent::RedrawRequested => {
					if self.exit_requested == false {
						// self.update();
						self.window.as_ref().unwrap().request_redraw();
					}
				}
				_ => (),
			}
		}
}

fn main() {
	let event_loop = EventLoop::new().unwrap();
	event_loop.set_control_flow(ControlFlow::Poll);

	let mut app = Triangle::new(PhysicalSize { width: 640, height: 480, });

	let _result = event_loop.run_app(&mut app);
	println!("run_app: {:?}", _result);
}

