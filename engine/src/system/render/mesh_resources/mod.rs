use std::{mem::size_of_val, ptr::copy_nonoverlapping};
use ash::{vk, version::DeviceV1_0};
use crate::{component::mesh::Material, geometry3d::{Geometry3D, SubmissionInfo}, pool::{Pool, Handle}, vulkan::{Buffer, Context}};
use super::MATERIALS_COUNT;

mod creation;
use creation::*;

pub struct MeshResources {
	pub pipeline_layout: vk::PipelineLayout,
	pub line_pipeline: vk::Pipeline,
	pub basic_pipeline: vk::Pipeline,
	pub normal_pipeline: vk::Pipeline,
	pub lambert_pipeline: vk::Pipeline,
	pub line_static_descriptor_set: vk::DescriptorSet,
	pub basic_static_descriptor_set: vk::DescriptorSet,
	pub normal_static_descriptor_set: vk::DescriptorSet,
	pub lambert_static_descriptor_set: vk::DescriptorSet,
	pub static_geometry_buffer: Buffer,
	pub static_geometry_infos: Vec<StaticGeometryInfo>,
	pub static_instance_groups: Vec<StaticInstanceGroup>,
	pub static_material_counts: [usize; MATERIALS_COUNT],
	static_geometry_submission_generation: usize
}

#[derive(Clone)]
pub struct StaticGeometryInfo {
	pub index_array_offset: usize,
	pub attribute_array_offset: usize,
	pub indices_count: usize
}

pub struct StaticInstanceGroup {
	pub geometry_info_index: usize,
	pub material: Material,
	pub instance_count: usize,
	pub first_instance: usize
}

impl MeshResources {
	pub fn new(
		logical_device: &ash::Device,
		frame_data_descriptor_set_layout: vk::DescriptorSetLayout,
		instance_data_descriptor_set_layout: vk::DescriptorSetLayout,
		extent: vk::Extent2D,
		render_pass: vk::RenderPass,
		descriptor_pool: vk::DescriptorPool)
		-> Self
	{
		let pipeline_layout = create_pipeline_layout(logical_device, frame_data_descriptor_set_layout, instance_data_descriptor_set_layout);
		let pipelines = create_pipelines(logical_device, extent, pipeline_layout, render_pass);
		let static_descriptor_sets = create_static_descriptor_sets(logical_device, descriptor_pool, instance_data_descriptor_set_layout);

		let static_geometry_buffer = Buffer::null(
			vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::VERTEX_BUFFER,
			vk::MemoryPropertyFlags::DEVICE_LOCAL);

		Self {
			pipeline_layout,
			line_pipeline: pipelines[0],
			basic_pipeline: pipelines[1],
			normal_pipeline: pipelines[2],
			lambert_pipeline: pipelines[3],
			line_static_descriptor_set: static_descriptor_sets[0],
			basic_static_descriptor_set: static_descriptor_sets[1],
			normal_static_descriptor_set: static_descriptor_sets[2],
			lambert_static_descriptor_set: static_descriptor_sets[3],
			static_geometry_buffer,
			static_geometry_infos: vec![],
			static_instance_groups: vec![],
			static_material_counts: [0; MATERIALS_COUNT],
			static_geometry_submission_generation: 0
		}
	}

	pub fn resize(&mut self, logical_device: &ash::Device, extent: vk::Extent2D, render_pass: vk::RenderPass) {
		unsafe {
			logical_device.destroy_pipeline(self.lambert_pipeline, None);
			logical_device.destroy_pipeline(self.normal_pipeline, None);
			logical_device.destroy_pipeline(self.basic_pipeline, None);
			logical_device.destroy_pipeline(self.line_pipeline, None);
		}

		let pipelines = create_pipelines(logical_device, extent, self.pipeline_layout, render_pass);

		self.line_pipeline = pipelines[0];
		self.basic_pipeline = pipelines[1];
		self.normal_pipeline = pipelines[2];
		self.lambert_pipeline = pipelines[3];
	}

