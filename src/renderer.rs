use ash::{vk, version::EntryV1_0, version::InstanceV1_0, version::DeviceV1_0, extensions::ext, extensions::khr, vk::Handle};
use glfw::Context;
use std::ffi::{CString, CStr};
use std::os::raw::{c_void, c_char};

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct Renderer {
	entry: ash::Entry,
	instance: ash::Instance,
	debug_utils: DebugUtils,
	surface: Surface,
	physical_device: vk::PhysicalDevice,
	logical_device: ash::Device,
	graphics_queue_family: QueueFamily,
	present_queue_family: QueueFamily,
	swapchain: Swapchain,
	render_pass: vk::RenderPass,
	pipeline: Pipeline,
	framebuffers: Vec<vk::Framebuffer>,
	command_pool: vk::CommandPool,
	command_buffers: Vec<vk::CommandBuffer>,
	image_available_semaphores: Vec<vk::Semaphore>,
	render_finished_semaphores: Vec<vk::Semaphore>,
	parallel_frame_fences: Vec<vk::Fence>,
	swapchain_frame_fences: Vec<vk::Fence>,
	current_frame: usize
}

struct DebugUtils {
	extension: ext::DebugUtils,
	messenger_handle: vk::DebugUtilsMessengerEXT
}

struct Surface {
	extension: khr::Surface,
	handle: vk::SurfaceKHR
}

struct QueueFamily {
	index: u32,
	queue: vk::Queue
}

struct Swapchain {
	extension: khr::Swapchain,
	handle: vk::SwapchainKHR,
	format: vk::SurfaceFormatKHR,
	extent: vk::Extent2D,
	images: Vec<vk::Image>,
	image_views: Vec<vk::ImageView>
}

struct Pipeline {
	handle: vk::Pipeline,
	layout: vk::PipelineLayout
}

