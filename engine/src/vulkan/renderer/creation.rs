use std::{ffi::CString, fs, mem::{MaybeUninit, transmute}};
use ash::{vk, version::DeviceV1_0, version::InstanceV1_0, extensions::khr};
use crate::vulkan::{Context, Buffer};
use super::*;

pub(super) fn create_render_pass(context: &Context) -> vk::RenderPass {
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

pub(super) fn create_swapchain(context: &Context, framebuffer_width: u32, framebuffer_height: u32, render_pass: vk::RenderPass) -> Swapchain {
	// Get present mode
	let present_modes = unsafe { context.surface.extension.get_physical_device_surface_present_modes(context.physical_device.handle, context.surface.handle).unwrap() };
	let present_mode_option = present_modes.iter().find(|&&m| m == vk::PresentModeKHR::FIFO);
	let present_mode = *present_mode_option.unwrap_or_else(|| &present_modes[0]);

	// Create extent
	let capabilities = unsafe { context.surface.extension.get_physical_device_surface_capabilities(context.physical_device.handle, context.surface.handle).unwrap() };
	let extent = if capabilities.current_extent.width == std::u32::MAX {
		vk::Extent2D::builder()
			.width(std::cmp::max(capabilities.current_extent.width, std::cmp::min(capabilities.current_extent.width, framebuffer_width)))
			.height(std::cmp::max(capabilities.current_extent.height, std::cmp::min(capabilities.current_extent.height, framebuffer_height)))
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
	for image in images {
		// Create image view
		let image_view_create_info = vk::ImageViewCreateInfo::builder()
			.image(image)
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
			.render_pass(render_pass)
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

pub(super) fn create_descriptor_pool(context: &Context) -> vk::DescriptorPool {
	let max_frames = IN_FLIGHT_FRAMES_COUNT as u32;

	// It's own set
	// Frame data, offsets are not dynamic, one for each in flight frame
	let uniform_buffer_pool_size = vk::DescriptorPoolSize::builder()
		.ty(vk::DescriptorType::UNIFORM_BUFFER)
		.descriptor_count(max_frames);

	// It's own set
	// Mesh data and text data from the dynamic buffer, offsets are dynamic, a pair for each in flight frame
	// Single descriptor for the mesh data from the static buffer, offsets are dynamic
	let uniform_buffer_dynamic_pool_size = vk::DescriptorPoolSize::builder()
		.ty(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
		.descriptor_count(max_frames * 2 + 1);
	
	// Set of two descriptors
	// Single sampler used to sample from the font atlas
	/*let sampler_pool_size = vk::DescriptorPoolSize::builder()
		.ty(vk::DescriptorType::SAMPLER)
		.descriptor_count(1);
	
	// The array of font atlases
	let sampled_image_pool_size = vk::DescriptorPoolSize::builder()
		.ty(vk::DescriptorType::SAMPLED_IMAGE)
		.descriptor_count(MAX_FONTS as u32);*/
	
	let pool_sizes = [
		uniform_buffer_pool_size.build(),
		uniform_buffer_dynamic_pool_size.build(),
		// sampler_pool_size.build(),
		// sampled_image_pool_size.build()
	];
	
	let create_info = vk::DescriptorPoolCreateInfo::builder()
		.pool_sizes(&pool_sizes)

		// 3 times each in flight frame for the frame data, mesh data & text data
		// One for the static mesh data
		// One for the text sampler and atlas textures
		.max_sets(20);
	
	unsafe { context.logical_device.create_descriptor_pool(&create_info, None).unwrap() }
}

pub(super) fn create_command_pool(context: &Context) -> vk::CommandPool {
	let create_info = vk::CommandPoolCreateInfo::builder()
		.queue_family_index(context.physical_device.graphics_queue_family)
		.flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

	unsafe { context.logical_device.create_command_pool(&create_info, None).unwrap() }
}

fn create_shader_module(logical_device: &ash::Device, filename: &str) -> vk::ShaderModule {
	let mut file_path = String::from("target/shaders/");
	file_path.push_str(filename);

	let mut file = fs::File::open(file_path).unwrap();
	let file_contents = ash::util::read_spv(&mut file).unwrap();

	let create_info = vk::ShaderModuleCreateInfo::builder()
		.code(&file_contents);

	unsafe { logical_device.create_shader_module(&create_info, None).unwrap() }
}

pub(super) fn create_frame_data_descriptor_set_layout(logical_device: &ash::Device) -> vk::DescriptorSetLayout {
	let layout_binding = vk::DescriptorSetLayoutBinding::builder()
		.binding(0)
		.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
		.descriptor_count(1)
		.stage_flags(vk::ShaderStageFlags::VERTEX);
	let layout_bindings = [layout_binding.build()];

	let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
		.bindings(&layout_bindings);

	unsafe { logical_device.create_descriptor_set_layout(&create_info, None) }.unwrap()
}

pub(super) fn create_instance_data_descriptor_set_layout(logical_device: &ash::Device) -> vk::DescriptorSetLayout {
	let layout_binding = vk::DescriptorSetLayoutBinding::builder()
		.binding(0)
		.descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
		.descriptor_count(1)
		.stage_flags(vk::ShaderStageFlags::VERTEX);
	
	let layout_bindings = [layout_binding.build()];

	let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
		.bindings(&layout_bindings);

	unsafe { logical_device.create_descriptor_set_layout(&create_info, None) }.unwrap()
}

pub(super) fn create_pipeline_layout(
	logical_device: &ash::Device,
	frame_data_descriptor_set_layout: vk::DescriptorSetLayout,
	instance_data_descriptor_set_layout: vk::DescriptorSetLayout)
	-> vk::PipelineLayout
{
	let descriptor_set_layouts = [frame_data_descriptor_set_layout, instance_data_descriptor_set_layout];

	let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
		.set_layouts(&descriptor_set_layouts);

	unsafe { logical_device.create_pipeline_layout(&pipeline_layout_create_info, None) }.unwrap()
}

pub(super) fn create_pipelines(logical_device: &ash::Device, extent: vk::Extent2D, pipeline_layout: vk::PipelineLayout, render_pass: vk::RenderPass) -> Vec<vk::Pipeline> {
	// Shared
	let entry_point = CString::new("main").unwrap();
	let entry_point_cstr = entry_point.as_c_str();

	let input_binding_description = vk::VertexInputBindingDescription::builder()
		.binding(0)
		.stride(24)
		.input_rate(vk::VertexInputRate::VERTEX);
	let input_binding_descriptions = [input_binding_description.build()];

	let input_attribute_description_position = vk::VertexInputAttributeDescription::builder()	
		.binding(0)
		.location(0)
		.format(vk::Format::R32G32B32_SFLOAT)
		.offset(0)
		.build();
	
	let input_attribute_description_normal = vk::VertexInputAttributeDescription::builder()	
		.binding(0)
		.location(1)
		.format(vk::Format::R32G32B32_SFLOAT)
		.offset(12)
		.build();
	
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
		.extent(extent);
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

	// Basic
	let basic_vert_module = create_shader_module(logical_device, "basic.vert.spv");
	let basic_vert_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
		.stage(vk::ShaderStageFlags::VERTEX)
		.module(basic_vert_module)
		.name(entry_point_cstr);
	
	let basic_frag_module = create_shader_module(logical_device, "basic.frag.spv");
	let basic_frag_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
		.stage(vk::ShaderStageFlags::FRAGMENT)
		.module(basic_frag_module)
		.name(entry_point_cstr);
	
	let basic_stage_create_infos = [basic_vert_stage_create_info.build(), basic_frag_stage_create_info.build()];
	let basic_input_attribute_descriptions = [input_attribute_description_position];

	let basic_vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo::builder()
		.vertex_binding_descriptions(&input_binding_descriptions)
		.vertex_attribute_descriptions(&basic_input_attribute_descriptions);

	let basic_pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
		.stages(&basic_stage_create_infos)
		.vertex_input_state(&basic_vertex_input_state_create_info)
		.input_assembly_state(&input_assembly_state_create_info)
		.viewport_state(&viewport_state_create_info)
		.rasterization_state(&rasterization_state_create_info)
		.multisample_state(&multisample_state_create_info)
		.depth_stencil_state(&depth_stencil_state_create_info)
		.color_blend_state(&color_blend_state_create_info)
		.layout(pipeline_layout)
		.render_pass(render_pass)
		.subpass(0);
	
	// Normal
	let normal_vert_module = create_shader_module(logical_device, "normal.vert.spv");
	let normal_vert_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
		.stage(vk::ShaderStageFlags::VERTEX)
		.module(normal_vert_module)
		.name(entry_point_cstr);

	let normal_frag_module =  create_shader_module(logical_device, "normal.frag.spv");
	let normal_frag_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
		.stage(vk::ShaderStageFlags::FRAGMENT)
		.module(normal_frag_module)
		.name(entry_point_cstr);
	
	let normal_stage_create_infos = [normal_vert_stage_create_info.build(), normal_frag_stage_create_info.build()];
	let normal_input_attribute_descriptions = [input_attribute_description_position, input_attribute_description_normal];

	let normal_vert_input_state_create_info = vk::PipelineVertexInputStateCreateInfo::builder()
		.vertex_binding_descriptions(&input_binding_descriptions)
		.vertex_attribute_descriptions(&normal_input_attribute_descriptions);
	
	let normal_pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
		.stages(&normal_stage_create_infos)
		.vertex_input_state(&normal_vert_input_state_create_info)
		.input_assembly_state(&input_assembly_state_create_info)
		.viewport_state(&viewport_state_create_info)
		.rasterization_state(&rasterization_state_create_info)
		.multisample_state(&multisample_state_create_info)
		.depth_stencil_state(&depth_stencil_state_create_info)
		.color_blend_state(&color_blend_state_create_info)
		.layout(pipeline_layout)
		.render_pass(render_pass)
		.subpass(0);
	
	// Lambert
	let lambert_vert_module = create_shader_module(logical_device, "lambert.vert.spv");
	let lambert_vert_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
		.stage(vk::ShaderStageFlags::VERTEX)
		.module(lambert_vert_module)
		.name(entry_point_cstr);

	let lambert_frag_module =  create_shader_module(logical_device, "lambert.frag.spv");
	let lambert_frag_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
		.stage(vk::ShaderStageFlags::FRAGMENT)
		.module(lambert_frag_module)
		.name(entry_point_cstr);

	let lambert_stage_create_infos = [lambert_vert_stage_create_info.build(), lambert_frag_stage_create_info.build()];
	let lambert_input_attribute_descriptions = [input_attribute_description_position, input_attribute_description_normal];

	let lambert_vert_input_state_create_info = vk::PipelineVertexInputStateCreateInfo::builder()
		.vertex_binding_descriptions(&input_binding_descriptions)
		.vertex_attribute_descriptions(&lambert_input_attribute_descriptions);

	let lambert_pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
		.stages(&lambert_stage_create_infos)
		.vertex_input_state(&lambert_vert_input_state_create_info)
		.input_assembly_state(&input_assembly_state_create_info)
		.viewport_state(&viewport_state_create_info)
		.rasterization_state(&rasterization_state_create_info)
		.multisample_state(&multisample_state_create_info)
		.depth_stencil_state(&depth_stencil_state_create_info)
		.color_blend_state(&color_blend_state_create_info)
		.layout(pipeline_layout)
		.render_pass(render_pass)
		.subpass(0);
	
	// Create pipelines
	let pipeline_create_infos = [
		basic_pipeline_create_info.build(),
		normal_pipeline_create_info.build(),
		lambert_pipeline_create_info.build()];
	
	let pipelines = unsafe { logical_device.create_graphics_pipelines(vk::PipelineCache::null(), &pipeline_create_infos, None) }.unwrap();

	unsafe {
		logical_device.destroy_shader_module(basic_vert_module, None);
		logical_device.destroy_shader_module(basic_frag_module, None);

		logical_device.destroy_shader_module(normal_vert_module, None);
		logical_device.destroy_shader_module(normal_frag_module, None);

		logical_device.destroy_shader_module(lambert_vert_module, None);
		logical_device.destroy_shader_module(lambert_frag_module, None);
	}

	pipelines
}

pub(super) fn create_static_descriptor_sets(logical_device: &ash::Device, descriptor_pool: vk::DescriptorPool, instance_data_descriptor_set_layout: vk::DescriptorSetLayout) -> Vec<vk::DescriptorSet> {
	let descriptor_set_layouts = [instance_data_descriptor_set_layout, instance_data_descriptor_set_layout, instance_data_descriptor_set_layout];
	let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
		.descriptor_pool(descriptor_pool)
		.set_layouts(&descriptor_set_layouts);
	
	unsafe { logical_device.allocate_descriptor_sets(&descriptor_set_allocate_info) }.unwrap()
}

pub(super) fn create_in_flight_frames(
	context: &Context,
	descriptor_pool: &vk::DescriptorPool,
	command_pool: &vk::CommandPool,
	frame_data_descriptor_set_layout: &vk::DescriptorSetLayout,
	instance_data_descriptor_set_layout: &vk::DescriptorSetLayout)
	-> [InFlightFrame; IN_FLIGHT_FRAMES_COUNT]
{
	let semaphore_create_info = vk::SemaphoreCreateInfo::builder();

	let fence_create_info = vk::FenceCreateInfo::builder()
		.flags(vk::FenceCreateFlags::SIGNALED);

	let primary_command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
		.command_pool(*command_pool)
		.level(vk::CommandBufferLevel::PRIMARY)
		.command_buffer_count(IN_FLIGHT_FRAMES_COUNT as u32);
	
	let primary_command_buffers = unsafe { context.logical_device.allocate_command_buffers(&primary_command_buffer_allocate_info) }.unwrap();

	let secondary_command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
		.command_pool(*command_pool)
		.level(vk::CommandBufferLevel::SECONDARY)
		.command_buffer_count(IN_FLIGHT_FRAMES_COUNT as u32 * 6);
	
	let secondary_command_buffers = unsafe { context.logical_device.allocate_command_buffers(&secondary_command_buffer_allocate_info) }.unwrap();

	let descriptor_set_layouts = [*frame_data_descriptor_set_layout, *instance_data_descriptor_set_layout, *instance_data_descriptor_set_layout, *instance_data_descriptor_set_layout];
	let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
		.descriptor_pool(*descriptor_pool)
		.set_layouts(&descriptor_set_layouts);

	let mut frames: [MaybeUninit<InFlightFrame>; IN_FLIGHT_FRAMES_COUNT] = unsafe { MaybeUninit::uninit().assume_init() };
	
	for (index, frame) in frames.iter_mut().enumerate() {
		let image_available = unsafe { context.logical_device.create_semaphore(&semaphore_create_info, None) }.unwrap();
		let render_finished = unsafe { context.logical_device.create_semaphore(&semaphore_create_info, None) }.unwrap();
		let fence = unsafe { context.logical_device.create_fence(&fence_create_info, None) }.unwrap();
		let descriptor_sets = unsafe { context.logical_device.allocate_descriptor_sets(&descriptor_set_allocate_info) }.unwrap();
		let frame_data_descriptor_set = descriptor_sets[0];
		let primary_command_buffer = primary_command_buffers[index];

		let frame_buffer = Buffer::new(context, FRAME_DATA_MEMORY_SIZE as u64, vk::BufferUsageFlags::UNIFORM_BUFFER, vk::MemoryPropertyFlags::HOST_VISIBLE);

		let mesh_buffer = Buffer::new(
			context,
			1,
			vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::STORAGE_BUFFER,
			vk::MemoryPropertyFlags::HOST_VISIBLE);
		
		let frame_data_descriptor_buffer_info = vk::DescriptorBufferInfo::builder()
			.buffer(frame_buffer.handle)
			.offset(0)
			.range(vk::WHOLE_SIZE);
		let frame_data_descriptor_buffer_infos = [frame_data_descriptor_buffer_info.build()];

		let frame_data_write_descriptor_set = vk::WriteDescriptorSet::builder()
			.dst_set(frame_data_descriptor_set)
			.dst_binding(0)
			.dst_array_element(0)
			.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
			.buffer_info(&frame_data_descriptor_buffer_infos);
		
		let write_descriptor_sets = [frame_data_write_descriptor_set.build()];
		unsafe { context.logical_device.update_descriptor_sets(&write_descriptor_sets, &[]) };

		let basic_material_data = MaterialData {
			descriptor_set: descriptor_sets[1],
			secondary_command_buffer: secondary_command_buffers[6 * index],
			secondary_static_command_buffer: secondary_command_buffers[6 * index + 1],
			array_offset: 0,
			array_size: 0
		};

		let normal_material_data = MaterialData {
			descriptor_set: descriptor_sets[2],
			secondary_command_buffer: secondary_command_buffers[6 * index + 2],
			secondary_static_command_buffer: secondary_command_buffers[6 * index + 3],
			array_offset: 0,
			array_size: 0
		};

		let lambert_material_data = MaterialData {
			descriptor_set: descriptor_sets[3],
			secondary_command_buffer: secondary_command_buffers[6 * index + 4],
			secondary_static_command_buffer: secondary_command_buffers[6 * index + 5],
			array_offset: 0,
			array_size: 0
		};

		*frame = MaybeUninit::new(InFlightFrame {
			image_available,
			render_finished,
			fence,
			frame_data_descriptor_set,
			primary_command_buffer,
			frame_buffer,
			mesh_buffer,
			basic_material_data,
			normal_material_data,
			lambert_material_data,
			index_arrays_offset: 0
		});
	}

	unsafe { transmute::<_, [InFlightFrame; IN_FLIGHT_FRAMES_COUNT]>(frames) }
}