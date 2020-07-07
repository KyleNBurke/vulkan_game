use ash::{vk, version::DeviceV1_0, version::InstanceV1_0, extensions::khr};
use crate::{
	vulkan::{Context, Buffer},
	mesh, Mesh,
	Camera,
	math::Vector3,
	lights::{AmbientLight, PointLight}
};
use std::{mem, mem::size_of, ffi::CStr, ffi::CString, ptr};

const IN_FLIGHT_FRAMES_COUNT: usize = 2;
const FRAME_DATA_MEMORY_SIZE: usize = 76 * size_of::<f32>();
const MAX_POINT_LIGHTS: usize = 5;

pub struct Renderer<'a> {
	context: &'a Context,
	render_pass: vk::RenderPass,
	swapchain: Swapchain,
	shared_pipeline_resources: SharedPipelineResources,
	basic_pipeline: vk::Pipeline,
	lambert_pipeline: vk::Pipeline,
	descriptor_pool: vk::DescriptorPool,
	command_pool: vk::CommandPool,
	in_flight_frames: [InFlightFrame<'a>; IN_FLIGHT_FRAMES_COUNT],
	current_in_flight_frame: usize,
	static_mesh_resources: StaticMeshResouces<'a>
}

struct Swapchain {
	extension: khr::Swapchain,
	handle: vk::SwapchainKHR,
	extent: vk::Extent2D,
	depth_image_resources: DepthImageResources,
	frames: Vec<SwapchainFrame>
}

struct DepthImageResources {
	image: vk::Image,
	image_view: vk::ImageView,
	memory: vk::DeviceMemory
}

struct SwapchainFrame {
	image_view: vk::ImageView,
	framebuffer: vk::Framebuffer,
	fence: vk::Fence
}

struct SharedPipelineResources {
	static_descriptor_set_layout: vk::DescriptorSetLayout,
	dynamic_descriptor_set_layout: vk::DescriptorSetLayout,
	pipeline_layout: vk::PipelineLayout
}

struct InFlightFrame<'a> {
	image_available: vk::Semaphore,
	render_finished: vk::Semaphore,
	fence: vk::Fence,
	primary_command_buffer: vk::CommandBuffer,
	basic_secondary_command_buffer: vk::CommandBuffer,
	lambert_secondary_command_buffer: vk::CommandBuffer,
	buffer: Buffer<'a>,
	frame_data_descriptor_set: vk::DescriptorSet,
	model_matrix_descriptor_set: vk::DescriptorSet
}

struct StaticMeshResouces<'a> {
	buffer: Buffer<'a>,
	model_matrix_descriptor_set: vk::DescriptorSet,
	render_info: Vec<RenderInfo>
}

struct RenderInfo {
	base_offset: usize,
	index_count: usize,
	index_size: usize,
	index_padding_size: usize,
	attribute_size: usize,
	attribute_padding_size: usize,
	material: mesh::Material
}

impl<'a> Renderer<'a> {
	pub fn new(context: &'a Context, width: u32, height: u32) -> Self {
		let render_pass = Self::create_render_pass(&context);
		let swapchain = Self::create_swapchain(&context, width, height, &render_pass);
		let shared_pipeline_resources = Self::create_shared_pipeline_resouces(context);
		let (basic_pipeline, lambert_pipeline) = Self::create_pipelines(context, swapchain.extent, &shared_pipeline_resources.pipeline_layout, &render_pass);
		let descriptor_pool = Self::create_descriptor_pool(&context);
		let command_pool = Self::create_command_pool(&context);
		let in_flight_frames = Self::create_in_flight_frames(&context, &descriptor_pool, &command_pool, &shared_pipeline_resources.static_descriptor_set_layout, &shared_pipeline_resources.dynamic_descriptor_set_layout);
		let static_mesh_resources = Self::create_static_mesh_resources(&context, &descriptor_pool, &shared_pipeline_resources.dynamic_descriptor_set_layout);

		Self {
			context,
			render_pass,
			command_pool,
			swapchain,
			shared_pipeline_resources,
			basic_pipeline,
			lambert_pipeline,
			descriptor_pool,
			in_flight_frames,
			current_in_flight_frame: 0,
			static_mesh_resources
		}
	}

	fn create_render_pass(context: &Context) -> vk::RenderPass {
		let color_attachment_description = vk::AttachmentDescription::builder()
			.format(context.surface.format.format)
			.samples(vk::SampleCountFlags::TYPE_1)
			.load_op(vk::AttachmentLoadOp::CLEAR)
			.store_op(vk::AttachmentStoreOp::STORE)
			.stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
			.stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
			.initial_layout(vk::ImageLayout::UNDEFINED)
			.final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

		let depth_attachment_description = vk::AttachmentDescription::builder()
			.format(vk::Format::D32_SFLOAT)
			.samples(vk::SampleCountFlags::TYPE_1)
			.load_op(vk::AttachmentLoadOp::CLEAR)
			.store_op(vk::AttachmentStoreOp::DONT_CARE)
			.stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
			.stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
			.initial_layout(vk::ImageLayout::UNDEFINED)
			.final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

		let attachment_descriptions = [color_attachment_description.build(), depth_attachment_description.build()];
		
		let color_attachment_ref = vk::AttachmentReference::builder()
			.attachment(0)
			.layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
		let color_attachment_refs = [color_attachment_ref.build()];

		let depth_attachment_ref = vk::AttachmentReference::builder()
			.attachment(1)
			.layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
		
		let subpass_description = vk::SubpassDescription::builder()
			.pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
			.color_attachments(&color_attachment_refs)
			.depth_stencil_attachment(&depth_attachment_ref);
		let subpass_descriptions = [subpass_description.build()];

		let subpass_dependency = vk::SubpassDependency::builder()
			.src_subpass(vk::SUBPASS_EXTERNAL)
			.dst_subpass(0)
			.src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
			.src_access_mask(vk::AccessFlags::empty())
			.dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
			.dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE);
		let subpass_dependencies = [subpass_dependency.build()];
		
		let render_pass_create_info = vk::RenderPassCreateInfo::builder()
			.attachments(&attachment_descriptions)
			.subpasses(&subpass_descriptions)
			.dependencies(&subpass_dependencies);
		
