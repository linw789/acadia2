use crate::vulkan::device::Device;
use ::ash::vk::{self, MemoryPropertyFlags};
use std::{
	cell::{RefCell, RefMut},
	ffi::c_void,
	ptr::copy_nonoverlapping,
	rc::Rc,
};

#[derive(Default)]
struct Pointer {
	size: u64,
	ptr: *mut c_void,
}

pub struct Buffer {
	device: Rc<Device>,
	pub buf: vk::Buffer,
	pub mem: vk::DeviceMemory,
	ptr: RefCell<Pointer>,
}

/// `BufferWriter` allows sequential writing into `Buffer` objects of difference sizes without
/// explicitly managing offset.
pub struct BufferWriter<'a> {
	offset: u64,
	ptr: RefMut<'a, Pointer>,
}

impl Pointer {
	fn copy_bytes(&mut self, offset: u64, bytes: &[u8]) {
		assert!(
			self.size >= (offset + bytes.len() as u64),
			"buffer size: {}, offset: {}, data size: {}",
			self.size,
			offset,
			bytes.len()
		);
		unsafe {
			let dst_ptr = (self.ptr as *mut u8).add(offset as usize);
			copy_nonoverlapping(bytes.as_ptr(), dst_ptr, bytes.len());
		}
	}
}

impl Buffer {
	pub fn new(
		device: Rc<Device>,
		size: u64,
		usage: vk::BufferUsageFlags,
	) -> Self {
		let buffer_createinfo = vk::BufferCreateInfo::default()
			.size(size)
			.usage(usage)
			.sharing_mode(vk::SharingMode::EXCLUSIVE);
		let buf = unsafe { device.api.create_buffer(&buffer_createinfo, None).unwrap() };

		let buf_mem_req = unsafe { device.api.get_buffer_memory_requirements(buf) };
		let mem_type_index = device.find_memory_type_index(
			buf_mem_req,
			MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
		)
		.expect("Failed to find suitable memory type.");

		let required_size = buf_mem_req.size;
		let mem_alloc_info = vk::MemoryAllocateInfo::default()
			.allocation_size(required_size)
			.memory_type_index(mem_type_index);
		let mem = unsafe { device.api.allocate_memory(&mem_alloc_info, None).unwrap() };

		unsafe {
			device.api.bind_buffer_memory(buf, mem, 0).unwrap();
		}

		let ptr = unsafe {
			device.api.map_memory(mem, 0, size, vk::MemoryMapFlags::empty()).unwrap()
			// No need to unmap memory after copy (persistent mapping).
		};

		Self {
			device,
			buf,
			mem,
			ptr: RefCell::new(Pointer { size, ptr }),
		}
	}

	pub fn buffer_writer(&self) -> BufferWriter<'_> {
		BufferWriter {
			offset: 0,
			ptr: self.ptr.borrow_mut(),
		}
	}
}

impl Drop for Buffer {
	fn drop(&mut self) {
		unsafe {
			self.device.api.free_memory(self.mem, None);
			self.device.api.destroy_buffer(self.buf, None);
		}
	}
}

impl<'a> BufferWriter<'a> {
	pub fn write(&mut self, slice: &[u8]) {
		self.ptr.copy_bytes(self.offset, slice);
		self.offset += slice.len() as u64;
	}
}
