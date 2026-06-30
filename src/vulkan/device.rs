use ash::{Device as AshDevice, vk};

pub struct Device {
	pub api: AshDevice,
	pub present_queue: vk::Queue,
	physical_memory_properties: vk::PhysicalDeviceMemoryProperties,
	// pipeline_layouts: Vec<vk::Pipeline>,
}

impl Device {
	pub fn new(
		api: AshDevice,
		graphics_queue_family_index: u32,
		physical_memory_properties: vk::PhysicalDeviceMemoryProperties,
	) -> Self {
		let present_queue = unsafe { api.get_device_queue(graphics_queue_family_index, 0) };
		Self {
			api,
			present_queue,
			physical_memory_properties,
		}
	}

	pub fn find_memory_type_index(
		&self,
		memory_requirements: vk::MemoryRequirements,
		memory_prop_flags: vk::MemoryPropertyFlags,
	) -> Option<u32> {
		self.physical_memory_properties.memory_types[..self.physical_memory_properties.memory_type_count as _]
			.iter()
			.enumerate()
			.find(|(index, memory_type)| {
				((1 << index) & memory_requirements.memory_type_bits) != 0
					&& (memory_type.property_flags & memory_prop_flags) == memory_prop_flags
			})
			.map(|(index, _memory_type)| index as _)
	}

	pub fn destruct(&mut self) {
		unsafe {
			self.api.destroy_device(None);
		}
	}
}
