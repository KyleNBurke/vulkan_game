use std::{cmp::max, convert::TryInto, fs, mem::{self, MaybeUninit, transmute, size_of, size_of_val}, ptr::copy_nonoverlapping};
use ash::{vk, version::DeviceV1_0, version::InstanceV1_0, extensions::khr};
use crate::{
	Geometry3D,
	math::{Matrix3, Matrix4, Vector3},
	Mesh,
	Material,
	pool::{Pool, Handle},
	scene::Scene,
	vulkan::{Context, Buffer, MeshManager, TextManager, text_manager::MAX_FONTS, Font}};

const IN_FLIGHT_FRAMES_COUNT: usize = 2;
const FRAME_DATA_MEMORY_SIZE: usize = 76 * size_of::<f32>();
const MAX_POINT_LIGHTS: usize = 5;

pub struct Renderer {
	context: Context,
	render_pass: vk::RenderPass,
	swapchain: Swapchain,
	descriptor_pool: vk::DescriptorPool,
	command_pool: vk::CommandPool,
	mesh_manager: MeshManager,
	text_manager: TextManager,
	in_flight_frames: [InFlightFrame; IN_FLIGHT_FRAMES_COUNT],
	current_in_flight_frame: usize,
	current_group_index: usize,
	submit_fonts: bool,
	inverse_view_matrix: Matrix4,
	ui_projection_matrix: Matrix3
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

struct InFlightFrame {
	image_available: vk::Semaphore,
	render_finished: vk::Semaphore,
	fence: vk::Fence,
	frame_data_descriptor_set: vk::DescriptorSet,
	primary_command_buffer: vk::CommandBuffer,
	frame_data_buffer: Buffer,
	mesh_data_buffer: Buffer,
	basic_material_data: MaterialData,
	normal_material_data: MaterialData,
	lambert_material_data: MaterialData,
	index_arrays_offset: usize,
}

#[derive(Clone, Copy)]
struct MaterialData {
	descriptor_set: vk::DescriptorSet,
	secondary_command_buffer: vk::CommandBuffer,
	static_secondary_command_buffer: vk::CommandBuffer,
	array_offset: usize,
	array_size: usize
}

impl InFlightFrame {
	fn update_descriptor_sets(
		&mut self,
		logical_device: &ash::Device,
		basic_instance_data_array_offset: usize,
		basic_instance_data_array_size: usize,
		normal_instance_data_array_offset: usize,
		normal_instance_data_array_size: usize,
		lambert_instance_data_array_offset: usize,
		lambert_instance_data_array_size: usize,
		index_arrays_offset: usize)
	{
		// Basic
		let basic_descriptor_buffer_info = vk::DescriptorBufferInfo::builder()
			.buffer(self.mesh_data_buffer.handle)
			.offset(basic_instance_data_array_offset as u64)
			.range(max(1, basic_instance_data_array_size) as u64);
		let basic_descriptor_buffer_infos = [basic_descriptor_buffer_info.build()];

		let basic_write_descriptor_set = vk::WriteDescriptorSet::builder()
			.dst_set(self.basic_material_data.descriptor_set)
			.dst_binding(0)
			.dst_array_element(0)
			.descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
			.buffer_info(&basic_descriptor_buffer_infos);
		
		// Normal
		let normal_descriptor_buffer_info = vk::DescriptorBufferInfo::builder()
			.buffer(self.mesh_data_buffer.handle)
			.offset(normal_instance_data_array_offset as u64)
			.range(max(1, normal_instance_data_array_size) as u64);
		let normal_descriptor_buffer_infos = [normal_descriptor_buffer_info.build()];

		let normal_write_descriptor_set = vk::WriteDescriptorSet::builder()
			.dst_set(self.normal_material_data.descriptor_set)
			.dst_binding(0)
			.dst_array_element(0)
			.descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
			.buffer_info(&normal_descriptor_buffer_infos);
		
		// Lambert
		let lambert_descriptor_buffer_info = vk::DescriptorBufferInfo::builder()
			.buffer(self.mesh_data_buffer.handle)
			.offset(lambert_instance_data_array_offset as u64)
			.range(max(1, lambert_instance_data_array_size) as u64);
		let lambert_descriptor_buffer_infos = [lambert_descriptor_buffer_info.build()];

		let lambert_write_descriptor_set = vk::WriteDescriptorSet::builder()
			.dst_set(self.lambert_material_data.descriptor_set)
			.dst_binding(0)
			.dst_array_element(0)
			.descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
			.buffer_info(&lambert_descriptor_buffer_infos);
		
		// Update descriptor sets
		let write_descriptor_sets = [
			basic_write_descriptor_set.build(),
			normal_write_descriptor_set.build(),
			lambert_write_descriptor_set.build()
		];
		
		unsafe { logical_device.update_descriptor_sets(&write_descriptor_sets, &[]) };

		// Set offsets and sizes
		self.basic_material_data.array_offset = basic_instance_data_array_offset;
		self.basic_material_data.array_size = basic_instance_data_array_size;

		self.normal_material_data.array_offset = normal_instance_data_array_offset;
		self.normal_material_data.array_size = normal_instance_data_array_size;

		self.lambert_material_data.array_offset = lambert_instance_data_array_offset;
		self.lambert_material_data.array_size = lambert_instance_data_array_size;

		self.index_arrays_offset = index_arrays_offset;
	}
}

impl Renderer {
	pub fn new(glfw: &glfw::Glfw, window: &glfw::Window) -> Self {
		let context = Context::new(glfw, window);
		let render_pass = Self::create_render_pass(&context);
		let (framebuffer_width, framebuffer_height) = window.get_framebuffer_size();
		let swapchain = Self::create_swapchain(&context, framebuffer_width as u32, framebuffer_height as u32, render_pass);
		let descriptor_pool = Self::create_descriptor_pool(&context);
		let command_pool = Self::create_command_pool(&context);
		let mesh_manager = MeshManager::new(&context.logical_device, swapchain.extent, render_pass, descriptor_pool);
		let text_manager = TextManager::new(&context, swapchain.extent, render_pass, descriptor_pool, command_pool);

		let in_flight_frames = Self::create_in_flight_frames(
			&context,
			&descriptor_pool,
			&command_pool,
			&mesh_manager.frame_data_descriptor_set_layout,
			&mesh_manager.instance_data_descriptor_set_layout,
			&text_manager.text_data_descriptor_set_layout);
		
		let ui_projection_matrix = Matrix3::from([
			[2.0 / framebuffer_width as f32, 0.0, -1.0],
			[0.0, 2.0 / framebuffer_height as f32, -1.0],
			[0.0, 0.0, 1.0]
		]);

		Self {
			context,
			render_pass,
			swapchain,
			descriptor_pool,
			command_pool,
			mesh_manager,
			text_manager,
			in_flight_frames,
			current_in_flight_frame: 0,
			current_group_index: 0,
			submit_fonts: false,
			inverse_view_matrix: Matrix4::new(),
			ui_projection_matrix
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

	fn create_swapchain(context: &Context, framebuffer_width: u32, framebuffer_height: u32, render_pass: vk::RenderPass) -> Swapchain {
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

	fn create_descriptor_pool(context: &Context) -> vk::DescriptorPool {
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
			let sampler_pool_size = vk::DescriptorPoolSize::builder()
				.ty(vk::DescriptorType::SAMPLER)
				.descriptor_count(1);
			
			// The array of font atlases
			let sampled_image_pool_size = vk::DescriptorPoolSize::builder()
				.ty(vk::DescriptorType::SAMPLED_IMAGE)
				.descriptor_count(MAX_FONTS as u32);
		
		let pool_sizes = [
			uniform_buffer_pool_size.build(),
			uniform_buffer_dynamic_pool_size.build(),
			sampler_pool_size.build(),
			sampled_image_pool_size.build()
		];
		
		let create_info = vk::DescriptorPoolCreateInfo::builder()
			.pool_sizes(&pool_sizes)

			// 3 times each in flight frame for the frame data, mesh data & text data
			// One for the static mesh data
			// One for the text sampler and atlas textures
			.max_sets(20);
		
		unsafe { context.logical_device.create_descriptor_pool(&create_info, None).unwrap() }
	}

	fn create_command_pool(context: &Context) -> vk::CommandPool {
		let create_info = vk::CommandPoolCreateInfo::builder()
			.queue_family_index(context.physical_device.graphics_queue_family)
			.flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

		unsafe { context.logical_device.create_command_pool(&create_info, None).unwrap() }
	}

	pub fn create_shader_module(logical_device: &ash::Device, filename: &str) -> vk::ShaderModule {
		let mut file_path = String::from("target/shaders/");
		file_path.push_str(filename);
	
		let mut file = fs::File::open(file_path).unwrap();
		let file_contents = ash::util::read_spv(&mut file).unwrap();
	
		let create_info = vk::ShaderModuleCreateInfo::builder()
			.code(&file_contents);
	
		unsafe { logical_device.create_shader_module(&create_info, None).unwrap() }
	}

	fn create_in_flight_frames(
		context: &Context,
		descriptor_pool: &vk::DescriptorPool,
		command_pool: &vk::CommandPool,
		frame_data_descriptor_set_layout: &vk::DescriptorSetLayout,
		instance_data_descriptor_set_layout: &vk::DescriptorSetLayout,
		text_data_descriptor_set_layout: &vk::DescriptorSetLayout) -> [InFlightFrame; IN_FLIGHT_FRAMES_COUNT]
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

			let frame_data_buffer = Buffer::new(context, FRAME_DATA_MEMORY_SIZE as u64, vk::BufferUsageFlags::UNIFORM_BUFFER, vk::MemoryPropertyFlags::HOST_VISIBLE);

			let mesh_data_buffer = Buffer::new(
				context,
				1,
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
				secondary_command_buffer: secondary_command_buffers[6 * index],
				static_secondary_command_buffer: secondary_command_buffers[6 * index + 1],
				array_offset: 0,
				array_size: 0
			};

			let normal_material_data = MaterialData {
				descriptor_set: descriptor_sets[2],
				secondary_command_buffer: secondary_command_buffers[6 * index + 2],
				static_secondary_command_buffer: secondary_command_buffers[6 * index + 3],
				array_offset: 0,
				array_size: 0
			};

			let lambert_material_data = MaterialData {
				descriptor_set: descriptor_sets[3],
				secondary_command_buffer: secondary_command_buffers[6 * index + 4],
				static_secondary_command_buffer: secondary_command_buffers[6 * index + 5],
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
				mesh_data_buffer,
				basic_material_data,
				normal_material_data,
				lambert_material_data,
				index_arrays_offset: 0
			});
		}

		unsafe { transmute::<_, [InFlightFrame; IN_FLIGHT_FRAMES_COUNT]>(frames) }
	}

