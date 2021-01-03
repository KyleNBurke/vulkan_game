use std::{fs, io, ptr, ffi::CString, io::{Read, Seek}, mem::size_of};
use ash::{vk, version::DeviceV1_0};
use crate::{Pool, vulkan::{Context, Renderer, Buffer, Font}};

pub const MAX_FONTS: usize = 10;

struct SubmittedFontResources {
	image: vk::Image,
	image_view: vk::ImageView
}

pub struct TextRenderer<'a> {
	context: &'a Context,
	pub sampler_and_atlases_descriptor_set_layout: vk::DescriptorSetLayout,
	pub text_data_descriptor_set_layout: vk::DescriptorSetLayout,
	pub pipeline_layout: vk::PipelineLayout,
	pub pipeline: vk::Pipeline,
	pub sampler_and_atlases_descriptor_set: vk::DescriptorSet,
	pub sampler: vk::Sampler,
	empty_image_memory: vk::DeviceMemory,
	empty_image: vk::Image,
	empty_image_view: vk::ImageView,
	memory: vk::DeviceMemory,
	pub fonts: Pool<Font>,
	submitted_font_resources: Vec<SubmittedFontResources>,
	pub atlas_index_uniform_relative_offset: usize
}

impl<'a> TextRenderer<'a> {
	pub fn new(context: &'a Context, extent: vk::Extent2D, render_pass: vk::RenderPass, descriptor_pool: vk::DescriptorPool, command_pool: vk::CommandPool) -> Self {
		let (sampler_and_atlases_descriptor_set_layout, text_data_descriptor_set_layout) = Self::create_descriptor_set_layouts(&context.logical_device);
		let pipeline_layout = Self::create_pipeline_layout(&context.logical_device, sampler_and_atlases_descriptor_set_layout, text_data_descriptor_set_layout);
		let pipeline = Self::create_pipeline(&context.logical_device, extent, pipeline_layout, render_pass);
		let sampler_and_atlases_descriptor_set = Self::create_descriptor_set(&context.logical_device, sampler_and_atlases_descriptor_set_layout, descriptor_pool);
		let sampler = Self::create_sampler_and_update_descriptor(&context.logical_device, sampler_and_atlases_descriptor_set);
		let (empty_image_memory, empty_image, empty_image_view) = Self::create_empty_image(context, command_pool, sampler_and_atlases_descriptor_set);
		let atlas_index_uniform_relative_offset = Self::calculate_atlas_index_uniform_relative_offset(context.physical_device.min_uniform_buffer_offset_alignment as usize);

		Self {
			context,
			sampler_and_atlases_descriptor_set_layout,
			text_data_descriptor_set_layout,
			pipeline_layout,
			pipeline,
			sampler_and_atlases_descriptor_set,
			sampler,
			empty_image_memory,
			empty_image,
			empty_image_view,
			memory: vk::DeviceMemory::null(),
			fonts: Pool::<Font>::new(),
			submitted_font_resources: vec![],
			atlas_index_uniform_relative_offset
		}
	}

	fn create_descriptor_set_layouts(logical_device: &ash::Device) -> (vk::DescriptorSetLayout, vk::DescriptorSetLayout) {
		// Sampler and atlases set
		let sampler_descriptor_set_layout_binding = vk::DescriptorSetLayoutBinding::builder()
			.binding(0)
			.descriptor_type(vk::DescriptorType::SAMPLER)
			.descriptor_count(1)
			.stage_flags(vk::ShaderStageFlags::FRAGMENT);

		let atlases_descriptor_set_layout_binding = vk::DescriptorSetLayoutBinding::builder()
			.binding(1)
			.descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
			.descriptor_count(MAX_FONTS as u32)
			.stage_flags(vk::ShaderStageFlags::FRAGMENT);

		let sampler_and_atlases_descriptor_set_layout_bindings = [sampler_descriptor_set_layout_binding.build(), atlases_descriptor_set_layout_binding.build()];
		let sampler_and_atlases_descriptor_set_layout_create_info = vk::DescriptorSetLayoutCreateInfo::builder()
			.bindings(&sampler_and_atlases_descriptor_set_layout_bindings);
		
		let sampler_and_atlases_descriptor_set_layout = unsafe { logical_device.create_descriptor_set_layout(&sampler_and_atlases_descriptor_set_layout_create_info, None) }.unwrap();

		// Text data set
		let matrix_descriptor_set_layout_binding = vk::DescriptorSetLayoutBinding::builder()
			.binding(0)
			.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
			.descriptor_count(1)
			.stage_flags(vk::ShaderStageFlags::VERTEX);
		
		let atlas_index_descriptor_set_layout_binding = vk::DescriptorSetLayoutBinding::builder()
			.binding(1)
			.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
			.descriptor_count(1)
			.stage_flags(vk::ShaderStageFlags::FRAGMENT);

		let text_data_descriptor_set_layout_bindings = [matrix_descriptor_set_layout_binding.build(), atlas_index_descriptor_set_layout_binding.build()];
		let text_data_descriptor_set_layout_create_info = vk::DescriptorSetLayoutCreateInfo::builder()
			.bindings(&text_data_descriptor_set_layout_bindings);

		let text_data_descriptor_set_layout = unsafe { logical_device.create_descriptor_set_layout(&text_data_descriptor_set_layout_create_info, None) }.unwrap();

		(sampler_and_atlases_descriptor_set_layout, text_data_descriptor_set_layout)
	}

