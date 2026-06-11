use ash::{Entry, Instance, vk, khr::swapchain};
use winit::{raw_window_handle::{HasDisplayHandle, HasWindowHandle}, window::Window};
use std::rc::Rc;

pub struct Wsi {
	surface: vk::SurfaceKHR,
	swapchain: Option<vk::SwapchainKHR>,
	// Swapchain's images used for presenting.
	present_images: Vec<vk::Image>,
	// Signal when present images have been acquired.
	present_acquired_semaphores: Vec<vk::Semaphore>,

	swapchain_width: u32,
	swapchain_height: u32,
}

impl Wsi {
	pub fn new(
		instance: &vk::Instance,
		device: &vk::Device,
		swapchain_width: u32,
		swapchain_height: u32,
	) -> Self {
		let swapchain_loader = swapchain::Device::new(instance, device);

	}

	fn create_swapchain(window: &Window, instance: &Instance) {
		let vk_entry = Entry::linked();

		let surface = unsafe {
			ash_window::create_surface(
				&vk_entry,
				instance,
				window.display_handle().unwrap().as_raw(),
				window.window_handle().unwrap().as_raw(),
				None,
			)
			.unwrap()
		};

		let surface_capabilities = 
	}
}
