use glam::{vec3, vec4};
use acadia::renderer::Renderer;
use ::winit::{
	application::ApplicationHandler,
	dpi::PhysicalSize,
	event::WindowEvent,
	event_loop::{ControlFlow, EventLoop, ActiveEventLoop},
	window::{Window, WindowId},
};
use std::rc::Rc;

struct Triangle {
	window: Option<Window>,
	window_size: PhysicalSize<u32>,
	exit_requested: bool,

	renderer: Option<Renderer>,
}

impl Triangle {
	fn new(window_size: PhysicalSize<u32>) -> Self {
		Self {
			window: None,
			window_size: PhysicalSize { width: 640, height: 480, },
			exit_requested: false,
			renderer: None,
		}
	}

	fn init_renderer(&mut self) {
		self.renderer = Some(Renderer::new(self.window.as_ref().unwrap()));
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

		let renderer = self.renderer.as_mut().unwrap();

		let mut cmd_buf = renderer.begin_frame();
		{
			let program = renderer
				.shader_manager
				.find_program(&["assets/shaders/triangle.vert.spv", "assets/shaders/triangle.frag.spv"])
				.unwrap();

			let cmd_buf = Rc::get_mut(&mut cmd_buf).unwrap();

			cmd_buf.begin_rendering();
			cmd_buf.set_program(program);
			cmd_buf.end_rendering();
		}

		renderer.end_frame(cmd_buf.as_ref());
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
				self.init_renderer();
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