impl Renderer {
	pub fn new(window: &glfw::Window) -> Self {
		let entry = ash::Entry::new().unwrap();

		let required_layers = [CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
		let required_layers_c_str: Vec<&CStr> = required_layers.iter().map(|layer| layer.as_c_str()).collect();

		let required_instance_extensions = [
			CString::new("VK_KHR_surface").unwrap(),
			CString::new("VK_KHR_win32_surface").unwrap(),
			CString::new("VK_EXT_debug_utils").unwrap()
		];
		let required_instance_extensions_c_str: Vec<&CStr> = required_instance_extensions.iter().map(|extension| extension.as_c_str()).collect();

		let required_device_extensions = [CString::new("VK_KHR_swapchain").unwrap()];
		let required_device_extensions_c_str: Vec<&CStr> = required_device_extensions.iter().map(|extension| extension.as_c_str()).collect();
		
		let instance = Self::create_instance(&entry, &required_layers_c_str, &required_instance_extensions_c_str);
		let debug_utils = Self::create_debug_utils(&entry, &instance);
		let surface = Self::create_surface(&entry, &instance, window);
		
		let (physical_device, graphics_queue_family, present_queue_family) = Self::choose_physical_device(&instance,
			&required_device_extensions_c_str,
			&surface);
		
		let (logical_device, graphics_queue_handle, present_queue_handle) = Self::create_logical_device(&instance,
			&required_layers_c_str,
			&required_device_extensions_c_str,
			&physical_device,
			graphics_queue_family,
			present_queue_family);
		
		let swapchain = Self::create_swapchain(&instance,
			&physical_device,
			&logical_device,
			&surface,
			graphics_queue_family,
			present_queue_family);
		
		let render_pass = Self::create_render_pass(&logical_device, &swapchain.format);
		let pipeline = Self::create_pipeline(&logical_device, &swapchain.extent, &render_pass);
		let framebuffers = Self::create_framebuffers(&logical_device, &swapchain.image_views, &swapchain.extent, &render_pass);
		let command_pool = Self::create_command_pool(&logical_device, graphics_queue_family);

		let command_buffers = Self::create_command_buffers(&logical_device,
			framebuffers.len() as u32,
			&command_pool,
			&render_pass,
			&framebuffers,
			&swapchain.extent,
			&pipeline.handle);
		
		let (image_available_semaphores,
			render_finished_semaphores,
			parallel_frame_fences,
			swapchain_frame_fences) = Self::create_sync_objects(&logical_device, swapchain.images.len());
		
		Renderer {
			entry,
			instance,
			debug_utils,
			surface,
			physical_device,
			logical_device,
			graphics_queue_family: QueueFamily {
				index: graphics_queue_family,
				queue: graphics_queue_handle
			},
			present_queue_family: QueueFamily {
				index: present_queue_family,
				queue: present_queue_handle
			},
			swapchain,
			render_pass,
			pipeline,
			framebuffers,
			command_pool,
			command_buffers,
			image_available_semaphores,
			render_finished_semaphores,
			parallel_frame_fences,
			swapchain_frame_fences,
			current_frame: 0
		}
	}

	fn create_instance(entry: &ash::Entry, required_layers: &Vec<&CStr>, required_extensions: &Vec<&CStr>) -> ash::Instance {
		let engine_name = CString::new("Vulkan Engine").unwrap();
		let app_info = vk::ApplicationInfo::builder()
			.engine_name(engine_name.as_c_str())
			.engine_version(ash::vk_make_version!(1, 0, 0))
			.api_version(ash::vk_make_version!(1, 2, 0));
		
		let mut required_layers_ffi = Vec::with_capacity(required_layers.len());
		let available_layers = entry.enumerate_instance_layer_properties().unwrap();

		for req_lay in required_layers {
			available_layers.iter()
				.find(|&&avail_lay| &unsafe { CStr::from_ptr(avail_lay.layer_name.as_ptr()) } == req_lay)
				.expect(&format!("Required layer {} not supported", req_lay.to_str().unwrap()));
			required_layers_ffi.push(req_lay.as_ptr());
		}
		
		let mut required_extensions_ffi = Vec::with_capacity(required_extensions.len());
		let available_extensions = entry.enumerate_instance_extension_properties().unwrap();

		for req_ext in required_extensions {
			available_extensions.iter()
				.find(|&&avail_ext| &unsafe { CStr::from_ptr(avail_ext.extension_name.as_ptr()) } == req_ext)
				.expect(&format!("Required extension {} not supported", req_ext.to_str().unwrap()));
			required_extensions_ffi.push(req_ext.as_ptr());
		}

		let mut debug_messenger_create_info = Self::create_debug_messenger_create_info();

		let create_info = vk::InstanceCreateInfo::builder()
			.application_info(&app_info)
			.enabled_extension_names(&required_extensions_ffi)
			.enabled_layer_names(&required_layers_ffi)
			.push_next(&mut debug_messenger_create_info);
		
		unsafe { entry.create_instance(&create_info, None).unwrap() }
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

	fn create_debug_utils(entry: &ash::Entry, instance: &ash::Instance) -> DebugUtils {
		let create_info = Self::create_debug_messenger_create_info();
		let extension = ext::DebugUtils::new(entry, instance);
		let messenger_handle = unsafe { extension.create_debug_utils_messenger(&create_info, None).unwrap() };
		
		DebugUtils {
			extension,
			messenger_handle
		}
	}

	fn create_surface(entry: &ash::Entry, instance: &ash::Instance, window: &glfw::Window) -> Surface {
		let instance_raw = instance.handle().as_raw();
		let mut surface_raw: u64 = 0;
		
		let result = unsafe { glfw::ffi::glfwCreateWindowSurface(instance_raw as usize, window.window_ptr(), std::ptr::null(), &mut surface_raw as *mut u64) };
		if result != 0 {
			panic!("Couldn't create window surface");
		}
		
		let extension = khr::Surface::new(entry, instance);
		let handle = vk::SurfaceKHR::from_raw(surface_raw);

		Surface {
			extension,
			handle
		}
	}

	fn choose_physical_device(instance: &ash::Instance, required_extensions: &Vec<&CStr>, surface: &Surface)
		-> (vk::PhysicalDevice, u32, u32)
	{
		let devices = unsafe { instance.enumerate_physical_devices().unwrap() };

		for device in devices {
			let device_properties = unsafe { instance.get_physical_device_properties(device) };
			let device_features = unsafe { instance.get_physical_device_features(device) };

			if device_properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU && device_features.geometry_shader == vk::TRUE {
				let queue_family_properties = unsafe { instance.get_physical_device_queue_family_properties(device) };
				let mut graphics_queue_family = None;
				let mut present_queue_family = None;
				for (i, property) in queue_family_properties.iter().enumerate() {
					if property.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
						graphics_queue_family = Some(i);
					}

					if unsafe { surface.extension.get_physical_device_surface_support(device, i as u32, surface.handle) } {
						present_queue_family = Some(i);
					}
				}

				//should this go first in this loop?
				let extension_properties = unsafe { instance.enumerate_device_extension_properties(device).unwrap() };
				for req_ext in required_extensions {
					if extension_properties.iter().find(|ext| &unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) } == req_ext).is_none() {
						continue;
					}
				}

				let formats = unsafe { surface.extension.get_physical_device_surface_formats(device, surface.handle).unwrap() };
				let present_modes = unsafe { surface.extension.get_physical_device_surface_present_modes(device, surface.handle).unwrap() };

				if graphics_queue_family.is_some() && present_queue_family.is_some() && formats.len() != 0 && present_modes.len() != 0 {
					return (device, graphics_queue_family.unwrap() as u32, present_queue_family.unwrap() as u32)
				}
			}
		}

