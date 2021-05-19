use std::{cmp::max, fs::File, mem::size_of_val, ptr::copy_nonoverlapping};
use ash::{vk, version::DeviceV1_0, extensions::khr};
use crate::{Font, Geometry3D, Material, StaticMesh, Scene, Text, math::Vector3, pool::Pool, vulkan::{Context, Buffer}, graph::{Node, Object}};

mod creation;
use creation::*;

mod mesh_resources;
use mesh_resources::*;

mod text_resources;
use text_resources::*;

const IN_FLIGHT_FRAMES_COUNT: usize = 2;
const FRAME_DATA_MEMORY_SIZE: usize = 76 * 4;
const MATERIALS_COUNT: usize = 3;
const MAX_POINT_LIGHTS: usize = 5;
const MAX_FONTS: usize = 10;

pub struct Renderer {
	context: Context,
	render_pass: vk::RenderPass,
	swapchain: Swapchain,
	descriptor_pool: vk::DescriptorPool,
	command_pool: vk::CommandPool,
	frame_data_descriptor_set_layout: vk::DescriptorSetLayout,
	instance_data_descriptor_set_layout: vk::DescriptorSetLayout,
	in_flight_frames: [InFlightFrame; IN_FLIGHT_FRAMES_COUNT],
	current_in_flight_frame_index: usize,
	mesh_resources: MeshResources,
	text_resources: TextResources
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
	instance_data_buffer: Buffer,
	basic_instance_data_resources: InstanceDataResources,
	normal_instance_data_resources: InstanceDataResources,
	lambert_instance_data_resources: InstanceDataResources,
	text_instance_data_resources: InstanceDataResources,
	index_arrays_offset: usize,
}

struct InstanceDataResources {
	descriptor_set: vk::DescriptorSet,
	secondary_command_buffer: vk::CommandBuffer,
	array_offset: usize,
	array_size: usize
}

fn create_shader_module(logical_device: &ash::Device, filename: &str) -> vk::ShaderModule {
	let mut file_path = String::from("target/shaders/");
	file_path.push_str(filename);

	let mut file = File::open(file_path).unwrap();
	let file_contents = ash::util::read_spv(&mut file).unwrap();

	let create_info = vk::ShaderModuleCreateInfo::builder()
		.code(&file_contents);

	unsafe { logical_device.create_shader_module(&create_info, None) }.unwrap()
}

