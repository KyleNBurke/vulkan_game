use ash::{vk, version::DeviceV1_0, version::InstanceV1_0, extensions::khr};
use crate::{vulkan::Context, vulkan::Buffer, Mesh, Camera};
use std::{mem, mem::size_of};

const IN_FLIGHT_FRAMES_COUNT: usize = 2;

pub struct Renderer<'a> {
	context: &'a Context,
	render_pass: vk::RenderPass,
	command_pool: vk::CommandPool,
	swapchain: Swapchain,
	pipeline: Pipeline,
	descriptor_pool: vk::DescriptorPool,
	in_flight_frames: [InFlightFrame<'a>; IN_FLIGHT_FRAMES_COUNT],
	current_in_flight_frame: usize,
	static_mesh_content: StaticMeshContent<'a>
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
	command_buffer: vk::CommandBuffer,
	fence: vk::Fence
}

struct Pipeline {
	handle: vk::Pipeline,
	pipeline_layout: vk::PipelineLayout,
	static_descriptor_set_layout: vk::DescriptorSetLayout,
	dynamic_descriptor_set_layout: vk::DescriptorSetLayout
}

struct InFlightFrame<'a> {
	image_available: vk::Semaphore,
	render_finished: vk::Semaphore,
	fence: vk::Fence,
	buffer: Buffer<'a>,
	projection_view_matrix_descriptor_set: vk::DescriptorSet,
	model_matrix_descriptor_set: vk::DescriptorSet
}

struct StaticMeshContent<'a> {
	buffer: Buffer<'a>,
	model_matrix_descriptor_set: vk::DescriptorSet,
	chunk_sizes: Vec<[usize; 5]>
}

impl<'a> Renderer<'a> {
	pub fn new(context: &'a Context, width: u32, height: u32) -> Self {
		let render_pass = Self::render_pass(&context);
		let command_pool = Self::create_command_pool(&context);
		let swapchain = Self::create_swapchain(&context, width, height, &command_pool, &render_pass);
		let pipeline = Self::create_pipeline(&context, &swapchain, &render_pass);
		let descriptor_pool = Self::create_descriptor_pool(&context);
		let in_flight_frames = Self::create_in_flight_frames(&context, &descriptor_pool, &pipeline.static_descriptor_set_layout, &pipeline.dynamic_descriptor_set_layout);
		let static_mesh_content = Self::create_static_mesh_content(&context, &descriptor_pool, &pipeline.dynamic_descriptor_set_layout);

		Self {
			context,
			render_pass,
			command_pool,
			swapchain,
			pipeline,
			descriptor_pool,
			in_flight_frames,
			current_in_flight_frame: 0,
			static_mesh_content
		}
	}

