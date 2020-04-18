use ash::{vk, version::EntryV1_0, version::InstanceV1_0, version::DeviceV1_0, extensions::ext, extensions::khr, vk::Handle};
use glfw::Context;
use std::ffi::{CString, CStr};
use std::os::raw::{c_void, c_char};
use crate::Mesh;

const REQUIRED_LAYERS: &[&str] = &["VK_LAYER_KHRONOS_validation"];
const REQUIRED_INSTANCE_EXTENSIONS: &[&str] = &["VK_EXT_debug_utils"];
const REQUIRED_DEVICE_EXTENSIONS: &[&str] = &["VK_KHR_swapchain"];

const MAX_FRAMES_IN_FLIGHT: usize = 2;

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

pub struct Renderer {
	instance: ash::Instance,
	debug_utils: DebugUtils,
	surface: Surface,
	physical_device: vk::PhysicalDevice,
	logical_device: ash::Device,
	graphics_queue_family: QueueFamily,
	present_queue_family: QueueFamily,
	command_pool: vk::CommandPool,
	render_pass: vk::RenderPass,
	pipeline: Pipeline,
	in_flight_frames: Vec<InFlightFrame>,
	current_in_flight_frame: usize,
	swapchain: Swapchain,
	vertex_buffer: Buffer
}

struct DebugUtils {
	extension: ext::DebugUtils,
	messenger_handle: vk::DebugUtilsMessengerEXT
}

struct Surface {
	extension: khr::Surface,
	handle: vk::SurfaceKHR,
	format: vk::SurfaceFormatKHR
}

struct QueueFamily {
	index: u32,
	queue: vk::Queue
}

struct Swapchain {
	extension: khr::Swapchain,
	handle: vk::SwapchainKHR,
	extent: vk::Extent2D,
	frames: Vec<Frame>
}

struct Frame {
	image_view: vk::ImageView,
	framebuffer: vk::Framebuffer,
	command_buffer: vk::CommandBuffer,
	fence: vk::Fence
}

struct Pipeline {
	handle: vk::Pipeline,
	layout: vk::PipelineLayout
}

struct InFlightFrame {
	image_available: vk::Semaphore,
	render_finished: vk::Semaphore,
	fence: vk::Fence
}

struct Buffer {
	handle: vk::Buffer,
	memory: vk::DeviceMemory
}

impl Renderer {
	pub fn new(glfw: &glfw::Glfw, window: &glfw::Window) -> Self {
		let entry = ash::Entry::new().unwrap();

		let layers_c_string: Vec<CString> = REQUIRED_LAYERS.iter().map(|&s| CString::new(s).unwrap()).collect();

		let glfw_instance_extensions_string = glfw.get_required_instance_extensions().unwrap();
		let mut glfw_instance_extensions_c_string: Vec<CString> = glfw_instance_extensions_string.iter().map(|s| CString::new(s.as_str()).unwrap()).collect();
		let mut instance_extensions_c_string: Vec<CString> = REQUIRED_INSTANCE_EXTENSIONS.iter().map(|&s| CString::new(s).unwrap()).collect();
		instance_extensions_c_string.append(&mut glfw_instance_extensions_c_string);
		
		let device_extensions_c_string: Vec<CString> = REQUIRED_DEVICE_EXTENSIONS.iter().map(|&s| CString::new(s).unwrap()).collect();

		let instance = Self::create_instance(&entry, &layers_c_string, &instance_extensions_c_string);
		let debug_utils = Self::create_debug_utils(&entry, &instance);
		let (surface_extension, surface_handle) = Self::create_surface(&entry, &instance, window);

		let (physical_device, graphics_queue_family_index, present_queue_family_index) = Self::choose_physical_device(
			&instance,
			&device_extensions_c_string,
			&surface_extension,
			&surface_handle);
		
		let surface_format = Self::create_surface_format(&surface_extension, &surface_handle, &physical_device);

		let (logical_device, graphics_queue_handle, present_queue_handle) = Self::create_logical_device(
			&instance,
			&layers_c_string,
			&device_extensions_c_string,
			&physical_device,
			graphics_queue_family_index,
			present_queue_family_index);

		let (framebuffer_width, framebuffer_height) = window.get_framebuffer_size();
		let (swapchain_extension, swapchain_handle, swapchain_extent, swapchain_image_views) = Self::create_swapchain(
			&instance,
			&physical_device,
			&logical_device,
			&surface_extension,
			&surface_handle,
			&surface_format,
			graphics_queue_family_index,
			present_queue_family_index,
			framebuffer_width as u32,
			framebuffer_height as u32);
		
		let render_pass = Self::create_render_pass(&logical_device, &surface_format);
		let pipeline = Self::create_pipeline(&logical_device, &swapchain_extent, &render_pass);
		let framebuffers = Self::create_framebuffers(&logical_device, &swapchain_image_views, &swapchain_extent, &render_pass);
		let command_pool = Self::create_command_pool(&logical_device, graphics_queue_family_index);
		let command_buffers = Self::create_command_buffers(&logical_device, &command_pool, swapchain_image_views.len() as u32);
		let in_flight_frames = Self::create_in_flight_frames(&logical_device);

		let mut swapchain_frames: Vec<Frame> = Vec::with_capacity(swapchain_image_views.len());
		for i in 0..swapchain_image_views.len() {
			swapchain_frames.push(Frame {
				image_view: swapchain_image_views[i],
				framebuffer: framebuffers[i],
				command_buffer: command_buffers[i],
				fence: vk::Fence::null()
			});
		}

		Renderer {
			instance,
			debug_utils,
			surface: Surface {
				extension: surface_extension,
				handle: surface_handle,
				format: surface_format
			},
			physical_device,
			logical_device,
			graphics_queue_family: QueueFamily {
				index: graphics_queue_family_index,
				queue: graphics_queue_handle
			},
			present_queue_family: QueueFamily {
				index: present_queue_family_index,
				queue: present_queue_handle
			},
			command_pool,
			render_pass,
			pipeline,
			in_flight_frames,
			current_in_flight_frame: 0,
			swapchain: Swapchain {
				extension: swapchain_extension,
				handle: swapchain_handle,
				extent: swapchain_extent,
				frames: swapchain_frames
			},
			vertex_buffer: Buffer {
				handle: vk::Buffer::null(),
				memory: vk::DeviceMemory::null()
			}
		}
	}