impl InFlightFrame {
	#[allow(clippy::clippy::too_many_arguments)]
	fn update_descriptor_sets(
		&mut self,
		logical_device: &ash::Device,
		basic_instance_data_array_offset: usize,
		basic_instance_data_array_size: usize,
		normal_instance_data_array_offset: usize,
		normal_instance_data_array_size: usize,
		lambert_instance_data_array_offset: usize,
		lambert_instance_data_array_size: usize,
		text_instance_data_array_offset: usize,
		text_instance_data_array_size: usize,
		index_arrays_offset: usize)
	{
		// Basic
		let basic_descriptor_buffer_info = vk::DescriptorBufferInfo::builder()
			.buffer(self.instance_data_buffer.handle)
			.offset(basic_instance_data_array_offset as u64)
			.range(max(1, basic_instance_data_array_size) as u64);
		let basic_descriptor_buffer_infos = [basic_descriptor_buffer_info.build()];

		let basic_write_descriptor_set = vk::WriteDescriptorSet::builder()
			.dst_set(self.basic_instance_data_resources.descriptor_set)
			.dst_binding(0)
			.dst_array_element(0)
			.descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
			.buffer_info(&basic_descriptor_buffer_infos);
		
		// Normal
		let normal_descriptor_buffer_info = vk::DescriptorBufferInfo::builder()
			.buffer(self.instance_data_buffer.handle)
			.offset(normal_instance_data_array_offset as u64)
			.range(max(1, normal_instance_data_array_size) as u64);
		let normal_descriptor_buffer_infos = [normal_descriptor_buffer_info.build()];

		let normal_write_descriptor_set = vk::WriteDescriptorSet::builder()
			.dst_set(self.normal_instance_data_resources.descriptor_set)
			.dst_binding(0)
			.dst_array_element(0)
			.descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
			.buffer_info(&normal_descriptor_buffer_infos);
		
		// Lambert
		let lambert_descriptor_buffer_info = vk::DescriptorBufferInfo::builder()
			.buffer(self.instance_data_buffer.handle)
			.offset(lambert_instance_data_array_offset as u64)
			.range(max(1, lambert_instance_data_array_size) as u64);
		let lambert_descriptor_buffer_infos = [lambert_descriptor_buffer_info.build()];

		let lambert_write_descriptor_set = vk::WriteDescriptorSet::builder()
			.dst_set(self.lambert_instance_data_resources.descriptor_set)
			.dst_binding(0)
			.dst_array_element(0)
			.descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
			.buffer_info(&lambert_descriptor_buffer_infos);
		
		// Text
		let text_descriptor_buffer_info = vk::DescriptorBufferInfo::builder()
			.buffer(self.instance_data_buffer.handle)
			.offset(text_instance_data_array_offset as u64)
			.range(max(1, text_instance_data_array_size) as u64);
		let text_descriptor_buffer_infos = [text_descriptor_buffer_info.build()];

		let text_write_descriptor_set = vk::WriteDescriptorSet::builder()
			.dst_set(self.text_instance_data_resources.descriptor_set)
			.dst_binding(0)
			.dst_array_element(0)
			.descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
			.buffer_info(&text_descriptor_buffer_infos);
		
		// Update descriptor sets
		let write_descriptor_sets = [
			basic_write_descriptor_set.build(),
			normal_write_descriptor_set.build(),
			lambert_write_descriptor_set.build(),
			text_write_descriptor_set.build()
		];
		
		unsafe { logical_device.update_descriptor_sets(&write_descriptor_sets, &[]) };

		// Set offsets and sizes
		self.basic_instance_data_resources.array_offset = basic_instance_data_array_offset;
		self.basic_instance_data_resources.array_size = basic_instance_data_array_size;

		self.normal_instance_data_resources.array_offset = normal_instance_data_array_offset;
		self.normal_instance_data_resources.array_size = normal_instance_data_array_size;

		self.lambert_instance_data_resources.array_offset = lambert_instance_data_array_offset;
		self.lambert_instance_data_resources.array_size = lambert_instance_data_array_size;

		self.text_instance_data_resources.array_offset = text_instance_data_array_offset;
		self.text_instance_data_resources.array_size = text_instance_data_array_size;

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
		let in_flight_frames = create_in_flight_frames(&context, descriptor_pool, command_pool, frame_data_descriptor_set_layout, instance_data_descriptor_set_layout);
		let mesh_resources = MeshResources::new(&context.logical_device, frame_data_descriptor_set_layout, instance_data_descriptor_set_layout, swapchain.extent, render_pass, descriptor_pool);
		let text_renderer = TextResources::new(&context.logical_device, instance_data_descriptor_set_layout, swapchain.extent, render_pass, descriptor_pool);

		Self {
			context,
			render_pass,
			swapchain,
			descriptor_pool,
			command_pool,
			frame_data_descriptor_set_layout,
			instance_data_descriptor_set_layout,
			in_flight_frames,
			current_in_flight_frame_index: 0,
			mesh_resources,
			text_resources: text_renderer
		}
	}

	pub fn get_swapchain_extent(&self) -> vk::Extent2D {
		self.swapchain.extent
	}

	pub fn resize(&mut self, framebuffer_width: i32, framebuffer_height: i32) {
		let logical_device = &self.context.logical_device;

		unsafe {
			logical_device.device_wait_idle().unwrap();

			self.swapchain.extension.destroy_swapchain(self.swapchain.handle, None);
			logical_device.destroy_image(self.swapchain.depth_image_resources.image, None);
			logical_device.destroy_image_view(self.swapchain.depth_image_resources.image_view, None);
			logical_device.free_memory(self.swapchain.depth_image_resources.memory, None);

			for frame in &self.swapchain.frames {
				logical_device.destroy_image_view(frame.image_view, None);
				logical_device.destroy_framebuffer(frame.framebuffer, None);
			}
		}

		self.swapchain = create_swapchain(&self.context, framebuffer_width as u32, framebuffer_height as u32, self.render_pass);
		self.mesh_resources.resize(&self.context.logical_device, self.swapchain.extent, self.render_pass);
		self.text_resources.resize(&self.context.logical_device, self.swapchain.extent, self.render_pass);

		println!("Renderer resized");
	}

	pub fn submit_static_meshes(&mut self, geometries: &Pool<Geometry3D>, meshes: &[StaticMesh]) {
		self.mesh_resources.submit_static_meshes(&self.context, self.command_pool, geometries, meshes);
		println!("Static meshes submitted");
	}

	pub fn submit_fonts(&mut self, fonts: &mut Pool<Font>) {
		self.text_resources.submit_fonts(&self.context, self.command_pool, fonts);
		println!("Fonts submitted");
	}

