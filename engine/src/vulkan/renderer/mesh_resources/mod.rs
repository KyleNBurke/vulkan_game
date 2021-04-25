use std::{mem::size_of_val, ptr::copy_nonoverlapping, cmp::max};
use ash::{vk, version::DeviceV1_0};
use crate::{vulkan::{Buffer, Context}, pool::Pool, Geometry3D, Mesh, Material};

mod creation;
use creation::*;

pub struct MeshResources {
	pub pipeline_layout: vk::PipelineLayout,
	pub basic_pipeline: vk::Pipeline,
	pub normal_pipeline: vk::Pipeline,
	pub lambert_pipeline: vk::Pipeline,
	pub basic_static_descriptor_set: vk::DescriptorSet,
	pub normal_static_descriptor_set: vk::DescriptorSet,
	pub lambert_static_descriptor_set: vk::DescriptorSet,
	pub static_mesh_buffer: Buffer,
	pub static_geometry_infos: Vec<StaticGeometryInfo>,
	pub basic_static_instance_infos: Vec<StaticInstanceInfo>,
	pub normal_static_instance_infos: Vec<StaticInstanceInfo>,
	pub lambert_static_instance_infos: Vec<StaticInstanceInfo>
}

pub struct StaticGeometryInfo {
	pub index_array_offset: usize,
	pub attribute_array_offset: usize,
	pub indices_count: usize
}

pub struct StaticInstanceInfo {
	pub geometry_info_index: usize,
	pub instance_count: usize
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

		let static_mesh_buffer = Buffer::null(
			vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::STORAGE_BUFFER,
			vk::MemoryPropertyFlags::DEVICE_LOCAL);

