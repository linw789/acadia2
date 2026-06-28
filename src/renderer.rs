use crate::vulkan::{
	base::Base,
	cmdbuf::CmdBuf,
	frame::Frame,
	wsi::{MAX_FRAMES_IN_FLIGHT, Wsi},
};
use ash::{Device, khr, vk};
use std::rc::Rc;
use winit::{
	raw_window_handle::{HasDisplayHandle, HasWindowHandle},
	window::Window,
};

pub struct Renderer {
	base: Base,
	device: Rc<Device>,
	wsi: Wsi,
	frames: [Frame; MAX_FRAMES_IN_FLIGHT],
	frame_count: u64,
}

impl Renderer {
	pub fn new(window: &Window) -> Self {
		let mut base = Base::new(window);

		// Create surface.

		let surface = unsafe {
			ash_window::create_surface(
				&base.entry,
				&base.instance,
				window.display_handle().unwrap().as_raw(),
				window.window_handle().unwrap().as_raw(),
				None,
			)
			.unwrap()
		};

		base.init_physical_device(surface);

		// Create logical device.

		let device = {
			let device_extension_names_raw = [khr::swapchain::NAME.as_ptr()];
			let features = vk::PhysicalDeviceFeatures::default()
				.shader_clip_distance(true)
				.sampler_anisotropy(true)
				.wide_lines(true);
			let mut vk13_features = vk::PhysicalDeviceVulkan13Features::default()
				.synchronization2(true)
				.dynamic_rendering(true);

			let priorities = [1.0];
			let queue_info = vk::DeviceQueueCreateInfo::default()
				.queue_family_index(base.graphics_queue_family_index)
				.queue_priorities(&priorities);
			let device_createinfo = vk::DeviceCreateInfo::default()
				.queue_create_infos(std::slice::from_ref(&queue_info))
				.enabled_extension_names(&device_extension_names_raw)
				.enabled_features(&features)
				.push_next(&mut vk13_features);
			unsafe {
				Rc::new(
					base.instance
						.create_device(base.physical_device, &device_createinfo, None)
						.unwrap(),
				)
			}
		};

		let wsi = {
			let swapchain_extent = vk::Extent2D {
				width: window.inner_size().width,
				height: window.inner_size().height,
			};
			Wsi::new(surface, swapchain_extent, &base, Rc::clone(&device))
		};

		let frames = std::array::from_fn(|_| Frame::new(Rc::clone(&device), base.graphics_queue_family_index));

		Self {
			base,
			device,
			wsi,
			frames,
			frame_count: 0,
		}
	}

	pub fn destruct(&mut self) {
		unsafe {
			for frame in self.frames.iter_mut() {
				frame.destruct();
			}
			self.device.destroy_device(None);
			self.wsi.destruct();
			self.base.destruct();
		}
	}

	pub fn begin_frame(&mut self) -> CmdBuf {
		let in_flight_frame_index = (self.frame_count % (MAX_FRAMES_IN_FLIGHT as u64)) as usize;
		self.wsi.begin_frame(in_flight_frame_index);

		let frame = self.frames[in_flight_frame_index];
		frame.cmd_buf()
	}

	pub fn end_frame(&mut self) {
		self.frame_count += 1;
	}
}
