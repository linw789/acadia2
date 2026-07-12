use crate::vulkan::{
	base::Base,
	cmdbuf::CmdBuf,
	device::Device,
	frame::Frame,
	shader::ShaderManager,
	wsi::{MAX_FRAMES_IN_FLIGHT, Wsi},
};
use ash::{khr, vk};
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
	frame_fences: [vk::Fence; MAX_FRAMES_IN_FLIGHT],
	frame_count: u64,
	shader_manager: ShaderManager,
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
			let device_api = unsafe {
				base.instance
					.create_device(base.physical_device, &device_createinfo, None)
					.unwrap()
			};
			Rc::new(Device::new(
				device_api,
				base.graphics_queue_family_index,
				base.physical_device_mem_props,
			))
		};

		let wsi = {
			let swapchain_extent = vk::Extent2D {
				width: window.inner_size().width,
				height: window.inner_size().height,
			};
			Wsi::new(surface, swapchain_extent, &base, Rc::clone(&device))
		};

		let frames = std::array::from_fn(|_| Frame::new(Rc::clone(&device), base.graphics_queue_family_index));

		let frame_fences = std::array::from_fn(|_| {
			let createinfo = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
			unsafe { device.api.create_fence(&createinfo, None).unwrap() }
		});

		let mut shader_manager = ShaderManager::new(Rc::clone(&device));
		shader_manager.add_graphics_program("assets/shaders/triangle.vert.spv", "assets/shaders/triangle.frag.spv");

		Self {
			base,
			device,
			wsi,
			frames,
			frame_fences,
			frame_count: 0,
			shader_manager,
		}
	}

	pub fn destruct(&mut self) {
		for fence in self.frame_fences.iter() {
			unsafe {
				self.device.api.destroy_fence(*fence, None);
			}
		}
		for frame in self.frames.iter_mut() {
			frame.destruct();
		}
		Rc::get_mut(&mut self.device).unwrap().destruct();
		self.wsi.destruct();
		self.base.destruct();
	}

	pub fn record_frame(&mut self, f: impl FnOnce(&mut CmdBuf, &mut ShaderManager)) {
		let in_flight_frame_index = self.begin_frame();

		let cmd_buf = self.frames[in_flight_frame_index].cmd_buf_mut();
		cmd_buf.set_present_image_and_view(self.wsi.present_image(), self.wsi.present_image_view());
		cmd_buf.set_color_format(self.wsi.surface_format());

		f(cmd_buf, &mut self.shader_manager);

		self.end_frame(in_flight_frame_index);
	}

	fn begin_frame(&mut self) -> usize {
		let in_flight_frame_index = (self.frame_count as usize) % MAX_FRAMES_IN_FLIGHT;

		let frame_fence = self.frame_fences[in_flight_frame_index];
		unsafe {
			self.device.api.wait_for_fences(&[frame_fence], true, u64::MAX).unwrap();
			self.device.api.reset_fences(&[frame_fence]).unwrap();
		}

		// TODO: Why does this (acquire-next-image) need to happen after waiting for the frame fence?
		self.wsi.begin_frame(in_flight_frame_index);

		in_flight_frame_index
	}

	fn end_frame(&mut self, in_flight_frame_index: usize) {
		{
			let frame_fence = self.frame_fences[in_flight_frame_index];
			let present_image_ready_semaphore = self.wsi.present_image_ready_semaphore();
			let render_complete_semaphore = self.wsi.render_complete_semaphore();
			let cmd_buf = self.frames[in_flight_frame_index].cmd_buf();
			let submit_info = vk::SubmitInfo::default()
				.wait_semaphores(std::slice::from_ref(&present_image_ready_semaphore))
				.wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
				.command_buffers(std::slice::from_ref(&cmd_buf.handle))
				.signal_semaphores(std::slice::from_ref(&render_complete_semaphore));
			unsafe {
				self.device
					.api
					.queue_submit(self.device.present_queue, &[submit_info], frame_fence)
					.unwrap();
			}
		}

		self.wsi.end_frame();
		self.frame_count += 1;
	}
}
