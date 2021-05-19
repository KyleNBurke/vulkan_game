use std::{fs::File, io::{Read, Seek, SeekFrom}, ptr::copy_nonoverlapping, mem::size_of};
use ash::{vk, version::DeviceV1_0};
use crate::{pool::Pool, font::{Font, SubmissionInfo}, vulkan::{Context, Buffer}, math::Matrix3};
use super::MAX_FONTS;

mod creation;
use creation::*;

pub struct TextResources {
	sampler_descriptor_set_layout: vk::DescriptorSetLayout,
	atlases_descriptor_set_layout: vk::DescriptorSetLayout,
	pub pipeline_layout: vk::PipelineLayout,
	pub pipeline: vk::Pipeline,
	pub sampler_descriptor_set: vk::DescriptorSet,
	pub atlases_descriptor_set: vk::DescriptorSet,
	sampler: vk::Sampler,
	memory: vk::DeviceMemory,
	atlases: Vec<Atlas>,
	empty_image: vk::Image,
	empty_image_view: vk::ImageView,
	pub submission_generation: usize,
	pub projection_matrix: Matrix3
}

struct Atlas {
	image: vk::Image,
	image_view: vk::ImageView
}

impl TextResources {
	pub fn new(logical_device: &ash::Device, instance_data_descriptor_set_layout: vk::DescriptorSetLayout, extent: vk::Extent2D, render_pass: vk::RenderPass, descriptor_pool: vk::DescriptorPool) -> Self {
		let sampler_descriptor_set_layout = create_sampler_descriptor_set_layout(logical_device);
		let atlases_descriptor_set_layout = create_atlases_descriptor_set_layout(logical_device);
		let pipeline_layout = create_pipeline_layout(logical_device, instance_data_descriptor_set_layout, sampler_descriptor_set_layout, atlases_descriptor_set_layout);
		let pipeline = create_pipeline(logical_device, extent, pipeline_layout, render_pass);
		let descriptor_sets = create_descriptor_sets(logical_device, sampler_descriptor_set_layout, atlases_descriptor_set_layout, descriptor_pool);
		let sampler = create_sampler(logical_device);
		update_sampler(logical_device, sampler, descriptor_sets[0]);

		let projection_matrix = Matrix3::from([
			[2.0 / extent.width as f32, 0.0, -1.0],
			[0.0, 2.0 / extent.height as f32, -1.0],
			[0.0, 0.0, 1.0]]);

		Self {
			sampler_descriptor_set_layout,
			atlases_descriptor_set_layout,
			pipeline_layout,
			pipeline,
			sampler_descriptor_set: descriptor_sets[0],
			atlases_descriptor_set: descriptor_sets[1],
			sampler,
			memory: vk::DeviceMemory::null(),
			atlases: vec![],
			empty_image: vk::Image::null(),
			empty_image_view: vk::ImageView::null(),
			submission_generation: 0,
			projection_matrix
		}
	}

	pub fn resize(&mut self, logical_device: &ash::Device, extent: vk::Extent2D, render_pass: vk::RenderPass) {
		unsafe { logical_device.destroy_pipeline(self.pipeline, None) };

		self.pipeline = create_pipeline(logical_device, extent, self.pipeline_layout, render_pass);
		
		self.projection_matrix.elements[0][0] = 2.0 / extent.width as f32;
		self.projection_matrix.elements[1][1] = 2.0 / extent.height as f32;
	}

	pub fn submit_fonts(&mut self, context: &Context, command_pool: vk::CommandPool, fonts: &mut Pool<Font>) {
		let logical_device = &context.logical_device;

		// Free memory and destroy resources
		unsafe {
			logical_device.queue_wait_idle(context.graphics_queue).unwrap();
			logical_device.free_memory(self.memory, None);
			logical_device.destroy_image_view(self.empty_image_view, None);
			logical_device.destroy_image(self.empty_image, None);

			for atlas in &self.atlases {
				logical_device.destroy_image_view(atlas.image_view, None);
				logical_device.destroy_image(atlas.image, None);
			}
		}

		self.atlases.clear();
		self.submission_generation += 1;

		// Don't do anything if there are no fonts
		if fonts.is_empty() {
			return;
		}

		// Ensure there are not more fonts than what's allowed
		assert!(fonts.occupied_record_count() <= MAX_FONTS, "Cannot submit fonts, {} is more than the allowed {}", fonts.occupied_record_count(), MAX_FONTS);

		// Create images and calculate buffer size
		struct TempFontInfo<'a> {
			font: &'a mut Font,
			image: vk::Image,
			image_view: vk::ImageView,
			offset: u64
		}

		let mut font_infos: Vec<TempFontInfo> = vec![];
		let mut offset = 0;