		panic!("No suitable physical device found");
	}

	fn create_logical_device(
		instance: &ash::Instance,
		required_layers: &Vec<&CStr>,
		required_extensions: &Vec<&CStr>,
		physical_device: &vk::PhysicalDevice,
		graphics_queue_family_index: u32,
		present_queue_family_index: u32) -> (ash::Device, vk::Queue, vk::Queue)
	{
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

		let required_layers_ffi: Vec<*const c_char> = required_layers.iter().map(|layer| layer.as_ptr()).collect();
		let required_extensions_ffi: Vec<*const c_char> = required_extensions.iter().map(|extension| extension.as_ptr()).collect();
		let features = vk::PhysicalDeviceFeatures::builder();

		let device_create_info = vk::DeviceCreateInfo::builder()
			.queue_create_infos(&device_queue_create_infos)
			.enabled_features(&features)
			.enabled_layer_names(&required_layers_ffi)
			.enabled_extension_names(&required_extensions_ffi);
		
		let logical_device = unsafe { instance.create_device(*physical_device, &device_create_info, None).unwrap() };
		let graphics_queue = unsafe { logical_device.get_device_queue(graphics_queue_family_index, 0) };
		let present_queue = unsafe { logical_device.get_device_queue(present_queue_family_index, 0) };

		(logical_device, graphics_queue, present_queue)
	}

	fn create_swapchain(instance: &ash::Instance,
		physical_device: &vk::PhysicalDevice,
		logical_device: &ash::Device,
		surface: &Surface,
		graphics_queue_family: u32,
		present_queue_family: u32) -> Swapchain
	{
		let formats = unsafe { surface.extension.get_physical_device_surface_formats(*physical_device, surface.handle).unwrap() };
		let format = formats.iter().find(|f| f.format == vk::Format::B8G8R8A8_SRGB && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR);
		let format = if format.is_some() { *format.unwrap() } else { formats[0] };

		let present_modes = unsafe { surface.extension.get_physical_device_surface_present_modes(*physical_device, surface.handle).unwrap() };
		let present_mode = present_modes.iter().find(|&&m| m == vk::PresentModeKHR::MAILBOX);
		let present_mode = if present_mode.is_some() { *present_mode.unwrap() } else { present_modes[0] };

		let capabilities = unsafe { surface.extension.get_physical_device_surface_capabilities(*physical_device, surface.handle).unwrap() };
		let mut extent = capabilities.current_extent;
		if capabilities.current_extent.width == std::u32::MAX {
			let actual_extent_width = 300;
			let actual_extent_height = 300;
			extent = vk::Extent2D::builder()
				.width(std::cmp::max(capabilities.current_extent.width, std::cmp::min(capabilities.current_extent.width, actual_extent_width)))
				.height(std::cmp::max(capabilities.current_extent.height, std::cmp::min(capabilities.current_extent.height, actual_extent_height)))
				.build();
		}

		let mut image_count = capabilities.min_image_count + 1;
		if capabilities.max_image_count > 0 && image_count > capabilities.max_image_count {
			image_count = capabilities.max_image_count;
		}

		let mut swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
			.surface(surface.handle)
			.min_image_count(image_count)
			.image_format(format.format)
			.image_color_space(format.color_space)
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
				.format(format.format)
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

		Swapchain {
			extension,
			handle,
			format,
			extent,
			images,
			image_views
		}
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

		let vert_input_state_create_info = vk::PipelineVertexInputStateCreateInfo::builder();

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

		for image_view in image_views {
			let attachments = [*image_view];

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
			.queue_family_index(graphics_queue_family);

		unsafe { device.create_command_pool(&create_info, None).unwrap() }
	}

	fn create_command_buffers(device: &ash::Device,
		count: u32,
		command_pool: &vk::CommandPool,
		render_pass: &vk::RenderPass,
		frame_buffers: &Vec<vk::Framebuffer>,
		extent: &vk::Extent2D,
		pipeline: &vk::Pipeline) -> Vec<vk::CommandBuffer>
	{
		let create_info = vk::CommandBufferAllocateInfo::builder()
			.command_pool(*command_pool)
			.level(vk::CommandBufferLevel::PRIMARY)
			.command_buffer_count(count);
		
		let command_buffers = unsafe { device.allocate_command_buffers(&create_info).unwrap() };
		
		for (i, command_buffer) in command_buffers.iter().enumerate() {
			let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder();

			unsafe { device.begin_command_buffer(*command_buffer, &command_buffer_begin_info).unwrap() };

			let clear_color = vk::ClearValue {
				color: vk::ClearColorValue {
					float32: [0.0, 0.0, 0.0, 1.0]
				}
			};
			let clear_colors = [clear_color];

			let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
				.render_pass(*render_pass)
				.framebuffer(frame_buffers[i])
				.render_area(vk::Rect2D::builder()
					.offset(vk::Offset2D::builder().x(0).y(0).build())
					.extent(*extent)
					.build())
				.clear_values(&clear_colors);
			
			unsafe {
				device.cmd_begin_render_pass(*command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
				device.cmd_bind_pipeline(*command_buffer, vk::PipelineBindPoint::GRAPHICS, *pipeline);
				device.cmd_draw(*command_buffer, 3, 1, 0, 0);
				device.cmd_end_render_pass(*command_buffer);

				device.end_command_buffer(*command_buffer).unwrap();
			}
		}

		command_buffers
	}

	fn create_sync_objects(device: &ash::Device, swapchain_image_count: usize) -> (Vec<vk::Semaphore>, Vec<vk::Semaphore>, Vec<vk::Fence>, Vec<vk::Fence>) {
		let semaphore_create_info = vk::SemaphoreCreateInfo::builder();
		let fence_create_info = vk::FenceCreateInfo::builder()
			.flags(vk::FenceCreateFlags::SIGNALED);

		let mut image_available_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
		let mut render_finished_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
		let mut parallel_frame_fences = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);

		for _ in 0..MAX_FRAMES_IN_FLIGHT {
			image_available_semaphores.push(unsafe { device.create_semaphore(&semaphore_create_info, None).unwrap() });
			render_finished_semaphores.push(unsafe { device.create_semaphore(&semaphore_create_info, None).unwrap() });
			parallel_frame_fences.push(unsafe { device.create_fence(&fence_create_info, None).unwrap() });
		}

		let mut swapchain_frame_fences = Vec::with_capacity(swapchain_image_count);

		for _ in 0..swapchain_image_count {
			swapchain_frame_fences.push(vk::Fence::null());
		}

		(image_available_semaphores, render_finished_semaphores, parallel_frame_fences, swapchain_frame_fences)
	}

	pub fn render(&mut self) {
		let current_frame_fence = self.parallel_frame_fences[self.current_frame];
		let current_frame_fences = [current_frame_fence];

		unsafe { self.logical_device.wait_for_fences(&current_frame_fences, true, std::u64::MAX).unwrap() };
				
		let current_image_available_semaphore = self.image_available_semaphores[self.current_frame];

		let image_index = unsafe {
			self.swapchain.extension.acquire_next_image(self.swapchain.handle,
				std::u64::MAX,
				current_image_available_semaphore,
				vk::Fence::null()
			).unwrap().0
		};

		let swapchain_frame_fence = self.swapchain_frame_fences[image_index as usize];

		if swapchain_frame_fence != vk::Fence::null() {
			let swapchain_frame_fences = [swapchain_frame_fence];
			unsafe { self.logical_device.wait_for_fences(&swapchain_frame_fences, true, std::u64::MAX).unwrap() };
		}

		self.swapchain_frame_fences[image_index as usize] = current_frame_fence;
		
		let current_render_finished_semaphore = self.render_finished_semaphores[self.current_frame];
		let wait_semaphores = [current_image_available_semaphore];
		let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
		let command_buffers = [self.command_buffers[image_index as usize]];
		let signal_semaphores = [current_render_finished_semaphore];

		let submit_info = vk::SubmitInfo::builder()
			.wait_semaphores(&wait_semaphores)
			.wait_dst_stage_mask(&wait_stages)
			.command_buffers(&command_buffers)
			.signal_semaphores(&signal_semaphores);
		let submit_infos = [submit_info.build()];

		unsafe {
			self.logical_device.reset_fences(&current_frame_fences).unwrap();
			self.logical_device.queue_submit(self.graphics_queue_family.queue, &submit_infos, current_frame_fence).unwrap();
		}

		let swapchains = [self.swapchain.handle];
		let image_indices = [image_index];

		let present_info = vk::PresentInfoKHR::builder()
			.wait_semaphores(&signal_semaphores)
			.swapchains(&swapchains)
			.image_indices(&image_indices);
		
		unsafe { self.swapchain.extension.queue_present(self.graphics_queue_family.queue, &present_info).unwrap() };

		self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
	}
}