	fn create_pipeline_layout(logical_device: &ash::Device, sampler_and_atlases_descriptor_set_layout: vk::DescriptorSetLayout, text_data_descriptor_set_layout: vk::DescriptorSetLayout) -> vk::PipelineLayout {
		let descriptor_set_layouts = [sampler_and_atlases_descriptor_set_layout, text_data_descriptor_set_layout];

		let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
			.set_layouts(&descriptor_set_layouts);

		unsafe { logical_device.create_pipeline_layout(&pipeline_layout_create_info, None) }.unwrap()
	}

	fn create_pipeline(logical_device: &ash::Device, extent: vk::Extent2D, pipeline_layout: vk::PipelineLayout, render_pass: vk::RenderPass) -> vk::Pipeline {
		// Create entry point string
		let entry_point = CString::new("main").unwrap();
		let entry_point_cstr = entry_point.as_c_str();

		// Create shader stage create infos
		let vert_module = Renderer::create_shader_module(logical_device, "text.vert.spv");
		let vert_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
			.stage(vk::ShaderStageFlags::VERTEX)
			.module(vert_module)
			.name(entry_point_cstr);
		
		let frag_module =  Renderer::create_shader_module(logical_device, "text.frag.spv");
		let frag_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
			.stage(vk::ShaderStageFlags::FRAGMENT)
			.module(frag_module)
			.name(entry_point_cstr);
		
		let stage_create_infos = [vert_stage_create_info.build(), frag_stage_create_info.build()];

		// Create vertex input state create info
		let input_binding_description = vk::VertexInputBindingDescription::builder()
			.binding(0)
			.stride(16)
			.input_rate(vk::VertexInputRate::VERTEX);
		let input_binding_descriptions = [input_binding_description.build()];

		let input_attribute_description_position = vk::VertexInputAttributeDescription::builder()	
			.binding(0)
			.location(0)
			.format(vk::Format::R32G32_SFLOAT)
			.offset(0)
			.build();
		
		let input_attribute_description_texture_position = vk::VertexInputAttributeDescription::builder()	
			.binding(0)
			.location(1)
			.format(vk::Format::R32G32_SFLOAT)
			.offset(8)
			.build();

		let input_attribute_descriptions = [input_attribute_description_position, input_attribute_description_texture_position];

		let vert_input_state_create_info = vk::PipelineVertexInputStateCreateInfo::builder()
			.vertex_binding_descriptions(&input_binding_descriptions)
			.vertex_attribute_descriptions(&input_attribute_descriptions);
		
		// Create input assembly state create info
		let input_assembly_state_create_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
			.topology(vk::PrimitiveTopology::TRIANGLE_LIST)
			.primitive_restart_enable(false);
		
		// Create viewport state create info
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
		
		// Create rasterization state create info
		let rasterization_state_create_info = vk::PipelineRasterizationStateCreateInfo::builder()
			.depth_clamp_enable(false)
			.rasterizer_discard_enable(false)
			.polygon_mode(vk::PolygonMode::FILL)
			.line_width(1.0)
			.cull_mode(vk::CullModeFlags::BACK)
			.front_face(vk::FrontFace::CLOCKWISE)
			.depth_bias_enable(false);
		
		// Create multisample state create info
		let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo::builder()
			.sample_shading_enable(false)
			.rasterization_samples(vk::SampleCountFlags::TYPE_1);
		
		// Create depth stencil state create info
		let depth_stencil_state_create_info = vk::PipelineDepthStencilStateCreateInfo::builder()
			.depth_test_enable(false)
			.depth_bounds_test_enable(false)
			.stencil_test_enable(false);
		
		// Create color blend state create info
		let color_blend_attachment_state = vk::PipelineColorBlendAttachmentState::builder()
			.color_write_mask(vk::ColorComponentFlags::all())
			.blend_enable(true)
			.src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
			.dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
			.color_blend_op(vk::BlendOp::ADD)
			.src_alpha_blend_factor(vk::BlendFactor::ONE)
			.dst_alpha_blend_factor(vk::BlendFactor::ZERO)
			.alpha_blend_op(vk::BlendOp::ADD);
		let color_blend_attachment_states = [color_blend_attachment_state.build()];

		let color_blend_state_create_info = vk::PipelineColorBlendStateCreateInfo::builder()
			.logic_op_enable(false)
			.attachments(&color_blend_attachment_states);
		
		// Create pipeline
		let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
			.stages(&stage_create_infos)
			.vertex_input_state(&vert_input_state_create_info)
			.input_assembly_state(&input_assembly_state_create_info)
			.viewport_state(&viewport_state_create_info)
			.rasterization_state(&rasterization_state_create_info)
			.multisample_state(&multisample_state_create_info)
			.depth_stencil_state(&depth_stencil_state_create_info)
			.color_blend_state(&color_blend_state_create_info)
			.layout(pipeline_layout)
			.render_pass(render_pass)
			.subpass(0);
		
		let pipeline = unsafe { logical_device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_create_info.build()], None) }.unwrap()[0];

		// Destroy shader modules
		unsafe {
			logical_device.destroy_shader_module(vert_module, None);
			logical_device.destroy_shader_module(frag_module, None);
		}

		pipeline
	}

	fn create_descriptor_set(logical_device: &ash::Device, descriptor_set_layout: vk::DescriptorSetLayout, descriptor_pool: vk::DescriptorPool) -> vk::DescriptorSet {
		let descriptor_set_layouts = [descriptor_set_layout];
		let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
			.descriptor_pool(descriptor_pool)
			.set_layouts(&descriptor_set_layouts);
		
		unsafe { logical_device.allocate_descriptor_sets(&descriptor_set_allocate_info) }.unwrap()[0]
	}

	fn create_sampler_and_update_descriptor(logical_device: &ash::Device, sampler_and_atlases_descriptor_set: vk::DescriptorSet) -> vk::Sampler {
		// Create sampler
		let sampler_create_info = vk::SamplerCreateInfo::builder()
			.mag_filter(vk::Filter::LINEAR)
			.min_filter(vk::Filter::LINEAR)
			.address_mode_u(vk::SamplerAddressMode::CLAMP_TO_BORDER)
			.address_mode_v(vk::SamplerAddressMode::CLAMP_TO_BORDER)
			.address_mode_w(vk::SamplerAddressMode::CLAMP_TO_BORDER)
			.anisotropy_enable(false)
			.border_color(vk::BorderColor::FLOAT_OPAQUE_BLACK)
			.unnormalized_coordinates(true)
			.compare_enable(false)
			.mipmap_mode(vk::SamplerMipmapMode::NEAREST)
			.mip_lod_bias(0.0)
			.min_lod(0.0)
			.max_lod(0.0);
		
		let sampler = unsafe { logical_device.create_sampler(&sampler_create_info, None) }.unwrap();

		// Update descriptor
		let descriptor_image_info = vk::DescriptorImageInfo::builder()
			.sampler(sampler);
		let descriptor_image_infos = [descriptor_image_info.build()];
		
		let write_descriptor_set = vk::WriteDescriptorSet::builder()
			.dst_set(sampler_and_atlases_descriptor_set)
			.dst_binding(0)
			.dst_array_element(0)
			.descriptor_type(vk::DescriptorType::SAMPLER)
			.image_info(&descriptor_image_infos)
			.build();

		unsafe { logical_device.update_descriptor_sets(&[write_descriptor_set], &[]) };

		sampler

	}

	pub fn create_empty_image(context: &Context, command_pool: vk::CommandPool, sampler_and_atlases_descriptor_set: vk::DescriptorSet) -> (vk::DeviceMemory, vk::Image, vk::ImageView) {
		let logical_device = &context.logical_device;

		// Create image
		let image_create_info = vk::ImageCreateInfo::builder()
			.image_type(vk::ImageType::TYPE_2D)
			.extent(vk::Extent3D::builder().width(1).height(1).depth(1).build())
			.mip_levels(1)
			.array_layers(1)
			.format(vk::Format::R8_UNORM)
			.tiling(vk::ImageTiling::OPTIMAL)
			.initial_layout(vk::ImageLayout::UNDEFINED)
			.usage(vk::ImageUsageFlags::SAMPLED)
			.sharing_mode(vk::SharingMode::EXCLUSIVE)
			.samples(vk::SampleCountFlags::TYPE_1);
	
		let image = unsafe { logical_device.create_image(&image_create_info, None) }.unwrap();

		// Create device local buffer
		let image_memory_requirements = unsafe { logical_device.get_image_memory_requirements(image) };
		let memory_type_index = context.physical_device.find_memory_type_index(image_memory_requirements.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL);

		let memory_allocate_info = vk::MemoryAllocateInfo::builder()
			.allocation_size(image_memory_requirements.size)
			.memory_type_index(memory_type_index as u32);
	
		let memory = unsafe { logical_device.allocate_memory(&memory_allocate_info, None) }.unwrap();

		// Bind image to buffer
		unsafe { logical_device.bind_image_memory(image, memory, 0) }.unwrap();

		// Create image view
		let image_view_create_info = vk::ImageViewCreateInfo::builder()
			.image(image)
			.view_type(vk::ImageViewType::TYPE_2D)
			.format(vk::Format::R8_UNORM)
			.subresource_range(vk::ImageSubresourceRange::builder()
				.aspect_mask(vk::ImageAspectFlags::COLOR)
				.base_mip_level(0)
				.level_count(1)
				.base_array_layer(0)
				.layer_count(1)
				.build());
		
		let image_view = unsafe { logical_device.create_image_view(&image_view_create_info, None) }.unwrap();

		// Build a command buffer to transition the layout
		let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
			.level(vk::CommandBufferLevel::PRIMARY)
			.command_pool(command_pool)
			.command_buffer_count(1);

		let command_buffer = unsafe { logical_device.allocate_command_buffers(&command_buffer_allocate_info) }.unwrap()[0];

		let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
			.flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
		
		let image_memory_barrier = vk::ImageMemoryBarrier::builder()
			.old_layout(vk::ImageLayout::UNDEFINED)
			.new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
			.src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
			.dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
			.image(image)
			.subresource_range(vk::ImageSubresourceRange::builder()
				.aspect_mask(vk::ImageAspectFlags::COLOR)
				.base_mip_level(0)
				.level_count(1)
				.base_array_layer(0)
				.layer_count(1)
				.build())
			.src_access_mask(vk::AccessFlags::empty())
			.dst_access_mask(vk::AccessFlags::SHADER_READ)
			.build();
		
		unsafe {
			logical_device.begin_command_buffer(command_buffer, &command_buffer_begin_info).unwrap();
			logical_device.cmd_pipeline_barrier(command_buffer, vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::FRAGMENT_SHADER, vk::DependencyFlags::empty(), &[], &[], &[image_memory_barrier]);
			logical_device.end_command_buffer(command_buffer).unwrap();
		}

		// Submit the command buffer
		let command_buffers = [command_buffer];
		let submit_info = vk::SubmitInfo::builder()
			.command_buffers(&command_buffers);
		
		unsafe {
			logical_device.queue_submit(context.graphics_queue, &[submit_info.build()], vk::Fence::null()).unwrap();
			logical_device.queue_wait_idle(context.graphics_queue).unwrap();
			logical_device.free_command_buffers(command_pool, &command_buffers);
		}

		// Update descriptors
		let mut descriptor_image_infos = Vec::with_capacity(MAX_FONTS);
		for _ in 0..MAX_FONTS {
			let descriptor_image_info = vk::DescriptorImageInfo::builder()
				.image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
				.image_view(image_view)
				.build();
			
			descriptor_image_infos.push(descriptor_image_info);
		}

		let write_descriptor_set = vk::WriteDescriptorSet::builder()
			.dst_set(sampler_and_atlases_descriptor_set)
			.dst_binding(1)
			.dst_array_element(0)
			.descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
			.image_info(&descriptor_image_infos)
			.build();

		unsafe { logical_device.update_descriptor_sets(&[write_descriptor_set], &[]) };

		(memory, image, image_view)
	}

	fn calculate_atlas_index_uniform_relative_offset(min_uniform_buffer_offset_alignment: usize) -> usize {
		let matrix_uniform_size = 12 * size_of::<f32>();
		let matrix_uniform_padding = (min_uniform_buffer_offset_alignment - (matrix_uniform_size % min_uniform_buffer_offset_alignment)) % min_uniform_buffer_offset_alignment;
		matrix_uniform_size + matrix_uniform_padding
	}

	pub fn submit_fonts(&mut self, command_pool: vk::CommandPool) {
		let logical_device = &self.context.logical_device;

		// Free memory and destory resources
		unsafe {
			logical_device.queue_wait_idle(self.context.graphics_queue).unwrap();
			logical_device.free_memory(self.memory, None);
		}

		for font in &self.submitted_font_resources {
			unsafe {
				logical_device.destroy_image(font.image, None);
				logical_device.destroy_image_view(font.image_view, None);
			}
		}

		self.submitted_font_resources.clear();

		struct TempResources<'a> {
			font: &'a Font,
			image: vk::Image,
			image_view: vk::ImageView,
			staging_buffer_offset: usize,
			local_buffer_offset: usize
		};

		// Set the submission index, create images, calculate buffer size and calculate image offsets
		let mut temp_resources: Vec<TempResources> = vec![];
		let mut staging_buffer_offset = 0;
		let mut local_buffer_offset = 0;
		for (index, font) in self.fonts.iter_mut().enumerate() {
			font.submission_index = index;

			let image_create_info = vk::ImageCreateInfo::builder()
				.image_type(vk::ImageType::TYPE_2D)
				.extent(vk::Extent3D::builder().width(font.atlas_width as u32).height(font.atlas_height as u32).depth(1).build())
				.mip_levels(1)
				.array_layers(1)
				.format(vk::Format::R8_UNORM)
				.tiling(vk::ImageTiling::OPTIMAL)
				.initial_layout(vk::ImageLayout::UNDEFINED)
				.usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
				.sharing_mode(vk::SharingMode::EXCLUSIVE)
				.samples(vk::SampleCountFlags::TYPE_1);
		
			let image = unsafe { logical_device.create_image(&image_create_info, None) }.unwrap();

			let image_memory_requirements = unsafe { logical_device.get_image_memory_requirements(image) };
			let alignment = image_memory_requirements.alignment;
			let padding = (alignment - local_buffer_offset % alignment) % alignment;
			local_buffer_offset += padding;

			temp_resources.push(TempResources {
				font,
				image,
				image_view: vk::ImageView::null(),
				staging_buffer_offset,
				local_buffer_offset: local_buffer_offset as usize
			});

			staging_buffer_offset += font.atlas_width * font.atlas_height;
			local_buffer_offset += image_memory_requirements.size;
		}

		// If there are no fonts to submit, fill in all atlas descriptor slots with the empty image and return
		if temp_resources.len() == 0 {
			let mut descriptor_image_infos = Vec::with_capacity(MAX_FONTS);

			for _ in 0..MAX_FONTS {
				let descriptor_image_info = vk::DescriptorImageInfo::builder()
					.image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
					.image_view(self.empty_image_view)
					.build();
				
				descriptor_image_infos.push(descriptor_image_info);
			}
	
			let write_descriptor_set = vk::WriteDescriptorSet::builder()
				.dst_set(self.sampler_and_atlases_descriptor_set)
				.dst_binding(1)
				.dst_array_element(0)
				.descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
				.image_info(&descriptor_image_infos)
				.build();
	
			unsafe { logical_device.update_descriptor_sets(&[write_descriptor_set], &[]) };

			return;
		}

		// Ensure there are not more fonts than what's allowed
		assert!(temp_resources.len() < MAX_FONTS, "{} fonts is more than the allowed {}", temp_resources.len(), MAX_FONTS);

		// Create staging buffer
		let staging_buffer = Buffer::new(self.context, staging_buffer_offset as u64, vk::BufferUsageFlags::TRANSFER_SRC, vk::MemoryPropertyFlags::HOST_VISIBLE);

		// Copy atlases into staging buffer
		let staging_buffer_ptr = unsafe { logical_device.map_memory(staging_buffer.memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty()) }.unwrap();

		for resource in &temp_resources {
			unsafe {
				let mut file = fs::File::open(&resource.font.fnt_path).unwrap();
				file.seek(io::SeekFrom::Start(2 * size_of::<u32>() as u64)).unwrap();
				let mut atlas = vec![0u8; resource.font.atlas_width * resource.font.atlas_height];
				file.read_exact(&mut atlas).unwrap();

				let dst_ptr = staging_buffer_ptr.add(resource.staging_buffer_offset) as *mut u8;
				ptr::copy_nonoverlapping(atlas.as_ptr(), dst_ptr, resource.font.atlas_width * resource.font.atlas_height);
			}
		}

		let flush_range = vk::MappedMemoryRange::builder()
			.memory(staging_buffer.memory)
			.offset(0)
			.size(vk::WHOLE_SIZE);
		
		unsafe {
			logical_device.flush_mapped_memory_ranges(&[flush_range.build()]).unwrap();
			logical_device.unmap_memory(staging_buffer.memory);
		}

		// Create device local buffer
		let first_image = temp_resources[0].image;
		let first_image_memory_requirements = unsafe { logical_device.get_image_memory_requirements(first_image) };
		let memory_type_index = self.context.physical_device.find_memory_type_index(first_image_memory_requirements.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL);

		let memory_allocate_info = vk::MemoryAllocateInfo::builder()
			.allocation_size(local_buffer_offset as u64)
			.memory_type_index(memory_type_index as u32);
	
		self.memory = unsafe { logical_device.allocate_memory(&memory_allocate_info, None) }.unwrap();

		// Bind the images to the device local buffer at the corresponding offset & create image views
		for resource in &mut temp_resources {
			unsafe { logical_device.bind_image_memory(resource.image, self.memory, resource.local_buffer_offset as u64) }.unwrap();

			let image_view_create_info = vk::ImageViewCreateInfo::builder()
				.image(resource.image)
				.view_type(vk::ImageViewType::TYPE_2D)
				.format(vk::Format::R8_UNORM)
				.subresource_range(vk::ImageSubresourceRange::builder()
					.aspect_mask(vk::ImageAspectFlags::COLOR)
					.base_mip_level(0)
					.level_count(1)
					.base_array_layer(0)
					.layer_count(1)
					.build());
			
			resource.image_view = unsafe { logical_device.create_image_view(&image_view_create_info, None) }.unwrap();
		}

		// Build a command buffer to copy the images from the staging buffer to the device local buffer
		let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
			.level(vk::CommandBufferLevel::PRIMARY)
			.command_pool(command_pool)
			.command_buffer_count(1);

		let command_buffer = unsafe { logical_device.allocate_command_buffers(&command_buffer_allocate_info) }.unwrap()[0];

		let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
			.flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
		
		let mut transfer_image_memory_barriers = vec![];
		let mut shader_read_image_memory_barriers = vec![];
		for resource in &temp_resources {
			let transfer_image_memory_barrier = vk::ImageMemoryBarrier::builder()
				.old_layout(vk::ImageLayout::UNDEFINED)
				.new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
				.src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
				.dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
				.image(resource.image)
				.subresource_range(vk::ImageSubresourceRange::builder()
					.aspect_mask(vk::ImageAspectFlags::COLOR)
					.base_mip_level(0)
					.level_count(1)
					.base_array_layer(0)
					.layer_count(1)
					.build())
				.src_access_mask(vk::AccessFlags::empty())
				.dst_access_mask(vk::AccessFlags::TRANSFER_WRITE);
			
			transfer_image_memory_barriers.push(transfer_image_memory_barrier.build());

			let shader_read_image_memory_barrier = vk::ImageMemoryBarrier::builder()
				.old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
				.new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
				.src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
				.dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
				.image(resource.image)
				.subresource_range(vk::ImageSubresourceRange::builder()
					.aspect_mask(vk::ImageAspectFlags::COLOR)
					.base_mip_level(0)
					.level_count(1)
					.base_array_layer(0)
					.layer_count(1)
					.build())
				.src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
				.dst_access_mask(vk::AccessFlags::SHADER_READ);
			
			shader_read_image_memory_barriers.push(shader_read_image_memory_barrier.build());
		}

		unsafe {
			logical_device.begin_command_buffer(command_buffer, &command_buffer_begin_info).unwrap();
			logical_device.cmd_pipeline_barrier(command_buffer, vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER, vk::DependencyFlags::empty(), &[], &[], &transfer_image_memory_barriers);
		}

		for resource in &temp_resources {
			let region = vk::BufferImageCopy::builder()
				.buffer_offset(resource.staging_buffer_offset as u64)
				.buffer_row_length(0)
				.buffer_image_height(0)
				.image_subresource(vk::ImageSubresourceLayers::builder()
					.aspect_mask(vk::ImageAspectFlags::COLOR)
					.mip_level(0)
					.base_array_layer(0)
					.layer_count(1)
					.build())
				.image_offset(vk::Offset3D::builder().x(0).y(0).z(0).build())
				.image_extent(vk::Extent3D::builder().width(resource.font.atlas_width as u32).height(resource.font.atlas_height as u32).depth(1).build());

			unsafe { logical_device.cmd_copy_buffer_to_image(command_buffer, staging_buffer.handle, resource.image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[region.build()]) };
		}

		unsafe {
			logical_device.cmd_pipeline_barrier(command_buffer, vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::FRAGMENT_SHADER, vk::DependencyFlags::empty(), &[], &[], &shader_read_image_memory_barriers);
			logical_device.end_command_buffer(command_buffer).unwrap();
		}

		// Submit the command buffer
		let command_buffers = [command_buffer];
		let submit_info = vk::SubmitInfo::builder()
			.command_buffers(&command_buffers);
		
		unsafe {
			logical_device.queue_submit(self.context.graphics_queue, &[submit_info.build()], vk::Fence::null()).unwrap();
			logical_device.queue_wait_idle(self.context.graphics_queue).unwrap();
			logical_device.free_command_buffers(command_pool, &command_buffers);
		}

		// Update atlases descriptor to reference device local buffer
		// Fill empty slots in the atlases descriptor set array with the empty image
		let mut descriptor_image_infos = Vec::with_capacity(MAX_FONTS);
		for resouce in &temp_resources {
			let descriptor_image_info = vk::DescriptorImageInfo::builder()
				.image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
				.image_view(resouce.image_view)
				.build();
			
			descriptor_image_infos.push(descriptor_image_info);
		}

		for _ in 0..(MAX_FONTS - temp_resources.len()) {
			let descriptor_image_info = vk::DescriptorImageInfo::builder()
				.image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
				.image_view(self.empty_image_view)
				.build();
			
			descriptor_image_infos.push(descriptor_image_info);
		}

		let write_descriptor_set = vk::WriteDescriptorSet::builder()
			.dst_set(self.sampler_and_atlases_descriptor_set)
			.dst_binding(1)
			.dst_array_element(0)
			.descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
			.image_info(&descriptor_image_infos)
			.build();

		unsafe { logical_device.update_descriptor_sets(&[write_descriptor_set], &[]) };

		for resource in temp_resources {
			self.submitted_font_resources.push(SubmittedFontResources {
				image: resource.image,
				image_view: resource.image_view
			});
		}
	}

	pub fn handle_resize(&mut self, extent: vk::Extent2D, render_pass: vk::RenderPass) {
		unsafe { self.context.logical_device.destroy_pipeline(self.pipeline, None) };
		self.pipeline = Self::create_pipeline(&self.context.logical_device, extent, self.pipeline_layout, render_pass);
	}
}

impl<'a> Drop for TextRenderer<'a> {
	fn drop(&mut self) {
		let logical_device = &self.context.logical_device;

		unsafe {
			logical_device.destroy_descriptor_set_layout(self.sampler_and_atlases_descriptor_set_layout, None);
			logical_device.destroy_descriptor_set_layout(self.text_data_descriptor_set_layout, None);
			logical_device.destroy_pipeline_layout(self.pipeline_layout, None);
			logical_device.destroy_pipeline(self.pipeline, None);
			logical_device.destroy_sampler(self.sampler, None);
			logical_device.free_memory(self.empty_image_memory, None);
			logical_device.destroy_image(self.empty_image, None);
			logical_device.destroy_image_view(self.empty_image_view, None);
			logical_device.free_memory(self.memory, None);
		}

		for font in &self.submitted_font_resources {
			unsafe {
				logical_device.destroy_image(font.image, None);
				logical_device.destroy_image_view(font.image_view, None);
			}
		}
	}
}