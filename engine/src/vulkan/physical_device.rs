use ash::{vk, version::InstanceV1_0, extensions::khr};
use std::ffi::{CString, CStr};

pub struct PhysicalDevice {
	pub handle: vk::PhysicalDevice,
	pub graphics_queue_family: u32,
	pub present_queue_family: u32,
	pub memory_properties: vk::PhysicalDeviceMemoryProperties,
	pub min_uniform_buffer_offset_alignment: u64
}

impl PhysicalDevice {
	pub fn new(instance: &ash::Instance, surface_handle: vk::SurfaceKHR, surface_extension: &khr::Surface, device_extensions: &[CString]) -> Self {
		let physical_devices = unsafe { instance.enumerate_physical_devices().unwrap() };

		'main: for device in physical_devices {
			let properties = unsafe { instance.get_physical_device_properties(device) };
			if properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU {
				continue;
			}
			
			let features = unsafe { instance.get_physical_device_features(device) };
			if features.geometry_shader == vk::FALSE {
				continue;
			}

			let queue_family_properties = unsafe { instance.get_physical_device_queue_family_properties(device) };
			let mut graphics_queue_family = None;
			let mut present_queue_family = None;
			for (i, property) in queue_family_properties.iter().enumerate() {
				if property.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
					graphics_queue_family = Some(i);
				}

				if unsafe { surface_extension.get_physical_device_surface_support(device, i as u32, surface_handle).unwrap() } {
					present_queue_family = Some(i);
				}
			}

			if graphics_queue_family.is_none() || present_queue_family.is_none() {
				continue;
			}

			let available_device_extensions = unsafe { instance.enumerate_device_extension_properties(device).unwrap() };
			for device_extension in device_extensions {
				let extension = available_device_extensions.iter().find(|e| unsafe { CStr::from_ptr(e.extension_name.as_ptr()) } == device_extension.as_c_str());
				if extension.is_none() {
					continue 'main;
				}
			}

			let formats = unsafe { surface_extension.get_physical_device_surface_formats(device, surface_handle).unwrap() };
			if formats.is_empty() {
				continue;
			}

			let present_modes = unsafe { surface_extension.get_physical_device_surface_present_modes(device, surface_handle).unwrap() };
			if present_modes.is_empty() {
				continue;
			}

			return Self {
				handle: device,
				graphics_queue_family: graphics_queue_family.unwrap() as u32,
				present_queue_family: present_queue_family.unwrap() as u32,
				memory_properties: unsafe { instance.get_physical_device_memory_properties(device) },
				min_uniform_buffer_offset_alignment: properties.limits.min_uniform_buffer_offset_alignment
			}
		}

		panic!("No suitable physical device found");
	}

	pub fn find_memory_type_index(&self, r#type: u32, properties: vk::MemoryPropertyFlags) -> usize {
		let available_types = self.memory_properties.memory_types;

		(0..available_types.len())
			.find(|&i| r#type & (1 << i) != 0 && available_types[i].property_flags.contains(properties))
			.expect("Could not find suitable memory type")
	}
}