	pub fn handle_resize(&mut self, framebuffer_width: i32, framebuffer_height: i32) {
		let logical_device = &self.context.logical_device;

		unsafe {
			logical_device.device_wait_idle().unwrap();

			for frame in &self.swapchain.frames {
				logical_device.destroy_framebuffer(frame.framebuffer, None);
				logical_device.destroy_image_view(frame.image_view, None);
			}

			logical_device.free_memory(self.swapchain.depth_image_resources.memory, None);
			logical_device.destroy_image_view(self.swapchain.depth_image_resources.image_view, None);
			logical_device.destroy_image(self.swapchain.depth_image_resources.image, None);
			self.swapchain.extension.destroy_swapchain(self.swapchain.handle, None);
		}

		self.swapchain = Self::create_swapchain(&self.context, framebuffer_width as u32, framebuffer_height as u32, self.render_pass);
		self.mesh_manager.handle_resize(&self.context.logical_device, self.swapchain.extent, self.render_pass);
		self.text_manager.handle_resize(&self.context.logical_device, self.swapchain.extent, self.render_pass);

		self.ui_projection_matrix.elements[0][0] = 2.0 / framebuffer_width as f32;
		self.ui_projection_matrix.elements[1][1] = 2.0 / framebuffer_height as f32;

		println!("Swapchain recreated");
	}