	pub fn render(&mut self, scene: &mut Scene) -> bool {
		let logical_device = &self.context.logical_device;
		let in_flight_frame = &mut self.in_flight_frames[self.current_in_flight_frame_index];
		
		// Wait for this in flight frame to become available
		unsafe { logical_device.wait_for_fences(&[in_flight_frame.fence], true, std::u64::MAX) }.unwrap();
		
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
			unsafe { logical_device.wait_for_fences(&[swapchain_frame.fence], true, std::u64::MAX) }.unwrap();
		}

		swapchain_frame.fence = in_flight_frame.fence;

		// Map frame data buffer
		let frame_data_buffer_ptr = unsafe { logical_device.map_memory(in_flight_frame.frame_data_buffer.memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty()) }.unwrap();
		
		// Copy camera data into frame data buffer
		let camera_node = scene.graph.get(&scene.camera_handle).unwrap();
		let camera_object = &camera_node.object;
		let camera = camera_object.camera().unwrap();
		
		let projection_matrix = &camera.projection_matrix.elements;
		let projection_matrix_dst_ptr = frame_data_buffer_ptr as *mut [f32; 4];
		unsafe { copy_nonoverlapping(projection_matrix.as_ptr(), projection_matrix_dst_ptr, 4) };

		let mut inverse_view_matrix = camera_node.transform.global_matrix;
		inverse_view_matrix.invert();
		unsafe {
			let inverse_view_matrix_dst_ptr = frame_data_buffer_ptr.add(16 * 4) as *mut [f32; 4];
			copy_nonoverlapping(inverse_view_matrix.elements.as_ptr(), inverse_view_matrix_dst_ptr, 4);
		}