	fn render_pass(context: &Context) -> vk::RenderPass {
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

	fn create_command_pool(context: &Context) -> vk::CommandPool {
		let create_info = vk::CommandPoolCreateInfo::builder()
			.queue_family_index(context.physical_device.graphics_queue_family.index)
			.flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

		unsafe { context.logical_device.create_command_pool(&create_info, None).unwrap() }
	}

	fn create_swapchain(context: &Context, width: u32, height: u32, command_pool: &vk::CommandPool, render_pass: &vk::RenderPass) -> Swapchain {
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
		
		let graphics_queue_family_index = context.physical_device.graphics_queue_family.index;
		let present_queue_family_index = context.physical_device.present_quue_family.index;
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

		// Create command buffers
		let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
			.command_pool(*command_pool)
			.level(vk::CommandBufferLevel::PRIMARY)
			.command_buffer_count(images.len() as u32);
		
		let command_buffers = unsafe { context.logical_device.allocate_command_buffers(&command_buffer_allocate_info).unwrap() };

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
		let memory_properties = unsafe { context.instance.get_physical_device_memory_properties(context.physical_device.handle) };
		
		let memory_type_index = (0..memory_properties.memory_types.len())
			.find(|&i| memory_requirements.memory_type_bits & (1 << i) != 0 && memory_properties.memory_types[i].property_flags.contains(vk::MemoryPropertyFlags::DEVICE_LOCAL))
			.expect("Could not find suitable memory type for depth buffering") as u32;

		let allocate_info = vk::MemoryAllocateInfo::builder()
			.allocation_size(memory_requirements.size)
			.memory_type_index(memory_type_index);

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
				command_buffer: command_buffers[i],
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

	fn create_pipeline(context: &Context, swapchain: &Swapchain, render_pass: &vk::RenderPass) -> Pipeline {
		// Create descriptor set layouts
		let static_binding = vk::DescriptorSetLayoutBinding::builder()
			.binding(0)
			.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
			.descriptor_count(1)
			.stage_flags(vk::ShaderStageFlags::VERTEX);
		let static_bindings = [static_binding.build()];

		let static_create_info = vk::DescriptorSetLayoutCreateInfo::builder()
			.bindings(&static_bindings);

		let static_descriptor_set_layout = unsafe { context.logical_device.create_descriptor_set_layout(&static_create_info, None).unwrap() };

		let dynamic_binding = vk::DescriptorSetLayoutBinding::builder()
			.binding(0)
			.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
			.descriptor_count(1)
			.stage_flags(vk::ShaderStageFlags::VERTEX);
		let dynamic_bindings = [dynamic_binding.build()];

		let dynamic_create_info = vk::DescriptorSetLayoutCreateInfo::builder()
			.bindings(&dynamic_bindings);

		let dynamic_descriptor_set_layout = unsafe { context.logical_device.create_descriptor_set_layout(&dynamic_create_info, None).unwrap() };

		let descriptor_set_layouts = [static_descriptor_set_layout, dynamic_descriptor_set_layout];

		// Create layout
		let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
			.set_layouts(&descriptor_set_layouts);

		let pipeline_layout = unsafe { context.logical_device.create_pipeline_layout(&pipeline_layout_create_info, None).unwrap() };

		// Create handle
		let handle = Self::create_pipeline_handle(context, swapchain, &pipeline_layout, render_pass);

		Pipeline {
			handle,
			pipeline_layout,
			static_descriptor_set_layout,
			dynamic_descriptor_set_layout
		}
	}

	fn create_pipeline_handle(context: &Context, swapchain: &Swapchain, pipeline_layout: &vk::PipelineLayout, render_pass: &vk::RenderPass) -> vk::Pipeline {
		let mut curr_dir = std::env::current_exe().unwrap();
		curr_dir.pop();

		let mut vert_file = std::fs::File::open(curr_dir.join("vert.spv").as_path()).unwrap();
		let mut frag_file = std::fs::File::open(curr_dir.join("frag.spv").as_path()).unwrap();
		let vert_file_contents = ash::util::read_spv(&mut vert_file).unwrap();
		let frag_file_contents = ash::util::read_spv(&mut frag_file).unwrap();
		
		let vert_create_info = vk::ShaderModuleCreateInfo::builder().code(&vert_file_contents);
		let frag_create_info = vk::ShaderModuleCreateInfo::builder().code(&frag_file_contents);
		let vert_shader_module = unsafe { context.logical_device.create_shader_module(&vert_create_info, None).unwrap() };
		let frag_shader_module = unsafe { context.logical_device.create_shader_module(&frag_create_info, None).unwrap() };

		let entry_point = std::ffi::CString::new("main").unwrap();

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
			.width(swapchain.extent.width as f32)
			.height(swapchain.extent.height as f32)
			.min_depth(0.0)
			.max_depth(1.0);
		let viewports = [viewport.build()];
		
		let scissor = vk::Rect2D::builder()
			.offset(vk::Offset2D::builder().x(0).y(0).build())
			.extent(swapchain.extent);
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

		let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
			.stages(&stages)
			.vertex_input_state(&vert_input_state_create_info)
			.input_assembly_state(&input_assembly_state_create_info)
			.viewport_state(&viewport_state_create_info)
			.rasterization_state(&rasterization_state_create_info)
			.multisample_state(&multisample_state_create_info)
			.depth_stencil_state(&depth_stencil_state_create_info)
			.color_blend_state(&color_blend_state_create_info)
			.layout(*pipeline_layout)
			.render_pass(*render_pass)
			.subpass(0);
		let pipeline_create_infos = [pipeline_create_info.build()];
		
		let pipelines = unsafe { context.logical_device.create_graphics_pipelines(vk::PipelineCache::null(), &pipeline_create_infos, None).unwrap() };

		unsafe {
			context.logical_device.destroy_shader_module(vert_shader_module, None);
			context.logical_device.destroy_shader_module(frag_shader_module, None);
		}

		pipelines[0]
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

	fn create_in_flight_frames(
		context: &Context,
		descriptor_pool: &vk::DescriptorPool,
		static_descriptor_set_layout: &vk::DescriptorSetLayout,
		dynamic_descriptor_set_layout: &vk::DescriptorSetLayout) -> [InFlightFrame<'a>; IN_FLIGHT_FRAMES_COUNT]
	{
		let semaphore_create_info = vk::SemaphoreCreateInfo::builder();
		let fence_create_info = vk::FenceCreateInfo::builder()
			.flags(vk::FenceCreateFlags::SIGNALED);

		let dynamic_descriptor_set_layouts = [*static_descriptor_set_layout, *dynamic_descriptor_set_layout];
		let dynamic_descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
			.descriptor_pool(*descriptor_pool)
			.set_layouts(&dynamic_descriptor_set_layouts);

		let mut frames: [mem::MaybeUninit<InFlightFrame>; IN_FLIGHT_FRAMES_COUNT] = unsafe { mem::MaybeUninit::uninit().assume_init() };
		for frame in &mut frames {
			let image_available = unsafe { context.logical_device.create_semaphore(&semaphore_create_info, None).unwrap() };
			let render_finished = unsafe { context.logical_device.create_semaphore(&semaphore_create_info, None).unwrap() };
			let fence = unsafe { context.logical_device.create_fence(&fence_create_info, None).unwrap() };
			
			let buffer = Buffer::new(
				context,
				128,
				vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::UNIFORM_BUFFER,
				vk::MemoryPropertyFlags::HOST_VISIBLE);
			
			let descriptor_sets = unsafe { context.logical_device.allocate_descriptor_sets(&dynamic_descriptor_set_allocate_info).unwrap() };
			let projection_view_matrix_descriptor_set = descriptor_sets[0];
			let model_matrix_descriptor_set = descriptor_sets[1];

			Self::update_in_flight_frame_descriptor_sets(
				&context.logical_device,
				&projection_view_matrix_descriptor_set,
				&model_matrix_descriptor_set,
				&buffer.handle);

			*frame = mem::MaybeUninit::new(InFlightFrame {
				image_available,
				render_finished,
				fence,
				buffer,
				projection_view_matrix_descriptor_set,
				model_matrix_descriptor_set
			});
		}

		unsafe { mem::transmute::<_, [InFlightFrame; IN_FLIGHT_FRAMES_COUNT]>(frames) }
	}

	fn update_in_flight_frame_descriptor_sets(
		logical_device: &ash::Device,
		projection_view_matrix_descriptor_set: &vk::DescriptorSet,
		model_matrix_descriptor_set: &vk::DescriptorSet,
		buffer: &vk::Buffer)
	{
		// Projection & view
		let projection_view_matrix_descriptor_buffer_info = vk::DescriptorBufferInfo::builder()
			.buffer(*buffer)
			.offset(0)
			.range(32 * std::mem::size_of::<f32>() as u64);
		let projection_view_matrix_descriptor_buffer_infos = [projection_view_matrix_descriptor_buffer_info.build()];

		let projection_view_matrix_write_descriptor_set = vk::WriteDescriptorSet::builder()
			.dst_set(*projection_view_matrix_descriptor_set)
			.dst_binding(0)
			.dst_array_element(0)
			.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
			.buffer_info(&projection_view_matrix_descriptor_buffer_infos);

		// Model
		let model_matrix_descriptor_buffer_info = vk::DescriptorBufferInfo::builder()
			.buffer(*buffer)
			.offset(0)
			.range(16 * std::mem::size_of::<f32>() as u64);
		let model_matrix_descriptor_buffer_infos = [model_matrix_descriptor_buffer_info.build()];

		let model_matrix_write_descriptor_set = vk::WriteDescriptorSet::builder()
			.dst_set(*model_matrix_descriptor_set)
			.dst_binding(0)
			.dst_array_element(0)
			.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
			.buffer_info(&model_matrix_descriptor_buffer_infos);

		let write_descriptor_sets = [
			projection_view_matrix_write_descriptor_set.build(),
			model_matrix_write_descriptor_set.build()
		];
		let copy_descriptor_sets = [];

		unsafe { logical_device.update_descriptor_sets(&write_descriptor_sets, &copy_descriptor_sets) };
	}

	fn create_static_mesh_content(context: &'a Context, descriptor_pool: &vk::DescriptorPool, dynamic_descriptor_set_layout: &vk::DescriptorSetLayout) -> StaticMeshContent<'a> {
		let buffer = Buffer::null(
			context,
			vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::UNIFORM_BUFFER,
			vk::MemoryPropertyFlags::DEVICE_LOCAL);
		
		let descriptor_set_layout = [*dynamic_descriptor_set_layout];
		let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
			.descriptor_pool(*descriptor_pool)
			.set_layouts(&descriptor_set_layout);
		
		let model_matrix_descriptor_set = unsafe { context.logical_device.allocate_descriptor_sets(&descriptor_set_allocate_info).unwrap()[0] };
	
		StaticMeshContent {
			buffer,
			model_matrix_descriptor_set,
			chunk_sizes: vec![]
		}
	}

	pub fn recreate_swapchain(&mut self, width: u32, height: u32) {
		let logical_device = &self.context.logical_device;

		unsafe {
			logical_device.device_wait_idle().unwrap();
			logical_device.destroy_pipeline(self.pipeline.handle, None);
			let mut command_buffers: Vec<vk::CommandBuffer> = Vec::with_capacity(self.swapchain.frames.len());

			for frame in &self.swapchain.frames {
				command_buffers.push(frame.command_buffer);
				logical_device.destroy_framebuffer(frame.framebuffer, None);
				logical_device.destroy_image_view(frame.image_view, None);
			}

			logical_device.free_command_buffers(self.command_pool, &command_buffers);
			logical_device.free_memory(self.swapchain.depth_image_resources.memory, None);
			logical_device.destroy_image_view(self.swapchain.depth_image_resources.image_view, None);
			logical_device.destroy_image(self.swapchain.depth_image_resources.image, None);
			self.swapchain.extension.destroy_swapchain(self.swapchain.handle, None);
		}

		self.swapchain = Self::create_swapchain(&self.context, width, height, &self.command_pool, &self.render_pass);
		self.pipeline.handle = Self::create_pipeline_handle(&self.context, &self.swapchain, &self.pipeline.pipeline_layout, &self.render_pass);
	}

	pub fn submit_static_meshes(&mut self, meshes: &[Mesh]) {
		let logical_device = &self.context.logical_device;

		// Wait for rendering operations to finish
		unsafe { logical_device.queue_wait_idle(self.context.physical_device.graphics_queue_family.queue).unwrap() };

		// Calculate total memory size and chunk sizes
		let mut total_size = 0;
		let mut chunk_sizes: Vec<[usize; 5]> = Vec::with_capacity(meshes.len());
		let uniform_alignment = self.context.physical_device.min_uniform_buffer_offset_alignment as usize;

		for mesh in meshes {
			let indices = mesh.geometry.get_vertex_indices();
			let attributes = mesh.geometry.get_vertex_attributes();
			let index_size = indices.len() * size_of::<u16>();
			let index_padding_size = size_of::<f32>() - (total_size + index_size) % size_of::<f32>();
			let attribute_size = attributes.len() * size_of::<f32>();
			let attribute_padding = uniform_alignment - (total_size + index_size + index_padding_size + attribute_size) % uniform_alignment;
			let uniform_size = 16 * size_of::<f32>();
			chunk_sizes.push([index_size, index_padding_size, attribute_size, attribute_padding, indices.len()]);
			total_size += index_size + index_padding_size + attribute_size + attribute_padding + uniform_size;
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
		let mut mesh_offset = 0;

		for (i, mesh) in meshes.iter().enumerate() {
			let indices = mesh.geometry.get_vertex_indices();
			let attributes = mesh.geometry.get_vertex_attributes();
			let index_size = chunk_sizes[i][0];
			let index_padding_size = chunk_sizes[i][1];
			let attribute_size = chunk_sizes[i][2];
			let attribute_padding_size = chunk_sizes[i][3];
			let uniform_size = 16 * size_of::<f32>();

			unsafe {
				let index_offset = mesh_offset;
				let index_dst_ptr = buffer_ptr.add(index_offset) as *mut u16;
				std::ptr::copy_nonoverlapping(indices.as_ptr(), index_dst_ptr, indices.len());

				let attribute_offset = index_offset + index_size + index_padding_size;
				let attribute_dst_ptr = buffer_ptr.add(attribute_offset) as *mut f32;
				std::ptr::copy_nonoverlapping(attributes.as_ptr(), attribute_dst_ptr, attributes.len());

				let model_matrix_offset = attribute_offset + attribute_size + attribute_padding_size;
				let model_matrix_dst_ptr = buffer_ptr.add(model_matrix_offset) as *mut [f32; 4];
				let model_matrix = &mesh.model_matrix.elements;
				std::ptr::copy_nonoverlapping(model_matrix.as_ptr(), model_matrix_dst_ptr, model_matrix.len());
			}

			mesh_offset += index_size + index_padding_size + attribute_size + attribute_padding_size + uniform_size;
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
		if total_size > self.static_mesh_content.buffer.capacity {
			self.static_mesh_content.buffer.reallocate(total_size);
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
			logical_device.cmd_copy_buffer(command_buffer, staging_buffer.handle, self.static_mesh_content.buffer.handle, &regions);
			logical_device.end_command_buffer(command_buffer).unwrap();
		}

		let command_buffers = [command_buffer];
		let submit_info = vk::SubmitInfo::builder()
			.command_buffers(&command_buffers);
		let submit_infos = [submit_info.build()];
		
		unsafe {
			logical_device.queue_submit(self.context.physical_device.graphics_queue_family.queue, &submit_infos, vk::Fence::null()).unwrap();
			logical_device.queue_wait_idle(self.context.physical_device.graphics_queue_family.queue).unwrap();
			logical_device.free_command_buffers(self.command_pool, &command_buffers);
		}
		
		// Update the descriptor set to reference the device local buffer
		let model_matrix_buffer_info = vk::DescriptorBufferInfo::builder()
			.buffer(self.static_mesh_content.buffer.handle)
			.offset(0)
			.range(16 * size_of::<f32>() as u64);
		let model_matrix_buffer_infos = [model_matrix_buffer_info.build()];

		let model_matrix_write_descriptor_set = vk::WriteDescriptorSet::builder()
			.dst_set(self.static_mesh_content.model_matrix_descriptor_set)
			.dst_binding(0)
			.dst_array_element(0)
			.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
			.buffer_info(&model_matrix_buffer_infos);
		
		let write_descriptor_sets = [model_matrix_write_descriptor_set.build()];
		let copy_descriptor_sets = [];
		
		unsafe { logical_device.update_descriptor_sets(&write_descriptor_sets, &copy_descriptor_sets) };

		// Save chunk sizes for recording command buffers later
		self.static_mesh_content.chunk_sizes = chunk_sizes;
	}

	pub fn render(&mut self, window: &glfw::Window, camera: &Camera, dynamic_meshes: &[Mesh]) {
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
		let dynamic_mesh_initial_chunk_size = 32 * size_of::<f32>();
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
				&in_flight_frame.projection_view_matrix_descriptor_set,
				&in_flight_frame.model_matrix_descriptor_set,
				&in_flight_frame.buffer.handle);
		}

		// Record the command buffers
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

		let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder();

		let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
			.render_pass(self.render_pass)
			.framebuffer(swapchain_frame.framebuffer)
			.render_area(vk::Rect2D::builder()
				.offset(vk::Offset2D::builder().x(0).y(0).build())
				.extent(self.swapchain.extent)
				.build())
			.clear_values(&clear_colors);
		
		unsafe {
			logical_device.begin_command_buffer(swapchain_frame.command_buffer, &command_buffer_begin_info).unwrap();
			logical_device.cmd_begin_render_pass(swapchain_frame.command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
			logical_device.cmd_bind_pipeline(swapchain_frame.command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline.handle);
			
			let descriptor_sets = [in_flight_frame.projection_view_matrix_descriptor_set];
			let dynamic_offsets = [];
			logical_device.cmd_bind_descriptor_sets(
				swapchain_frame.command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.pipeline.pipeline_layout,
				0,
				&descriptor_sets,
				&dynamic_offsets);
		}
		
		let buffer_ptr = unsafe { logical_device.map_memory(in_flight_frame.buffer.memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty()).unwrap() };
		
		// Copy projection and view matrix into dynamic memory buffer
		unsafe {
			let projection_matrix_dst_ptr = buffer_ptr as *mut [f32; 4];
			let projection_matrix = &camera.projection_matrix.elements;
			std::ptr::copy_nonoverlapping(projection_matrix.as_ptr(), projection_matrix_dst_ptr, projection_matrix.len());

			let inverse_view_matrix_dst_ptr = buffer_ptr.add(16 * size_of::<f32>()) as *mut [f32; 4];
			let inverse_view_matrix = &camera.inverse_view_matrix;
			std::ptr::copy_nonoverlapping(inverse_view_matrix.elements.as_ptr(), inverse_view_matrix_dst_ptr, inverse_view_matrix.elements.len());
		}

		// Record static mesh command buffers
		let mut static_mesh_offset = 0;
		for chunk_size in &self.static_mesh_content.chunk_sizes {
			let index_size = chunk_size[0];
			let index_padding_size = chunk_size[1];
			let attribute_size = chunk_size[2];
			let attribute_padding_size = chunk_size[3];
			let uniform_size = 16 * size_of::<f32>();
			let index_count = chunk_size[4];

			unsafe {
				logical_device.cmd_bind_index_buffer(
					swapchain_frame.command_buffer,
					self.static_mesh_content.buffer.handle,
					static_mesh_offset as u64,
					vk::IndexType::UINT16);
				
				let vertex_buffers = [self.static_mesh_content.buffer.handle];
				let vertex_offsets = [(static_mesh_offset + index_size + index_padding_size) as u64];
				logical_device.cmd_bind_vertex_buffers(swapchain_frame.command_buffer, 0, &vertex_buffers, &vertex_offsets);
				
				let descriptor_sets = [self.static_mesh_content.model_matrix_descriptor_set];
				let dynamic_offsets = [(static_mesh_offset + index_size + index_padding_size + attribute_size + attribute_padding_size) as u32];
				logical_device.cmd_bind_descriptor_sets(
					swapchain_frame.command_buffer,
					vk::PipelineBindPoint::GRAPHICS,
					self.pipeline.pipeline_layout,
					1,
					&descriptor_sets,
					&dynamic_offsets);
				
				logical_device.cmd_draw_indexed(swapchain_frame.command_buffer, index_count as u32, 1, 0, 0, 0);
			}

			static_mesh_offset += index_size + index_padding_size + attribute_size + attribute_padding_size + uniform_size;
		}

		// Copy dynamic mesh data into buffer and record dynamic mesh command buffers
		let mut dynamic_mesh_offset = dynamic_mesh_initial_chunk_size;
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
				let index_offset = dynamic_mesh_offset;
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
				logical_device.cmd_bind_index_buffer(
					swapchain_frame.command_buffer,
					in_flight_frame.buffer.handle,
					dynamic_mesh_offset as u64,
					vk::IndexType::UINT16);
				
				let vertex_buffers = [in_flight_frame.buffer.handle];
				let vertex_offsets = [(dynamic_mesh_offset + index_size + index_padding_size) as u64];
				logical_device.cmd_bind_vertex_buffers(swapchain_frame.command_buffer, 0, &vertex_buffers, &vertex_offsets);
				
				let descriptor_sets = [in_flight_frame.model_matrix_descriptor_set];
				let dynamic_offsets = [(dynamic_mesh_offset + index_size + index_padding_size + attribute_size + attribute_padding_size) as u32];
				logical_device.cmd_bind_descriptor_sets(
					swapchain_frame.command_buffer,
					vk::PipelineBindPoint::GRAPHICS,
					self.pipeline.pipeline_layout,
					1,
					&descriptor_sets,
					&dynamic_offsets);
				
				logical_device.cmd_draw_indexed(swapchain_frame.command_buffer, indices.len() as u32, 1, 0, 0, 0);
			}

			dynamic_mesh_offset += index_size + index_padding_size + attribute_size + attribute_padding_size + uniform_size;
		}

		unsafe {
			logical_device.cmd_end_render_pass(swapchain_frame.command_buffer);
			logical_device.end_command_buffer(swapchain_frame.command_buffer).unwrap();

			let ranges = [vk::MappedMemoryRange::builder()
				.memory(in_flight_frame.buffer.memory)
				.offset(0)
				.size(vk::WHOLE_SIZE)
				.build()];
			logical_device.flush_mapped_memory_ranges(&ranges).unwrap();

			logical_device.unmap_memory(in_flight_frame.buffer.memory);
		}

		// Wait for image to be available then submit command buffer
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
			logical_device.reset_fences(&fences).unwrap();
			logical_device.queue_submit(self.context.physical_device.graphics_queue_family.queue, &submit_infos, in_flight_frame.fence).unwrap();
		}

		// Wait for render to finish then present swapchain image
		let swapchains = [self.swapchain.handle];
		let image_indices = [image_index];
		let present_info = vk::PresentInfoKHR::builder()
			.wait_semaphores(&render_finished_semaphores)
			.swapchains(&swapchains)
			.image_indices(&image_indices);
		
		let result = unsafe { self.swapchain.extension.queue_present(self.context.physical_device.graphics_queue_family.queue, &present_info) };

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
			logical_device.destroy_descriptor_set_layout(self.pipeline.dynamic_descriptor_set_layout, None);
			logical_device.destroy_descriptor_set_layout(self.pipeline.static_descriptor_set_layout, None);
			logical_device.destroy_pipeline_layout(self.pipeline.pipeline_layout, None);
			logical_device.destroy_pipeline(self.pipeline.handle, None);
			
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