	pub fn submit_static_meshes(&mut self, geometries: &Pool<Geometry3D>, meshes: &mut [Mesh]) {
		let logical_device = &self.context.logical_device;

		// Iterate over meshes to
		// - Group meshes together which share the same geometry and material
		// - Calculate the offsets and size of the data
		struct GeometryGroup<'a> {
			index_array_relative_offset: usize,
			attribute_array_relative_offset: usize,
			material_groups: Vec<Vec<&'a Mesh>>,
			copied: bool
		}

		let mut geometry_groups: Vec<GeometryGroup> = vec![];
		let mut index_arrays_size = 0;
		let mut attribute_arrays_size = 0;
		let mut instance_counts = [0; 3];

		for mesh in meshes {
			let other_group_index = (self.current_group_index + 1) % 2;
			mesh.geometry_group_indices[other_group_index] = None;
			mesh.material_group_indices[other_group_index] = None;

			let current_geometry_group_index = &mut mesh.geometry_group_indices[self.current_group_index];
			let current_material_group_index = &mut mesh.material_group_indices[self.current_group_index];

			if let Some(geometry_group_index) = current_geometry_group_index {
				let geometry_group = &mut geometry_groups[*geometry_group_index];

				if let Some(material_group_index) = current_material_group_index {
					geometry_group.material_groups[*material_group_index].push(mesh);
				}
				else {
					*current_material_group_index = Some(geometry_group.material_groups.len());

					geometry_group.material_groups.push(vec![mesh]);
				}
			}
			else {
				*current_geometry_group_index = Some(geometry_groups.len());
				*current_material_group_index = Some(0);

				let geometry_group = GeometryGroup {
					index_array_relative_offset: index_arrays_size,
					attribute_array_relative_offset: attribute_arrays_size,
					material_groups: vec![vec![mesh]],
					copied: false
				};

				geometry_groups.push(geometry_group);

				let geometry = geometries.get(&mesh.geometry_handle).unwrap();
				index_arrays_size += size_of_val(&geometry.indices[..]);
				attribute_arrays_size += size_of_val(&geometry.attributes[..]);
			}

			instance_counts[mesh.material as usize] += 1;
		}

		let basic_instance_data_array_offset = 0;
		let basic_instance_data_array_size = 4 * 16 * instance_counts[Material::Basic as usize];

		let normal_instance_data_array_offset = basic_instance_data_array_offset + basic_instance_data_array_size;
		let normal_instance_data_array_size = 4 * 16 * instance_counts[Material::Normal as usize];
		
		let lambert_instance_data_array_offset = normal_instance_data_array_offset + normal_instance_data_array_size;
		let lambert_instance_data_array_size = 4 * 16 * instance_counts[Material::Lambert as usize];

		let index_arrays_offset = lambert_instance_data_array_offset + lambert_instance_data_array_size;
		
		let unaligned_attribute_arrays_offset = index_arrays_offset + index_arrays_size;
		let attribute_arrays_padding = (4 - unaligned_attribute_arrays_offset % 4) % 4;
		let attribute_arrays_offset = unaligned_attribute_arrays_offset + attribute_arrays_padding;

		let buffer_size = (attribute_arrays_offset + attribute_arrays_size) as u64;

		// Create a host visible staging buffer
		let mut staging_buffer = Buffer::new(&self.context, buffer_size, vk::BufferUsageFlags::TRANSFER_SRC, vk::MemoryPropertyFlags::HOST_VISIBLE);

		// Allocate larger device local buffer if necessary and update descriptor sets to reference new buffer
		if buffer_size > self.mesh_manager.static_mesh_buffer.capacity {
			unsafe { logical_device.queue_wait_idle(self.context.graphics_queue) }.unwrap();
			self.mesh_manager.static_mesh_buffer.reallocate(&self.context, buffer_size);
		}

		// Update the descriptor sets to potentially use the new device local buffer and to use the calculated offsets and sizes
		{
			// Basic
			let basic_descriptor_buffer_info = vk::DescriptorBufferInfo::builder()
				.buffer(self.mesh_manager.static_mesh_buffer.handle)
				.offset(basic_instance_data_array_offset as u64)
				.range(max(1, basic_instance_data_array_size) as u64);
			let basic_descriptor_buffer_infos = [basic_descriptor_buffer_info.build()];

			let basic_write_descriptor_set = vk::WriteDescriptorSet::builder()
				.dst_set(self.mesh_manager.static_basic_descriptor_set)
				.dst_binding(0)
				.dst_array_element(0)
				.descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
				.buffer_info(&basic_descriptor_buffer_infos);
			
			// Normal
			let normal_descriptor_buffer_info = vk::DescriptorBufferInfo::builder()
				.buffer(self.mesh_manager.static_mesh_buffer.handle)
				.offset(normal_instance_data_array_offset as u64)
				.range(max(1, normal_instance_data_array_size) as u64);
			let normal_descriptor_buffer_infos = [normal_descriptor_buffer_info.build()];

			let normal_write_descriptor_set = vk::WriteDescriptorSet::builder()
				.dst_set(self.mesh_manager.static_normal_descriptor_set)
				.dst_binding(0)
				.dst_array_element(0)
				.descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
				.buffer_info(&normal_descriptor_buffer_infos);
			
			// Lambert
			let lambert_descriptor_buffer_info = vk::DescriptorBufferInfo::builder()
				.buffer(self.mesh_manager.static_mesh_buffer.handle)
				.offset(lambert_instance_data_array_offset as u64)
				.range(max(1, lambert_instance_data_array_size) as u64);
			let lambert_descriptor_buffer_infos = [lambert_descriptor_buffer_info.build()];

			let lambert_write_descriptor_set = vk::WriteDescriptorSet::builder()
				.dst_set(self.mesh_manager.static_lambert_descriptor_set)
				.dst_binding(0)
				.dst_array_element(0)
				.descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
				.buffer_info(&lambert_descriptor_buffer_infos);
			
			// Update descriptor sets
			let write_descriptor_sets = [
				basic_write_descriptor_set.build(),
				normal_write_descriptor_set.build(),
				lambert_write_descriptor_set.build()
			];
			
			unsafe { logical_device.update_descriptor_sets(&write_descriptor_sets, &[]) };
		}

