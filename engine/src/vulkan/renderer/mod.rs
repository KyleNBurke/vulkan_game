use std::{cmp::max, mem::{size_of, size_of_val}, ptr::copy_nonoverlapping};
use ash::{vk, version::DeviceV1_0, extensions::khr};
use crate::{
	Geometry3D,
	math::{Matrix3, Matrix4, Vector3},
	Mesh,
	Material,
	pool::{Pool, Handle},
	scene::Scene,
	vulkan::{Context, Buffer, Font}
};

mod creation;
use creation::*;

const IN_FLIGHT_FRAMES_COUNT: usize = 2;
const FRAME_DATA_MEMORY_SIZE: usize = 76 * size_of::<f32>();
const MAX_POINT_LIGHTS: usize = 5;

pub struct Renderer {
	context: Context,
	render_pass: vk::RenderPass,
	swapchain: Swapchain,
	descriptor_pool: vk::DescriptorPool,
	command_pool: vk::CommandPool,
	frame_data_descriptor_set_layout: vk::DescriptorSetLayout,
	instance_data_descriptor_set_layout: vk::DescriptorSetLayout,
	pipeline_layout: vk::PipelineLayout,
	basic_pipeline: vk::Pipeline,
	normal_pipeline: vk::Pipeline,
	lambert_pipeline: vk::Pipeline,
	static_basic_descriptor_set: vk::DescriptorSet,
	static_normal_descriptor_set: vk::DescriptorSet,
	static_lambert_descriptor_set: vk::DescriptorSet,
	static_mesh_buffer: Buffer,
	in_flight_frames: [InFlightFrame; IN_FLIGHT_FRAMES_COUNT],
	current_in_flight_frame: usize,
	current_group_index: usize,
	static_geometry_groups: Vec<StaticGeometryGroup>,
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

struct StaticGeometryGroup {
	index_array_offset: usize,
	attribute_array_offset: usize,
	indices_count: usize,
	material_groups: Vec<StaticMaterialGroup>
}

struct StaticMaterialGroup {
	material: Material,
	instance_count: usize
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
		let render_pass = create_render_pass(&context);
		let (framebuffer_width, framebuffer_height) = window.get_framebuffer_size();
		let swapchain = create_swapchain(&context, framebuffer_width as u32, framebuffer_height as u32, render_pass);
		let descriptor_pool = create_descriptor_pool(&context);
		let command_pool = create_command_pool(&context);
		let frame_data_descriptor_set_layout = create_frame_data_descriptor_set_layout(&context.logical_device);
		let instance_data_descriptor_set_layout = create_instance_data_descriptor_set_layout(&context.logical_device);
		let pipeline_layout = create_pipeline_layout(&context.logical_device, frame_data_descriptor_set_layout, instance_data_descriptor_set_layout);
		let pipelines = create_pipelines(&context.logical_device, swapchain.extent, pipeline_layout, render_pass);
		let descriptor_sets = create_static_descriptor_sets(&context.logical_device, descriptor_pool, instance_data_descriptor_set_layout);

		let static_mesh_buffer = Buffer::null(
			vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::STORAGE_BUFFER,
			vk::MemoryPropertyFlags::DEVICE_LOCAL);

		let in_flight_frames = create_in_flight_frames(
			&context,
			&descriptor_pool,
			&command_pool,
			&frame_data_descriptor_set_layout,
			&instance_data_descriptor_set_layout);
		
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
			frame_data_descriptor_set_layout,
			instance_data_descriptor_set_layout,
			basic_pipeline: pipelines[0],
			normal_pipeline: pipelines[1],
			lambert_pipeline: pipelines[2],
			static_basic_descriptor_set: descriptor_sets[0],
			static_normal_descriptor_set: descriptor_sets[1],
			static_lambert_descriptor_set: descriptor_sets[2],
			static_mesh_buffer,
			pipeline_layout,
			in_flight_frames,
			current_in_flight_frame: 0,
			current_group_index: 0,
			static_geometry_groups: vec![],
			submit_fonts: false,
			inverse_view_matrix: Matrix4::new(),
			ui_projection_matrix
		}
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
			