		for font in fonts.iter_mut() {
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
			let padding = (alignment - offset % alignment) % alignment;
			let size = image_memory_requirements.size;

			font_infos.push(TempFontInfo {
				font,
				image,
				image_view: vk::ImageView::null(),
				offset
			});

			offset += padding + size;
		}

		let empty_image_create_info = vk::ImageCreateInfo::builder()
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

		self.empty_image = unsafe { logical_device.create_image(&empty_image_create_info, None) }.unwrap();

		// Create staging buffer
		let staging_buffer = Buffer::new(context, offset, vk::BufferUsageFlags::TRANSFER_SRC, vk::MemoryPropertyFlags::HOST_VISIBLE);

		// Copy atlases into staging buffer
		let staging_buffer_ptr = unsafe { logical_device.map_memory(staging_buffer.memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty()) }.unwrap();

		for font_info in &font_infos {
			let font = &font_info.font;

			let mut file = File::open(&font.fnt_path).unwrap();
			file.seek(SeekFrom::Start(2 * size_of::<u32>() as u64)).unwrap();
			let mut atlas = vec![0u8; font.atlas_width * font.atlas_height];
			file.read_exact(&mut atlas).unwrap();

			unsafe {
				let dst_ptr = staging_buffer_ptr.add(font_info.offset as usize) as *mut u8;
				copy_nonoverlapping(atlas.as_ptr(), dst_ptr, font.atlas_width * font.atlas_height);
			}
		}

		let range = vk::MappedMemoryRange::builder()
			.memory(staging_buffer.memory)
			.offset(0)
			.size(vk::WHOLE_SIZE);
		
		unsafe {
			logical_device.flush_mapped_memory_ranges(&[range.build()]).unwrap();
			logical_device.unmap_memory(staging_buffer.memory);
		}

		// Create device local buffer
		let first_image = font_infos[0].image;
		let first_image_memory_requirements = unsafe { logical_device.get_image_memory_requirements(first_image) };
		let memory_type_index = context.physical_device.find_memory_type_index(first_image_memory_requirements.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL);

		let memory_allocate_info = vk::MemoryAllocateInfo::builder()
			.allocation_size(offset as u64)
			.memory_type_index(memory_type_index as u32);
	
		self.memory = unsafe { logical_device.allocate_memory(&memory_allocate_info, None) }.unwrap();

		// Bind images to device local buffer and create image view
		for font_info in &mut font_infos {
			unsafe { logical_device.bind_image_memory(font_info.image, self.memory, font_info.offset) }.unwrap();

			let image_view_create_info = vk::ImageViewCreateInfo::builder()
				.image(font_info.image)
				.view_type(vk::ImageViewType::TYPE_2D)
				.format(vk::Format::R8_UNORM)
				.subresource_range(vk::ImageSubresourceRange::builder()
					.aspect_mask(vk::ImageAspectFlags::COLOR)
					.base_mip_level(0)
					.level_count(1)
					.base_array_layer(0)
					.layer_count(1)
					.build());
			
			font_info.image_view = unsafe { logical_device.create_image_view(&image_view_create_info, None) }.unwrap();
		}

		unsafe { logical_device.bind_image_memory(self.empty_image, self.memory, 0) }.unwrap();

		let empty_image_view_create_info = vk::ImageViewCreateInfo::builder()
			.image(self.empty_image)
			.view_type(vk::ImageViewType::TYPE_2D)
			.format(vk::Format::R8_UNORM)
			.subresource_range(vk::ImageSubresourceRange::builder()
				.aspect_mask(vk::ImageAspectFlags::COLOR)
				.base_mip_level(0)
				.level_count(1)
				.base_array_layer(0)
				.layer_count(1)
				.build());
		
		self.empty_image_view = unsafe { logical_device.create_image_view(&empty_image_view_create_info, None) }.unwrap();

