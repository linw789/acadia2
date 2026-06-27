use crate::vulkan::{base::Base, wsi::Wsi};
use ash::{
	Device,
	khr,
	vk,
};
use std::rc::Rc;
use winit::{
	raw_window_handle::{HasDisplayHandle, HasWindowHandle},
	window::Window,
};

pub struct Renderer {
	base: Base,
	device: Rc<Device>,
	wsi: Wsi,
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
				.queue_family_index(base.graphics_family_queue_index)
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

		Self { base, device, wsi }
	}

	pub fn destruct(&mut self) {
		unsafe {
			self.device.destroy_device(None);
			self.wsi.destruct();
			self.base.destruct();
		}
	}

	pub fn begin_frame(&mut self) {}
	pub fn end_frame(&mut self) {}
}
