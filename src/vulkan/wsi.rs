use crate::vulkan::base::Base;
use ash::{
	Device,
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

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct Wsi {
	context: Base,
	surface: vk::SurfaceKHR,
	device: Rc<Device>,

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

	frame_fences: Vec<vk::Fence>,

	frame_count: u64,
}

impl Wsi {
	pub fn new(window: &Window) -> Self {
		let mut context = Base::new(window);

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

		let surface_formats = unsafe {
			surface_loader
				.get_physical_device_surface_formats(context.physical_device, surface)
				.unwrap()
		};
		let surface_format = pick_surface_format(&surface_formats);

		let surface_capabilities = unsafe {
			surface_loader
				.get_physical_device_surface_capabilities(context.physical_device, surface)
				.unwrap()
		};

		let present_mode = unsafe {
			surface_loader
				.get_physical_device_surface_present_modes(context.physical_device, surface)
				.unwrap()
				.iter()
				.find(|&&mode| mode == vk::PresentModeKHR::MAILBOX)
				.copied()
				.unwrap_or(vk::PresentModeKHR::FIFO)
		};

		// Create swapchain.

		let swapchain_extent = vk::Extent2D {
			width: window.inner_size().width,
			height: window.inner_size().height,
		};
		let swapchain_loader = swapchain::Device::new(&context.instance, &device);
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
			unsafe { present_image_ready_semaphores.push(device.create_semaphore(&createinfo, None).unwrap()) };
		}

		let mut render_complete_semaphores = Vec::new();
		for _ in 0..present_images.len() {
			let createinfo = vk::SemaphoreCreateInfo::default();
			unsafe { render_complete_semaphores.push(device.create_semaphore(&createinfo, None).unwrap()) };
		}

		let mut frame_fences = Vec::new();
		for _ in 0..MAX_FRAMES_IN_FLIGHT {
			let createinfo = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
			unsafe {
				frame_fences.push(device.create_fence(&createinfo, None).unwrap());
			}
		}

		Self {
			context,
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
			frame_count: 0,
			frame_fences,
		}
	}

	pub fn begin_frame(&mut self) {
		let in_flight_frame_index = (self.frame_count % (MAX_FRAMES_IN_FLIGHT as u64)) as usize;
		let present_image_ready_semaphore = self.present_image_ready_semaphores[in_flight_frame_index];
		let frame_fence = self.frame_fences[in_flight_frame_index];

		unsafe {
			self.device.wait_for_fences(&[frame_fence], true, u64::MAX).unwrap();
			self.device.reset_fences(&[frame_fence]).unwrap();
		}

		let present_image_index = unsafe {
			self.swapchain_loader
				.acquire_next_image(
					self.swapchain,
					u64::MAX,
					present_image_ready_semaphore,
					vk::Fence::null(),
				)
				.unwrap()
		};
	}

	pub fn end_frame(&mut self) {}
}

impl Drop for Wsi {
	fn drop(&mut self) {
		unsafe {
			self.swapchain_loader.destroy_swapchain(self.swapchain, None);
			self.surface_loader.destroy_surface(self.surface, None);
			self.device.destroy_device(None);
		}
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