		// Copy mesh data into staging buffer and record draw commands
		let buffer_ptr = unsafe { logical_device.map_memory(staging_buffer.memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty()).unwrap() };

		let command_buffer_inheritance_info = vk::CommandBufferInheritanceInfo::builder()
			.render_pass(self.render_pass)
			.subpass(0);

		let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
			.flags(vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE)
			.inheritance_info(&command_buffer_inheritance_info);

		println!("{} | {} | {} | {} | {}",
			basic_instance_data_array_size,
			normal_instance_data_array_size,
			lambert_instance_data_array_size,
			index_arrays_size,
			attribute_arrays_size);
		
		for in_flight_frame in &self.in_flight_frames {
			let basic_static_secondary_command_buffer = in_flight_frame.basic_material_data.static_secondary_command_buffer;
			let normal_static_secondary_command_buffer = in_flight_frame.normal_material_data.static_secondary_command_buffer;
			let lambert_static_secondary_command_buffer = in_flight_frame.lambert_material_data.static_secondary_command_buffer;

			// Begin secondary command buffers, bind pipelines and descriptor sets
			unsafe {
				logical_device.begin_command_buffer(basic_static_secondary_command_buffer, &command_buffer_begin_info).unwrap();
				logical_device.cmd_bind_pipeline(basic_static_secondary_command_buffer, vk::PipelineBindPoint::GRAPHICS, self.mesh_manager.basic_pipeline);
				logical_device.cmd_bind_descriptor_sets(
					basic_static_secondary_command_buffer,
					vk::PipelineBindPoint::GRAPHICS,
					self.mesh_manager.pipeline_layout,
					0,
					&[in_flight_frame.frame_data_descriptor_set],
					&[]);
				logical_device.cmd_bind_descriptor_sets(
					basic_static_secondary_command_buffer,
					vk::PipelineBindPoint::GRAPHICS,
					self.mesh_manager.pipeline_layout,
					1,
					&[self.mesh_manager.static_basic_descriptor_set],
					&[]);
				
				logical_device.begin_command_buffer(normal_static_secondary_command_buffer, &command_buffer_begin_info).unwrap();
				logical_device.cmd_bind_pipeline(normal_static_secondary_command_buffer, vk::PipelineBindPoint::GRAPHICS, self.mesh_manager.normal_pipeline);
				logical_device.cmd_bind_descriptor_sets(
					normal_static_secondary_command_buffer,
					vk::PipelineBindPoint::GRAPHICS,
					self.mesh_manager.pipeline_layout,
					0,
					&[in_flight_frame.frame_data_descriptor_set],
					&[]);
				logical_device.cmd_bind_descriptor_sets(
					normal_static_secondary_command_buffer,
					vk::PipelineBindPoint::GRAPHICS,
					self.mesh_manager.pipeline_layout,
					1,
					&[self.mesh_manager.static_normal_descriptor_set],
					&[]);
				
				logical_device.begin_command_buffer(lambert_static_secondary_command_buffer, &command_buffer_begin_info).unwrap();
				logical_device.cmd_bind_pipeline(lambert_static_secondary_command_buffer, vk::PipelineBindPoint::GRAPHICS, self.mesh_manager.lambert_pipeline);
				logical_device.cmd_bind_descriptor_sets(
					lambert_static_secondary_command_buffer,
					vk::PipelineBindPoint::GRAPHICS,
					self.mesh_manager.pipeline_layout,
					0,
					&[in_flight_frame.frame_data_descriptor_set],
					&[]);
				logical_device.cmd_bind_descriptor_sets(
					lambert_static_secondary_command_buffer,
					vk::PipelineBindPoint::GRAPHICS,
					self.mesh_manager.pipeline_layout,
					1,
					&[self.mesh_manager.static_lambert_descriptor_set],
					&[]);
			}

			let mut current_instance_indices = [0; 3];

			for geometry_group in &mut geometry_groups {
				let geometry_handle = &geometry_group.material_groups[0][0].geometry_handle;
				let geometry = geometries.get(geometry_handle).unwrap();

				let index_array_offset = index_arrays_offset + geometry_group.index_array_relative_offset;
				let attribute_array_offset = attribute_arrays_offset + geometry_group.attribute_array_relative_offset;

				// Ensure geometry data is copied into the staging buffer
				if !geometry_group.copied {
					let indices = &geometry.indices;
					let attributes = &geometry.attributes;

					unsafe {
						let index_array_dst_ptr = buffer_ptr.add(index_array_offset) as *mut u16;
						copy_nonoverlapping(indices.as_ptr(), index_array_dst_ptr, indices.len());

						let attribute_array_dst_ptr = buffer_ptr.add(attribute_array_offset) as *mut f32;
						copy_nonoverlapping(attributes.as_ptr(), attribute_array_dst_ptr, attributes.len());
					}

					geometry_group.copied = true;
				}

				for material_group in &geometry_group.material_groups {
					let material = material_group[0].material;
					let current_instance_index = current_instance_indices[material as usize];

					let secondary_command_buffer = match material {
						Material::Basic => in_flight_frame.basic_material_data.static_secondary_command_buffer,
						Material::Normal => in_flight_frame.normal_material_data.static_secondary_command_buffer,
						Material::Lambert => in_flight_frame.lambert_material_data.static_secondary_command_buffer,
					};

					// Record draw commands
					unsafe {
						logical_device.cmd_bind_index_buffer(secondary_command_buffer, self.mesh_manager.static_mesh_buffer.handle, index_array_offset as u64, vk::IndexType::UINT16);
						logical_device.cmd_bind_vertex_buffers(secondary_command_buffer, 0, &[self.mesh_manager.static_mesh_buffer.handle], &[attribute_array_offset as u64]);
						logical_device.cmd_draw_indexed(secondary_command_buffer, geometry.indices.len() as u32, material_group.len() as u32, 0, 0, current_instance_index as u32);
					}

					// Ensure the instance data is copied into the staging buffer
					for instance in material_group {
						match material {
							Material::Basic => {
								let instance_data_offset = basic_instance_data_array_offset + 4 * 16 * current_instance_index;

								unsafe {
									let instance_data_dst_ptr = buffer_ptr.add(instance_data_offset) as *mut [f32; 4];
									copy_nonoverlapping(instance.transform.matrix.elements.as_ptr(), instance_data_dst_ptr, 4);
								}
							},
							Material::Normal => {
								let instance_data_offset = normal_instance_data_array_offset + 4 * 16 * current_instance_index;

								unsafe {
									let instance_data_dst_ptr = buffer_ptr.add(instance_data_offset) as *mut [f32; 4];
									copy_nonoverlapping(instance.transform.matrix.elements.as_ptr(), instance_data_dst_ptr, 4);
								}
							},
							Material::Lambert => {
								let instance_data_offset = lambert_instance_data_array_offset + 4 * 16 * current_instance_index;

								unsafe {
									let instance_data_dst_ptr = buffer_ptr.add(instance_data_offset) as *mut [f32; 4];
									copy_nonoverlapping(instance.transform.matrix.elements.as_ptr(), instance_data_dst_ptr, 4);
								}
							}
						}
					}

					current_instance_indices[material as usize] += material_group.len();
				}
			}

			// End command buffers
			unsafe {
				logical_device.end_command_buffer(basic_static_secondary_command_buffer).unwrap();
				logical_device.end_command_buffer(normal_static_secondary_command_buffer).unwrap();
				logical_device.end_command_buffer(lambert_static_secondary_command_buffer).unwrap();
			}
		}

