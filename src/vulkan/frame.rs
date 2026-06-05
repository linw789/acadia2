use ash::{Device, vk};

/// `Frame` holds resources needed for each frame.
pub struct Frame {
	present_acquired_semaphore: vk::Semaphore,
	render_complete_semaphre: vk::Semaphore,
	frame_fence: vk::Fence,
}

impl Frame {
	pub fn new(device: &Device) -> Self {
		let frame_fence = {
			let create_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
			unsafe { device.create_fence(&create_info, None).unwrap() }
		};
		let present_acquired_semaphore = {
			let create_info = vk::SemaphoreCreateInfo::default();
			unsafe { device.create_semaphore(&create_info, None).unwrap() }
		};
		let render_complete_semaphre = {
			let create_info = vk::SemaphoreCreateInfo::default();
			unsafe { device.create_semaphore(&create_info, None).unwrap() }
		};
		Self {
			present_acquired_semaphore,
			render_complete_semaphre,
			frame_fence,
		}
	}
}