			logical_device.destroy_pipeline(self.basic_pipeline, None);
			logical_device.destroy_pipeline(self.normal_pipeline, None);
			logical_device.destroy_pipeline(self.lambert_pipeline, None);
		}

		self.swapchain = create_swapchain(&self.context, framebuffer_width as u32, framebuffer_height as u32, self.render_pass);
		let pipelines = create_pipelines(logical_device, self.swapchain.extent, self.pipeline_layout, self.render_pass);

		self.basic_pipeline = pipelines[0];
		self.normal_pipeline = pipelines[1];
		self.lambert_pipeline = pipelines[2];

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
			copied: bool,
			material_groups: Vec<Vec<&'a Mesh>>
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
					copied: false,
					material_groups: vec![vec![mesh]]
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
		if buffer_size > self.static_mesh_buffer.capacity {
			unsafe { logical_device.queue_wait_idle(self.context.graphics_queue) }.unwrap();
			self.static_mesh_buffer.reallocate(&self.context, buffer_size);
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
				.dst_set(self.static_basic_descriptor_set)
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
				.dst_set(self.static_normal_descriptor_set)
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
				.dst_set(self.static_lambert_descriptor_set)
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
		
		let mut current_instance_indices = [0; 3];
		self.static_geometry_groups.clear();

		for (geometry_group_index, geometry_group) in geometry_groups.iter_mut().enumerate() {
			let geometry_handle = &geometry_group.material_groups[0][0].geometry_handle;
			let geometry = geometries.get(geometry_handle).unwrap();

			let index_array_offset = index_arrays_offset + geometry_group.index_array_relative_offset;
			let attribute_array_offset = attribute_arrays_offset + geometry_group.attribute_array_relative_offset;

			// Ensure geometry data is copied into staging buffer
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

			// Save geometry draw information
			self.static_geometry_groups.push(StaticGeometryGroup {
				index_array_offset,
				attribute_array_offset,
				indices_count: geometry.indices.len(),
				material_groups: Vec::with_capacity(geometry_group.material_groups.len())
			});

			for material_group in &geometry_group.material_groups {
				let material = material_group[0].material;

				// Save material draw information
				self.static_geometry_groups[geometry_group_index].material_groups.push(StaticMaterialGroup {
					material,
					instance_count: material_group.len()
				});

				// Copy instance data into staging buffer
				let current_instance_index = current_instance_indices[material as usize];
				
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
			logical_device.cmd_copy_buffer(command_buffer, staging_buffer.handle, self.static_mesh_buffer.handle, &[region.build()]);
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
		// self.text_manager.fonts.add(Font::new(file_path, size))
		Handle::null()
	}

	pub fn remove_font(&mut self, handle: &Handle<Font>) {
		// self.text_manager.fonts.remove(handle);
		self.submit_fonts = true;
	}

	pub fn render(&mut self, scene: &mut Scene) -> bool {
		// If new fonts have been added or removed, submit them
		/*if self.submit_fonts {
			self.text_manager.submit_fonts(&self.context, self.command_pool);
			self.submit_fonts = false;
		}*/

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
			copied: bool,
			material_groups: Vec<Vec<&'a Mesh>>
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
					copied: false,
					material_groups: vec![vec![mesh]]
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
			logical_device.cmd_bind_pipeline(basic_material_data.secondary_command_buffer, vk::PipelineBindPoint::GRAPHICS, self.basic_pipeline);
			logical_device.cmd_bind_descriptor_sets(
				basic_material_data.secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.pipeline_layout,
				0,
				&[in_flight_frame.frame_data_descriptor_set],
				&[]);
			logical_device.cmd_bind_descriptor_sets(
				basic_material_data.secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.pipeline_layout,
				1,
				&[basic_material_data.descriptor_set],
				&[]);
			
			logical_device.begin_command_buffer(normal_material_data.secondary_command_buffer, &command_buffer_begin_info).unwrap();
			logical_device.cmd_bind_pipeline(normal_material_data.secondary_command_buffer, vk::PipelineBindPoint::GRAPHICS, self.normal_pipeline);
			logical_device.cmd_bind_descriptor_sets(
				normal_material_data.secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.pipeline_layout,
				0,
				&[in_flight_frame.frame_data_descriptor_set],
				&[]);
			logical_device.cmd_bind_descriptor_sets(
				normal_material_data.secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.pipeline_layout,
				1,
				&[normal_material_data.descriptor_set],
				&[]);
			
			logical_device.begin_command_buffer(lambert_material_data.secondary_command_buffer, &command_buffer_begin_info).unwrap();
			logical_device.cmd_bind_pipeline(lambert_material_data.secondary_command_buffer, vk::PipelineBindPoint::GRAPHICS, self.lambert_pipeline);
			logical_device.cmd_bind_descriptor_sets(
				lambert_material_data.secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.pipeline_layout,
				0,
				&[in_flight_frame.frame_data_descriptor_set],
				&[]);
			logical_device.cmd_bind_descriptor_sets(
				lambert_material_data.secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.pipeline_layout,
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

		// Record static mesh draw commands
		unsafe {
			logical_device.begin_command_buffer(basic_material_data.static_secondary_command_buffer, &command_buffer_begin_info).unwrap();
			logical_device.cmd_bind_pipeline(basic_material_data.static_secondary_command_buffer, vk::PipelineBindPoint::GRAPHICS, self.basic_pipeline);
			logical_device.cmd_bind_descriptor_sets(
				basic_material_data.static_secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.pipeline_layout,
				0,
				&[in_flight_frame.frame_data_descriptor_set],
				&[]);
			logical_device.cmd_bind_descriptor_sets(
				basic_material_data.static_secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.pipeline_layout,
				1,
				&[self.static_basic_descriptor_set],
				&[]);
			
			logical_device.begin_command_buffer(normal_material_data.static_secondary_command_buffer, &command_buffer_begin_info).unwrap();
			logical_device.cmd_bind_pipeline(normal_material_data.static_secondary_command_buffer, vk::PipelineBindPoint::GRAPHICS, self.normal_pipeline);
			logical_device.cmd_bind_descriptor_sets(
				normal_material_data.static_secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.pipeline_layout,
				0,
				&[in_flight_frame.frame_data_descriptor_set],
				&[]);
			logical_device.cmd_bind_descriptor_sets(
				normal_material_data.static_secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.pipeline_layout,
				1,
				&[self.static_normal_descriptor_set],
				&[]);
			
			logical_device.begin_command_buffer(lambert_material_data.static_secondary_command_buffer, &command_buffer_begin_info).unwrap();
			logical_device.cmd_bind_pipeline(lambert_material_data.static_secondary_command_buffer, vk::PipelineBindPoint::GRAPHICS, self.lambert_pipeline);
			logical_device.cmd_bind_descriptor_sets(
				lambert_material_data.static_secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.pipeline_layout,
				0,
				&[in_flight_frame.frame_data_descriptor_set],
				&[]);
			logical_device.cmd_bind_descriptor_sets(
				lambert_material_data.static_secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.pipeline_layout,
				1,
				&[self.static_lambert_descriptor_set],
				&[]);
		}

		let mut current_instance_indices = [0; 3];

		for geometry_group in &self.static_geometry_groups {
			for material_group in &geometry_group.material_groups {
				let secondary_command_buffer = match material_group.material {
					Material::Basic => basic_material_data.static_secondary_command_buffer,
					Material::Normal => normal_material_data.static_secondary_command_buffer,
					Material::Lambert => lambert_material_data.static_secondary_command_buffer,
				};

				let current_instance_index = current_instance_indices[material_group.material as usize];

				unsafe {
					logical_device.cmd_bind_index_buffer(secondary_command_buffer, self.static_mesh_buffer.handle, geometry_group.index_array_offset as u64, vk::IndexType::UINT16);
					logical_device.cmd_bind_vertex_buffers(secondary_command_buffer, 0, &[self.static_mesh_buffer.handle], &[geometry_group.attribute_array_offset as u64]);
					logical_device.cmd_draw_indexed(secondary_command_buffer, geometry_group.indices_count as u32, material_group.instance_count as u32, 0, 0, current_instance_index as u32);
				}

				current_instance_indices[material_group.material as usize] += material_group.instance_count;
			}
		}

		unsafe {
			logical_device.end_command_buffer(basic_material_data.static_secondary_command_buffer).unwrap();
			logical_device.end_command_buffer(normal_material_data.static_secondary_command_buffer).unwrap();
			logical_device.end_command_buffer(lambert_material_data.static_secondary_command_buffer).unwrap();
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
			lambert_material_data.static_secondary_command_buffer
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

			for frame in &mut self.in_flight_frames {
				logical_device.destroy_semaphore(frame.image_available, None);
				logical_device.destroy_semaphore(frame.render_finished, None);
				logical_device.destroy_fence(frame.fence, None);
				frame.frame_data_buffer.drop(&self.context.logical_device);
				frame.mesh_data_buffer.drop(&self.context.logical_device);
			}
			
			logical_device.destroy_pipeline(self.lambert_pipeline, None);
			logical_device.destroy_pipeline(self.normal_pipeline, None);
			logical_device.destroy_pipeline(self.basic_pipeline, None);
			logical_device.destroy_pipeline_layout(self.pipeline_layout, None);
			logical_device.destroy_descriptor_set_layout(self.instance_data_descriptor_set_layout, None);
			logical_device.destroy_descriptor_set_layout(self.frame_data_descriptor_set_layout, None);
			logical_device.destroy_command_pool(self.command_pool, None);
			logical_device.destroy_descriptor_pool(self.descriptor_pool, None);

			self.swapchain.extension.destroy_swapchain(self.swapchain.handle, None);
			logical_device.destroy_image(self.swapchain.depth_image_resources.image, None);
			logical_device.destroy_image_view(self.swapchain.depth_image_resources.image_view, None);
			logical_device.free_memory(self.swapchain.depth_image_resources.memory, None);

			for frame in &self.swapchain.frames {
				logical_device.destroy_image_view(frame.image_view, None);
				logical_device.destroy_framebuffer(frame.framebuffer, None);
			}

			logical_device.destroy_render_pass(self.render_pass, None);
		}

		self.static_mesh_buffer.drop(logical_device);
	}
}