		// Flush and unmap staging buffer
		let range = vk::MappedMemoryRange::builder()
			.memory(staging_buffer.memory)
			.offset(0)
			.size(vk::WHOLE_SIZE);

		unsafe {
			logical_device.flush_mapped_memory_ranges(&[range.build()]).unwrap();
			logical_device.unmap_memory(staging_buffer.memory);
		}

		// Record a command buffer to copy the data from the staging buffer to the device local buffer
		let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
			.level(vk::CommandBufferLevel::PRIMARY)
			.command_pool(self.command_pool)
			.command_buffer_count(1);

		let command_buffer = unsafe { logical_device.allocate_command_buffers(&command_buffer_allocate_info) }.unwrap()[0];

		let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
			.flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
		
		let region = vk::BufferCopy::builder()
			.size(buffer_size);
		
		unsafe {
			logical_device.begin_command_buffer(command_buffer, &command_buffer_begin_info).unwrap();
			logical_device.cmd_copy_buffer(command_buffer, staging_buffer.handle, self.mesh_manager.static_mesh_buffer.handle, &[region.build()]);
			logical_device.end_command_buffer(command_buffer).unwrap();
		}

		// Submit the command buffer
		let command_buffers = [command_buffer];
		let submit_info = vk::SubmitInfo::builder()
			.command_buffers(&command_buffers);
		
		unsafe {
			logical_device.queue_wait_idle(self.context.graphics_queue).unwrap();
			logical_device.queue_submit(self.context.graphics_queue, &[submit_info.build()], vk::Fence::null()).unwrap();
			logical_device.queue_wait_idle(self.context.graphics_queue).unwrap();
			logical_device.free_command_buffers(self.command_pool, &command_buffers);
		}

		staging_buffer.drop(logical_device);
		self.current_group_index = (self.current_group_index + 1) % 2;
	}

	pub fn add_font(&mut self, file_path: &str, size: u32) -> Handle<Font> {
		self.submit_fonts = true;
		self.text_manager.fonts.add(Font::new(file_path, size))
	}

	pub fn remove_font(&mut self, handle: &Handle<Font>) {
		self.text_manager.fonts.remove(handle);
		self.submit_fonts = true;
	}

