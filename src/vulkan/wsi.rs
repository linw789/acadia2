use crate::vulkan::context::Context;
use ash::{
	Device, Entry, Instance,
	khr::{self, swapchain},
	vk,
};
use std::{
	io::{Error, ErrorKind, Result},
	rc::Rc,
};
use winit::{
	raw_window_handle::{HasDisplayHandle, HasWindowHandle},
	window::Window,
};

pub struct Wsi {
	context: Context,
	surface: vk::SurfaceKHR,
	device: Rc<Device>,

	surface_loader: khr::surface::Instance,
	surface_format: vk::SurfaceFormatKHR,
	surface_capabilities: vk::SurfaceCapabilitiesKHR,

	swapchain_loader: swapchain::Device,
	swapchain: Option<vk::SwapchainKHR>,
	// Swapchain's images used for presenting.
	present_images: Vec<vk::Image>,
	// Signal when present images have been acquired.
	present_acquired_semaphores: Vec<vk::Semaphore>,

	swapchain_width: u32,
	swapchain_height: u32,
}

impl Wsi {
	pub fn new(window: &Window, swapchain_width: u32, swapchain_height: u32) -> Self {
		let mut context = Context::new(window);

		// Create surface.

		let surface = unsafe {
			ash_window::create_surface(
				&context.entry,
				&context.instance,
				window.display_handle().unwrap().as_raw(),
				window.window_handle().unwrap().as_raw(),
				None,
			)
			.unwrap()
		};

		context.init_physical_device(surface);

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
				.queue_family_index(context.graphics_family_queue_index)
				.queue_priorities(&priorities);
			let device_createinfo = vk::DeviceCreateInfo::default()
				.queue_create_infos(std::slice::from_ref(&queue_info))
				.enabled_extension_names(&device_extension_names_raw)
				.enabled_features(&features)
				.push_next(&mut vk13_features);
			unsafe {
				Rc::new(
					context
						.instance
						.create_device(context.physical_device, &device_createinfo, None)
						.unwrap(),
				)
			}
		};

		// Create swapchain.

		let surface_loader = khr::surface::Instance::new(&context.entry, &context.instance);
		let swapchain_loader = swapchain::Device::new(&context.instance, &device);

		let surface_format = unsafe {
			surface_loader.get_physical_device_surface_formats(context.physical_device, surface).unwrap()
		};
		let surface_capabilities = unsafe {
			surface_loader
				.get_physical_device_surface_capabilities(context.physical_device, surface)
				.unwrap()
		};

		let present_mode = unsafe {
			surface_loader.get_physical_device_surface_present_modes(context.physical_device, surface)
				.unwrap()
				.iter()
				.find(|&&mode| mode == vk::PresentModeKHR::MAILBOX)
				.copied()
				.unwrap_or(vk::PresentModeKHR::FIFO)
		};

		// Create swapchain.

		let swapchain_loader = swapchain::Device::new(&context.instance, &device);
		let swapchain = create_swapchain(
			&swapchain_loader,
			surface,



		Self { context, surface, device,
			surface_loader,
			surface_format,
			surface_capabilities,
		}
	}
}

fn create_swapchain(
	loader: &swapchain::Device,
	surface: vk::SurfaceKHR,
	surface_format: vk::SurfaceFormatKHR,
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

	let desired_image_count =
			u32::min(surface_capabilities.min_image_count + 1, max_image_count);

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

	let swapchain = {
		unsafe { loader.create_swapchain(&swapchain_createinfo, None).unwrap() }
	};
	swapchain
}
