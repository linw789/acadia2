use ::winit::{
	application::ApplicationHandler,
	dpi::PhysicalSize,
	event::WindowEvent,
	event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
	window::{Window, WindowId},
};
use acadia::{vulkan::cmdbuf::RenderingInfo, renderer::Renderer};
use ash::vk;
use glam::{Vec2, Vec4, vec2, vec4};
use std::{rc::Rc, slice};

#[macro_export]
macro_rules! offset_of {
    ($base:path, $field:ident) => {{
        #[allow(unused_unsafe)]
        unsafe {
            let b: $base = std::mem::zeroed();
            std::ptr::addr_of!(b.$field) as isize - std::ptr::addr_of!(b) as isize
        }
    }};
}

#[repr(C, packed)]
struct Vertex {
	position: Vec2,
	color: Vec4,
}

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
			window_size,
			exit_requested: false,
			renderer: None,
		}
	}

	fn init_renderer(&mut self) {
		self.renderer = Some(Renderer::new(self.window.as_ref().unwrap()));
	}

	fn draw_frame(&mut self) {
		let vertices = [
			Vertex {
				position: vec2(-0.5, -0.5),
				color: vec4(0.0, 1.0, 0.0, 1.0),
			},
			Vertex {
				position: vec2(0.5, -0.5),
				color: vec4(0.0, 0.0, 1.0, 1.0),
			},
			Vertex {
				position: vec2(-0.5, 0.5),
				color: vec4(1.0, 0.0, 0.0, 1.0),
			},
		];

		let renderer = self.renderer.as_mut().unwrap();
		renderer.record_frame(|cmd_buf, shader_manager| {
			let program = shader_manager
				.find_program(&["assets/shaders/triangle.vert.spv", "assets/shaders/triangle.frag.spv"])
				.unwrap();

			let rendering_info = RenderingInfo {
				render_area: vk::Rect2D {
					offset: vk::Offset2D { x: 0, y: 0, },
					extent: vk::Extent2D { width: self.window_size.width, height: self.window_size.height, },
				}
			};
			cmd_buf.begin_rendering(rendering_info);
			cmd_buf.set_program(program);

			if !cmd_buf.is_vertex_data_allocated() {
				{
					let mut buf_writer = cmd_buf.alloc_vertex_data(
						0,
						(size_of::<Vertex>() * vertices.len()) as u64,
						size_of::<Vertex>() as u32,
						vk::VertexInputRate::VERTEX,
					);
					let vert_data = unsafe {
						slice::from_raw_parts(vertices.as_ptr() as *const u8, vertices.len() * size_of::<Vertex>())
					};
					buf_writer.write(vert_data);
				}
				cmd_buf.set_vertex_attrib(0, 0, vk::Format::R32G32_SFLOAT, 0);
				cmd_buf.set_vertex_attrib(1, 0, vk::Format::R32G32B32A32_SFLOAT, offset_of!(Vertex, color) as u32);
			}

			cmd_buf.draw(3);

			cmd_buf.end_rendering();
		});
	}

	fn destruct(&mut self) {
	}
}

impl ApplicationHandler for Triangle {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		if self.window.is_none() {
			let window = event_loop
					.create_window(
						Window::default_attributes()
							.with_inner_size(self.window_size)
							.with_title("Acadia"),
					)
					.unwrap();
			self.window_size = window.inner_size();
			self.window = Some(window);
			self.init_renderer();
		}
	}

	fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
		match event {
			WindowEvent::CloseRequested => {
				// self.destruct();
				event_loop.exit();
				self.exit_requested = true;
			}

			WindowEvent::RedrawRequested => {
				if self.exit_requested == false {
					self.draw_frame();
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

	let mut app = Triangle::new(PhysicalSize {
		width: 640,
		height: 480,
	});

	let _result = event_loop.run_app(&mut app);
	println!("run_app: {:?}", _result);
}