	pub fn render(&mut self, scene: &mut Scene) -> bool {
		// If new fonts have been added or removed, submit them
		if self.submit_fonts {
			self.text_manager.submit_fonts(&self.context, self.command_pool);
			self.submit_fonts = false;
		}

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

		match result {
			Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => return true,
			Err(e) => panic!("Could not aquire a swapchain image: {}", e),
			_ => ()
		}

		let image_index = result.unwrap().0;
		let swapchain_frame = &mut self.swapchain.frames[image_index as usize];

		// Wait for swapchain frame to become available
		if swapchain_frame.fence != vk::Fence::null() {
			let fences = [swapchain_frame.fence];
			unsafe { logical_device.wait_for_fences(&fences, true, std::u64::MAX).unwrap() };
		}

		swapchain_frame.fence = in_flight_frame.fence;

		// Copy frame data into frame data buffer
		{
			let buffer_ptr = unsafe { logical_device.map_memory(in_flight_frame.frame_data_buffer.memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty()) }.unwrap();
			
			let camera = &mut scene.camera;
			let projection_matrix = &camera.projection_matrix.elements;
			let projection_matrix_dst_ptr = buffer_ptr as *mut [f32; 4];
			unsafe { copy_nonoverlapping(projection_matrix.as_ptr(), projection_matrix_dst_ptr, projection_matrix.len()) };

			if camera.auto_update_view_matrix {
				camera.transform.update_matrix();
			}

			self.inverse_view_matrix = camera.transform.matrix;
			self.inverse_view_matrix.invert();
			unsafe {
				let inverse_view_matrix_dst_ptr = buffer_ptr.add(16 * size_of::<f32>()) as *mut [f32; 4];
				copy_nonoverlapping(self.inverse_view_matrix.elements.as_ptr(), inverse_view_matrix_dst_ptr, self.inverse_view_matrix.elements.len());
			}

			let ambient_light = &scene.ambient_light;
			let ambient_light_intensified_color = ambient_light.color * ambient_light.intensity;
			unsafe {
				let ambient_light_dst_ptr = buffer_ptr.add(32 * size_of::<f32>()) as *mut Vector3;
				copy_nonoverlapping(&ambient_light_intensified_color as *const Vector3, ambient_light_dst_ptr, 1);
			}

			let mut point_light_count = 0;
			let position_base_offest = 36 * size_of::<f32>();
			let color_base_offest = 40 * size_of::<f32>();
			let stride = 8 * size_of::<f32>();

			for point_light in scene.point_lights.iter() {
				let intensified_color = point_light.color * point_light.intensity;

				unsafe {
					let position_dst_ptr = buffer_ptr.add(position_base_offest + stride * point_light_count) as *mut Vector3;
					copy_nonoverlapping(&point_light.position as *const Vector3, position_dst_ptr, 1);

					let color_dst_ptr = buffer_ptr.add(color_base_offest + stride * point_light_count) as *mut Vector3;
					copy_nonoverlapping(&intensified_color as *const Vector3, color_dst_ptr, 1);
				}

				point_light_count += 1;
			}

			assert!(point_light_count <= MAX_POINT_LIGHTS, "Only {} point lights allowed", MAX_POINT_LIGHTS);
			unsafe {
				let point_light_count_dst_ptr = buffer_ptr.add(35 * size_of::<f32>()) as *mut u32;
				copy_nonoverlapping(&(point_light_count as u32) as *const u32, point_light_count_dst_ptr, 1);
			}

			// Flush and unmap buffer
			let range = vk::MappedMemoryRange::builder()
				.memory(in_flight_frame.frame_data_buffer.memory)
				.offset(0)
				.size(vk::WHOLE_SIZE);
			
			unsafe {		
				logical_device.flush_mapped_memory_ranges(&[range.build()]).unwrap();
				logical_device.unmap_memory(in_flight_frame.frame_data_buffer.memory);
			}
		}

		// Iterate over meshes to
		// - Group meshes together which share the same geometry and material
		// - Calculate the offsets and size of the data
		struct GeometryGroup<'a> {
			index_array_relative_offset: usize,
			attribute_array_relative_offset: usize,
			material_groups: Vec<Vec<&'a Mesh>>,
			copied: bool
		}

		let mut geometry_groups: Vec<GeometryGroup> = vec![];
		let mut index_arrays_size = 0;
		let mut attribute_arrays_size = 0;
		let mut instance_counts = [0; 3];

		for mesh in scene.meshes.iter_mut() {
			let other_group_index = (self.current_group_index + 1) % 2;
			mesh.geometry_group_indices[other_group_index] = None;
			mesh.material_group_indices[other_group_index] = None;

			let current_geometry_group_index = &mut mesh.geometry_group_indices[self.current_group_index];
			let current_material_group_index = &mut mesh.material_group_indices[self.current_group_index];

			if let Some(geometry_group_index) = current_geometry_group_index {
				let geometry_group = &mut geometry_groups[*geometry_group_index];

				if let Some(material_group_index) = current_material_group_index {
					geometry_group.material_groups[*material_group_index].push(mesh);
				}
				else {
					*current_material_group_index = Some(geometry_group.material_groups.len());

					geometry_group.material_groups.push(vec![mesh]);
				}
			}
			else {
				*current_geometry_group_index = Some(geometry_groups.len());
				*current_material_group_index = Some(0);

				let geometry_group = GeometryGroup {
					index_array_relative_offset: index_arrays_size,
					attribute_array_relative_offset: attribute_arrays_size,
					material_groups: vec![vec![mesh]],
					copied: false
				};

				geometry_groups.push(geometry_group);

				let geometry = scene.geometries.get(&mesh.geometry_handle).unwrap();
				index_arrays_size += size_of_val(&geometry.indices[..]);
				attribute_arrays_size += size_of_val(&geometry.attributes[..]);
			}

			instance_counts[mesh.material as usize] += 1;
		}

		let basic_instance_data_array_offset = 0;
		let basic_instance_data_array_size = 4 * 16 * instance_counts[Material::Basic as usize];

		let normal_instance_data_array_offset = basic_instance_data_array_offset + basic_instance_data_array_size;
		let normal_instance_data_array_size = 4 * 16 * instance_counts[Material::Normal as usize];
		
		let lambert_instance_data_array_offset = normal_instance_data_array_offset + normal_instance_data_array_size;
		let lambert_instance_data_array_size = 4 * 16 * instance_counts[Material::Lambert as usize];

		let index_arrays_offset = lambert_instance_data_array_offset + lambert_instance_data_array_size;
		
		let unaligned_attribute_arrays_offset = index_arrays_offset + index_arrays_size;
		let attribute_arrays_padding = (4 - unaligned_attribute_arrays_offset % 4) % 4;
		let attribute_arrays_offset = unaligned_attribute_arrays_offset + attribute_arrays_padding;

