use ash::vk;
use std::rc::Rc;
use crate::vulkan::{cmdbuf::CmdBuf, device::Device};

/// `FrameData` holds resources needed for each frame.
pub struct Frame {
	device: Rc<Device>,
	cmd_pool: vk::CommandPool,
	cmd_bufs: Vec<Rc<CmdBuf>>,
}

impl Frame {
	pub fn new(device: Rc<Device>, graphics_queue_index: u32) -> Self {
		let cmd_pool = unsafe {
			let pool_createinfo = vk::CommandPoolCreateInfo::default()
				.flags(vk::CommandPoolCreateFlags::TRANSIENT)
				.queue_family_index(graphics_queue_index);
			device.api.create_command_pool(&pool_createinfo, None).unwrap()
		};

		let cmd_bufs = unsafe {
			let allocinfo = vk::CommandBufferAllocateInfo::default()
				.command_pool(cmd_pool)
				.level(vk::CommandBufferLevel::PRIMARY)
				.command_buffer_count(1);
			let cbs = device.api.allocate_command_buffers(&allocinfo).unwrap();
			cbs.into_iter().map(|cb| Rc::new(CmdBuf::new(Rc::clone(&device), cb))).collect()
		};

		Self {
			device, 
			cmd_pool,
			cmd_bufs,
		}
	}

	pub fn cmd_buf(&self) -> Rc<CmdBuf> {
		Rc::clone(&self.cmd_bufs[0])
	}

	pub fn destruct(&mut self) {
		self.cmd_bufs.clear();
		unsafe {
			self.device.api.destroy_command_pool(self.cmd_pool, None);
			self.cmd_pool = vk::CommandPool::null();
		}
	}
}