	pub fn create_instance(entry: &ash::Entry, layers: &Vec<CString>, instance_extensions: &Vec<CString>) -> ash::Instance {
		let available_layers = entry.enumerate_instance_layer_properties().unwrap();
		let mut layers_ptr = Vec::with_capacity(layers.len());
		for layer in layers {
			available_layers.iter()
				.find(|l| unsafe { CStr::from_ptr(l.layer_name.as_ptr()) } == layer.as_c_str())
				.expect(&format!("Required layer {} not supported", layer.to_str().unwrap()));
			layers_ptr.push(layer.as_ptr());
		}

		let available_instance_extensions = entry.enumerate_instance_extension_properties().unwrap();
		let mut instance_extensions_ptr = Vec::with_capacity(instance_extensions.len());
		for instance_extension in instance_extensions {
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
		
		let mut debug_messenger_create_info = Self::create_debug_messenger_create_info();

		let create_info = vk::InstanceCreateInfo::builder()
			.application_info(&app_info)
			.enabled_extension_names(&instance_extensions_ptr)
			.enabled_layer_names(&layers_ptr)
			.push_next(&mut debug_messenger_create_info);
		
		unsafe { entry.create_instance(&create_info, None).unwrap() }
	}

	fn create_debug_utils(entry: &ash::Entry, instance: &ash::Instance) -> DebugUtils {
		let create_info = Self::create_debug_messenger_create_info();
		let extension = ext::DebugUtils::new(entry, instance);
		let messenger_handle = unsafe { extension.create_debug_utils_messenger(&create_info, None).unwrap() };
		
		DebugUtils {
			extension,
			messenger_handle
		}
	}

	fn create_debug_messenger_create_info() ->  vk::DebugUtilsMessengerCreateInfoEXT {
		vk::DebugUtilsMessengerCreateInfoEXT::builder()
			.message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
				| vk::DebugUtilsMessageSeverityFlagsEXT::ERROR)
			.message_type(vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
				| vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
				| vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE)
			.pfn_user_callback(Some(debug_message_callback))
			.build()
	}

	fn create_surface(entry: &ash::Entry, instance: &ash::Instance, window: &glfw::Window) -> (khr::Surface, vk::SurfaceKHR) {
		let instance_raw = instance.handle().as_raw();
		let mut surface_raw: u64 = 0;
		
		let result = unsafe { glfw::ffi::glfwCreateWindowSurface(instance_raw as usize, window.window_ptr(), std::ptr::null(), &mut surface_raw as *mut u64) };
		if result != 0 {
			panic!("Could not create window surface");
		}

		(khr::Surface::new(entry, instance), vk::SurfaceKHR::from_raw(surface_raw))
	}

