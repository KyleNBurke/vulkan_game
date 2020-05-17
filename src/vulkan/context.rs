use ash::{vk, version::EntryV1_0, version::InstanceV1_0, version::DeviceV1_0, extensions::ext, extensions::khr, vk::Handle};
use glfw::Context as glfwContext;
use std::ffi::{CString, CStr};
use std::os::raw::{c_void, c_char};

const REQUIRED_LAYERS: &[&str] = &["VK_LAYER_KHRONOS_validation"];
const REQUIRED_INSTANCE_EXTENSIONS: &[&str] = &["VK_EXT_debug_utils"];
const REQUIRED_DEVICE_EXTENSIONS: &[&str] = &["VK_KHR_swapchain"];

pub struct Context {
	pub instance: ash::Instance,
	pub debug_utils: DebugUtils,
	pub physical_device: PhysicalDevice,
	pub surface: Surface,
	pub logical_device: ash::Device,
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

pub struct PhysicalDevice {
	pub handle: vk::PhysicalDevice,
	pub graphics_queue_family: QueueFamily,
	pub present_quue_family: QueueFamily,
	pub min_uniform_buffer_offset_alignment: u64
}

pub struct QueueFamily {
	pub index: u32,
	pub queue: vk::Queue
}

impl Context {
	pub fn new(glfw: &glfw::Glfw, window: &glfw::Window) -> Self {
		// Create ash entry
		let entry = ash::Entry::new().unwrap();

		// Setup layer and extension strings for foreign function interfacing
		let layers_c_string: Vec<CString> = REQUIRED_LAYERS.iter().map(|&s| CString::new(s).unwrap()).collect();
		let layers_ptr_c_char: Vec<*const c_char> = layers_c_string.iter().map(|e| e.as_ptr()).collect();

		let glfw_instance_extensions_string = glfw.get_required_instance_extensions().unwrap();
		let mut glfw_instance_extensions_c_string: Vec<CString> = glfw_instance_extensions_string.iter().map(|s| CString::new(s.as_str()).unwrap()).collect();
		let mut instance_extensions_c_string: Vec<CString> = REQUIRED_INSTANCE_EXTENSIONS.iter().map(|&s| CString::new(s).unwrap()).collect();
		instance_extensions_c_string.append(&mut glfw_instance_extensions_c_string);
		
		let device_extensions_c_string: Vec<CString> = REQUIRED_DEVICE_EXTENSIONS.iter().map(|&s| CString::new(s).unwrap()).collect();
		let device_extensions_ptr_c_char: Vec<*const c_char> = device_extensions_c_string.iter().map(|e| e.as_ptr()).collect();

		// Create instance
		let available_layers = entry.enumerate_instance_layer_properties().unwrap();
		for layer in &layers_c_string {
			available_layers.iter()
				.find(|l| unsafe { CStr::from_ptr(l.layer_name.as_ptr()) } == layer.as_c_str())
				.expect(&format!("Required layer {} not supported", layer.to_str().unwrap()));
		}
	
		let available_instance_extensions = entry.enumerate_instance_extension_properties().unwrap();
		let mut instance_extensions_ptr = Vec::with_capacity(instance_extensions_c_string.len());
		for instance_extension in &instance_extensions_c_string {
			available_instance_extensions.iter()
				.find(|e| unsafe { CStr::from_ptr(e.extension_name.as_ptr()) } == instance_extension.as_c_str())
				.expect(&format!("Required instance extension {} not supported", instance_extension.to_str().unwrap()));
			instance_extensions_ptr.push(instance_extension.as_ptr());
		}
	
		let engine_name = CString::new("Vulkan Engine").unwrap();
		let app_info = vk::ApplicationInfo::builder()
			.engine_name(engine_name.as_c_str())
			.engine_version(vk::make_version(1, 0, 0))
			.api_version(vk::make_version(1, 2, 0));
		
		let debug_messenger_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
			.message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
				| vk::DebugUtilsMessageSeverityFlagsEXT::ERROR)
			.message_type(vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
				| vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
				| vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE)
			.pfn_user_callback(Some(Self::debug_message_callback))
			.build();
		
		let mut debug_messenger_create_info_mut = debug_messenger_create_info;
	
		let instance_create_info = vk::InstanceCreateInfo::builder()
			.application_info(&app_info)
			.enabled_extension_names(&instance_extensions_ptr)
			.enabled_layer_names(&layers_ptr_c_char)
			.push_next(&mut debug_messenger_create_info_mut);
		
		let instance = unsafe { entry.create_instance(&instance_create_info, None).unwrap() };

		// Create debug utils
		let debug_utils_extension = ext::DebugUtils::new(&entry, &instance);
		let debug_utils_messenger_handle = unsafe { debug_utils_extension.create_debug_utils_messenger(&debug_messenger_create_info, None).unwrap() };
		let debug_utils = DebugUtils {
			extension: debug_utils_extension,
			messenger_handle: debug_utils_messenger_handle
		};

