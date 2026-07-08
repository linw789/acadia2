use ash::{
	Entry, Instance,
	ext::debug_utils,
	khr::surface,
	vk::{self, PhysicalDevice, PhysicalDeviceFeatures, PhysicalDeviceMemoryProperties},
};
use std::{borrow::Cow, ffi::CStr};
use winit::{raw_window_handle::HasDisplayHandle, window::Window};

#[cfg(feature = "vulkan_debug")]
extern "system" fn vulkan_debug_callback(
	message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
	message_type: vk::DebugUtilsMessageTypeFlagsEXT,
	p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
	_user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
	let callback_data = unsafe { *p_callback_data };
	let message_id_number = callback_data.message_id_number;

	if message_severity == vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
		|| message_severity == vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
	{
		let message_id_name = if callback_data.p_message_id_name.is_null() {
			Cow::from("?")
		} else {
			unsafe { CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy() }
		};

		let message = if callback_data.p_message.is_null() {
			Cow::from("?")
		} else {
			unsafe { CStr::from_ptr(callback_data.p_message).to_string_lossy() }
		};

		println!(
			"[Vulkan {message_severity:?}:{message_type:?}] [{message_id_name} ({message_id_number})] : {message}\n",
		);

		if message_severity == vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
			panic!("Vulkan validation failed.");
		}
	}

	vk::FALSE
}

pub struct Base {
	pub entry: Entry,
	pub instance: Instance,
	pub physical_device: PhysicalDevice,
	pub physical_device_mem_props: PhysicalDeviceMemoryProperties,
	pub physical_device_features: PhysicalDeviceFeatures,
	pub graphics_queue_family_index: u32,

	debug_utils_instance: debug_utils::Instance,
	debug_messenger: vk::DebugUtilsMessengerEXT,
}

impl Base {
	pub fn new(window: &Window) -> Self {
		// Initialize vulkan instance.

		let entry = Entry::linked();
		let instance = {
			let mut layer_names = Vec::new();
			#[cfg(feature = "vulkan_debug")]
			layer_names.push(c"VK_LAYER_KHRONOS_validation".as_ptr());

			let mut extension_names =
				ash_window::enumerate_required_extensions(window.display_handle().unwrap().as_raw())
					.unwrap()
					.to_vec();
			#[cfg(feature = "vulkan_debug")]
			extension_names.push(debug_utils::NAME.as_ptr());

			let appinfo = vk::ApplicationInfo::default()
				.application_name(c"Acadia")
				.application_version(0)
				.engine_name(c"Acadia Vulkan Renderer")
				.engine_version(0)
				.api_version(vk::make_api_version(0, 1, 3, 0));

			let create_flags = vk::InstanceCreateFlags::default();

			let createinfo = vk::InstanceCreateInfo::default()
				.application_info(&appinfo)
				.enabled_layer_names(&layer_names)
				.enabled_extension_names(&extension_names)
				.flags(create_flags);

			unsafe { entry.create_instance(&createinfo, None).unwrap() }
		};

		let (debug_utils_instance, debug_messenger) = {
			let debuginfo = vk::DebugUtilsMessengerCreateInfoEXT::default()
				.message_severity(
					vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
						| vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
						| vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
				)
				.message_type(
					vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
						| vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
						| vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
				)
				.pfn_user_callback(Some(vulkan_debug_callback));

			let debug_util_instance = debug_utils::Instance::new(&entry, &instance);
			let debug_messenger = unsafe {
				debug_util_instance
					.create_debug_utils_messenger(&debuginfo, None)
					.unwrap()
			};
			(debug_util_instance, debug_messenger)
		};

		Self {
			entry,
			instance,
			physical_device: PhysicalDevice::null(),
			physical_device_mem_props: PhysicalDeviceMemoryProperties::default(),
			physical_device_features: PhysicalDeviceFeatures::default(),
			graphics_queue_family_index: u32::MAX,

			debug_utils_instance,
			debug_messenger,
		}
	}

	pub fn init_physical_device(&mut self, surface: vk::SurfaceKHR) {
		// Initialize vulkan physical device.

		let surface_loader = surface::Instance::new(&self.entry, &self.instance);

		let (physical_device, graphics_queue_family_index) = unsafe {
			let physical_devices = self.instance.enumerate_physical_devices().unwrap();
			let (physical_device, graphics_queue_family_index) = physical_devices
				.iter()
				.find_map(|physical_device| {
					self.instance
						.get_physical_device_queue_family_properties(*physical_device)
						.iter()
						.enumerate()
						.find_map(|(index, info)| {
							let support_graphics_and_surface = info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
								&& surface_loader
									.get_physical_device_surface_support(*physical_device, index as u32, surface)
									.unwrap();
							if support_graphics_and_surface {
								Some((*physical_device, index))
							} else {
								None
							}
						})
				})
				.expect("Couldn't find suitable physical device.");

			(physical_device, graphics_queue_family_index as u32)
		};

		let features = unsafe { self.instance.get_physical_device_features(physical_device) };

		let physical_device_mem_props = unsafe { self.instance.get_physical_device_memory_properties(physical_device) };

		self.physical_device = physical_device;
		self.physical_device_mem_props = physical_device_mem_props;
		self.physical_device_features = features;
		self.graphics_queue_family_index = graphics_queue_family_index;
	}

	pub fn destruct(&mut self) {
		unsafe {
			self.debug_utils_instance
				.destroy_debug_utils_messenger(self.debug_messenger, None);
			self.instance.destroy_instance(None);
		}
	}
}
