use ash::{vk, version::EntryV1_0, version::InstanceV1_0, version::DeviceV1_0, extensions::ext, extensions::khr, vk::Handle};
use glfw::Context;
use std::ffi::{CString, CStr};
use std::os::raw::{c_void, c_char};

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
	pub min_uniform_buffer_offset_alignment: u64
}

pub struct QueueFamily {
	pub index: u32,
	pub queue: vk::Queue
}

pub struct Swapchain {
	pub extension: khr::Swapchain,
	pub handle: vk::SwapchainKHR,
	pub extent: vk::Extent2D,
	pub frames: Vec<Frame>
}

pub struct Frame {
	pub image_view: vk::ImageView,
	pub framebuffer: vk::Framebuffer,
	pub command_buffer: vk::CommandBuffer,
	pub fence: vk::Fence
}

pub struct DescriptorSet {
	pub layout: vk::DescriptorSetLayout,
	pub handle: vk::DescriptorSet
}

pub struct Pipeline {
	pub handle: vk::Pipeline,
	pub layout: vk::PipelineLayout
}

pub struct InFlightFrame {
	pub image_available: vk::Semaphore,
	pub render_finished: vk::Semaphore,
	pub fence: vk::Fence
}

pub struct Buffer {
	pub handle: vk::Buffer,
	pub memory: vk::DeviceMemory
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
	
	let mut debug_messenger_create_info = create_debug_messenger_create_info();

	let create_info = vk::InstanceCreateInfo::builder()
		.application_info(&app_info)
		.enabled_extension_names(&instance_extensions_ptr)
		.enabled_layer_names(&layers_ptr)
		.push_next(&mut debug_messenger_create_info);
	
	unsafe { entry.create_instance(&create_info, None).unwrap() }
}

pub fn create_debug_utils(entry: &ash::Entry, instance: &ash::Instance) -> DebugUtils {
	let create_info = create_debug_messenger_create_info();
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

pub fn create_surface(entry: &ash::Entry, instance: &ash::Instance, window: &glfw::Window) -> (khr::Surface, vk::SurfaceKHR) {
	let instance_raw = instance.handle().as_raw();
	let mut surface_raw: u64 = 0;
	
	let result = unsafe { glfw::ffi::glfwCreateWindowSurface(instance_raw as usize, window.window_ptr(), std::ptr::null(), &mut surface_raw as *mut u64) };
	if result != 0 {
		panic!("Could not create window surface");
	}

	(khr::Surface::new(entry, instance), vk::SurfaceKHR::from_raw(surface_raw))
}

pub fn choose_physical_device(
	instance: &ash::Instance,
	device_extensions: &Vec<CString>,
	surface_extension: &khr::Surface,
	surface_handle: &vk::SurfaceKHR) -> (PhysicalDevice, u32, u32)
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

		let physical_device = PhysicalDevice {
			handle: device,
			min_uniform_buffer_offset_alignment: properties.limits.min_uniform_buffer_offset_alignment
		};

		return (physical_device, graphics_queue_family.unwrap() as u32, present_queue_family.unwrap() as u32);
	}

	panic!("No suitable physical device found");
}

pub fn create_surface_format(surface_extension: &khr::Surface, surface_handle: &vk::SurfaceKHR, physical_device: &vk::PhysicalDevice) -> vk::SurfaceFormatKHR {
	let formats = unsafe { surface_extension.get_physical_device_surface_formats(*physical_device, *surface_handle).unwrap() };
	let format = formats.iter().find(|f| f.format == vk::Format::B8G8R8A8_SRGB && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR);
	if format.is_some() { *format.unwrap() } else { formats[0] }
}