impl Drop for Renderer {
	fn drop(&mut self) {
		unsafe {
			self.logical_device.device_wait_idle().unwrap();

			for i in 0..MAX_FRAMES_IN_FLIGHT {
				self.logical_device.destroy_fence(self.parallel_frame_fences[i], None);
				self.logical_device.destroy_semaphore(self.render_finished_semaphores[i], None);
				self.logical_device.destroy_semaphore(self.image_available_semaphores[i], None);
			}

			self.logical_device.destroy_command_pool(self.command_pool, None);

			for framebuffer in &self.framebuffers {
				self.logical_device.destroy_framebuffer(*framebuffer, None);
			}

			self.logical_device.destroy_pipeline(self.pipeline.handle, None);
			self.logical_device.destroy_pipeline_layout(self.pipeline.layout, None);
			self.logical_device.destroy_render_pass(self.render_pass, None);

			for image_view in &self.swapchain.image_views {
				self.logical_device.destroy_image_view(*image_view, None);
			}
			
			self.swapchain.extension.destroy_swapchain(self.swapchain.handle, None);
			self.logical_device.destroy_device(None);
			self.surface.extension.destroy_surface(self.surface.handle, None);
			self.debug_utils.extension.destroy_debug_utils_messenger(self.debug_utils.messenger_handle, None);
			self.instance.destroy_instance(None);
		}
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