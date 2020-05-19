use ash::{vk, version::DeviceV1_0, version::InstanceV1_0};
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
	pub fn new(context: &'a Context, required_capacity: vk::DeviceSize, usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags) -> Self {
		let (handle, memory, capacity) = Self::allocate(context, required_capacity, usage, properties);

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

	pub fn resize(&mut self, required_capacity: vk::DeviceSize) {
		unsafe {
			self.context.logical_device.free_memory(self.memory, None);
			self.context.logical_device.destroy_buffer(self.handle, None);
		}

		let (handle, memory, capacity) = Self::allocate(self.context, required_capacity, self.usage, self.properties);

		self.handle = handle;
		self.memory = memory;
		self.capacity = capacity;
	}

	fn allocate(
		context: &'a Context,
		required_capacity: vk::DeviceSize,
		usage: vk::BufferUsageFlags,
		properties: vk::MemoryPropertyFlags) -> (vk::Buffer, vk::DeviceMemory, vk::DeviceSize)
	{
		let create_info = vk::BufferCreateInfo::builder()
			.size(required_capacity)
			.usage(usage)
			.sharing_mode(vk::SharingMode::EXCLUSIVE);
		
		let handle = unsafe { context.logical_device.create_buffer(&create_info, None).unwrap() };
		let memory_requirements = unsafe { context.logical_device.get_buffer_memory_requirements(handle) };
		let capacity = memory_requirements.size;
		let memory_properties = unsafe { context.instance.get_physical_device_memory_properties(context.physical_device.handle) };
		
		let memory_type_index = (0..memory_properties.memory_types.len())
			.find(|&i| memory_requirements.memory_type_bits & (1 << i) != 0 &&
				memory_properties.memory_types[i].property_flags.contains(properties))
			.expect("Could not find suitable memory type") as u32;

		let allocate_info = vk::MemoryAllocateInfo::builder()
			.allocation_size(capacity)
			.memory_type_index(memory_type_index);
	
		let memory = unsafe { context.logical_device.allocate_memory(&allocate_info, None).unwrap() };
		unsafe { context.logical_device.bind_buffer_memory(handle, memory, 0).unwrap() };

		(handle, memory, capacity)
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