		// Allocate larger mesh data buffer and update descriptor sets if necessary
		let buffer_size = (attribute_arrays_offset + attribute_arrays_size) as u64;

		if buffer_size > in_flight_frame.mesh_data_buffer.capacity {
			in_flight_frame.mesh_data_buffer.reallocate(&self.context, buffer_size);

			in_flight_frame.update_descriptor_sets(
				logical_device,
				basic_instance_data_array_offset,
				basic_instance_data_array_size,
				normal_instance_data_array_offset,
				normal_instance_data_array_size,
				lambert_instance_data_array_offset,
				lambert_instance_data_array_size,
				index_arrays_offset);
		}
		else if
			basic_instance_data_array_size > in_flight_frame.basic_material_data.array_size ||
			normal_instance_data_array_size > in_flight_frame.normal_material_data.array_size ||
			lambert_instance_data_array_size > in_flight_frame.lambert_material_data.array_size
		{
			in_flight_frame.update_descriptor_sets(
				logical_device,
				basic_instance_data_array_offset,
				basic_instance_data_array_size,
				normal_instance_data_array_offset,
				normal_instance_data_array_size,
				lambert_instance_data_array_offset,
				lambert_instance_data_array_size,
				index_arrays_offset);
		}

		let basic_material_data = &in_flight_frame.basic_material_data;
		let normal_material_data = &in_flight_frame.normal_material_data;
		let lambert_material_data = &in_flight_frame.lambert_material_data;

		// Copy mesh data into mesh data buffer and record draw commands
		let mesh_data_buffer_ptr = unsafe { logical_device.map_memory(in_flight_frame.mesh_data_buffer.memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty()) }.unwrap();

		let command_buffer_inheritance_info = vk::CommandBufferInheritanceInfo::builder()
			.render_pass(self.render_pass)
			.subpass(0)
			.framebuffer(swapchain_frame.framebuffer);