		unsafe { context.logical_device.create_render_pass(&render_pass_create_info, None).unwrap() }
	}

	fn create_swapchain(context: &Context, width: u32, height: u32, render_pass: &vk::RenderPass) -> Swapchain {
		// Get present mode
		let present_modes = unsafe { context.surface.extension.get_physical_device_surface_present_modes(context.physical_device.handle, context.surface.handle).unwrap() };
		let present_mode_option = present_modes.iter().find(|&&m| m == vk::PresentModeKHR::MAILBOX);
		let present_mode = *present_mode_option.unwrap_or_else(|| &present_modes[0]);

		// Create extent
		let capabilities = unsafe { context.surface.extension.get_physical_device_surface_capabilities(context.physical_device.handle, context.surface.handle).unwrap() };
		let extent = if capabilities.current_extent.width == std::u32::MAX {
			vk::Extent2D::builder()
				.width(std::cmp::max(capabilities.current_extent.width, std::cmp::min(capabilities.current_extent.width, width)))
				.height(std::cmp::max(capabilities.current_extent.height, std::cmp::min(capabilities.current_extent.height, height)))
				.build()
		}
		else {
			capabilities.current_extent
		};

		// Create swapchain extension, handle & images
		let mut image_count = capabilities.min_image_count + 1;
		if capabilities.max_image_count > 0 && image_count > capabilities.max_image_count {
			image_count = capabilities.max_image_count;
		}

		let mut swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
			.surface(context.surface.handle)
			.min_image_count(image_count)
			.image_format(context.surface.format.format)
			.image_color_space(context.surface.format.color_space)
			.image_extent(extent)
			.image_array_layers(1)
			.image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
			.pre_transform(capabilities.current_transform)
			.composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
			.present_mode(present_mode)
			.clipped(true);
		
		let graphics_queue_family_index = context.physical_device.graphics_queue_family;
		let present_queue_family_index = context.physical_device.present_queue_family;
		let queue_families = [graphics_queue_family_index, present_queue_family_index];
		if graphics_queue_family_index == present_queue_family_index {
			swapchain_create_info = swapchain_create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE);
		}
		else {
			swapchain_create_info = swapchain_create_info
				.image_sharing_mode(vk::SharingMode::CONCURRENT)
				.queue_family_indices(&queue_families);
		}

		let extension = khr::Swapchain::new(&context.instance, &context.logical_device);
		let handle = unsafe { extension.create_swapchain(&swapchain_create_info, None).unwrap() };
		let images = unsafe { extension.get_swapchain_images(handle).unwrap() };

		// Ensure D32_SFLOAT format is supported for depth buffering
		let required_format = vk::Format::D32_SFLOAT;
		let format_properties = unsafe { context.instance.get_physical_device_format_properties(context.physical_device.handle, required_format) };
		let required_format_feature = vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT;
		if format_properties.optimal_tiling_features & required_format_feature != required_format_feature {
			panic!("Required format for depth buffering not supported");
		}

		// Create depth image
		let image_create_info = vk::ImageCreateInfo::builder()
			.image_type(vk::ImageType::TYPE_2D)
			.extent(vk::Extent3D::builder()
				.width(extent.width)
				.height(extent.height)
				.depth(1)
				.build())
			.mip_levels(1)
			.array_layers(1)
			.format(required_format)
			.tiling(vk::ImageTiling::OPTIMAL)
			.initial_layout(vk::ImageLayout::UNDEFINED)
			.usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
			.samples(vk::SampleCountFlags::TYPE_1)
			.sharing_mode(vk::SharingMode::EXCLUSIVE);
	
		let depth_image = unsafe { context.logical_device.create_image(&image_create_info, None).unwrap() };

		// Allocate depth image memory and bind it to the image
		let memory_requirements = unsafe { context.logical_device.get_image_memory_requirements(depth_image) };
		let memory_type_index = context.physical_device.find_memory_type_index(memory_requirements.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL);

		let allocate_info = vk::MemoryAllocateInfo::builder()
			.allocation_size(memory_requirements.size)
			.memory_type_index(memory_type_index as u32);

		let depth_image_memory = unsafe { context.logical_device.allocate_memory(&allocate_info, None).unwrap() };
		unsafe { context.logical_device.bind_image_memory(depth_image, depth_image_memory, 0).unwrap() };

		// Create depth image view
		let image_view_create_info = vk::ImageViewCreateInfo::builder()
			.image(depth_image)
			.view_type(vk::ImageViewType::TYPE_2D)
			.format(required_format)
			.subresource_range(vk::ImageSubresourceRange::builder()
				.aspect_mask(vk::ImageAspectFlags::DEPTH)
				.base_mip_level(0)
				.level_count(1)
				.base_array_layer(0)
				.layer_count(1)
				.build());
		
		let depth_image_view = unsafe { context.logical_device.create_image_view(&image_view_create_info, None).unwrap() };
		
		// Create the container struct
		let depth_image_resources = DepthImageResources {
			image: depth_image,
			image_view: depth_image_view,
			memory: depth_image_memory
		};

		// Create swapchain frames
		let mut frames = Vec::with_capacity(images.len());
		for i in 0..images.len() {
			// Create image view
			let image_view_create_info = vk::ImageViewCreateInfo::builder()
				.image(images[i])
				.view_type(vk::ImageViewType::TYPE_2D)
				.format(context.surface.format.format)
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

			let image_view = unsafe { context.logical_device.create_image_view(&image_view_create_info, None).unwrap() };

			// Create framebuffer
			let attachments = [image_view, depth_image_view];

			let create_info = vk::FramebufferCreateInfo::builder()
				.render_pass(*render_pass)
				.attachments(&attachments)
				.width(extent.width)
				.height(extent.height)
				.layers(1);
			
			let framebuffer = unsafe { context.logical_device.create_framebuffer(&create_info, None).unwrap() };

			// Create fence
			let fence = vk::Fence::null();

			frames.push(SwapchainFrame {
				image_view,
				framebuffer,
				fence
			});
		}

		Swapchain {
			extension,
			handle,
			extent,
			depth_image_resources,
			frames
		}
	}

	fn create_shared_pipeline_resouces(context: &Context) -> SharedPipelineResources {
		// Create static descriptor set layout
		let static_binding = vk::DescriptorSetLayoutBinding::builder()
			.binding(0)
			.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
			.descriptor_count(1)
			.stage_flags(vk::ShaderStageFlags::VERTEX);
		let static_bindings = [static_binding.build()];

		let static_create_info = vk::DescriptorSetLayoutCreateInfo::builder()
			.bindings(&static_bindings);

		let static_descriptor_set_layout = unsafe { context.logical_device.create_descriptor_set_layout(&static_create_info, None).unwrap() };

		// Create dynamic descriptor set layout
		let dynamic_binding = vk::DescriptorSetLayoutBinding::builder()
			.binding(0)
			.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
			.descriptor_count(1)
			.stage_flags(vk::ShaderStageFlags::VERTEX);
		let dynamic_bindings = [dynamic_binding.build()];

		let dynamic_create_info = vk::DescriptorSetLayoutCreateInfo::builder()
			.bindings(&dynamic_bindings);

		let dynamic_descriptor_set_layout = unsafe { context.logical_device.create_descriptor_set_layout(&dynamic_create_info, None).unwrap() };

		// Create pipeline layout
		let descriptor_set_layouts = [static_descriptor_set_layout, dynamic_descriptor_set_layout];

		let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
			.set_layouts(&descriptor_set_layouts);

		let pipeline_layout = unsafe { context.logical_device.create_pipeline_layout(&pipeline_layout_create_info, None).unwrap() };

		SharedPipelineResources {
			static_descriptor_set_layout,
			dynamic_descriptor_set_layout,
			pipeline_layout
		}
	}

	fn create_pipelines(context: &Context, swapchain_extent: vk::Extent2D, pipeline_layout: &vk::PipelineLayout, render_pass: &vk::RenderPass) -> (vk::Pipeline, vk::Pipeline) {
		let shader_entry_point_cstring = CString::new("main").unwrap();
		let shader_entry_point_cstr = shader_entry_point_cstring.as_c_str();
		let (basic_shader_modules, basic_stages) = Self::create_pipeline_stages(context, "basic", shader_entry_point_cstr);
		let (lambert_shader_modules, lambert_stages) = Self::create_pipeline_stages(context, "lambert", shader_entry_point_cstr);

		let vert_input_binding_description = vk::VertexInputBindingDescription::builder()
			.binding(0)
			.stride(24)
			.input_rate(vk::VertexInputRate::VERTEX);
		let vert_input_binding_descriptions = [vert_input_binding_description.build()];
		
		let vert_input_attribute_description_position = vk::VertexInputAttributeDescription::builder()	
			.binding(0)
			.location(0)
			.format(vk::Format::R32G32B32_SFLOAT)
			.offset(0)
			.build();
		
		let vert_input_attribute_description_normal = vk::VertexInputAttributeDescription::builder()	
			.binding(0)
			.location(1)
			.format(vk::Format::R32G32B32_SFLOAT)
			.offset(12)
			.build();

		let basic_vert_input_attribute_descriptions = [vert_input_attribute_description_position];
		let lambert_vert_input_attribute_descriptions = [vert_input_attribute_description_position, vert_input_attribute_description_normal];

		let basic_vert_input_state_create_info = vk::PipelineVertexInputStateCreateInfo::builder()
			.vertex_binding_descriptions(&vert_input_binding_descriptions)
			.vertex_attribute_descriptions(&basic_vert_input_attribute_descriptions);

		let lambert_vert_input_state_create_info = vk::PipelineVertexInputStateCreateInfo::builder()
			.vertex_binding_descriptions(&vert_input_binding_descriptions)
			.vertex_attribute_descriptions(&lambert_vert_input_attribute_descriptions);
		
		let input_assembly_state_create_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
			.topology(vk::PrimitiveTopology::TRIANGLE_LIST)
			.primitive_restart_enable(false);

		let viewport = vk::Viewport::builder()
			.x(0.0)
			.y(0.0)
			.width(swapchain_extent.width as f32)
			.height(swapchain_extent.height as f32)
			.min_depth(0.0)
			.max_depth(1.0);
		let viewports = [viewport.build()];
		
		let scissor = vk::Rect2D::builder()
			.offset(vk::Offset2D::builder().x(0).y(0).build())
			.extent(swapchain_extent);
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
		
		let depth_stencil_state_create_info = vk::PipelineDepthStencilStateCreateInfo::builder()
			.depth_test_enable(true)
			.depth_write_enable(true)
			.depth_compare_op(vk::CompareOp::LESS)
			.depth_bounds_test_enable(false)
			.stencil_test_enable(false);
		
		let color_blend_attachment_state = vk::PipelineColorBlendAttachmentState::builder()
			.color_write_mask(vk::ColorComponentFlags::all())
			.blend_enable(false);
		let color_blend_attachment_states = [color_blend_attachment_state.build()];
		
		let color_blend_state_create_info = vk::PipelineColorBlendStateCreateInfo::builder()
			.logic_op_enable(false)
			.attachments(&color_blend_attachment_states);

		let basic_pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
			.stages(&basic_stages)
			.vertex_input_state(&basic_vert_input_state_create_info)
			.input_assembly_state(&input_assembly_state_create_info)
			.viewport_state(&viewport_state_create_info)
			.rasterization_state(&rasterization_state_create_info)
			.multisample_state(&multisample_state_create_info)
			.depth_stencil_state(&depth_stencil_state_create_info)
			.color_blend_state(&color_blend_state_create_info)
			.layout(*pipeline_layout)
			.render_pass(*render_pass)
			.subpass(0);
		
		let lambert_pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
			.stages(&lambert_stages)
			.vertex_input_state(&lambert_vert_input_state_create_info)
			.input_assembly_state(&input_assembly_state_create_info)
			.viewport_state(&viewport_state_create_info)
			.rasterization_state(&rasterization_state_create_info)
			.multisample_state(&multisample_state_create_info)
			.depth_stencil_state(&depth_stencil_state_create_info)
			.color_blend_state(&color_blend_state_create_info)
			.layout(*pipeline_layout)
			.render_pass(*render_pass)
			.subpass(0);
		
		let pipeline_create_infos = [basic_pipeline_create_info.build(), lambert_pipeline_create_info.build()];
		let pipelines = unsafe { context.logical_device.create_graphics_pipelines(vk::PipelineCache::null(), &pipeline_create_infos, None).unwrap() };

		unsafe {
			context.logical_device.destroy_shader_module(basic_shader_modules[0], None);
			context.logical_device.destroy_shader_module(basic_shader_modules[1], None);
			context.logical_device.destroy_shader_module(lambert_shader_modules[0], None);
			context.logical_device.destroy_shader_module(lambert_shader_modules[1], None);
		}

		(pipelines[0], pipelines[1])
	}

	fn create_pipeline_stages(context: &Context, filename: &str, entry_point: &CStr) -> ([vk::ShaderModule; 2], [vk::PipelineShaderStageCreateInfo; 2]) {
		let mut curr_dir = std::env::current_exe().unwrap();
		curr_dir.pop();

		let mut vert_file = std::fs::File::open(curr_dir.join(filename.to_owned() + ".vert.spv").as_path()).unwrap();
		let mut frag_file = std::fs::File::open(curr_dir.join(filename.to_owned() + ".frag.spv").as_path()).unwrap();
		let vert_file_contents = ash::util::read_spv(&mut vert_file).unwrap();
		let frag_file_contents = ash::util::read_spv(&mut frag_file).unwrap();
		
		let vert_create_info = vk::ShaderModuleCreateInfo::builder().code(&vert_file_contents);
		let frag_create_info = vk::ShaderModuleCreateInfo::builder().code(&frag_file_contents);
		let vert_shader_module = unsafe { context.logical_device.create_shader_module(&vert_create_info, None).unwrap() };
		let frag_shader_module = unsafe { context.logical_device.create_shader_module(&frag_create_info, None).unwrap() };

		let vert_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
			.stage(vk::ShaderStageFlags::VERTEX)
			.module(vert_shader_module)
			.name(entry_point);

		let frag_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
			.stage(vk::ShaderStageFlags::FRAGMENT)
			.module(frag_shader_module)
			.name(entry_point);
		
		([vert_shader_module, frag_shader_module], [vert_stage_create_info.build(), frag_stage_create_info.build()])
	}

	fn create_descriptor_pool(context: &Context) -> vk::DescriptorPool {
		let max_frames_in_flight_u32 = IN_FLIGHT_FRAMES_COUNT as u32;

		let static_pool_size = vk::DescriptorPoolSize::builder()
			.ty(vk::DescriptorType::UNIFORM_BUFFER)
			.descriptor_count(max_frames_in_flight_u32);

		let dynamic_pool_size = vk::DescriptorPoolSize::builder()
			.ty(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
			.descriptor_count(max_frames_in_flight_u32 + 1);
		
		let pool_sizes = [static_pool_size.build(), dynamic_pool_size.build()];
		
		let create_info = vk::DescriptorPoolCreateInfo::builder()
			.pool_sizes(&pool_sizes)
			.max_sets(2 * max_frames_in_flight_u32 + 1);
		
		unsafe { context.logical_device.create_descriptor_pool(&create_info, None).unwrap() }
	}

	fn create_command_pool(context: &Context) -> vk::CommandPool {
		let create_info = vk::CommandPoolCreateInfo::builder()
			.queue_family_index(context.physical_device.graphics_queue_family)
			.flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

		unsafe { context.logical_device.create_command_pool(&create_info, None).unwrap() }
	}

	fn create_in_flight_frames(
		context: &Context,
		descriptor_pool: &vk::DescriptorPool,
		command_pool: &vk::CommandPool,
		static_descriptor_set_layout: &vk::DescriptorSetLayout,
		dynamic_descriptor_set_layout: &vk::DescriptorSetLayout) -> [InFlightFrame<'a>; IN_FLIGHT_FRAMES_COUNT]
	{
		let semaphore_create_info = vk::SemaphoreCreateInfo::builder();
		let fence_create_info = vk::FenceCreateInfo::builder()
			.flags(vk::FenceCreateFlags::SIGNALED);

		let primary_command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
			.command_pool(*command_pool)
			.level(vk::CommandBufferLevel::PRIMARY)
			.command_buffer_count(IN_FLIGHT_FRAMES_COUNT as u32);
		
		let primary_command_buffers = unsafe { context.logical_device.allocate_command_buffers(&primary_command_buffer_allocate_info).unwrap() };

		let secondary_command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
			.command_pool(*command_pool)
			.level(vk::CommandBufferLevel::SECONDARY)
			.command_buffer_count(IN_FLIGHT_FRAMES_COUNT as u32 * 2);
		
		let secondary_command_buffers = unsafe { context.logical_device.allocate_command_buffers(&secondary_command_buffer_allocate_info).unwrap() };

		let dynamic_descriptor_set_layouts = [*static_descriptor_set_layout, *dynamic_descriptor_set_layout];
		let dynamic_descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
			.descriptor_pool(*descriptor_pool)
			.set_layouts(&dynamic_descriptor_set_layouts);

		let mut frames: [mem::MaybeUninit<InFlightFrame>; IN_FLIGHT_FRAMES_COUNT] = unsafe { mem::MaybeUninit::uninit().assume_init() };
		for (i, frame) in frames.iter_mut().enumerate() {
			let image_available = unsafe { context.logical_device.create_semaphore(&semaphore_create_info, None).unwrap() };
			let render_finished = unsafe { context.logical_device.create_semaphore(&semaphore_create_info, None).unwrap() };
			let fence = unsafe { context.logical_device.create_fence(&fence_create_info, None).unwrap() };
			
			let buffer = Buffer::new(
				context,
				FRAME_DATA_MEMORY_SIZE as u64,
				vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::UNIFORM_BUFFER,
				vk::MemoryPropertyFlags::HOST_VISIBLE);
			
			let descriptor_sets = unsafe { context.logical_device.allocate_descriptor_sets(&dynamic_descriptor_set_allocate_info).unwrap() };
			let frame_data_descriptor_set = descriptor_sets[0];
			let model_matrix_descriptor_set = descriptor_sets[1];

			Self::update_in_flight_frame_descriptor_sets(
				&context.logical_device,
				&frame_data_descriptor_set,
				&model_matrix_descriptor_set,
				&buffer.handle);

			*frame = mem::MaybeUninit::new(InFlightFrame {
				image_available,
				render_finished,
				fence,
				primary_command_buffer: primary_command_buffers[i],
				basic_secondary_command_buffer: secondary_command_buffers[i * 2],
				lambert_secondary_command_buffer: secondary_command_buffers[i * 2 + 1],
				buffer,
				frame_data_descriptor_set,
				model_matrix_descriptor_set
			});
		}

		unsafe { mem::transmute::<_, [InFlightFrame; IN_FLIGHT_FRAMES_COUNT]>(frames) }
	}

	fn update_in_flight_frame_descriptor_sets(
		logical_device: &ash::Device,
		frame_data_descriptor_set: &vk::DescriptorSet,
		model_matrix_descriptor_set: &vk::DescriptorSet,
		buffer: &vk::Buffer)
	{
		// Frame data
		let frame_data_descriptor_buffer_info = vk::DescriptorBufferInfo::builder()
			.buffer(*buffer)
			.offset(0)
			.range(FRAME_DATA_MEMORY_SIZE as u64);
		let frame_data_descriptor_buffer_infos = [frame_data_descriptor_buffer_info.build()];

		let frame_data_write_descriptor_set = vk::WriteDescriptorSet::builder()
			.dst_set(*frame_data_descriptor_set)
			.dst_binding(0)
			.dst_array_element(0)
			.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
			.buffer_info(&frame_data_descriptor_buffer_infos);

		// Model
		let model_matrix_descriptor_buffer_info = vk::DescriptorBufferInfo::builder()
			.buffer(*buffer)
			.offset(0)
			.range(16 * size_of::<f32>() as u64);
		let model_matrix_descriptor_buffer_infos = [model_matrix_descriptor_buffer_info.build()];

		let model_matrix_write_descriptor_set = vk::WriteDescriptorSet::builder()
			.dst_set(*model_matrix_descriptor_set)
			.dst_binding(0)
			.dst_array_element(0)
			.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
			.buffer_info(&model_matrix_descriptor_buffer_infos);

		let write_descriptor_sets = [
			frame_data_write_descriptor_set.build(),
			model_matrix_write_descriptor_set.build()
		];
		let copy_descriptor_sets = [];

		unsafe { logical_device.update_descriptor_sets(&write_descriptor_sets, &copy_descriptor_sets) };
	}

	fn create_static_mesh_resources(context: &'a Context, descriptor_pool: &vk::DescriptorPool, dynamic_descriptor_set_layout: &vk::DescriptorSetLayout) -> StaticMeshResouces<'a> {
		let buffer = Buffer::null(
			context,
			vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::UNIFORM_BUFFER,
			vk::MemoryPropertyFlags::DEVICE_LOCAL);
		
		let descriptor_set_layout = [*dynamic_descriptor_set_layout];
		let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
			.descriptor_pool(*descriptor_pool)
			.set_layouts(&descriptor_set_layout);
		
		let model_matrix_descriptor_set = unsafe { context.logical_device.allocate_descriptor_sets(&descriptor_set_allocate_info).unwrap()[0] };
	
		StaticMeshResouces {
			buffer,
			model_matrix_descriptor_set,
			render_info: vec![]
		}
	}

	pub fn recreate_swapchain(&mut self, width: u32, height: u32) {
		let logical_device = &self.context.logical_device;

		unsafe {
			logical_device.device_wait_idle().unwrap();
			logical_device.destroy_pipeline(self.basic_pipeline, None);
			logical_device.destroy_pipeline(self.lambert_pipeline, None);

			for frame in &self.swapchain.frames {
				logical_device.destroy_framebuffer(frame.framebuffer, None);
				logical_device.destroy_image_view(frame.image_view, None);
			}

			logical_device.free_memory(self.swapchain.depth_image_resources.memory, None);
			logical_device.destroy_image_view(self.swapchain.depth_image_resources.image_view, None);
			logical_device.destroy_image(self.swapchain.depth_image_resources.image, None);
			self.swapchain.extension.destroy_swapchain(self.swapchain.handle, None);
		}

		self.swapchain = Self::create_swapchain(&self.context, width, height, &self.render_pass);
		let (basic_pipeline, lambert_pipeline) = Self::create_pipelines(self.context, self.swapchain.extent, &self.shared_pipeline_resources.pipeline_layout, &self.render_pass);
		self.basic_pipeline = basic_pipeline;
		self.lambert_pipeline = lambert_pipeline;

		println!("Swapchain recreated");
	}

	pub fn submit_static_meshes(&mut self, meshes: &[Mesh]) {
		let logical_device = &self.context.logical_device;

		// Wait for rendering operations to finish
		unsafe { logical_device.queue_wait_idle(self.context.graphics_queue).unwrap() };

		// Calculate total memory size and render info
		let mut total_size = 0;
		let uniform_alignment = self.context.physical_device.min_uniform_buffer_offset_alignment as usize;
		self.static_mesh_resources.render_info.clear();

		for mesh in meshes {
			let indices = mesh.geometry.get_vertex_indices();
			let attributes = mesh.geometry.get_vertex_attributes();
			let index_count = indices.len();
			let index_size = indices.len() * size_of::<u16>();
			let index_padding_size = size_of::<f32>() - (total_size + index_size) % size_of::<f32>();
			let attribute_size = attributes.len() * size_of::<f32>();
			let attribute_padding_size = uniform_alignment - (total_size + index_size + index_padding_size + attribute_size) % uniform_alignment;
			let uniform_size = 16 * size_of::<f32>();

			self.static_mesh_resources.render_info.push(RenderInfo {
				base_offset: total_size,
				index_count,
				index_size,
				index_padding_size,
				attribute_size,
				attribute_padding_size,
				material: mesh.material
			});

			total_size += index_size + index_padding_size + attribute_size + attribute_padding_size + uniform_size;
		}

		let total_size = total_size as u64;

		// Create a host visible staging buffer
		let staging_buffer = Buffer::new(
			self.context,
			total_size,
			vk::BufferUsageFlags::TRANSFER_SRC,
			vk::MemoryPropertyFlags::HOST_VISIBLE);
		
		// Copy mesh data into staging buffer
		let buffer_ptr = unsafe { logical_device.map_memory(staging_buffer.memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty()).unwrap() };

		for (i, mesh) in meshes.iter().enumerate() {
			let indices = mesh.geometry.get_vertex_indices();
			let attributes = mesh.geometry.get_vertex_attributes();
			let render_info = &self.static_mesh_resources.render_info[i];

			unsafe {
				let index_offset = render_info.base_offset;
				let index_dst_ptr = buffer_ptr.add(index_offset) as *mut u16;
				std::ptr::copy_nonoverlapping(indices.as_ptr(), index_dst_ptr, indices.len());

				let attribute_offset = index_offset + render_info.index_size + render_info.index_padding_size;
				let attribute_dst_ptr = buffer_ptr.add(attribute_offset) as *mut f32;
				std::ptr::copy_nonoverlapping(attributes.as_ptr(), attribute_dst_ptr, attributes.len());

				let model_matrix_offset = attribute_offset + render_info.attribute_size + render_info.attribute_padding_size;
				let model_matrix_dst_ptr = buffer_ptr.add(model_matrix_offset) as *mut [f32; 4];
				let model_matrix = &mesh.model_matrix.elements;
				std::ptr::copy_nonoverlapping(model_matrix.as_ptr(), model_matrix_dst_ptr, model_matrix.len());
			}
		}

		unsafe {
			let ranges = [vk::MappedMemoryRange::builder()
				.memory(staging_buffer.memory)
				.offset(0)
				.size(vk::WHOLE_SIZE)
				.build()];
			logical_device.flush_mapped_memory_ranges(&ranges).unwrap();
			logical_device.unmap_memory(staging_buffer.memory);
		}
		
		// Resize device local memory buffer if necessary
		if total_size > self.static_mesh_resources.buffer.capacity {
			self.static_mesh_resources.buffer.reallocate(total_size);
		}
		
		// Copy the data from the staging buffer into the device local buffer using a command buffer
		let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
			.level(vk::CommandBufferLevel::PRIMARY)
			.command_pool(self.command_pool)
			.command_buffer_count(1);

		let command_buffer = unsafe { logical_device.allocate_command_buffers(&command_buffer_allocate_info).unwrap()[0] };

		let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
			.flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
		
		let region = vk::BufferCopy::builder()
			.size(total_size);
		let regions = [region.build()];
		
		unsafe {
			logical_device.begin_command_buffer(command_buffer, &command_buffer_begin_info).unwrap();
			logical_device.cmd_copy_buffer(command_buffer, staging_buffer.handle, self.static_mesh_resources.buffer.handle, &regions);
			logical_device.end_command_buffer(command_buffer).unwrap();
		}

		let command_buffers = [command_buffer];
		let submit_info = vk::SubmitInfo::builder()
			.command_buffers(&command_buffers);
		let submit_infos = [submit_info.build()];
		
		unsafe {
			logical_device.queue_submit(self.context.graphics_queue, &submit_infos, vk::Fence::null()).unwrap();
			logical_device.queue_wait_idle(self.context.graphics_queue).unwrap();
			logical_device.free_command_buffers(self.command_pool, &command_buffers);
		}
		
		// Update the descriptor set to reference the device local buffer
		let model_matrix_buffer_info = vk::DescriptorBufferInfo::builder()
			.buffer(self.static_mesh_resources.buffer.handle)
			.offset(0)
			.range(16 * size_of::<f32>() as u64);
		let model_matrix_buffer_infos = [model_matrix_buffer_info.build()];

		let model_matrix_write_descriptor_set = vk::WriteDescriptorSet::builder()
			.dst_set(self.static_mesh_resources.model_matrix_descriptor_set)
			.dst_binding(0)
			.dst_array_element(0)
			.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
			.buffer_info(&model_matrix_buffer_infos);
		
		let write_descriptor_sets = [model_matrix_write_descriptor_set.build()];
		let copy_descriptor_sets = [];
		
		unsafe { logical_device.update_descriptor_sets(&write_descriptor_sets, &copy_descriptor_sets) };
	}

	pub fn render(&mut self, window: &glfw::Window, camera: &Camera, dynamic_meshes: &[Mesh], ambient_light: &AmbientLight, point_lights: &[PointLight]) {
		assert!(point_lights.len() <= MAX_POINT_LIGHTS, "Only {} point lights allowed", MAX_POINT_LIGHTS);

		let logical_device = &self.context.logical_device;
		let in_flight_frame = &mut self.in_flight_frames[self.current_in_flight_frame];
		
		// Wait for this in flight frame to become available
		let fences = [in_flight_frame.fence];
		unsafe { logical_device.wait_for_fences(&fences, true, std::u64::MAX).unwrap() };
		
		// Acquire a swapchain image to render to
		let result = unsafe {
			self.swapchain.extension.acquire_next_image(self.swapchain.handle,
				std::u64::MAX,
				in_flight_frame.image_available,
				vk::Fence::null())
		};

		if let Err(error) = result {
			if error == vk::Result::ERROR_OUT_OF_DATE_KHR {
				let (width, height) = window.get_framebuffer_size();
				self.recreate_swapchain(width as u32, height as u32);
				return;
			}

			panic!("Could not aquire a swapchain image");
		}

		let image_index = result.unwrap().0;
		let swapchain_frame = &mut self.swapchain.frames[image_index as usize];

		// Wait for swapchain frame to become available
		if swapchain_frame.fence != vk::Fence::null() {
			let fences = [swapchain_frame.fence];
			unsafe { logical_device.wait_for_fences(&fences, true, std::u64::MAX).unwrap() };
		}

		swapchain_frame.fence = in_flight_frame.fence;

		// Calculate total required dynamic mesh memory size and chunk sizes
		let dynamic_mesh_initial_chunk_size = FRAME_DATA_MEMORY_SIZE;
		let mut dynamic_mesh_total_size = dynamic_mesh_initial_chunk_size;
		let mut dynamic_mesh_chunk_sizes: Vec<[usize; 4]> = Vec::with_capacity(dynamic_meshes.len());
		let uniform_alignment = self.context.physical_device.min_uniform_buffer_offset_alignment as usize;

		for mesh in dynamic_meshes {
			let indices = mesh.geometry.get_vertex_indices();
			let attributes = mesh.geometry.get_vertex_attributes();
			let index_size = indices.len() * size_of::<u16>();
			let index_padding_size = size_of::<f32>() - (dynamic_mesh_total_size + index_size) % size_of::<f32>();
			let attribute_size = attributes.len() * size_of::<f32>();
			let attribute_padding = uniform_alignment - (dynamic_mesh_total_size + index_size + index_padding_size + attribute_size) % uniform_alignment;
			let uniform_size = 16 * size_of::<f32>();
			dynamic_mesh_chunk_sizes.push([index_size, index_padding_size, attribute_size, attribute_padding]);
			dynamic_mesh_total_size += index_size + index_padding_size + attribute_size + attribute_padding + uniform_size;
		}

		let dynamic_mesh_total_size = dynamic_mesh_total_size as vk::DeviceSize;

		// Allocate more memory in buffer for dynamic meshes if necessary
		if dynamic_mesh_total_size > in_flight_frame.buffer.capacity {
			in_flight_frame.buffer.reallocate(dynamic_mesh_total_size);

			// Update descriptor sets to refer to new memory buffer
			Self::update_in_flight_frame_descriptor_sets(
				&self.context.logical_device,
				&in_flight_frame.frame_data_descriptor_set,
				&in_flight_frame.model_matrix_descriptor_set,
				&in_flight_frame.buffer.handle);
		}

		// Copy frame data into dynamic memory buffer
		let buffer_ptr = unsafe { logical_device.map_memory(in_flight_frame.buffer.memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty()).unwrap() };
		
		unsafe {
			let projection_matrix_dst_ptr = buffer_ptr as *mut [f32; 4];
			let projection_matrix = &camera.projection_matrix.elements;
			ptr::copy_nonoverlapping(projection_matrix.as_ptr(), projection_matrix_dst_ptr, projection_matrix.len());

			let inverse_view_matrix_dst_ptr = buffer_ptr.add(16 * size_of::<f32>()) as *mut [f32; 4];
			let inverse_view_matrix = &camera.inverse_view_matrix;
			ptr::copy_nonoverlapping(inverse_view_matrix.elements.as_ptr(), inverse_view_matrix_dst_ptr, inverse_view_matrix.elements.len());

			let ambient_light_dst_ptr = buffer_ptr.add(32 * size_of::<f32>()) as *mut Vector3;
			let ambient_light_vec = ambient_light.color * ambient_light.intensity;
			ptr::copy_nonoverlapping(&ambient_light_vec as *const Vector3, ambient_light_dst_ptr, 1);

			let point_light_count_dst_ptr = buffer_ptr.add(35 * size_of::<f32>()) as *mut u32;
			let point_light_count = point_lights.len() as u32;
			ptr::copy_nonoverlapping(&point_light_count as *const u32, point_light_count_dst_ptr, 1);

			let position_base_offest = 36 * size_of::<f32>();
			let color_base_offest = 40 * size_of::<f32>();
			let stride = 8 * size_of::<f32>();

			for (i, light) in point_lights.iter().enumerate() {
				let position_dst_ptr = buffer_ptr.add(position_base_offest + stride * i) as *mut Vector3;
				ptr::copy_nonoverlapping(&light.position as *const Vector3, position_dst_ptr, 1);

				let color_dst_ptr = buffer_ptr.add(color_base_offest + stride * i) as *mut Vector3;
				let color_vec = light.color * light.intensity;
				ptr::copy_nonoverlapping(&color_vec as *const Vector3, color_dst_ptr, 1);
			}
		}

		// Begin the secondary command buffers
		let command_buffer_inheritance_info = vk::CommandBufferInheritanceInfo::builder()
			.render_pass(self.render_pass)
			.subpass(0)
			.framebuffer(swapchain_frame.framebuffer);

		let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
			.flags(vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE | vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
			.inheritance_info(&command_buffer_inheritance_info);
		
		let descriptor_sets = [in_flight_frame.frame_data_descriptor_set];
		let dynamic_offsets = [];
		
		unsafe {
			logical_device.begin_command_buffer(in_flight_frame.basic_secondary_command_buffer, &command_buffer_begin_info).unwrap();
			logical_device.cmd_bind_pipeline(in_flight_frame.basic_secondary_command_buffer, vk::PipelineBindPoint::GRAPHICS, self.basic_pipeline);
			logical_device.cmd_bind_descriptor_sets(
				in_flight_frame.basic_secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.shared_pipeline_resources.pipeline_layout,
				0,
				&descriptor_sets,
				&dynamic_offsets);

			logical_device.begin_command_buffer(in_flight_frame.lambert_secondary_command_buffer, &command_buffer_begin_info).unwrap();
			logical_device.cmd_bind_pipeline(in_flight_frame.lambert_secondary_command_buffer, vk::PipelineBindPoint::GRAPHICS, self.lambert_pipeline);
			logical_device.cmd_bind_descriptor_sets(
				in_flight_frame.lambert_secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.shared_pipeline_resources.pipeline_layout,
				0,
				&descriptor_sets,
				&dynamic_offsets);
		}

		let find_secondary_command_buffer_from_material = |m: mesh::Material|
			match m {
				mesh::Material::Basic => in_flight_frame.basic_secondary_command_buffer,
				mesh::Material::Lambert => in_flight_frame.lambert_secondary_command_buffer
			};

		// Record dynamic mesh commands and copy dynamic mesh data into dynamic buffer
		let mut base_offset = dynamic_mesh_initial_chunk_size;
		for (i, mesh) in dynamic_meshes.iter().enumerate() {
			let indices = mesh.geometry.get_vertex_indices();
			let attributes = mesh.geometry.get_vertex_attributes();
			let index_size = dynamic_mesh_chunk_sizes[i][0];
			let index_padding_size = dynamic_mesh_chunk_sizes[i][1];
			let attribute_size = dynamic_mesh_chunk_sizes[i][2];
			let attribute_padding_size = dynamic_mesh_chunk_sizes[i][3];
			let uniform_size = 16 * size_of::<f32>();

			unsafe {
				// Copy index, attribute and uniform buffer objects into memory buffer
				let index_offset = base_offset;
				let index_dst_ptr = buffer_ptr.add(index_offset) as *mut u16;
				std::ptr::copy_nonoverlapping(indices.as_ptr(), index_dst_ptr, indices.len());

				let attribute_offset = index_offset + index_size + index_padding_size;
				let attribute_dst_ptr = buffer_ptr.add(attribute_offset) as *mut f32;
				std::ptr::copy_nonoverlapping(attributes.as_ptr(), attribute_dst_ptr, attributes.len());

				let model_matrix_offset = attribute_offset + attribute_size + attribute_padding_size;
				let model_matrix_dst_ptr = buffer_ptr.add(model_matrix_offset) as *mut [f32; 4];
				let model_matrix = &mesh.model_matrix.elements;
				std::ptr::copy_nonoverlapping(model_matrix.as_ptr(), model_matrix_dst_ptr, model_matrix.len());

				// Record draw commands
				let secondary_command_buffer = find_secondary_command_buffer_from_material(mesh.material);

				logical_device.cmd_bind_index_buffer(
					secondary_command_buffer,
					in_flight_frame.buffer.handle,
					base_offset as u64,
					vk::IndexType::UINT16);
				
				let vertex_buffers = [in_flight_frame.buffer.handle];
				let vertex_offsets = [(base_offset + index_size + index_padding_size) as u64];
				logical_device.cmd_bind_vertex_buffers(secondary_command_buffer, 0, &vertex_buffers, &vertex_offsets);
				
				let descriptor_sets = [in_flight_frame.model_matrix_descriptor_set];
				let dynamic_offsets = [(base_offset + index_size + index_padding_size + attribute_size + attribute_padding_size) as u32];
				logical_device.cmd_bind_descriptor_sets(
					secondary_command_buffer,
					vk::PipelineBindPoint::GRAPHICS,
					self.shared_pipeline_resources.pipeline_layout,
					1,
					&descriptor_sets,
					&dynamic_offsets);
				
				logical_device.cmd_draw_indexed(secondary_command_buffer, indices.len() as u32, 1, 0, 0, 0);
			}

			base_offset += index_size + index_padding_size + attribute_size + attribute_padding_size + uniform_size;
		}

		// Record static mesh commands
		for render_info in &self.static_mesh_resources.render_info {
			let secondary_command_buffer = find_secondary_command_buffer_from_material(render_info.material);

			unsafe {
				logical_device.cmd_bind_index_buffer(
					secondary_command_buffer,
					self.static_mesh_resources.buffer.handle,
					render_info.base_offset as u64,
					vk::IndexType::UINT16);
				
				let vertex_buffers = [self.static_mesh_resources.buffer.handle];
				let vertex_offsets = [(render_info.base_offset + render_info.index_size + render_info.index_padding_size) as u64];
				logical_device.cmd_bind_vertex_buffers(secondary_command_buffer, 0, &vertex_buffers, &vertex_offsets);
				
				let descriptor_sets = [self.static_mesh_resources.model_matrix_descriptor_set];
				let dynamic_offsets = [(render_info.base_offset + render_info.index_size + render_info.index_padding_size + render_info.attribute_size + render_info.attribute_padding_size) as u32];
				logical_device.cmd_bind_descriptor_sets(
					secondary_command_buffer,
					vk::PipelineBindPoint::GRAPHICS,
					self.shared_pipeline_resources.pipeline_layout,
					1,
					&descriptor_sets,
					&dynamic_offsets);
				
				logical_device.cmd_draw_indexed(secondary_command_buffer, render_info.index_count as u32, 1, 0, 0, 0);
			}
		}

		// End secondary command buffers, flush & unmap dynamic memory buffer
		unsafe {
			logical_device.end_command_buffer(in_flight_frame.basic_secondary_command_buffer).unwrap();
			logical_device.end_command_buffer(in_flight_frame.lambert_secondary_command_buffer).unwrap();

			let ranges = [vk::MappedMemoryRange::builder()
				.memory(in_flight_frame.buffer.memory)
				.offset(0)
				.size(vk::WHOLE_SIZE)
				.build()];
			logical_device.flush_mapped_memory_ranges(&ranges).unwrap();

			logical_device.unmap_memory(in_flight_frame.buffer.memory);
		}

		// Record the primary command buffer
		let color_attachment_clear_value = vk::ClearValue {
			color: vk::ClearColorValue {
				float32: [0.0, 0.0, 0.0, 1.0]
			}
		};
		let depth_attachment_clear_value = vk::ClearValue {
			depth_stencil: vk::ClearDepthStencilValue {
				depth: 1.0,
				stencil: 0,
			}
		};
		let clear_colors = [color_attachment_clear_value, depth_attachment_clear_value];

		let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
			.flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

		let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
			.render_pass(self.render_pass)
			.framebuffer(swapchain_frame.framebuffer)
			.render_area(vk::Rect2D::builder()
				.offset(vk::Offset2D::builder().x(0).y(0).build())
				.extent(self.swapchain.extent)
				.build())
			.clear_values(&clear_colors);
		
		let secondary_command_buffers = [in_flight_frame.basic_secondary_command_buffer, in_flight_frame.lambert_secondary_command_buffer];
		
		unsafe {
			logical_device.begin_command_buffer(in_flight_frame.primary_command_buffer, &command_buffer_begin_info).unwrap();
			logical_device.cmd_begin_render_pass(in_flight_frame.primary_command_buffer, &render_pass_begin_info, vk::SubpassContents::SECONDARY_COMMAND_BUFFERS);
			logical_device.cmd_execute_commands(in_flight_frame.primary_command_buffer, &secondary_command_buffers);
			logical_device.cmd_end_render_pass(in_flight_frame.primary_command_buffer);
			logical_device.end_command_buffer(in_flight_frame.primary_command_buffer).unwrap();
		}

		// Wait for image to be available then submit command buffer
		let image_available_semaphores = [in_flight_frame.image_available];
		let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
		let command_buffers = [in_flight_frame.primary_command_buffer];
		let render_finished_semaphores = [in_flight_frame.render_finished];
		let submit_info = vk::SubmitInfo::builder()
			.wait_semaphores(&image_available_semaphores)
			.wait_dst_stage_mask(&wait_stages)
			.command_buffers(&command_buffers)
			.signal_semaphores(&render_finished_semaphores);
		let submit_infos = [submit_info.build()];

		unsafe {
			logical_device.reset_fences(&fences).unwrap();
			logical_device.queue_submit(self.context.graphics_queue, &submit_infos, in_flight_frame.fence).unwrap();
		}

		// Wait for render to finish then present swapchain image
		let swapchains = [self.swapchain.handle];
		let image_indices = [image_index];
		let present_info = vk::PresentInfoKHR::builder()
			.wait_semaphores(&render_finished_semaphores)
			.swapchains(&swapchains)
			.image_indices(&image_indices);
		
		let result = unsafe { self.swapchain.extension.queue_present(self.context.graphics_queue, &present_info) };

		if let Ok(true) = result {
			let (width, height) = window.get_framebuffer_size();
			self.recreate_swapchain(width as u32, height as u32);
		}
		else if let Err(error) = result {
			if error == vk::Result::ERROR_OUT_OF_DATE_KHR {
				let (width, height) = window.get_framebuffer_size();
				self.recreate_swapchain(width as u32, height as u32);
			}
			else {
				panic!("Could not present swapchain image");
			}
		}

		self.current_in_flight_frame = (self.current_in_flight_frame + 1) % IN_FLIGHT_FRAMES_COUNT;
	}
}

impl<'a> Drop for Renderer<'a> {
	fn drop(&mut self) {
		let logical_device = &self.context.logical_device;

		unsafe {
			logical_device.device_wait_idle().unwrap();

			for frame in &self.in_flight_frames {
				logical_device.destroy_fence(frame.fence, None);
				logical_device.destroy_semaphore(frame.render_finished, None);
				logical_device.destroy_semaphore(frame.image_available, None);
			}

			logical_device.destroy_descriptor_pool(self.descriptor_pool, None);
			logical_device.destroy_descriptor_set_layout(self.shared_pipeline_resources.dynamic_descriptor_set_layout, None);
			logical_device.destroy_descriptor_set_layout(self.shared_pipeline_resources.static_descriptor_set_layout, None);
			logical_device.destroy_pipeline_layout(self.shared_pipeline_resources.pipeline_layout, None);
			logical_device.destroy_pipeline(self.basic_pipeline, None);
			logical_device.destroy_pipeline(self.lambert_pipeline, None);
			
			for frame in &self.swapchain.frames {
				logical_device.destroy_framebuffer(frame.framebuffer, None);
				logical_device.destroy_image_view(frame.image_view, None);
			}

			logical_device.free_memory(self.swapchain.depth_image_resources.memory, None);
			logical_device.destroy_image_view(self.swapchain.depth_image_resources.image_view, None);
			logical_device.destroy_image(self.swapchain.depth_image_resources.image, None);
			self.swapchain.extension.destroy_swapchain(self.swapchain.handle, None);
			logical_device.destroy_command_pool(self.command_pool, None);
			logical_device.destroy_render_pass(self.render_pass, None);
		}
	}
}