		// Record command buffer to copy staging buffer to device local buffer
		let mut transfer_image_memory_barriers: Vec<vk::ImageMemoryBarrier> = Vec::with_capacity(font_infos.len());
		let mut shader_read_image_memory_barriers: Vec<vk::ImageMemoryBarrier> = Vec::with_capacity(font_infos.len());
		for font_info in &font_infos {
			let transfer_image_memory_barrier = vk::ImageMemoryBarrier::builder()
				.old_layout(vk::ImageLayout::UNDEFINED)
				.new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
				.src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
				.dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
				.image(font_info.image)
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
				.image(font_info.image)
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

		let empty_image_memory_barrier = vk::ImageMemoryBarrier::builder()
			.old_layout(vk::ImageLayout::UNDEFINED)
			.new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
			.src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
			.dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
			.image(self.empty_image)
			.subresource_range(vk::ImageSubresourceRange::builder()
				.aspect_mask(vk::ImageAspectFlags::COLOR)
				.base_mip_level(0)
				.level_count(1)
				.base_array_layer(0)
				.layer_count(1)
				.build())
			.src_access_mask(vk::AccessFlags::empty())
			.dst_access_mask(vk::AccessFlags::SHADER_READ);
		
		shader_read_image_memory_barriers.push(empty_image_memory_barrier.build());

		let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
			.level(vk::CommandBufferLevel::PRIMARY)
			.command_pool(command_pool)
			.command_buffer_count(1);

		let command_buffer = unsafe { logical_device.allocate_command_buffers(&command_buffer_allocate_info) }.unwrap()[0];

		let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
			.flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

		unsafe {
			logical_device.begin_command_buffer(command_buffer, &command_buffer_begin_info).unwrap();
			logical_device.cmd_pipeline_barrier(command_buffer, vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER, vk::DependencyFlags::empty(), &[], &[], &transfer_image_memory_barriers);
		}

		for font_info in &font_infos {
			let region = vk::BufferImageCopy::builder()
				.buffer_offset(font_info.offset)
				.buffer_row_length(0)
				.buffer_image_height(0)
				.image_subresource(vk::ImageSubresourceLayers::builder()
					.aspect_mask(vk::ImageAspectFlags::COLOR)
					.mip_level(0)
					.base_array_layer(0)
					.layer_count(1)
					.build())
				.image_offset(vk::Offset3D::builder().x(0).y(0).z(0).build())
				.image_extent(vk::Extent3D::builder().width(font_info.font.atlas_width as u32).height(font_info.font.atlas_height as u32).depth(1).build());

			unsafe { logical_device.cmd_copy_buffer_to_image(command_buffer, staging_buffer.handle, font_info.image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[region.build()]) };
		}

		unsafe {
			logical_device.cmd_pipeline_barrier(command_buffer, vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::FRAGMENT_SHADER, vk::DependencyFlags::empty(), &[], &[], &shader_read_image_memory_barriers);
			logical_device.end_command_buffer(command_buffer).unwrap();
		}

		// Submit command buffer
		let command_buffers = [command_buffer];
		let submit_info = vk::SubmitInfo::builder()
			.command_buffers(&command_buffers);
		
		unsafe {
			logical_device.queue_submit(context.graphics_queue, &[submit_info.build()], vk::Fence::null()).unwrap();
			logical_device.queue_wait_idle(context.graphics_queue).unwrap();
			logical_device.free_command_buffers(command_pool, &command_buffers);
		}

		// Destroy staging buffer
		staging_buffer.drop(logical_device);

		// Update descriptor sets
		let mut descriptor_image_infos: Vec<vk::DescriptorImageInfo> = Vec::with_capacity(MAX_FONTS);
		for font_info in &font_infos {
			let descriptor_image_info = vk::DescriptorImageInfo::builder()
				.image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
				.image_view(font_info.image_view)
				.build();
			
			descriptor_image_infos.push(descriptor_image_info);
		}

		for _ in 0..(MAX_FONTS - font_infos.len()) {
			let descriptor_image_info = vk::DescriptorImageInfo::builder()
				.image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
				.image_view(self.empty_image_view)
				.build();
			
			descriptor_image_infos.push(descriptor_image_info);
		}

		let write_descriptor_set = vk::WriteDescriptorSet::builder()
			.dst_set(self.atlases_descriptor_set)
			.dst_binding(0)
			.dst_array_element(0)
			.descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
			.image_info(&descriptor_image_infos)
			.build();

		unsafe { logical_device.update_descriptor_sets(&[write_descriptor_set], &[]) };
		
		// Save submission info, images and image views
		for (index, font_info) in font_infos.iter_mut().enumerate() {
			font_info.font.submission_info = Some(SubmissionInfo {
				generation: self.submission_generation,
				index
			});

			self.atlases.push(Atlas {
				image: font_info.image,
				image_view: font_info.image_view
			});
		}
	}

	pub fn drop(&self, logical_device: &ash::Device) {
		unsafe {
			if !self.atlases.is_empty() {
				logical_device.destroy_image_view(self.empty_image_view, None);
				logical_device.destroy_image(self.empty_image, None);
				logical_device.free_memory(self.memory, None);
			}

			for atlas in &self.atlases {
				logical_device.destroy_image_view(atlas.image_view, None);
				logical_device.destroy_image(atlas.image, None);
			}

			logical_device.destroy_sampler(self.sampler, None);
			logical_device.destroy_pipeline(self.pipeline, None);
			logical_device.destroy_pipeline_layout(self.pipeline_layout, None);
			logical_device.destroy_descriptor_set_layout(self.atlases_descriptor_set_layout, None);
			logical_device.destroy_descriptor_set_layout(self.sampler_descriptor_set_layout, None);
		}
	}
}