	fn choose_physical_device(
		instance: &ash::Instance,
		device_extensions: &Vec<CString>,
		surface_extension: &khr::Surface,
		surface_handle: &vk::SurfaceKHR) -> (vk::PhysicalDevice, u32, u32)
	{
		let devices = unsafe { instance.enumerate_physical_devices().unwrap() };

		'main: for device in devices {
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

				if unsafe { surface_extension.get_physical_device_surface_support(device, i as u32, *surface_handle).unwrap() } {
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

			let formats = unsafe { surface_extension.get_physical_device_surface_formats(device, *surface_handle).unwrap() };
			if formats.len() == 0 {
				continue;
			}

			let present_modes = unsafe { surface_extension.get_physical_device_surface_present_modes(device, *surface_handle).unwrap() };
			if present_modes.len() == 0 {
				continue;
			}

			return (device, graphics_queue_family.unwrap() as u32, present_queue_family.unwrap() as u32);
		}

		panic!("No suitable physical device found");
	}

	fn create_surface_format(surface_extension: &khr::Surface, surface_handle: &vk::SurfaceKHR, physical_device: &vk::PhysicalDevice) -> vk::SurfaceFormatKHR {
		let formats = unsafe { surface_extension.get_physical_device_surface_formats(*physical_device, *surface_handle).unwrap() };
		let format = formats.iter().find(|f| f.format == vk::Format::B8G8R8A8_SRGB && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR);
		if format.is_some() { *format.unwrap() } else { formats[0] }
	}

	fn create_logical_device(
		instance: &ash::Instance,
		layers: &Vec<CString>,
		device_extensions: &Vec<CString>,
		physical_device: &vk::PhysicalDevice,
		graphics_queue_family: u32,
		present_queue_family: u32) -> (ash::Device, vk::Queue, vk::Queue)
	{
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
		let layers_ptr: Vec<*const c_char> = layers.iter().map(|e| e.as_ptr()).collect();
		let device_extensions_ptr: Vec<*const c_char> = device_extensions.iter().map(|e| e.as_ptr()).collect();

		let device_create_info = vk::DeviceCreateInfo::builder()
			.queue_create_infos(&device_queue_create_infos)
			.enabled_features(&features)
			.enabled_layer_names(&layers_ptr)
			.enabled_extension_names(&device_extensions_ptr);
		
		let logical_device = unsafe { instance.create_device(*physical_device, &device_create_info, None).unwrap() };
		let graphics_queue = unsafe { logical_device.get_device_queue(graphics_queue_family, 0) };
		let present_queue = unsafe { logical_device.get_device_queue(present_queue_family, 0) };

		(logical_device, graphics_queue, present_queue)
	}

	fn create_swapchain(
		instance: &ash::Instance,
		physical_device: &vk::PhysicalDevice,
		logical_device: &ash::Device,
		surface_extension: &khr::Surface,
		surface_handle: &vk::SurfaceKHR,
		surface_format: &vk::SurfaceFormatKHR,
		graphics_queue_family: u32,
		present_queue_family: u32,
		framebuffer_width: u32,
		framebuffer_height: u32)
	-> (
		khr::Swapchain,
		vk::SwapchainKHR,
		vk::Extent2D,
		Vec<vk::ImageView>)
	{
		let present_modes = unsafe { surface_extension.get_physical_device_surface_present_modes(*physical_device, *surface_handle).unwrap() };
		let present_mode = present_modes.iter().find(|&&m| m == vk::PresentModeKHR::MAILBOX);
		let present_mode = if present_mode.is_some() { *present_mode.unwrap() } else { present_modes[0] };

		let capabilities = unsafe { surface_extension.get_physical_device_surface_capabilities(*physical_device, *surface_handle).unwrap() };
		let mut extent = capabilities.current_extent;
		if capabilities.current_extent.width == std::u32::MAX {
			extent = vk::Extent2D::builder()
				.width(std::cmp::max(capabilities.current_extent.width, std::cmp::min(capabilities.current_extent.width, framebuffer_width)))
				.height(std::cmp::max(capabilities.current_extent.height, std::cmp::min(capabilities.current_extent.height, framebuffer_height)))
				.build();
		}

		let mut image_count = capabilities.min_image_count + 1;
		if capabilities.max_image_count > 0 && image_count > capabilities.max_image_count {
			image_count = capabilities.max_image_count;
		}

		let mut swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
			.surface(*surface_handle)
			.min_image_count(image_count)
			.image_format(surface_format.format)
			.image_color_space(surface_format.color_space)
			.image_extent(extent)
			.image_array_layers(1)
			.image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
			.pre_transform(capabilities.current_transform)
			.composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
			.present_mode(present_mode)
			.clipped(true);
		
		let queue_families = [graphics_queue_family, present_queue_family];
		if graphics_queue_family == present_queue_family {
			swapchain_create_info = swapchain_create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE);
		}
		else {
			swapchain_create_info = swapchain_create_info
				.image_sharing_mode(vk::SharingMode::CONCURRENT)
				.queue_family_indices(&queue_families);
		}