		let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
			.flags(vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE | vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
			.inheritance_info(&command_buffer_inheritance_info);
		
		unsafe {
			logical_device.begin_command_buffer(basic_material_data.secondary_command_buffer, &command_buffer_begin_info).unwrap();
			logical_device.cmd_bind_pipeline(basic_material_data.secondary_command_buffer, vk::PipelineBindPoint::GRAPHICS, self.mesh_manager.basic_pipeline);
			logical_device.cmd_bind_descriptor_sets(
				basic_material_data.secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.mesh_manager.pipeline_layout,
				0,
				&[in_flight_frame.frame_data_descriptor_set],
				&[]);
			logical_device.cmd_bind_descriptor_sets(
				basic_material_data.secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.mesh_manager.pipeline_layout,
				1,
				&[basic_material_data.descriptor_set],
				&[]);
			
			logical_device.begin_command_buffer(normal_material_data.secondary_command_buffer, &command_buffer_begin_info).unwrap();
			logical_device.cmd_bind_pipeline(normal_material_data.secondary_command_buffer, vk::PipelineBindPoint::GRAPHICS, self.mesh_manager.normal_pipeline);
			logical_device.cmd_bind_descriptor_sets(
				normal_material_data.secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.mesh_manager.pipeline_layout,
				0,
				&[in_flight_frame.frame_data_descriptor_set],
				&[]);
			logical_device.cmd_bind_descriptor_sets(
				normal_material_data.secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.mesh_manager.pipeline_layout,
				1,
				&[normal_material_data.descriptor_set],
				&[]);
			
			logical_device.begin_command_buffer(lambert_material_data.secondary_command_buffer, &command_buffer_begin_info).unwrap();
			logical_device.cmd_bind_pipeline(lambert_material_data.secondary_command_buffer, vk::PipelineBindPoint::GRAPHICS, self.mesh_manager.lambert_pipeline);
			logical_device.cmd_bind_descriptor_sets(
				lambert_material_data.secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.mesh_manager.pipeline_layout,
				0,
				&[in_flight_frame.frame_data_descriptor_set],
				&[]);
			logical_device.cmd_bind_descriptor_sets(
				lambert_material_data.secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.mesh_manager.pipeline_layout,
				1,
				&[lambert_material_data.descriptor_set],
				&[]);
		}

		let index_arrays_offset = in_flight_frame.index_arrays_offset;
		let unaligned_attribute_arrays_offset = index_arrays_offset + index_arrays_size;
		let attribute_arrays_padding = (4 - unaligned_attribute_arrays_offset % 4) % 4;
		let attribute_arrays_offset = unaligned_attribute_arrays_offset + attribute_arrays_padding;

		let mut current_instance_indices = [0; 3];

		for geometry_group in &mut geometry_groups {
			let geometry_handle = &geometry_group.material_groups[0][0].geometry_handle;
			let geometry = scene.geometries.get(geometry_handle).unwrap();

			let index_array_offset = index_arrays_offset + geometry_group.index_array_relative_offset;
			let attribute_array_offset = attribute_arrays_offset + geometry_group.attribute_array_relative_offset;

			// Ensure geometry data is copied into the mesh data buffer
			if !geometry_group.copied {
				let indices = &geometry.indices;
				let attributes = &geometry.attributes;

				unsafe {
					let index_array_dst_ptr = mesh_data_buffer_ptr.add(index_array_offset) as *mut u16;
					copy_nonoverlapping(indices.as_ptr(), index_array_dst_ptr, indices.len());

					let attribute_array_dst_ptr = mesh_data_buffer_ptr.add(attribute_array_offset) as *mut f32;
					copy_nonoverlapping(attributes.as_ptr(), attribute_array_dst_ptr, attributes.len());
				}

				geometry_group.copied = true;
			}

			for material_group in &geometry_group.material_groups {
				let material = material_group[0].material;
				let current_instance_index = current_instance_indices[material as usize];

				let secondary_command_buffer = match material {
					Material::Basic => basic_material_data.secondary_command_buffer,
					Material::Normal => normal_material_data.secondary_command_buffer,
					Material::Lambert => lambert_material_data.secondary_command_buffer,
				};

				// Record draw commands
				unsafe {
					logical_device.cmd_bind_index_buffer(secondary_command_buffer, in_flight_frame.mesh_data_buffer.handle, index_array_offset as u64, vk::IndexType::UINT16);
					logical_device.cmd_bind_vertex_buffers(secondary_command_buffer, 0, &[in_flight_frame.mesh_data_buffer.handle], &[attribute_array_offset as u64]);
					logical_device.cmd_draw_indexed(secondary_command_buffer, geometry.indices.len() as u32, material_group.len() as u32, 0, 0, current_instance_index as u32);
				}

				// Ensure the instance data is copied into the mesh data buffer
				for instance in material_group {
					match material {
						Material::Basic => {
							let instance_data_offset = basic_material_data.array_offset + 4 * 16 * current_instance_index;

							unsafe {
								let instance_data_dst_ptr = mesh_data_buffer_ptr.add(instance_data_offset) as *mut [f32; 4];
								copy_nonoverlapping(instance.transform.matrix.elements.as_ptr(), instance_data_dst_ptr, 4);
							}
						},
						Material::Normal => {
							let instance_data_offset = normal_material_data.array_offset + 4 * 16 * current_instance_index;

							unsafe {
								let instance_data_dst_ptr = mesh_data_buffer_ptr.add(instance_data_offset) as *mut [f32; 4];
								copy_nonoverlapping(instance.transform.matrix.elements.as_ptr(), instance_data_dst_ptr, 4);
							}
						},
						Material::Lambert => {
							let instance_data_offset = lambert_material_data.array_offset + 4 * 16 * current_instance_index;

							unsafe {
								let instance_data_dst_ptr = mesh_data_buffer_ptr.add(instance_data_offset) as *mut [f32; 4];
								copy_nonoverlapping(instance.transform.matrix.elements.as_ptr(), instance_data_dst_ptr, 4);
							}
						}
					}
				}

				current_instance_indices[material as usize] += material_group.len();
			}
		}

		// End secondary command buffers, flush and unmap mesh data buffer
		let range = vk::MappedMemoryRange::builder()
			.memory(in_flight_frame.mesh_data_buffer.memory)
			.offset(0)
			.size(vk::WHOLE_SIZE);
		
		unsafe {
			logical_device.end_command_buffer(basic_material_data.secondary_command_buffer).unwrap();
			logical_device.end_command_buffer(normal_material_data.secondary_command_buffer).unwrap();
			logical_device.end_command_buffer(lambert_material_data.secondary_command_buffer).unwrap();
			
			logical_device.flush_mapped_memory_ranges(&[range.build()]).unwrap();
			logical_device.unmap_memory(in_flight_frame.mesh_data_buffer.memory);
		}

		// Record primary command buffer
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
		
		let secondary_command_buffers = [
			basic_material_data.secondary_command_buffer,
			basic_material_data.static_secondary_command_buffer,
			normal_material_data.secondary_command_buffer,
			normal_material_data.static_secondary_command_buffer,
			lambert_material_data.secondary_command_buffer,
			lambert_material_data.static_secondary_command_buffer,
		];
		
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

		unsafe {
			logical_device.reset_fences(&fences).unwrap();
			logical_device.queue_submit(self.context.graphics_queue, &[submit_info.build()], in_flight_frame.fence).unwrap();
		}

		// Wait for render to finish then present swapchain image
		let swapchains = [self.swapchain.handle];
		let image_indices = [image_index];
		let present_info = vk::PresentInfoKHR::builder()
			.wait_semaphores(&render_finished_semaphores)
			.swapchains(&swapchains)
			.image_indices(&image_indices);
		
		let result = unsafe { self.swapchain.extension.queue_present(self.context.graphics_queue, &present_info) };

		let surface_changed = match result {
			Ok(true) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => true,
			Err(e) => panic!("Could not present swapchain image: {}", e),
			_ => false
		};

		self.current_in_flight_frame = (self.current_in_flight_frame + 1) % IN_FLIGHT_FRAMES_COUNT;
		self.current_group_index = (self.current_group_index + 1) % 2;

		surface_changed
	}
}

impl Drop for Renderer {
	fn drop(&mut self) {
		let logical_device = &self.context.logical_device;

		unsafe {
			logical_device.device_wait_idle().unwrap();

			// In flight frames
			for frame in &mut self.in_flight_frames {
				logical_device.destroy_fence(frame.fence, None);
				logical_device.destroy_semaphore(frame.render_finished, None);
				logical_device.destroy_semaphore(frame.image_available, None);
				frame.frame_data_buffer.drop(&self.context.logical_device);
				frame.mesh_data_buffer.drop(&self.context.logical_device);
			}

			self.mesh_manager.drop(&self.context.logical_device);
			self.text_manager.drop(&self.context.logical_device);
			
			logical_device.destroy_command_pool(self.command_pool, None);
			logical_device.destroy_descriptor_pool(self.descriptor_pool, None);

			// Swapchain
			self.swapchain.extension.destroy_swapchain(self.swapchain.handle, None);
			logical_device.destroy_image(self.swapchain.depth_image_resources.image, None);
			logical_device.destroy_image_view(self.swapchain.depth_image_resources.image_view, None);
			logical_device.free_memory(self.swapchain.depth_image_resources.memory, None);

			for frame in &self.swapchain.frames {
				logical_device.destroy_framebuffer(frame.framebuffer, None);
				logical_device.destroy_image_view(frame.image_view, None);
			}

			logical_device.destroy_render_pass(self.render_pass, None);
		}
	}
}