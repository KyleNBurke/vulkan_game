use ash::{vk, version::EntryV1_0, version::InstanceV1_0, version::DeviceV1_0, extensions::ext, extensions::khr, vk::Handle};

use std::{
	ffi::{CString, CStr},
	os::raw::{c_void, c_char}
};

use super::PhysicalDevice;

pub struct Context {
	pub instance: ash::Instance,
	pub debug_utils: DebugUtils,
	pub physical_device: PhysicalDevice,
	pub surface: Surface,
	pub logical_device: ash::Device,
	pub graphics_queue: vk::Queue,
	pub present_queue: vk::Queue
}

pub struct DebugUtils {
	pub extension: ext::DebugUtils,
	pub messenger_handle: vk::DebugUtilsMessengerEXT
}

pub struct Surface {
	pub extension: khr::Surface,
	pub handle: vk::SurfaceKHR,
	pub format: vk::SurfaceFormatKHR
}

impl Context {
	pub fn new(glfw: &glfw::Glfw, window: &glfw::Window) -> Self {
		// Create entry
		let entry = ash::Entry::new().unwrap();

		// Create layer and extension lists
		let validation_layer = CString::new("VK_LAYER_KHRONOS_validation").unwrap();
		let required_layers = [validation_layer.as_c_str()];
		let required_device_extensions = [khr::Swapchain::name()];
		
		let mut required_instance_extensions = vec![ext::DebugUtils::name()];
		let required_glfw_instance_extensions_cstring: Vec<CString> = glfw.get_required_instance_extensions().unwrap().iter().map(|s| CString::new(s.as_str()).unwrap()).collect();
		let required_glfw_instance_extensions_cstr: Vec<&CStr> = required_glfw_instance_extensions_cstring.iter().map(|s| s.as_c_str()).collect();
		required_instance_extensions.extend_from_slice(&required_glfw_instance_extensions_cstr);

		// Check layer and extension support
		let available_layers = entry.enumerate_instance_layer_properties().unwrap();
		for required_layer in &required_layers {
			available_layers.iter()
				.find(|available_layer| unsafe { CStr::from_ptr(available_layer.layer_name.as_ptr()) } == *required_layer)
				.unwrap_or_else(|| panic!("Required layer {} not supported", required_layer.to_str().unwrap()));
		}

		let available_instance_extensions = entry.enumerate_instance_extension_properties().unwrap();
		for required_instance_extension in &required_instance_extensions {
			available_instance_extensions.iter()
				.find(|available_instance_extension| unsafe { CStr::from_ptr(available_instance_extension.extension_name.as_ptr()) } == *required_instance_extension)
				.unwrap_or_else(|| panic!("Required instance extension {} not supported", required_instance_extension.to_str().unwrap()));
		}

		// Create instance
		let engine_name = CString::new("Vulkan Engine").unwrap();
		let app_info = vk::ApplicationInfo::builder()
			.engine_name(engine_name.as_c_str())
			.engine_version(vk::make_version(1, 0, 0))
			.api_version(vk::make_version(1, 2, 0));
		
		let layers: Vec<*const c_char> = required_layers.iter().map(|layer| layer.as_ptr()).collect();
		let instance_extensions: Vec<*const c_char> = required_instance_extensions.iter().map(|extension| extension.as_ptr()).collect();
		
		let mut debug_messenger_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
			.message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
				| vk::DebugUtilsMessageSeverityFlagsEXT::ERROR)
			.message_type(vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
				| vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
				| vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE)
			.pfn_user_callback(Some(Self::debug_message_callback))
			.build();
	
		let instance_create_info = vk::InstanceCreateInfo::builder()
			.application_info(&app_info)
			.enabled_layer_names(&layers)
			.enabled_extension_names(&instance_extensions)
			.push_next(&mut debug_messenger_create_info);
		
		let instance = unsafe { entry.create_instance(&instance_create_info, None).unwrap() };

		// Create debug utils
		let debug_utils_extension = ext::DebugUtils::new(&entry, &instance);
		let debug_utils_messenger_handle = unsafe { debug_utils_extension.create_debug_utils_messenger(&debug_messenger_create_info, None).unwrap() };
		let debug_utils = DebugUtils {
			extension: debug_utils_extension,
			messenger_handle: debug_utils_messenger_handle
		};

		// Create surface extension and handle
		let surface_extension = khr::Surface::new(&entry, &instance);
		let mut surface_handle_raw: u64 = 0;
		let result = window.create_window_surface(instance.handle().as_raw() as usize, std::ptr::null(), &mut surface_handle_raw as *mut u64);
		assert_eq!(result, 0, "Could not create window surface");
		let surface_handle = vk::SurfaceKHR::from_raw(surface_handle_raw);

		// Create the physical device
		let device_extensions: Vec<CString> = required_device_extensions.iter().map(|extension| CString::new(extension.to_str().unwrap()).unwrap()).collect();
		let physical_device = PhysicalDevice::new(&instance, surface_handle, &surface_extension, &device_extensions);

		// Create surface format
		let surface_formats = unsafe { surface_extension.get_physical_device_surface_formats(physical_device.handle, surface_handle).unwrap() };
		let surface_format_option = surface_formats.iter().find(|f| f.format == vk::Format::B8G8R8A8_SRGB && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR);
		let surface_format = *surface_format_option.unwrap_or_else(|| &surface_formats[0]);

		// Create logical device and queues
		let graphics_queue_family = physical_device.graphics_queue_family;
		let present_queue_family = physical_device.present_queue_family;

		let mut device_queue_create_infos = vec![vk::DeviceQueueCreateInfo::builder()
			.queue_family_index(graphics_queue_family)
			.queue_priorities(&[1.0])
			.build()];

		if graphics_queue_family != present_queue_family {
			device_queue_create_infos.push(vk::DeviceQueueCreateInfo::builder()
				.queue_family_index(present_queue_family)
				.queue_priorities(&[1.0])
				.build());
		}

		let features = vk::PhysicalDeviceFeatures::builder();
		let device_extensions: Vec<*const c_char> = required_device_extensions.iter().map(|extension| extension.as_ptr()).collect();

		let device_create_info = vk::DeviceCreateInfo::builder()
			.queue_create_infos(&device_queue_create_infos)
			.enabled_features(&features)
			.enabled_layer_names(&layers)
			.enabled_extension_names(&device_extensions);
		
		let logical_device = unsafe { instance.create_device(physical_device.handle, &device_create_info, None).unwrap() };
		let graphics_queue = unsafe { logical_device.get_device_queue(graphics_queue_family, 0) };
		let present_queue = unsafe { logical_device.get_device_queue(present_queue_family, 0) };

		Self {
			instance,
			debug_utils,
			physical_device,
			surface: Surface {
				extension: surface_extension,
				handle: surface_handle,
				format: surface_format
			},
			logical_device,
			graphics_queue,
			present_queue
		}
	}
	
	unsafe extern "system" fn debug_message_callback(
		_message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
		_message_type: vk::DebugUtilsMessageTypeFlagsEXT,
		p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
		_p_user_data: *mut c_void) -> vk::Bool32
	{
		let message = CStr::from_ptr((*p_callback_data).p_message).to_str().unwrap();
		println!("{}\n", message);
		vk::FALSE
	}
}

impl Drop for Context {
	fn drop(&mut self) {
		unsafe {
			self.logical_device.destroy_device(None);
			self.surface.extension.destroy_surface(self.surface.handle, None);
			self.debug_utils.extension.destroy_debug_utils_messenger(self.debug_utils.messenger_handle, None);
			self.instance.destroy_instance(None);
		}
	}
}