		// Create surface extension and handle
		let instance_raw = instance.handle().as_raw();
		let mut surface_raw: u64 = 0;
		
		let result = unsafe { glfw::ffi::glfwCreateWindowSurface(instance_raw as usize, window.window_ptr(), std::ptr::null(), &mut surface_raw as *mut u64) };
		if result != 0 {
			panic!("Could not create window surface");
		}

		let surface_extension = khr::Surface::new(&entry, &instance);
		let surface_handle = vk::SurfaceKHR::from_raw(surface_raw);

		// Choose physical device and get properties
		struct PhysicalDeviceIntermediary {
			handle: vk::PhysicalDevice,
			graphics_queue_family_index: u32,
			present_queue_family_index: u32,
			min_uniform_buffer_offset_alignment: u64
		}

		let mut physical_device_intermediary = None;
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
			let mut graphics_queue_family_index = None;
			let mut present_queue_family_index = None;
			for (i, property) in queue_family_properties.iter().enumerate() {
				if property.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
					graphics_queue_family_index = Some(i);
				}

				if unsafe { surface_extension.get_physical_device_surface_support(device, i as u32, surface_handle).unwrap() } {
					present_queue_family_index = Some(i);
				}
			}

			if graphics_queue_family_index.is_none() || present_queue_family_index.is_none() {
				continue;
			}

			let available_device_extensions = unsafe { instance.enumerate_device_extension_properties(device).unwrap() };
			for device_extension in &device_extensions_c_string {
				let extension = available_device_extensions.iter().find(|e| unsafe { CStr::from_ptr(e.extension_name.as_ptr()) } == device_extension.as_c_str());
				if extension.is_none() {
					continue 'main;
				}
			}

			let formats = unsafe { surface_extension.get_physical_device_surface_formats(device, surface_handle).unwrap() };
			if formats.len() == 0 {
				continue;
			}

			let present_modes = unsafe { surface_extension.get_physical_device_surface_present_modes(device, surface_handle).unwrap() };
			if present_modes.len() == 0 {
				continue;
			}

			physical_device_intermediary = Some(PhysicalDeviceIntermediary {
				handle: device,
				graphics_queue_family_index: graphics_queue_family_index.unwrap() as u32,
				present_queue_family_index: present_queue_family_index.unwrap() as u32,
				min_uniform_buffer_offset_alignment: properties.limits.min_uniform_buffer_offset_alignment
			});

			break;
		}

		if physical_device_intermediary.is_none() {
			panic!("No suitable physical device found");
		}

		let physical_device_intermediary = physical_device_intermediary.unwrap();

		// Create surface format
		let surface_formats = unsafe { surface_extension.get_physical_device_surface_formats(physical_device_intermediary.handle, surface_handle).unwrap() };
		let surface_format_option = surface_formats.iter().find(|f| f.format == vk::Format::B8G8R8A8_SRGB && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR);
		let surface_format = if surface_format_option.is_some() { *surface_format_option.unwrap() } else { surface_formats[0] };

		// Create logical device and queues
		let graphics_queue_family_index = physical_device_intermediary.graphics_queue_family_index;
		let present_queue_family_index = physical_device_intermediary.present_queue_family_index;
		let mut device_queue_create_infos = vec![vk::DeviceQueueCreateInfo::builder()
			.queue_family_index(graphics_queue_family_index)
			.queue_priorities(&[1.0])
			.build()];

		if graphics_queue_family_index != present_queue_family_index {
			device_queue_create_infos.push(vk::DeviceQueueCreateInfo::builder()
				.queue_family_index(present_queue_family_index)
				.queue_priorities(&[1.0])
				.build());
		}

		let features = vk::PhysicalDeviceFeatures::builder();

		let device_create_info = vk::DeviceCreateInfo::builder()
			.queue_create_infos(&device_queue_create_infos)
			.enabled_features(&features)
			.enabled_layer_names(&layers_ptr_c_char)
			.enabled_extension_names(&device_extensions_ptr_c_char);
		
		let logical_device = unsafe { instance.create_device(physical_device_intermediary.handle, &device_create_info, None).unwrap() };
		let graphics_queue = unsafe { logical_device.get_device_queue(graphics_queue_family_index, 0) };
		let present_queue = unsafe { logical_device.get_device_queue(present_queue_family_index, 0) };
		
		Self {
			instance,
			debug_utils,
			physical_device: PhysicalDevice {
				handle: physical_device_intermediary.handle,
				graphics_queue_family: QueueFamily {
					index: physical_device_intermediary.graphics_queue_family_index,
					queue: graphics_queue
				},
				present_quue_family: QueueFamily {
					index: physical_device_intermediary.present_queue_family_index,
					queue: present_queue
				},
				min_uniform_buffer_offset_alignment: physical_device_intermediary.min_uniform_buffer_offset_alignment
			},
			surface: Surface {
				extension: surface_extension,
				handle: surface_handle,
				format: surface_format
			},
			logical_device
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