use ash::{vk, version::DeviceV1_0};
use crate::vulkan::Context;

pub struct Buffer<'a> {
	context: &'a Context,
	pub handle: vk::Buffer,
	pub memory: vk::DeviceMemory,
	usage: vk::BufferUsageFlags,
	properties: vk::MemoryPropertyFlags,
	pub capacity: vk::DeviceSize
}

impl<'a> Buffer<'a> {
	pub fn new(context: &'a Context, capacity: vk::DeviceSize, usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags) -> Self {
		let (handle, memory) = Self::allocate(context, capacity, usage, properties);

		Self {
			context,
			handle,
			memory,
			usage,
			properties,
			capacity
		}
	}

	pub fn null(context: &'a Context, usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags) -> Self {
		Self {
			context,
			handle: vk::Buffer::null(),
			memory: vk::DeviceMemory::null(),
			usage,
			properties,
			capacity: 0
		}
	}

	pub fn reallocate(&mut self, capacity: vk::DeviceSize) {
		unsafe {
			self.context.logical_device.free_memory(self.memory, None);
			self.context.logical_device.destroy_buffer(self.handle, None);
		}

		let (handle, memory) = Self::allocate(self.context, capacity, self.usage, self.properties);

		self.handle = handle;
		self.memory = memory;
		self.capacity = capacity;
	}

	fn allocate(
		context: &'a Context,
		capacity: vk::DeviceSize,
		usage: vk::BufferUsageFlags,
		properties: vk::MemoryPropertyFlags) -> (vk::Buffer, vk::DeviceMemory)
	{
		let create_info = vk::BufferCreateInfo::builder()
			.size(capacity)
			.usage(usage)
			.sharing_mode(vk::SharingMode::EXCLUSIVE);
		
		let handle = unsafe { context.logical_device.create_buffer(&create_info, None).unwrap() };
		let memory_requirements = unsafe { context.logical_device.get_buffer_memory_requirements(handle) };
		let memory_type_index = context.physical_device.find_memory_type_index(memory_requirements.memory_type_bits, properties);

		let allocate_info = vk::MemoryAllocateInfo::builder()
			.allocation_size(memory_requirements.size)
			.memory_type_index(memory_type_index as u32);
	
		let memory = unsafe { context.logical_device.allocate_memory(&allocate_info, None).unwrap() };
		unsafe { context.logical_device.bind_buffer_memory(handle, memory, 0).unwrap() };

		(handle, memory)
	}
}

impl<'a> Drop for Buffer<'a> {
	fn drop(&mut self) {
		unsafe {
			self.context.logical_device.free_memory(self.memory, None);
			self.context.logical_device.destroy_buffer(self.handle, None);
		}
	}
}