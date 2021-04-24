use std::{mem::{MaybeUninit, transmute}, cmp::{min, max}};
use ash::{vk, version::DeviceV1_0, version::InstanceV1_0, extensions::khr};
use crate::vulkan::{Context, Buffer};
use super::{Swapchain, DepthImageResources, SwapchainFrame, InFlightFrame, MaterialData, IN_FLIGHT_FRAMES_COUNT, FRAME_DATA_MEMORY_SIZE};

pub fn create_render_pass(context: &Context) -> vk::RenderPass {
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
	let extent = if capabilities.current_extent.width == u32::MAX {
		vk::Extent2D::builder()
			.width(max(capabilities.min_image_extent.width, min(capabilities.max_image_extent.width, framebuffer_width)))
			.height(max(capabilities.min_image_extent.height, min(capabilities.max_image_extent.height, framebuffer_height)))
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

pub fn create_descriptor_pool(context: &Context) -> vk::DescriptorPool {
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

pub fn create_command_pool(context: &Context) -> vk::CommandPool {
	let create_info = vk::CommandPoolCreateInfo::builder()
		.queue_family_index(context.physical_device.graphics_queue_family)
		.flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

	unsafe { context.logical_device.create_command_pool(&create_info, None).unwrap() }
}

pub fn create_frame_data_descriptor_set_layout(logical_device: &ash::Device) -> vk::DescriptorSetLayout {
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

pub fn create_instance_data_descriptor_set_layout(logical_device: &ash::Device) -> vk::DescriptorSetLayout {
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
		.command_buffer_count(IN_FLIGHT_FRAMES_COUNT as u32 * 4);
	
	let secondary_command_buffers = unsafe { context.logical_device.allocate_command_buffers(&secondary_command_buffer_allocate_info) }.unwrap();

	let descriptor_set_layouts = [
		*frame_data_descriptor_set_layout,
		*instance_data_descriptor_set_layout,
		*instance_data_descriptor_set_layout,
		*instance_data_descriptor_set_layout,
		*instance_data_descriptor_set_layout
	];

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

		let frame_data_buffer = Buffer::new(context, FRAME_DATA_MEMORY_SIZE as u64, vk::BufferUsageFlags::UNIFORM_BUFFER, vk::MemoryPropertyFlags::HOST_VISIBLE);

		let instance_data_buffer = Buffer::null(
			vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::STORAGE_BUFFER,
			vk::MemoryPropertyFlags::HOST_VISIBLE);
		
		let frame_data_descriptor_buffer_info = vk::DescriptorBufferInfo::builder()
			.buffer(frame_data_buffer.handle)
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
			secondary_command_buffer: secondary_command_buffers[4 * index],
			array_offset: 0,
			array_size: 0
		};

		let normal_material_data = MaterialData {
			descriptor_set: descriptor_sets[2],
			secondary_command_buffer: secondary_command_buffers[4 * index + 1],
			array_offset: 0,
			array_size: 0
		};

		let lambert_material_data = MaterialData {
			descriptor_set: descriptor_sets[3],
			secondary_command_buffer: secondary_command_buffers[4 * index + 2],
			array_offset: 0,
			array_size: 0
		};

		let text_material_data = MaterialData {
			descriptor_set: descriptor_sets[4],
			secondary_command_buffer: secondary_command_buffers[4 * index + 3],
			array_offset: 0,
			array_size: 0
		};

		*frame = MaybeUninit::new(InFlightFrame {
			image_available,
			render_finished,
			fence,
			frame_data_descriptor_set,
			primary_command_buffer,
			frame_data_buffer,
			instance_data_buffer,
			basic_material_data,
			normal_material_data,
			lambert_material_data,
			text_material_data,
			index_arrays_offset: 0
		});
	}

	unsafe { transmute::<_, [InFlightFrame; IN_FLIGHT_FRAMES_COUNT]>(frames) }
}