pub fn create_logical_device(
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

pub fn create_swapchain(
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

pub fn create_render_pass(device: &ash::Device, format: &vk::SurfaceFormatKHR) -> vk::RenderPass {
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

pub fn create_descriptor_pool(device: &ash::Device) -> vk::DescriptorPool {
	let uniform_buffer_pool_size = vk::DescriptorPoolSize::builder()
		.ty(vk::DescriptorType::UNIFORM_BUFFER)
		.descriptor_count(1);

	let dynamic_uniform_buffer_pool_size = vk::DescriptorPoolSize::builder()
		.ty(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
		.descriptor_count(1);
	
	let pool_sizes = [uniform_buffer_pool_size.build(), dynamic_uniform_buffer_pool_size.build()];
	
	let create_info = vk::DescriptorPoolCreateInfo::builder()
		.pool_sizes(&pool_sizes)
		.max_sets(2);
	
	unsafe { device.create_descriptor_pool(&create_info, None).unwrap() }
}

pub fn create_descriptor_sets(device: &ash::Device, pool: &vk::DescriptorPool) -> (DescriptorSet, DescriptorSet) {
	let layout_binding = vk::DescriptorSetLayoutBinding::builder()
		.binding(0)
		.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
		.descriptor_count(1)
		.stage_flags(vk::ShaderStageFlags::VERTEX);
	let layout_bindings = [layout_binding.build()];
	
	let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
		.bindings(&layout_bindings);
	
	let projection_matrix_layout = unsafe { device.create_descriptor_set_layout(&create_info, None).unwrap() };

	let layout_binding = vk::DescriptorSetLayoutBinding::builder()
		.binding(0)
		.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
		.descriptor_count(1)
		.stage_flags(vk::ShaderStageFlags::VERTEX);
	let layout_bindings = [layout_binding.build()];
	
	let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
		.bindings(&layout_bindings);
	
	let model_matrix_layout = unsafe { device.create_descriptor_set_layout(&create_info, None).unwrap() };

	let layouts = [projection_matrix_layout, model_matrix_layout];

	let allocate_info = vk::DescriptorSetAllocateInfo::builder()
		.descriptor_pool(*pool)
		.set_layouts(&layouts);
	
	let sets = unsafe { device.allocate_descriptor_sets(&allocate_info).unwrap() };

	let projection_matrix_descriptor_set = DescriptorSet {
		handle: sets[0],
		layout: projection_matrix_layout
	};

	let model_matrix_descriptor_set = DescriptorSet {
		handle: sets[1],
		layout: model_matrix_layout
	};

	(projection_matrix_descriptor_set, model_matrix_descriptor_set)
}

pub fn create_pipeline(
	device: &ash::Device,
	extent: &vk::Extent2D,
	render_pass: &vk::RenderPass,
	descriptor_set_layouts: &[vk::DescriptorSetLayout]) -> Pipeline
{
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
			
	let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
		.set_layouts(descriptor_set_layouts);

	let pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_create_info, None).unwrap() };

	let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
		.stages(&stages)
		.vertex_input_state(&vert_input_state_create_info)
		.input_assembly_state(&input_assembly_state_create_info)
		.viewport_state(&viewport_state_create_info)
		.rasterization_state(&rasterization_state_create_info)
		.multisample_state(&multisample_state_create_info)
		.color_blend_state(&color_blend_state_create_info)
		.layout(pipeline_layout)
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
		layout: pipeline_layout
	}
}

pub fn create_framebuffers(device: &ash::Device, image_views: &Vec<vk::ImageView>, extent: &vk::Extent2D, render_pass: &vk::RenderPass) -> Vec<vk::Framebuffer> {
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

pub fn create_command_pool(device: &ash::Device, graphics_queue_family: u32) -> vk::CommandPool {
	let create_info = vk::CommandPoolCreateInfo::builder()
		.queue_family_index(graphics_queue_family)
		.flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

	unsafe { device.create_command_pool(&create_info, None).unwrap() }
}

pub fn create_command_buffers(device: &ash::Device, command_pool: &vk::CommandPool, count: u32) -> Vec<vk::CommandBuffer> {
	let create_info = vk::CommandBufferAllocateInfo::builder()
		.command_pool(*command_pool)
		.level(vk::CommandBufferLevel::PRIMARY)
		.command_buffer_count(count);
	
	unsafe { device.allocate_command_buffers(&create_info).unwrap() }
}

pub fn create_in_flight_frames(device: &ash::Device, count: usize) -> Vec<InFlightFrame> {
	let semaphore_create_info = vk::SemaphoreCreateInfo::builder();
	let fence_create_info = vk::FenceCreateInfo::builder()
		.flags(vk::FenceCreateFlags::SIGNALED);

	let mut frames: Vec<InFlightFrame> = Vec::with_capacity(count);
	for _ in 0..count {
		frames.push(InFlightFrame {
			image_available: unsafe { device.create_semaphore(&semaphore_create_info, None).unwrap() },
			render_finished: unsafe { device.create_semaphore(&semaphore_create_info, None).unwrap() },
			fence: unsafe { device.create_fence(&fence_create_info, None).unwrap() }
		});
	}

	frames
}

pub fn create_buffer(
	instance: &ash::Instance,
	physical_device: &vk::PhysicalDevice,
	logical_device: &ash::Device,
	size: vk::DeviceSize,
	usage: vk::BufferUsageFlags,
	properties: vk::MemoryPropertyFlags) -> Buffer
{
	let create_info = vk::BufferCreateInfo::builder()
		.size(size)
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