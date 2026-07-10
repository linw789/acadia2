use crate::vulkan::{base::Base, device::Device};
use ash::{
	khr::{self, swapchain},
	vk,
};
use std::rc::Rc;

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct Wsi {
	device: Rc<Device>,

	surface: vk::SurfaceKHR,
	surface_loader: khr::surface::Instance,
	surface_format: vk::SurfaceFormatKHR,
	surface_capabilities: vk::SurfaceCapabilitiesKHR,

	swapchain_loader: swapchain::Device,
	swapchain: vk::SwapchainKHR,
	swapchain_extent: vk::Extent2D,
	// Swapchain's images used for presenting.
	present_images: Vec<vk::Image>,
	// Signal when a present image is ready to write to.
	present_image_ready_semaphores: Vec<vk::Semaphore>,
	// Signal when GPU finishes writing to a present image.
	render_complete_semaphores: Vec<vk::Semaphore>,

	present_image_ready_semaphore: vk::Semaphore,
	present_image_index: usize,
	in_flight_frame_index: usize,
}

impl Wsi {
	pub fn new(surface: vk::SurfaceKHR, swapchain_extent: vk::Extent2D, base: &Base, device: Rc<Device>) -> Self {
		let surface_loader = khr::surface::Instance::new(&base.entry, &base.instance);

		let surface_formats = unsafe {
			surface_loader
				.get_physical_device_surface_formats(base.physical_device, surface)
				.unwrap()
		};
		let surface_format = pick_surface_format(&surface_formats);

		let surface_capabilities = unsafe {
			surface_loader
				.get_physical_device_surface_capabilities(base.physical_device, surface)
				.unwrap()
		};

		let present_mode = unsafe {
			surface_loader
				.get_physical_device_surface_present_modes(base.physical_device, surface)
				.unwrap()
				.iter()
				.find(|&&mode| mode == vk::PresentModeKHR::MAILBOX)
				.copied()
				.unwrap_or(vk::PresentModeKHR::FIFO)
		};

		// Create swapchain.

		let swapchain_loader = swapchain::Device::new(&base.instance, &device.api);
		let swapchain = create_swapchain(
			&swapchain_loader,
			surface,
			&surface_format,
			&surface_capabilities,
			present_mode,
			swapchain_extent,
		);

		let present_images = unsafe { swapchain_loader.get_swapchain_images(swapchain).unwrap() };

		let mut present_image_ready_semaphores = Vec::new();
		for _ in 0..MAX_FRAMES_IN_FLIGHT {
			let createinfo = vk::SemaphoreCreateInfo::default();
			unsafe { present_image_ready_semaphores.push(device.api.create_semaphore(&createinfo, None).unwrap()) };
		}

		let mut render_complete_semaphores = Vec::new();
		for _ in 0..present_images.len() {
			let createinfo = vk::SemaphoreCreateInfo::default();
			unsafe { render_complete_semaphores.push(device.api.create_semaphore(&createinfo, None).unwrap()) };
		}

		Self {
			surface,
			device,
			surface_loader,
			surface_format,
			surface_capabilities,
			swapchain_loader,
			swapchain,
			swapchain_extent,
			present_images,
			present_image_ready_semaphores,
			render_complete_semaphores,
			present_image_ready_semaphore: vk::Semaphore::null(),
			present_image_index: 0,
			in_flight_frame_index: 0,
		}
	}

	pub fn destruct(&mut self) {
		unsafe {
			self.swapchain_loader.destroy_swapchain(self.swapchain, None);
			self.surface_loader.destroy_surface(self.surface, None);
		}
	}

	pub fn present_image(&self) -> vk::Image {
		self.present_images[self.present_image_index]
	}

	pub fn present_image_ready_semaphore(&self) -> vk::Semaphore {
		self.present_image_ready_semaphore
	}

	pub fn render_complete_semaphore(&self) -> vk::Semaphore {
		self.render_complete_semaphores[self.present_image_index]
	}

	pub fn begin_frame(&mut self, in_flight_frame_index: usize) {
		self.in_flight_frame_index = in_flight_frame_index;

		self.present_image_ready_semaphore = self.present_image_ready_semaphores[in_flight_frame_index];

		self.present_image_index = unsafe {
			let result = self.swapchain_loader
				.acquire_next_image(
					self.swapchain,
					u64::MAX,
					self.present_image_ready_semaphore,
					vk::Fence::null(),
				)
				.unwrap();
			
			// TODO: check result.1 to see if swapchain is suboptimal.
			result.0 as usize
		};
	}

	pub fn end_frame(&self) {
		let render_complete_semaphore = self.render_complete_semaphores[self.present_image_index];
		let present_image_index = self.present_image_index as u32;
		let present_info = vk::PresentInfoKHR::default()
			.wait_semaphores(std::slice::from_ref(&render_complete_semaphore))
			.swapchains(std::slice::from_ref(&self.swapchain))
			.image_indices(std::slice::from_ref(&present_image_index));
		unsafe { self.swapchain_loader.queue_present(self.device.present_queue, &present_info).unwrap(); }
	}
}

fn create_swapchain(
	loader: &swapchain::Device,
	surface: vk::SurfaceKHR,
	surface_format: &vk::SurfaceFormatKHR,
	surface_capabilities: &vk::SurfaceCapabilitiesKHR,
	present_mode: vk::PresentModeKHR,
	image_extent: vk::Extent2D,
) -> vk::SwapchainKHR {
	// 0 means there is no limit on max image count.
	let max_image_count = if surface_capabilities.max_image_count == 0 {
		u32::MAX
	} else {
		surface_capabilities.max_image_count
	};

	let desired_image_count = u32::min(surface_capabilities.min_image_count + 1, max_image_count);

	let min_image_extent = surface_capabilities.min_image_extent;
	let max_image_extent = surface_capabilities.max_image_extent;
	let actual_image_extent = match surface_capabilities.current_extent.width {
		u32::MAX => image_extent,
		_ => vk::Extent2D {
			width: u32::min(
				u32::max(min_image_extent.width, image_extent.width),
				max_image_extent.width,
			),
			height: u32::min(
				u32::max(min_image_extent.height, image_extent.height),
				max_image_extent.height,
			),
		},
	};

	let pre_transform = if surface_capabilities
		.supported_transforms
		.contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
	{
		vk::SurfaceTransformFlagsKHR::IDENTITY
	} else {
		surface_capabilities.current_transform
	};

	let swapchain_createinfo = vk::SwapchainCreateInfoKHR::default()
		.surface(surface)
		.min_image_count(desired_image_count)
		.image_color_space(surface_format.color_space)
		.image_format(surface_format.format)
		.image_extent(actual_image_extent)
		.image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
		.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
		.pre_transform(pre_transform)
		.composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
		.present_mode(present_mode)
		.clipped(true)
		.image_array_layers(1)
		// TODO: handle swapchain re-creation.
		.old_swapchain(vk::SwapchainKHR::null());

	let swapchain = { unsafe { loader.create_swapchain(&swapchain_createinfo, None).unwrap() } };
	swapchain
}

fn pick_surface_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
	let mut format_index = 0;
	for (i, sf) in formats.iter().enumerate() {
		if (sf.format == vk::Format::R8G8B8A8_UNORM) || (sf.format == vk::Format::B8G8R8A8_UNORM) {
			format_index = i;
			break;
		}
	}

	let result = formats[format_index];
	assert!(
		result.format != vk::Format::UNDEFINED,
		"Failed to find a proper surface format."
	);
	result
}