		// Iterate over meshes to
		// - Update transformation matrix
		// - Split nodes into separate lists
		// - Group meshes together which share the same geometry and material
		// - Calculate the offsets and size of the data
		struct GeometryInfo<'a> {
			geometry: &'a Geometry3D,
			index_array_relative_offset: usize,
			attribute_array_relative_offset: usize,
			copied: bool
		}

		struct InstanceGroup<'a> {
			geometry_info_index: usize,
			material: Material,
			nodes: Vec<&'a Node>
		}

		let mut geometry_infos: Vec<GeometryInfo> = vec![];
		let mut instance_groups: Vec<InstanceGroup> = vec![];
		let mut map: Vec<[Option<usize>; MATERIALS_COUNT + 1]> = vec![[None; MATERIALS_COUNT + 1]; scene.geometries.total_len()];
		let mut index_arrays_size = 0;
		let mut attribute_arrays_size = 0;
		let mut material_counts = [0; MATERIALS_COUNT];

		let mut total_ambient_light_color = Vector3::new();
		let mut total_ambient_light_intensity = 0.0;
		let mut point_lights: Vec<&Node> = vec![];

		for node in scene.graph.iter() {
			match &node.object {
				Object::AmbientLight(ambient_light) => {
					total_ambient_light_color += &ambient_light.color;
					total_ambient_light_intensity += ambient_light.intensity;
				},
				Object::PointLight(_) => {
					point_lights.push(node);
				},
				Object::Mesh(mesh) => {
					let geometry_index = mesh.geometry_handle.index;
					let material_index = mesh.material as usize + 1;
					
					if map[geometry_index][0].is_none() {
						let geometry = scene.geometries.get(&mesh.geometry_handle).unwrap();

						geometry_infos.push(GeometryInfo {
							geometry,
							index_array_relative_offset: index_arrays_size,
							attribute_array_relative_offset: attribute_arrays_size,
							copied: false
						});

						map[geometry_index][0] = Some(geometry_infos.len() - 1);
						index_arrays_size += size_of_val(&geometry.indices[..]);
						attribute_arrays_size += size_of_val(&geometry.attributes[..]);
					}

					if let Some(instance_group_index) = map[geometry_index][material_index] {
						instance_groups[instance_group_index].nodes.push(node);
					}
					else {
						instance_groups.push(InstanceGroup {
							geometry_info_index: map[geometry_index][0].unwrap(),
							material: mesh.material,
							nodes: vec![node]
						});

						map[geometry_index][material_index] = Some(instance_groups.len() - 1);
					}

					material_counts[mesh.material as usize] += 1;
				},
				_ => ()
			}
		}

		struct TextInfo<'a> {
			text: &'a Text,
			index_array_relative_offset: usize,
			attribute_array_relative_offset: usize
		}

		let mut text_infos: Vec<TextInfo> = Vec::with_capacity(scene.text.available_len());

		for text in scene.text.iter_mut() {
			if text.get_string().is_empty() {
				continue;
			}

			if text.generate {
				let font = scene.fonts.get(&text.font).unwrap();
				text.generate(&font);
			}

			let vertex_indices_size = size_of_val(text.get_vertex_indices());
			let vertex_attributes_size = size_of_val(text.get_vertex_attributes());

			text_infos.push(TextInfo {
				text,
				index_array_relative_offset: index_arrays_size,
				attribute_array_relative_offset: attribute_arrays_size
			});

			index_arrays_size += vertex_indices_size;
			attribute_arrays_size += vertex_attributes_size;
		}

		// Copy light data into frame data buffer
		let total_ambient_light_intensified_color = total_ambient_light_color * total_ambient_light_intensity;
		unsafe {
			let ambient_light_dst_ptr = frame_data_buffer_ptr.add(32 * 4) as *mut Vector3;
			copy_nonoverlapping(&total_ambient_light_intensified_color as *const Vector3, ambient_light_dst_ptr, 1);
		}

		let mut point_light_count = 0;
		let position_base_offest = 36 * 4;
		let color_base_offest = 40 * 4;
		let stride = 8 * 4;

		for point_light_node in point_lights {
			let point_light = point_light_node.object.point_light().unwrap();
			let intensified_color = point_light.color * point_light.intensity;

			unsafe {
				let position_dst_ptr = frame_data_buffer_ptr.add(position_base_offest + stride * point_light_count) as *mut Vector3;
				copy_nonoverlapping(&point_light_node.transform.position as *const Vector3, position_dst_ptr, 1); // Needs to be global position

				let color_dst_ptr = frame_data_buffer_ptr.add(color_base_offest + stride * point_light_count) as *mut Vector3;
				copy_nonoverlapping(&intensified_color as *const Vector3, color_dst_ptr, 1);
			}

			point_light_count += 1;
		}

		assert!(point_light_count <= MAX_POINT_LIGHTS, "Only {} point lights allowed", MAX_POINT_LIGHTS);
		unsafe {
			let point_light_count_dst_ptr = frame_data_buffer_ptr.add(35 * 4) as *mut u32;
			copy_nonoverlapping(&(point_light_count as u32) as *const u32, point_light_count_dst_ptr, 1);
		}

		// Flush and unmap frame data buffer
		let range = vk::MappedMemoryRange::builder()
			.memory(in_flight_frame.frame_data_buffer.memory)
			.offset(0)
			.size(vk::WHOLE_SIZE);
		
		unsafe {		
			logical_device.flush_mapped_memory_ranges(&[range.build()]).unwrap();
			logical_device.unmap_memory(in_flight_frame.frame_data_buffer.memory);
		}

		// Calculate offsets
		let alignment = self.context.physical_device.min_storage_buffer_offset_alignment as usize;

		let basic_instance_data_array_offset = 0;
		let basic_instance_data_array_size = 4 * 16 * material_counts[0];

		let unaligned_normal_instance_data_array_offset = basic_instance_data_array_offset + basic_instance_data_array_size;
		let normal_instance_data_array_padding = (alignment - unaligned_normal_instance_data_array_offset % alignment) % alignment;
		let normal_instance_data_array_offset = unaligned_normal_instance_data_array_offset + normal_instance_data_array_padding;
		let normal_instance_data_array_size = 4 * 16 * material_counts[1];
		
		let unaligned_lambert_instance_data_array_offset = normal_instance_data_array_offset + normal_instance_data_array_size;
		let lambert_instance_data_array_padding = (alignment - unaligned_lambert_instance_data_array_offset % alignment) % alignment;
		let lambert_instance_data_array_offset = unaligned_lambert_instance_data_array_offset + lambert_instance_data_array_padding;
		let lambert_instance_data_array_size = 4 * 16 * material_counts[2];

		let unaligned_text_instance_data_array_offset = lambert_instance_data_array_offset + lambert_instance_data_array_size;
		let text_instance_data_array_padding = (alignment - unaligned_text_instance_data_array_offset % alignment) % alignment;
		let text_instance_data_array_offset = unaligned_text_instance_data_array_offset + text_instance_data_array_padding;
		let text_instance_data_array_size = 4 * 16 * text_infos.len();

		let index_arrays_offset = text_instance_data_array_offset + text_instance_data_array_size;
		
		let unaligned_attribute_arrays_offset = index_arrays_offset + index_arrays_size;
		let attribute_arrays_padding = (4 - unaligned_attribute_arrays_offset % 4) % 4;
		let attribute_arrays_offset = unaligned_attribute_arrays_offset + attribute_arrays_padding;

		// Allocate larger mesh data buffer and update descriptor sets if necessary
		let buffer_size = (attribute_arrays_offset + attribute_arrays_size) as u64;

		if buffer_size > in_flight_frame.instance_data_buffer.capacity {
			in_flight_frame.instance_data_buffer.reallocate(&self.context, buffer_size);

			in_flight_frame.update_descriptor_sets(
				logical_device,
				basic_instance_data_array_offset,
				basic_instance_data_array_size,
				normal_instance_data_array_offset,
				normal_instance_data_array_size,
				lambert_instance_data_array_offset,
				lambert_instance_data_array_size,
				text_instance_data_array_offset,
				text_instance_data_array_size,
				index_arrays_offset);
			
			println!("In flight frame {} instance data buffer reallocated", self.current_in_flight_frame_index);
		}
		else if
			basic_instance_data_array_size > in_flight_frame.basic_instance_data_resources.array_size ||
			normal_instance_data_array_size > in_flight_frame.normal_instance_data_resources.array_size ||
			lambert_instance_data_array_size > in_flight_frame.lambert_instance_data_resources.array_size ||
			text_instance_data_array_size > in_flight_frame.text_instance_data_resources.array_size
		{
			in_flight_frame.update_descriptor_sets(
				logical_device,
				basic_instance_data_array_offset,
				basic_instance_data_array_size,
				normal_instance_data_array_offset,
				normal_instance_data_array_size,
				lambert_instance_data_array_offset,
				lambert_instance_data_array_size,
				text_instance_data_array_offset,
				text_instance_data_array_size,
				index_arrays_offset);
		}

		let in_flight_frame = &self.in_flight_frames[self.current_in_flight_frame_index];
		let basic_instance_data_resources = &in_flight_frame.basic_instance_data_resources;
		let normal_instance_data_resources = &in_flight_frame.normal_instance_data_resources;
		let lambert_instance_data_resources = &in_flight_frame.lambert_instance_data_resources;
		let text_instance_data_resources = &in_flight_frame.text_instance_data_resources;

		let instance_data_buffer_ptr = unsafe { logical_device.map_memory(in_flight_frame.instance_data_buffer.memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty()) }.unwrap();

		// Begin mesh command buffers
		let command_buffer_inheritance_info = vk::CommandBufferInheritanceInfo::builder()
			.render_pass(self.render_pass)
			.subpass(0)
			.framebuffer(swapchain_frame.framebuffer);

		let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
			.flags(vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE | vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
			.inheritance_info(&command_buffer_inheritance_info);

		unsafe {
			// Basic
			logical_device.begin_command_buffer(basic_instance_data_resources.secondary_command_buffer, &command_buffer_begin_info).unwrap();
			logical_device.cmd_bind_pipeline(basic_instance_data_resources.secondary_command_buffer, vk::PipelineBindPoint::GRAPHICS, self.mesh_resources.basic_pipeline);
			logical_device.cmd_bind_descriptor_sets(
				basic_instance_data_resources.secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.mesh_resources.pipeline_layout,
				0,
				&[in_flight_frame.frame_data_descriptor_set],
				&[]);
			logical_device.cmd_bind_descriptor_sets(
				basic_instance_data_resources.secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.mesh_resources.pipeline_layout,
				1,
				&[basic_instance_data_resources.descriptor_set],
				&[]);
			
			// Normal
			logical_device.begin_command_buffer(normal_instance_data_resources.secondary_command_buffer, &command_buffer_begin_info).unwrap();
			logical_device.cmd_bind_pipeline(normal_instance_data_resources.secondary_command_buffer, vk::PipelineBindPoint::GRAPHICS, self.mesh_resources.normal_pipeline);
			logical_device.cmd_bind_descriptor_sets(
				normal_instance_data_resources.secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.mesh_resources.pipeline_layout,
				0,
				&[in_flight_frame.frame_data_descriptor_set],
				&[]);
			logical_device.cmd_bind_descriptor_sets(
				normal_instance_data_resources.secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.mesh_resources.pipeline_layout,
				1,
				&[normal_instance_data_resources.descriptor_set],
				&[]);
			
			// Lambert
			logical_device.begin_command_buffer(lambert_instance_data_resources.secondary_command_buffer, &command_buffer_begin_info).unwrap();
			logical_device.cmd_bind_pipeline(lambert_instance_data_resources.secondary_command_buffer, vk::PipelineBindPoint::GRAPHICS, self.mesh_resources.lambert_pipeline);
			logical_device.cmd_bind_descriptor_sets(
				lambert_instance_data_resources.secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.mesh_resources.pipeline_layout,
				0,
				&[in_flight_frame.frame_data_descriptor_set],
				&[]);
			logical_device.cmd_bind_descriptor_sets(
				lambert_instance_data_resources.secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.mesh_resources.pipeline_layout,
				1,
				&[lambert_instance_data_resources.descriptor_set],
				&[]);
		}
		
		let index_arrays_offset = in_flight_frame.index_arrays_offset;
		let unaligned_attribute_arrays_offset = index_arrays_offset + index_arrays_size;
		let attribute_arrays_padding = (4 - unaligned_attribute_arrays_offset % 4) % 4;
		let attribute_arrays_offset = unaligned_attribute_arrays_offset + attribute_arrays_padding;

		let mut instance_group_indices = [0; MATERIALS_COUNT];

		for instance_group in &instance_groups {
			let geometry_info = &mut geometry_infos[instance_group.geometry_info_index];
			let index_array_offset = index_arrays_offset + geometry_info.index_array_relative_offset;
			let attribute_array_offset = attribute_arrays_offset + geometry_info.attribute_array_relative_offset;
			let geometry = geometry_info.geometry;

			// Copy geometry data
			if !geometry_info.copied {
				let indices = &geometry.indices;
				let attributes = &geometry.attributes;

				unsafe {
					let index_array_dst_ptr = instance_data_buffer_ptr.add(index_array_offset) as *mut u16;
					copy_nonoverlapping(indices.as_ptr(), index_array_dst_ptr, indices.len());

					let attribute_array_dst_ptr = instance_data_buffer_ptr.add(attribute_array_offset) as *mut f32;
					copy_nonoverlapping(attributes.as_ptr(), attribute_array_dst_ptr, attributes.len());
				}

				geometry_info.copied = true;
			}

			// Copy instance data
			let instance_group_index = &mut instance_group_indices[instance_group.material as usize];
			let secondary_command_buffer;

			match instance_group.material {
				Material::Basic => {
					for (instance_index, instance) in instance_group.nodes.iter().enumerate() {
						let offset = basic_instance_data_resources.array_offset + 4 * 16 * (*instance_group_index + instance_index);

						unsafe {
							let instance_data_dst_ptr = instance_data_buffer_ptr.add(offset) as *mut [f32; 4];
							copy_nonoverlapping(instance.transform.global_matrix.elements.as_ptr(), instance_data_dst_ptr, 4);
						}
					}

					secondary_command_buffer = basic_instance_data_resources.secondary_command_buffer;
				},
				Material::Normal => {
					for (instance_index, instance) in instance_group.nodes.iter().enumerate() {
						let instance_data_offset = normal_instance_data_resources.array_offset + 4 * 16 * (*instance_group_index + instance_index);

						unsafe {
							let instance_data_dst_ptr = instance_data_buffer_ptr.add(instance_data_offset) as *mut [f32; 4];
							copy_nonoverlapping(instance.transform.global_matrix.elements.as_ptr(), instance_data_dst_ptr, 4);
						}
					}

					secondary_command_buffer = normal_instance_data_resources.secondary_command_buffer;
				},
				Material::Lambert => {
					for (instance_index, instance) in instance_group.nodes.iter().enumerate() {
						let instance_data_offset = lambert_instance_data_resources.array_offset + 4 * 16 * (*instance_group_index + instance_index);

						unsafe {
							let instance_data_dst_ptr = instance_data_buffer_ptr.add(instance_data_offset) as *mut [f32; 4];
							copy_nonoverlapping(instance.transform.global_matrix.elements.as_ptr(), instance_data_dst_ptr, 4);
						}
					}

					secondary_command_buffer = lambert_instance_data_resources.secondary_command_buffer;
				}
			}

			// Record draw commands
			unsafe {
				logical_device.cmd_bind_index_buffer(secondary_command_buffer, in_flight_frame.instance_data_buffer.handle, index_array_offset as u64, vk::IndexType::UINT16);
				logical_device.cmd_bind_vertex_buffers(secondary_command_buffer, 0, &[in_flight_frame.instance_data_buffer.handle], &[attribute_array_offset as u64]);
				logical_device.cmd_draw_indexed(secondary_command_buffer, geometry.indices.len() as u32, instance_group.nodes.len() as u32, 0, 0, *instance_group_index as u32);
			}

			*instance_group_index += instance_group.nodes.len();
		}

		// Bind descriptor sets for static meshes
		unsafe {
			logical_device.cmd_bind_descriptor_sets(
				basic_instance_data_resources.secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.mesh_resources.pipeline_layout,
				1,
				&[self.mesh_resources.basic_static_descriptor_set],
				&[]);

			logical_device.cmd_bind_descriptor_sets(
				normal_instance_data_resources.secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.mesh_resources.pipeline_layout,
				1,
				&[self.mesh_resources.normal_static_descriptor_set],
				&[]);

			logical_device.cmd_bind_descriptor_sets(
				lambert_instance_data_resources.secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.mesh_resources.pipeline_layout,
				1,
				&[self.mesh_resources.lambert_static_descriptor_set],
				&[]);
		}

		// Record static mesh draw commands
		for instance_group in &self.mesh_resources.static_instance_groups {
			let geometry_info = &self.mesh_resources.static_geometry_infos[instance_group.geometry_info_index];

			let secondary_command_buffer = match instance_group.material {
				Material::Basic => basic_instance_data_resources.secondary_command_buffer,
				Material::Normal => normal_instance_data_resources.secondary_command_buffer,
				Material::Lambert => lambert_instance_data_resources.secondary_command_buffer
			};

			unsafe {
				logical_device.cmd_bind_index_buffer(secondary_command_buffer, self.mesh_resources.static_mesh_buffer.handle, geometry_info.index_array_offset as u64, vk::IndexType::UINT16);
				logical_device.cmd_bind_vertex_buffers(secondary_command_buffer, 0, &[self.mesh_resources.static_mesh_buffer.handle], &[geometry_info.attribute_array_offset as u64]);
				logical_device.cmd_draw_indexed(secondary_command_buffer, geometry_info.indices_count as u32, instance_group.instance_count as u32, 0, 0, instance_group.first_instance as u32);
			}
		}

		// End command buffers and add to submission list if there are meshes to draw
		unsafe {
			logical_device.end_command_buffer(basic_instance_data_resources.secondary_command_buffer).unwrap();
			logical_device.end_command_buffer(normal_instance_data_resources.secondary_command_buffer).unwrap();
			logical_device.end_command_buffer(lambert_instance_data_resources.secondary_command_buffer).unwrap();
		}

		let mut secondary_command_buffers = vec![];

		if material_counts[0] != 0 || self.mesh_resources.static_material_counts[0] != 0 {
			secondary_command_buffers.push(basic_instance_data_resources.secondary_command_buffer);
		}

		if material_counts[1] != 0 || self.mesh_resources.static_material_counts[1] != 0 {
			secondary_command_buffers.push(normal_instance_data_resources.secondary_command_buffer);
		}

		if material_counts[2] != 0 || self.mesh_resources.static_material_counts[2] != 0 {
			secondary_command_buffers.push(lambert_instance_data_resources.secondary_command_buffer);
		}

		// Begin text command buffer
		unsafe {
			logical_device.begin_command_buffer(text_instance_data_resources.secondary_command_buffer, &command_buffer_begin_info).unwrap();
			logical_device.cmd_bind_pipeline(text_instance_data_resources.secondary_command_buffer, vk::PipelineBindPoint::GRAPHICS, self.text_resources.pipeline);
			logical_device.cmd_bind_descriptor_sets(
				text_instance_data_resources.secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.text_resources.pipeline_layout,
				0,
				&[text_instance_data_resources.descriptor_set],
				&[]);
			logical_device.cmd_bind_descriptor_sets(
				text_instance_data_resources.secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.text_resources.pipeline_layout,
				1,
				&[self.text_resources.sampler_descriptor_set],
				&[]);
			logical_device.cmd_bind_descriptor_sets(
				text_instance_data_resources.secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.text_resources.pipeline_layout,
				2,
				&[self.text_resources.atlases_descriptor_set],
				&[]);
		}

		// Copy data into buffer and record draw commands
		for (index, text_info) in text_infos.iter().enumerate() {
			let text = text_info.text;
			let font = scene.fonts.get(&text.font).unwrap();
			let submission_info = font.submission_info.as_ref().unwrap();
			assert!(submission_info.generation == self.text_resources.submission_generation);

			let instance_data_offset = text_instance_data_resources.array_offset + 4 * 16 * index;
			let index_array_offset = index_arrays_offset + text_info.index_array_relative_offset;
			let attribute_array_offset = attribute_arrays_offset + text_info.attribute_array_relative_offset;

			let indices = text.get_vertex_indices();
			let attributes = text.get_vertex_attributes();

			let mut matrix = self.text_resources.projection_matrix;
			matrix *= &text.transform.matrix;

			unsafe {
				// Copy data
				let matrix_dst_ptr = instance_data_buffer_ptr.add(instance_data_offset) as *mut [f32; 4];
				copy_nonoverlapping(matrix.to_padded_array().as_ptr(), matrix_dst_ptr, 3);

				let atlas_index_dst_ptr = instance_data_buffer_ptr.add(instance_data_offset + 12 * 4) as *mut i32;
				copy_nonoverlapping(&(submission_info.index as i32), atlas_index_dst_ptr, 1);

				let index_array_dst_ptr = instance_data_buffer_ptr.add(index_array_offset) as *mut u16;
				copy_nonoverlapping(indices.as_ptr(), index_array_dst_ptr, indices.len());

				let attribute_array_dst_ptr = instance_data_buffer_ptr.add(attribute_array_offset) as *mut f32;
				copy_nonoverlapping(attributes.as_ptr(), attribute_array_dst_ptr, attributes.len());

				// Record draw commands
				logical_device.cmd_bind_index_buffer(text_instance_data_resources.secondary_command_buffer, in_flight_frame.instance_data_buffer.handle, index_array_offset as u64, vk::IndexType::UINT16);
				logical_device.cmd_bind_vertex_buffers(text_instance_data_resources.secondary_command_buffer, 0, &[in_flight_frame.instance_data_buffer.handle], &[attribute_array_offset as u64]);
				logical_device.cmd_draw_indexed(text_instance_data_resources.secondary_command_buffer, indices.len() as u32, 1, 0, 0, index as u32);
			}
		}

		// End command buffer and add to submission list if there are texts to draw
		unsafe { logical_device.end_command_buffer(text_instance_data_resources.secondary_command_buffer) }.unwrap();

		if !text_infos.is_empty() {
			secondary_command_buffers.push(text_instance_data_resources.secondary_command_buffer);
		}

		// Flush and unmap mesh buffer
		let range = vk::MappedMemoryRange::builder()
			.memory(in_flight_frame.instance_data_buffer.memory)
			.offset(0)
			.size(vk::WHOLE_SIZE);
		
		unsafe {
			logical_device.flush_mapped_memory_ranges(&[range.build()]).unwrap();
			logical_device.unmap_memory(in_flight_frame.instance_data_buffer.memory);
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
			.framebuffer(self.swapchain.frames[image_index as usize].framebuffer)
			.render_area(vk::Rect2D::builder()
				.offset(vk::Offset2D::builder().x(0).y(0).build())
				.extent(self.swapchain.extent)
				.build())
			.clear_values(&clear_colors);
		
		unsafe {
			logical_device.begin_command_buffer(in_flight_frame.primary_command_buffer, &command_buffer_begin_info).unwrap();
			logical_device.cmd_begin_render_pass(in_flight_frame.primary_command_buffer, &render_pass_begin_info, vk::SubpassContents::SECONDARY_COMMAND_BUFFERS);
			logical_device.cmd_execute_commands(in_flight_frame.primary_command_buffer, &secondary_command_buffers);
			logical_device.cmd_end_render_pass(in_flight_frame.primary_command_buffer);
			logical_device.end_command_buffer(in_flight_frame.primary_command_buffer).unwrap();
		}

		// Wait for image to be available then submit primary command buffer
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
			logical_device.reset_fences(&[in_flight_frame.fence]).unwrap();
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

		self.current_in_flight_frame_index = (self.current_in_flight_frame_index + 1) % IN_FLIGHT_FRAMES_COUNT;

		surface_changed
	}
}

impl Drop for Renderer {
	fn drop(&mut self) {
		let logical_device = &self.context.logical_device;

		unsafe { logical_device.device_wait_idle() }.unwrap();

		self.text_resources.drop(logical_device);
		self.mesh_resources.drop(logical_device);

		unsafe {
			for frame in &mut self.in_flight_frames {
				logical_device.destroy_semaphore(frame.image_available, None);
				logical_device.destroy_semaphore(frame.render_finished, None);
				logical_device.destroy_fence(frame.fence, None);
				frame.frame_data_buffer.drop(&self.context.logical_device);
				frame.instance_data_buffer.drop(&self.context.logical_device);
			}
			
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
	}
}