		Self {
			pipeline_layout,
			basic_pipeline: pipelines[0],
			normal_pipeline: pipelines[1],
			lambert_pipeline: pipelines[2],
			basic_static_descriptor_set: static_descriptor_sets[0],
			normal_static_descriptor_set: static_descriptor_sets[1],
			lambert_static_descriptor_set: static_descriptor_sets[2],
			static_mesh_buffer,
			static_geometry_infos: vec![],
			basic_static_instance_infos: vec![],
			normal_static_instance_infos: vec![],
			lambert_static_instance_infos: vec![],
		}
	}

	pub fn resize(&mut self, logical_device: &ash::Device, extent: vk::Extent2D, render_pass: vk::RenderPass) {
		unsafe {
			logical_device.destroy_pipeline(self.lambert_pipeline, None);
			logical_device.destroy_pipeline(self.normal_pipeline, None);
			logical_device.destroy_pipeline(self.basic_pipeline, None);
		}

		let pipelines = create_pipelines(logical_device, extent, self.pipeline_layout, render_pass);

		self.basic_pipeline = pipelines[0];
		self.normal_pipeline = pipelines[1];
		self.lambert_pipeline = pipelines[2];
	}

	pub fn submit_static_meshes(&mut self, context: &Context, command_pool: vk::CommandPool, geometries: &Pool<Geometry3D>, meshes: &[Mesh]) {
		self.static_geometry_infos.clear();
		self.basic_static_instance_infos.clear();
		self.normal_static_instance_infos.clear();
		self.lambert_static_instance_infos.clear();
		
		if meshes.is_empty() {
			return;
		}

		let logical_device = &context.logical_device;

		// Iterate over meshes to
		// - Group meshes together which share the same geometry and material
		// - Calculate the offsets and size of the data
		#[derive(Clone)]
		struct TempGeometryInfo<'a> {
			geometry: &'a Geometry3D,
			index_array_relative_offset: usize,
			attribute_array_relative_offset: usize,
			copied: bool,
			static_geometry_info_index: usize
		}

		let mut geometry_infos: Vec<Option<TempGeometryInfo>> = vec![None; geometries.len()];

		struct TempMaterialGroup<'a> {
			map: Vec<Option<usize>>,
			instance_groups: Vec<Vec<&'a Mesh>>,
			instance_count: usize
		}

		let mut material_groups: [TempMaterialGroup; 3] = [
			TempMaterialGroup {
				map: vec![None; geometries.len()],
				instance_groups: vec![],
				instance_count: 0
			},
			TempMaterialGroup {
				map: vec![None; geometries.len()],
				instance_groups: vec![],
				instance_count: 0
			},
			TempMaterialGroup {
				map: vec![None; geometries.len()],
				instance_groups: vec![],
				instance_count: 0
			}
		];
		
		let mut index_arrays_size = 0;
		let mut attribute_arrays_size = 0;

		for mesh in meshes.iter() {
			let geometry_info = &mut geometry_infos[mesh.geometry_handle.index];

			if geometry_info.is_none() {
				let geometry = geometries.get(&mesh.geometry_handle).unwrap();

				*geometry_info = Some(TempGeometryInfo {
					geometry,
					index_array_relative_offset: index_arrays_size,
					attribute_array_relative_offset: attribute_arrays_size,
					copied: false,
					static_geometry_info_index: self.static_geometry_infos.len()
				});

				self.static_geometry_infos.push(StaticGeometryInfo {
					index_array_offset: 0,
					attribute_array_offset: 0,
					indices_count: geometry.indices.len()
				});
				
				index_arrays_size += size_of_val(&geometry.indices[..]);
				attribute_arrays_size += size_of_val(&geometry.attributes[..]);
			}

			let material_group = &mut material_groups[mesh.material as usize];

			if let Some(instance_group_index) = material_group.map[mesh.geometry_handle.index] {
				material_group.instance_groups[instance_group_index].push(mesh);
			}
			else {
				material_group.map[mesh.geometry_handle.index] = Some(material_group.instance_groups.len());
				material_group.instance_groups.push(vec![mesh]);
			}

			material_group.instance_count += 1;
		}

		let alignment = context.physical_device.min_storage_buffer_offset_alignment as usize;

		let basic_instance_data_array_offset = 0;
		let basic_instance_data_array_size = 4 * 16 * material_groups[Material::Basic as usize].instance_count;

		let unaligned_normal_instance_data_array_offset = basic_instance_data_array_offset + basic_instance_data_array_size;
		let normal_instance_data_array_padding = (alignment - unaligned_normal_instance_data_array_offset % alignment) % alignment;
		let normal_instance_data_array_offset = unaligned_normal_instance_data_array_offset + normal_instance_data_array_padding;
		let normal_instance_data_array_size = 4 * 16 * material_groups[Material::Normal as usize].instance_count;
		
		let unaligned_lambert_instance_data_array_offset = normal_instance_data_array_offset + normal_instance_data_array_size;
		let lambert_instance_data_array_padding = (alignment - unaligned_lambert_instance_data_array_offset % alignment) % alignment;
		let lambert_instance_data_array_offset = unaligned_lambert_instance_data_array_offset + lambert_instance_data_array_padding;
		let lambert_instance_data_array_size = 4 * 16 * material_groups[Material::Lambert as usize].instance_count;

		let index_arrays_offset = lambert_instance_data_array_offset + lambert_instance_data_array_size;
		
		let unaligned_attribute_arrays_offset = index_arrays_offset + index_arrays_size;
		let attribute_arrays_padding = (4 - unaligned_attribute_arrays_offset % 4) % 4;
		let attribute_arrays_offset = unaligned_attribute_arrays_offset + attribute_arrays_padding;

		let buffer_size = (attribute_arrays_offset + attribute_arrays_size) as u64;

		// Create a host visible staging buffer
		let staging_buffer = Buffer::new(&context, buffer_size, vk::BufferUsageFlags::TRANSFER_SRC, vk::MemoryPropertyFlags::HOST_VISIBLE);

		// Allocate larger device local buffer if necessary and update descriptor sets to reference new buffer
		if buffer_size > self.static_mesh_buffer.capacity {
			unsafe { logical_device.queue_wait_idle(context.graphics_queue) }.unwrap();
			self.static_mesh_buffer.reallocate(&context, buffer_size);
			println!("Static mesh buffer reallocated");
		}

		// Update the descriptor sets to potentially use the new device local buffer and to use the calculated offsets and sizes
		{
			// Basic
			let basic_descriptor_buffer_info = vk::DescriptorBufferInfo::builder()
				.buffer(self.static_mesh_buffer.handle)
				.offset(basic_instance_data_array_offset as u64)
				.range(max(1, basic_instance_data_array_size) as u64);
			let basic_descriptor_buffer_infos = [basic_descriptor_buffer_info.build()];

			let basic_write_descriptor_set = vk::WriteDescriptorSet::builder()
				.dst_set(self.basic_static_descriptor_set)
				.dst_binding(0)
				.dst_array_element(0)
				.descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
				.buffer_info(&basic_descriptor_buffer_infos);
			
			// Normal
			let normal_descriptor_buffer_info = vk::DescriptorBufferInfo::builder()
				.buffer(self.static_mesh_buffer.handle)
				.offset(normal_instance_data_array_offset as u64)
				.range(max(1, normal_instance_data_array_size) as u64);
			let normal_descriptor_buffer_infos = [normal_descriptor_buffer_info.build()];

			let normal_write_descriptor_set = vk::WriteDescriptorSet::builder()
				.dst_set(self.normal_static_descriptor_set)
				.dst_binding(0)
				.dst_array_element(0)
				.descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
				.buffer_info(&normal_descriptor_buffer_infos);
			
			// Lambert
			let lambert_descriptor_buffer_info = vk::DescriptorBufferInfo::builder()
				.buffer(self.static_mesh_buffer.handle)
				.offset(lambert_instance_data_array_offset as u64)
				.range(max(1, lambert_instance_data_array_size) as u64);
			let lambert_descriptor_buffer_infos = [lambert_descriptor_buffer_info.build()];

			let lambert_write_descriptor_set = vk::WriteDescriptorSet::builder()
				.dst_set(self.lambert_static_descriptor_set)
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

		// Copy mesh data into staging buffer and save draw information
		let buffer_ptr = unsafe { logical_device.map_memory(staging_buffer.memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty()) }.unwrap();

		for material_group in &material_groups {
			let mut group_index = 0;

			for instance_group in &material_group.instance_groups {
				let geometry_handle = &instance_group[0].geometry_handle;
				let geometry_info = geometry_infos[geometry_handle.index].as_mut().unwrap();
				let geometry = geometries.get(geometry_handle).unwrap();

				let index_array_offset = index_arrays_offset + geometry_info.index_array_relative_offset;
				let attribute_array_offset = attribute_arrays_offset + geometry_info.attribute_array_relative_offset;

				// Ensure geometry data is copied into the buffer and save the absolute geometry offsets
				if !geometry_info.copied {
					let indices = &geometry.indices;
					let attributes = &geometry.attributes;

					unsafe {
						let index_array_dst_ptr = buffer_ptr.add(index_array_offset) as *mut u16;
						copy_nonoverlapping(indices.as_ptr(), index_array_dst_ptr, indices.len());

						let attribute_array_dst_ptr = buffer_ptr.add(attribute_array_offset) as *mut f32;
						copy_nonoverlapping(attributes.as_ptr(), attribute_array_dst_ptr, attributes.len());
					}

					geometry_info.copied = true;

					let static_geometry_info = &mut self.static_geometry_infos[geometry_info.static_geometry_info_index];
					static_geometry_info.index_array_offset = index_array_offset;
					static_geometry_info.attribute_array_offset = attribute_array_offset;
				}

				// Ensure the instance data is copied into the buffer and save instance group info
				match instance_group[0].material {
					Material::Basic => {
						for (instance_index, instance) in instance_group.iter().enumerate() {
							let instance_data_offset = basic_instance_data_array_offset + 4 * 16 * (group_index + instance_index);

							unsafe {
								let instance_data_dst_ptr = buffer_ptr.add(instance_data_offset) as *mut [f32; 4];
								copy_nonoverlapping(instance.transform.matrix.elements.as_ptr(), instance_data_dst_ptr, 4);
							}
						}

						self.basic_static_instance_infos.push(StaticInstanceInfo {
							geometry_info_index: geometry_info.static_geometry_info_index,
							instance_count: instance_group.len()
						});
					},
					Material::Normal => {
						for (instance_index, instance) in instance_group.iter().enumerate() {
							let instance_data_offset = normal_instance_data_array_offset + 4 * 16 * (group_index + instance_index);

							unsafe {
								let instance_data_dst_ptr = buffer_ptr.add(instance_data_offset) as *mut [f32; 4];
								copy_nonoverlapping(instance.transform.matrix.elements.as_ptr(), instance_data_dst_ptr, 4);
							}
						}

						self.normal_static_instance_infos.push(StaticInstanceInfo {
							geometry_info_index: geometry_info.static_geometry_info_index,
							instance_count: instance_group.len()
						});
					},
					Material::Lambert => {
						for (instance_index, instance) in instance_group.iter().enumerate() {
							let instance_data_offset = lambert_instance_data_array_offset + 4 * 16 * (group_index + instance_index);

							unsafe {
								let instance_data_dst_ptr = buffer_ptr.add(instance_data_offset) as *mut [f32; 4];
								copy_nonoverlapping(instance.transform.matrix.elements.as_ptr(), instance_data_dst_ptr, 4);
							}
						}

						self.lambert_static_instance_infos.push(StaticInstanceInfo {
							geometry_info_index: geometry_info.static_geometry_info_index,
							instance_count: instance_group.len()
						});
					}
				}

				group_index += instance_group.len();
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
			logical_device.cmd_copy_buffer(command_buffer, staging_buffer.handle, self.static_mesh_buffer.handle, &[region.build()]);
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
		self.static_mesh_buffer.drop(logical_device);
		
		unsafe {
			logical_device.destroy_pipeline(self.lambert_pipeline, None);
			logical_device.destroy_pipeline(self.normal_pipeline, None);
			logical_device.destroy_pipeline(self.basic_pipeline, None);
			logical_device.destroy_pipeline_layout(self.pipeline_layout, None);
		}
	}
}