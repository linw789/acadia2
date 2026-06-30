use crate::vulkan::device::Device;
use ash::vk;
use std::rc::Rc;

pub struct Image {
	image: vk::Image,
	view: vk::ImageView,
	memory: vk::DeviceMemory,
	extent: vk::Extent3D,
	device: Rc<Device>,
}

pub struct ImageParams {
	pub extent: vk::Extent3D,
	pub mip_levels: u32,
	pub layers: u32,
	pub dimensions: vk::ImageType,
	pub format: vk::Format,
	pub samples: vk::SampleCountFlags,
	pub tiling: vk::ImageTiling,
	pub usage: vk::ImageUsageFlags,
	pub sharing: vk::SharingMode,
	pub queue_families: Vec<u32>,
	pub initial_layout: vk::ImageLayout,
	pub aspect: vk::ImageAspectFlags, 
}

impl Image {
	pub fn new(device: Rc<Device>, params: &ImageParams) -> Self {
		let image = {
			let createinfo = vk::ImageCreateInfo::default()
				.image_type(params.dimensions)
				.format(params.format)
				.extent(params.extent)
				.mip_levels(params.mip_levels)
				.samples(params.samples)
				.tiling(params.tiling)
				.usage(params.usage)
				.sharing_mode(params.sharing)
				.queue_family_indices(&params.queue_families)
				.initial_layout(params.initial_layout);
			unsafe { device.api.create_image(&createinfo, None).unwrap() }
		};

		let memory = {
			let memory_requirements = unsafe { device.api.get_image_memory_requirements(image) };
			let mem_type_index =
				device.find_memory_type_index(memory_requirements, vk::MemoryPropertyFlags::LAZILY_ALLOCATED).unwrap();
			let alloc_info = vk::MemoryAllocateInfo::default()
				.allocation_size(memory_requirements.size)
				.memory_type_index(mem_type_index);
			unsafe { device.api.allocate_memory(&alloc_info, None).unwrap() }
		};

		unsafe {
			device.api.bind_image_memory(image, memory, 0).unwrap();
		}

		let view = {
			let create_info = vk::ImageViewCreateInfo::default()
				.image(image)
				.format(params.format)
				.view_type(vk::ImageViewType::TYPE_2D)
				.subresource_range(
					vk::ImageSubresourceRange::default()
						.aspect_mask(params.aspect)
						.level_count(params.mip_levels)
						.layer_count(params.layers)
				);
			unsafe { device.api.create_image_view(&create_info, None).unwrap() }
		};

		Self {
			image,
			view,
			memory,
			extent: params.extent,
			device,
		}
	}
}

impl ImageParams {
	pub fn default_depth_image(width: u32, height: u32) -> Self {
		Self {
			extent: vk::Extent3D {
				width,
				height,
				depth: 1,
			},
			mip_levels: 1,
			layers: 1,
			dimensions: vk::ImageType::TYPE_2D,
			format: vk::Format::D32_SFLOAT,
			samples: vk::SampleCountFlags::TYPE_1,
			tiling: vk::ImageTiling::OPTIMAL,
			usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT,
			sharing: vk::SharingMode::EXCLUSIVE,
			queue_families: Vec::new(),
			initial_layout: vk::ImageLayout::UNDEFINED,
			aspect: vk::ImageAspectFlags::COLOR,
		}
	}

	pub fn format(mut self, fmt: vk::Format) -> Self {
		self.format = fmt;
		self
	}
	
	pub fn aspect(mut self, aspect: vk::ImageAspectFlags) -> Self {
		self.aspect = aspect;
		self
	}
}