		let extension = khr::Swapchain::new(instance, logical_device);
		let handle = unsafe { extension.create_swapchain(&swapchain_create_info, None).unwrap() };
		let images = unsafe { extension.get_swapchain_images(handle).unwrap() };

		let mut image_views = Vec::with_capacity(images.len());
		for image in &images {
			let image_view_create_info = vk::ImageViewCreateInfo::builder()
				.image(*image)
				.view_type(vk::ImageViewType::TYPE_2D)
				.format(surface_format.format)
				.components(vk::ComponentMapping::builder()
			   		.r(vk::ComponentSwizzle::IDENTITY)
					.g(vk::ComponentSwizzle::IDENTITY)
					.b(vk::ComponentSwizzle::IDENTITY)
					.a(vk::ComponentSwizzle::IDENTITY)
					.build())
				.subresource_range(vk::ImageSubresourceRange::builder()
					.aspect_mask(vk::ImageAspectFlags::COLOR)
					.base_mip_level(0)
					.level_count(1)
					.base_array_layer(0)
					.layer_count(1)
					.build());

			image_views.push(unsafe { logical_device.create_image_view(&image_view_create_info, None).unwrap() });
		}

		(extension, handle, extent, image_views)
	}

	fn create_render_pass(device: &ash::Device, format: &vk::SurfaceFormatKHR) -> vk::RenderPass {
		let attachment_description = vk::AttachmentDescription::builder()
			.format(format.format)
			.samples(vk::SampleCountFlags::TYPE_1)
			.load_op(vk::AttachmentLoadOp::CLEAR)
			.store_op(vk::AttachmentStoreOp::STORE)
			.stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
			.stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
			.initial_layout(vk::ImageLayout::UNDEFINED)
			.final_layout(vk::ImageLayout::PRESENT_SRC_KHR);
		let attachment_descriptions = [attachment_description.build()];
		
		let color_attachment_ref = vk::AttachmentReference::builder()
			.attachment(0)
			.layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
		let color_attachment_refs = [color_attachment_ref.build()];
		
		let subpass_description = vk::SubpassDescription::builder()
			.pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
			.color_attachments(&color_attachment_refs);
		let subpass_descriptions = [subpass_description.build()];

		let subpass_dependency = vk::SubpassDependency::builder()
			.src_subpass(vk::SUBPASS_EXTERNAL)
			.dst_subpass(0)
			.src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
			.src_access_mask(vk::AccessFlags::empty())
			.dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
			.dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE);
		let subpass_dependencies = [subpass_dependency.build()];
		
		let create_info = vk::RenderPassCreateInfo::builder()
			.attachments(&attachment_descriptions)
			.subpasses(&subpass_descriptions)
			.dependencies(&subpass_dependencies);
		
		unsafe { device.create_render_pass(&create_info, None).unwrap() }
	}

	fn create_pipeline(device: &ash::Device, extent: &vk::Extent2D, render_pass: &vk::RenderPass) -> Pipeline {
		let mut curr_dir = std::env::current_exe().unwrap();
		curr_dir.pop();

		let mut vert_file = std::fs::File::open(curr_dir.join("vert.spv").as_path()).unwrap();
		let mut frag_file = std::fs::File::open(curr_dir.join("frag.spv").as_path()).unwrap();
		let vert_file_contents = ash::util::read_spv(&mut vert_file).unwrap();
		let frag_file_contents = ash::util::read_spv(&mut frag_file).unwrap();
		
		let vert_create_info = vk::ShaderModuleCreateInfo::builder().code(&vert_file_contents);
		let frag_create_info = vk::ShaderModuleCreateInfo::builder().code(&frag_file_contents);
		let vert_shader_module = unsafe { device.create_shader_module(&vert_create_info, None).unwrap() };
		let frag_shader_module = unsafe { device.create_shader_module(&frag_create_info, None).unwrap() };

		let entry_point = CString::new("main").unwrap();

		let vert_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
			.stage(vk::ShaderStageFlags::VERTEX)
			.module(vert_shader_module)
			.name(entry_point.as_c_str());

		let frag_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
			.stage(vk::ShaderStageFlags::FRAGMENT)
			.module(frag_shader_module)
			.name(entry_point.as_c_str());
		
		let stages = [vert_stage_create_info.build(), frag_stage_create_info.build()];

		let vert_input_binding_description = vk::VertexInputBindingDescription::builder()
			.binding(0)
			.stride(12)
			.input_rate(vk::VertexInputRate::VERTEX);
		let vert_input_binding_descriptions = [vert_input_binding_description.build()];
		
		let vert_input_attribute_description_position = vk::VertexInputAttributeDescription::builder()	
			.binding(0)
			.location(0)
			.format(vk::Format::R32G32B32_SFLOAT)
			.offset(0);
		let vert_input_attribute_descriptions = [vert_input_attribute_description_position.build()];

		let vert_input_state_create_info = vk::PipelineVertexInputStateCreateInfo::builder()
			.vertex_binding_descriptions(&vert_input_binding_descriptions)
			.vertex_attribute_descriptions(&vert_input_attribute_descriptions);

		let input_assembly_state_create_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
			.topology(vk::PrimitiveTopology::TRIANGLE_LIST)
			.primitive_restart_enable(false);

		let viewport = vk::Viewport::builder()
			.x(0.0)
			.y(0.0)
			.width(extent.width as f32)
			.height(extent.height as f32)
			.min_depth(0.0)
			.max_depth(1.0);
		let viewports = [viewport.build()];
		
		let scissor = vk::Rect2D::builder()
			.offset(vk::Offset2D::builder().x(0).y(0).build())
			.extent(*extent);
		let scissors = [scissor.build()];
		
		let viewport_state_create_info = vk::PipelineViewportStateCreateInfo::builder()
			.viewports(&viewports)
			.scissors(&scissors);
		
		let rasterization_state_create_info = vk::PipelineRasterizationStateCreateInfo::builder()
			.depth_clamp_enable(false)
			.rasterizer_discard_enable(false)
			.polygon_mode(vk::PolygonMode::FILL)
			.line_width(1.0)
			.cull_mode(vk::CullModeFlags::BACK)
			.front_face(vk::FrontFace::CLOCKWISE)
			.depth_bias_enable(false);
		
		let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo::builder()
			.sample_shading_enable(false)
			.rasterization_samples(vk::SampleCountFlags::TYPE_1);
		
		let color_blend_attachment_state = vk::PipelineColorBlendAttachmentState::builder()
			.color_write_mask(vk::ColorComponentFlags::all())
			.blend_enable(false);
		let color_blend_attachment_states = [color_blend_attachment_state.build()];
		
		let color_blend_state_create_info = vk::PipelineColorBlendStateCreateInfo::builder()
			.logic_op_enable(false)
			.attachments(&color_blend_attachment_states);
		
		let layout_create_info = vk::PipelineLayoutCreateInfo::builder();

		let layout = unsafe { device.create_pipeline_layout(&layout_create_info, None).unwrap() };
	
		let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
			.stages(&stages)
			.vertex_input_state(&vert_input_state_create_info)
			.input_assembly_state(&input_assembly_state_create_info)
			.viewport_state(&viewport_state_create_info)
			.rasterization_state(&rasterization_state_create_info)
			.multisample_state(&multisample_state_create_info)
			.color_blend_state(&color_blend_state_create_info)
			.layout(layout)
			.render_pass(*render_pass)
			.subpass(0);
		let pipeline_create_infos = [pipeline_create_info.build()];
		
		let pipelines = unsafe { device.create_graphics_pipelines(vk::PipelineCache::null(), &pipeline_create_infos, None).unwrap() };

		unsafe {
			device.destroy_shader_module(vert_shader_module, None);
			device.destroy_shader_module(frag_shader_module, None);
		}

		Pipeline {
			handle: pipelines[0],
			layout
		}
	}

	fn create_framebuffers(device: &ash::Device, image_views: &Vec<vk::ImageView>, extent: &vk::Extent2D, render_pass: &vk::RenderPass) -> Vec<vk::Framebuffer> {
		let mut framebuffers = Vec::with_capacity(image_views.len());

		for &image_view in image_views {
			let attachments = [image_view];

			let create_info = vk::FramebufferCreateInfo::builder()
				.render_pass(*render_pass)
				.attachments(&attachments)
				.width(extent.width)
				.height(extent.height)
				.layers(1);
			
			framebuffers.push(unsafe { device.create_framebuffer(&create_info, None).unwrap() });
		}

		framebuffers
	}

	fn create_command_pool(device: &ash::Device, graphics_queue_family: u32) -> vk::CommandPool {
		let create_info = vk::CommandPoolCreateInfo::builder()
			.queue_family_index(graphics_queue_family)
			.flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

		unsafe { device.create_command_pool(&create_info, None).unwrap() }
	}

	fn create_command_buffers(device: &ash::Device, command_pool: &vk::CommandPool, count: u32) -> Vec<vk::CommandBuffer> {
		let create_info = vk::CommandBufferAllocateInfo::builder()
			.command_pool(*command_pool)
			.level(vk::CommandBufferLevel::PRIMARY)
			.command_buffer_count(count);
		
		unsafe { device.allocate_command_buffers(&create_info).unwrap() }
	}

	fn create_in_flight_frames(device: &ash::Device) -> Vec<InFlightFrame> {
		let semaphore_create_info = vk::SemaphoreCreateInfo::builder();
		let fence_create_info = vk::FenceCreateInfo::builder()
			.flags(vk::FenceCreateFlags::SIGNALED);

		let mut frames: Vec<InFlightFrame> = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
		for _ in 0..MAX_FRAMES_IN_FLIGHT {
			frames.push(InFlightFrame {
				image_available: unsafe { device.create_semaphore(&semaphore_create_info, None).unwrap() },
				render_finished: unsafe { device.create_semaphore(&semaphore_create_info, None).unwrap() },
				fence: unsafe { device.create_fence(&fence_create_info, None).unwrap() }
			});
		}

		frames
	}

	fn create_buffer(
		instance: &ash::Instance,
		physical_device: &vk::PhysicalDevice,
		logical_device: &ash::Device,
		size: vk::DeviceSize,
		usage: vk::BufferUsageFlags,
		properties: vk::MemoryPropertyFlags) -> Buffer
	{
		let create_info = vk::BufferCreateInfo::builder()
			.size(size as u64)
			.usage(usage)
			.sharing_mode(vk::SharingMode::EXCLUSIVE);
		
		let handle = unsafe { logical_device.create_buffer(&create_info, None).unwrap() };
		let memory_requirements = unsafe { logical_device.get_buffer_memory_requirements(handle) };
		let memory_properties = unsafe { instance.get_physical_device_memory_properties(*physical_device) };
		
		let memory_type_index = (0..memory_properties.memory_types.len())
			.find(|&i| memory_requirements.memory_type_bits & (1 << i) != 0 &&
				memory_properties.memory_types[i].property_flags.contains(properties))
			.expect("Could not find suitable memory type");
		
		let allocate_info = vk::MemoryAllocateInfo::builder()
			.allocation_size(memory_requirements.size)
			.memory_type_index(memory_type_index as u32);

		let memory = unsafe { logical_device.allocate_memory(&allocate_info, None).unwrap() };
		unsafe { logical_device.bind_buffer_memory(handle, memory, 0).unwrap() };

		Buffer {
			handle,
			memory
		}
	}

	pub fn submit_static_meshes(&mut self, meshes: &Vec<Mesh>) {
		unsafe {
			self.logical_device.device_wait_idle().unwrap();
			self.logical_device.destroy_buffer(self.vertex_buffer.handle, None);
			self.logical_device.free_memory(self.vertex_buffer.memory, None);
		}

		let size = meshes.iter().map(|m| m.geometry.get_vertex_data().len()).sum::<usize>() * std::mem::size_of::<f32>();

		let staging_buffer = Self::create_buffer(
			&self.instance,
			&self.physical_device,
			&self.logical_device,
			size as u64,
			vk::BufferUsageFlags::TRANSFER_SRC,
			vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
		
		let mut offset = 0;
		for mesh in meshes {
			let src = mesh.geometry.get_vertex_data();
			unsafe {
				let dst = self.logical_device.map_memory(staging_buffer.memory, offset, size as u64, vk::MemoryMapFlags::empty()).unwrap();
				std::ptr::copy_nonoverlapping(src.as_ptr(), dst as *mut f32, src.len());
				self.logical_device.unmap_memory(staging_buffer.memory);
			}
			offset += (src.len() * std::mem::size_of::<f32>()) as u64;
		}
		
		let vertex_buffer = Self::create_buffer(
			&self.instance,
			&self.physical_device,
			&self.logical_device,
			size as u64,
			vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
			vk::MemoryPropertyFlags::DEVICE_LOCAL);
		
		let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
			.level(vk::CommandBufferLevel::PRIMARY)
			.command_pool(self.command_pool)
			.command_buffer_count(1);
		let command_buffer = unsafe { self.logical_device.allocate_command_buffers(&command_buffer_allocate_info).unwrap()[0] };

		let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
			.flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
		
		let region = vk::BufferCopy::builder()
			.size(size as u64);
		let regions = [region.build()];
		
		unsafe {
			self.logical_device.begin_command_buffer(command_buffer, &command_buffer_begin_info).unwrap();
			self.logical_device.cmd_copy_buffer(command_buffer, staging_buffer.handle, vertex_buffer.handle, &regions);
			self.logical_device.end_command_buffer(command_buffer).unwrap();
		}

		self.vertex_buffer = vertex_buffer;

		let command_buffers = [command_buffer];
		let submit_info = vk::SubmitInfo::builder()
			.command_buffers(&command_buffers);
		let submit_infos = [submit_info.build()];
		
		unsafe {
			self.logical_device.queue_submit(self.graphics_queue_family.queue, &submit_infos, vk::Fence::null()).unwrap();
			self.logical_device.queue_wait_idle(self.graphics_queue_family.queue).unwrap();
			self.logical_device.free_command_buffers(self.command_pool, &command_buffers);
			self.logical_device.destroy_buffer(staging_buffer.handle, None);
			self.logical_device.free_memory(staging_buffer.memory, None);
		}

		let clear_color = vk::ClearValue {
			color: vk::ClearColorValue {
				float32: [0.0, 0.0, 0.0, 1.0]
			}
		};
		let clear_colors = [clear_color];

		let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder();

		let mut render_pass_begin_info = vk::RenderPassBeginInfo::builder()
			.render_pass(self.render_pass)
			.render_area(vk::Rect2D::builder()
				.offset(vk::Offset2D::builder().x(0).y(0).build())
				.extent(self.swapchain.extent)
				.build())
			.clear_values(&clear_colors);

		for frame in &self.swapchain.frames {
			render_pass_begin_info = render_pass_begin_info.framebuffer(frame.framebuffer);
			
			unsafe {
				self.logical_device.begin_command_buffer(frame.command_buffer, &command_buffer_begin_info).unwrap();
				self.logical_device.cmd_begin_render_pass(frame.command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
				self.logical_device.cmd_bind_pipeline(frame.command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline.handle);
			}
			
			let mut offset = 0;
			for mesh in meshes {
				let vertex_data = mesh.geometry.get_vertex_data();
				let buffers = [self.vertex_buffer.handle];
				let offsets = [offset];
				unsafe {
					self.logical_device.cmd_bind_vertex_buffers(frame.command_buffer, 0, &buffers, &offsets);
					self.logical_device.cmd_draw(frame.command_buffer, vertex_data.len() as u32 / 3, 1, 0, 0);
				}
				offset += (vertex_data.len() * std::mem::size_of::<f32>()) as u64;
			}

			unsafe {
				self.logical_device.cmd_end_render_pass(frame.command_buffer);
				self.logical_device.end_command_buffer(frame.command_buffer).unwrap();
			}
		}
	}
	
	pub fn render(&mut self, window: &glfw::Window) {
		let in_flight_frame = &self.in_flight_frames[self.current_in_flight_frame];
		
		let fences = [in_flight_frame.fence];
		unsafe { self.logical_device.wait_for_fences(&fences, true, std::u64::MAX).unwrap() };
		
		let result = unsafe {
			self.swapchain.extension.acquire_next_image(self.swapchain.handle,
				std::u64::MAX,
				in_flight_frame.image_available,
				vk::Fence::null())
		};

		if result.is_err() {
			if result.unwrap_err() == vk::Result::ERROR_OUT_OF_DATE_KHR {
				let (width, height) = window.get_framebuffer_size();
				self.recreate_swapchain(width as u32, height as u32);
				return;
			}

			panic!("Could not aquire a swapchain image");
		}

		let image_index = result.unwrap().0;
		let swapchain_frame = &mut self.swapchain.frames[image_index as usize];

		if swapchain_frame.fence != vk::Fence::null() {
			let fences = [swapchain_frame.fence];
			unsafe { self.logical_device.wait_for_fences(&fences, true, std::u64::MAX).unwrap() };
		}

		swapchain_frame.fence = in_flight_frame.fence;
		let image_available_semaphores = [in_flight_frame.image_available];
		let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
		let command_buffers = [swapchain_frame.command_buffer];
		let render_finished_semaphores = [in_flight_frame.render_finished];
		let submit_info = vk::SubmitInfo::builder()
			.wait_semaphores(&image_available_semaphores)
			.wait_dst_stage_mask(&wait_stages)
			.command_buffers(&command_buffers)
			.signal_semaphores(&render_finished_semaphores);
		let submit_infos = [submit_info.build()];

		unsafe {
			self.logical_device.reset_fences(&fences).unwrap();
			self.logical_device.queue_submit(self.graphics_queue_family.queue, &submit_infos, in_flight_frame.fence).unwrap();
		}

		let swapchains = [self.swapchain.handle];
		let image_indices = [image_index];
		let present_info = vk::PresentInfoKHR::builder()
			.wait_semaphores(&render_finished_semaphores)
			.swapchains(&swapchains)
			.image_indices(&image_indices);
		
		let result = unsafe { self.swapchain.extension.queue_present(self.graphics_queue_family.queue, &present_info) };

		if result.is_err() {
			if result.unwrap_err() == vk::Result::ERROR_OUT_OF_DATE_KHR || result.unwrap() {
				let (width, height) = window.get_framebuffer_size();
				self.recreate_swapchain(width as u32, height as u32);
			}
			else {
				panic!("Could not present swapchain image");
			}
		}

		self.current_in_flight_frame = (self.current_in_flight_frame + 1) % MAX_FRAMES_IN_FLIGHT;
	}

	pub fn recreate_swapchain(&mut self, width: u32, height: u32) {
		unsafe {
			self.logical_device.device_wait_idle().unwrap();
			self.cleanup_swapchain();
		}

		let (swapchain_extension, swapchain_handle, swapchain_extent, swapchain_image_views) = Self::create_swapchain(
			&self.instance,
			&self.physical_device,
			&self.logical_device,
			&self.surface.extension,
			&self.surface.handle,
			&self.surface.format,
			self.graphics_queue_family.index,
			self.present_queue_family.index,
			width,
			height);
		
		self.pipeline = Self::create_pipeline(&self.logical_device, &swapchain_extent, &self.render_pass);
		let framebuffers = Self::create_framebuffers(&self.logical_device, &swapchain_image_views, &swapchain_extent, &self.render_pass);
		let command_buffers = Self::create_command_buffers(&self.logical_device, &self.command_pool, swapchain_image_views.len() as u32);

		let mut swapchain_frames: Vec<Frame> = Vec::with_capacity(swapchain_image_views.len());
		for i in 0..swapchain_image_views.len() {
			swapchain_frames.push(Frame {
				image_view: swapchain_image_views[i],
				framebuffer: framebuffers[i],
				command_buffer: command_buffers[i],
				fence: vk::Fence::null()
			});
		}

		self.swapchain = Swapchain {
			extension: swapchain_extension,
			handle: swapchain_handle,
			extent: swapchain_extent,
			frames: swapchain_frames
		};
	}

	unsafe fn cleanup_swapchain(&mut self) {
		let mut command_buffers: Vec<vk::CommandBuffer> = Vec::with_capacity(self.swapchain.frames.len());

		for frame in &self.swapchain.frames {
			self.logical_device.destroy_image_view(frame.image_view, None);
			self.logical_device.destroy_framebuffer(frame.framebuffer, None);
			command_buffers.push(frame.command_buffer);
		}
		
		self.logical_device.free_command_buffers(self.command_pool, &command_buffers);
		self.logical_device.destroy_pipeline_layout(self.pipeline.layout, None);
		self.logical_device.destroy_pipeline(self.pipeline.handle, None);
		self.swapchain.extension.destroy_swapchain(self.swapchain.handle, None);
	}
}

impl Drop for Renderer {
	fn drop(&mut self) {
		unsafe {
			self.logical_device.device_wait_idle().unwrap();
			self.cleanup_swapchain();

			for frame in &self.in_flight_frames {
				self.logical_device.destroy_semaphore(frame.image_available, None);
				self.logical_device.destroy_semaphore(frame.render_finished, None);
				self.logical_device.destroy_fence(frame.fence, None);
			}

			self.logical_device.destroy_buffer(self.vertex_buffer.handle, None);
			self.logical_device.free_memory(self.vertex_buffer.memory, None);
			self.logical_device.destroy_render_pass(self.render_pass, None);
			self.logical_device.destroy_command_pool(self.command_pool, None);
			self.logical_device.destroy_device(None);
			self.surface.extension.destroy_surface(self.surface.handle, None);
			self.debug_utils.extension.destroy_debug_utils_messenger(self.debug_utils.messenger_handle, None);
			self.instance.destroy_instance(None);
		}
	}
}