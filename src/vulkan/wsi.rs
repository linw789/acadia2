use ash::{
	Device, Entry, Instance,
	khr::{self, swapchain},
	vk,
};
use std::{io::Result, rc::Rc};
use winit::{
	raw_window_handle::{HasDisplayHandle, HasWindowHandle},
	window::Window,
};

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
		window: &Window,
		instance: &Instance,
		device: &Device,
		swapchain_width: u32,
		swapchain_height: u32,
	) -> Self {
		let swapchain_loader = swapchain::Device::new(instance, device);

		let vk_entry = Entry::linked();

		let vk_surface = unsafe {
			ash_window::create_surface(
				&vk_entry,
				instance,
				window.display_handle().unwrap().as_raw(),
				window.window_handle().unwrap().as_raw(),
				None,
			)
			.unwrap()
		};

		let surface_loader = khr::surface::Instance::new(&vk_entry, instance);

		let surface_capabilities = unsafe {
			surface_loader
				.get_physical_device_surface_capabilities(physical_device, vk_surface)
				.unwrap()
		};

		let present_mode = unsafe {
			surface_loader.get_physical_device_surface_present_modes(physical_device, vk_surface)
				.unwrap()
				.iter()
				.find(|&&mode| mode == vk::PresentModeKHR::MAILBOX)
				.copied()
				.unwrap_or(vk::PresentModeKHR::FIFO)
		};
	}

	fn create_swapchain(
		loader: &swapchain::Device,
		window: &Window,
		instance: &Instance,
		device: Rc<Device>,
		width: u32,
		height: u32,
	) -> Result<vk::SwapchainKHR> {
		let swapchain = {
			let swapchain_createinfo = vk::SwapchainCreateInfoKHR::default()
				.surface(surface)
				.min_image_count(desired_image_count)
				.image_color_space(surface_format.color_space)
				.image_format(surface_format.format)
				.image_extent(self.image_extent)
				.image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
				.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
				.pre_transform(pre_transform)
				.composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
				.present_mode(present_mode)
				.clipped(true)
				.image_array_layers(1)
				.old_swapchain(old_swapchain);
			unsafe { loader.create_swapchain(&swapchain_createinfo, None).unwrap() }
		};
	}
}