	pub fn submit_static_geometries(&mut self, context: &Context, command_pool: vk::CommandPool, geometries: &mut Pool<Geometry3D>, handles: &[Handle]) {
		// Don't forget to increment the submission generation
		let logical_device = &context.logical_device;

		let mut buffer_size = 0;

		for handle in handles {
			let geometry = geometries.borrow_mut(*handle);
			let index_array_size = size_of_val(geometry.indices());
			let attributes_array_size = size_of_val(geometry.attributes());

			let index_array_offset = buffer_size;
			let unaligned_attributes_array_offset = index_array_offset + index_array_size;
			let attributes_array_padding = (4 - unaligned_attributes_array_offset % 4) % 4;
			let attributes_array_offset = unaligned_attributes_array_offset + attributes_array_padding;

			geometry.submission_info = Some(SubmissionInfo {
				generation: self.static_geometry_submission_generation,
				index_array_offset,
				attributes_array_offset
			});

			buffer_size += index_array_size + attributes_array_padding + attributes_array_size;
		}
		
		let buffer_size = buffer_size as u64;

		// Create a host visible staging buffer
		let staging_buffer = Buffer::new(&context, buffer_size, vk::BufferUsageFlags::TRANSFER_SRC, vk::MemoryPropertyFlags::HOST_VISIBLE);

		// Allocate larger device local buffer if necessary and update descriptor sets to reference new buffer
		if buffer_size > self.static_geometry_buffer.capacity {
			unsafe { logical_device.queue_wait_idle(context.graphics_queue) }.unwrap();
			self.static_geometry_buffer.reallocate(&context, buffer_size);
			println!("Static mesh buffer reallocated");
		}

		// Copy mesh data into staging buffer and save draw information
		let buffer_ptr = unsafe { logical_device.map_memory(staging_buffer.memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty()) }.unwrap();

		for handle in handles {
			let geometry = geometries.borrow(*handle);
			let submission_info = geometry.submission_info.as_ref().unwrap();
			let indices = geometry.indices();
			let attributes = geometry.attributes();

			unsafe {
				let index_array_dst_ptr = buffer_ptr.add(submission_info.index_array_offset) as *mut u16;
				copy_nonoverlapping(indices.as_ptr(), index_array_dst_ptr, indices.len());

				let attribute_array_dst_ptr = buffer_ptr.add(submission_info.attributes_array_offset) as *mut f32;
				copy_nonoverlapping(attributes.as_ptr(), attribute_array_dst_ptr, attributes.len());
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
			.command_pool(command_pool)
			.command_buffer_count(1);

		let command_buffer = unsafe { logical_device.allocate_command_buffers(&command_buffer_allocate_info) }.unwrap()[0];

		let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
			.flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
		
		let region = vk::BufferCopy::builder()
			.size(buffer_size);
		
		unsafe {
			logical_device.begin_command_buffer(command_buffer, &command_buffer_begin_info).unwrap();
			logical_device.cmd_copy_buffer(command_buffer, staging_buffer.handle, self.static_geometry_buffer.handle, &[region.build()]);
			logical_device.end_command_buffer(command_buffer).unwrap();
		}

		// Submit the command buffer
		let command_buffers = [command_buffer];
		let submit_info = vk::SubmitInfo::builder()
			.command_buffers(&command_buffers);
		
		unsafe {
			logical_device.queue_wait_idle(context.graphics_queue).unwrap();
			logical_device.queue_submit(context.graphics_queue, &[submit_info.build()], vk::Fence::null()).unwrap();
			logical_device.queue_wait_idle(context.graphics_queue).unwrap();
			logical_device.free_command_buffers(command_pool, &command_buffers);
		}

		staging_buffer.drop(logical_device);
	}

	pub fn drop(&self, logical_device: &ash::Device) {
		self.static_geometry_buffer.drop(logical_device);
		
		unsafe {
			logical_device.destroy_pipeline(self.lambert_pipeline, None);
			logical_device.destroy_pipeline(self.normal_pipeline, None);
			logical_device.destroy_pipeline(self.basic_pipeline, None);
			logical_device.destroy_pipeline(self.line_pipeline, None);
			logical_device.destroy_pipeline_layout(self.pipeline_layout, None);
		}
	}
}