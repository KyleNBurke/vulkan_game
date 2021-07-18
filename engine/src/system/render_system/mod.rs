use std::{cmp::max, fs::File, mem::size_of_val, ptr::copy_nonoverlapping};
use crate::{
	Camera,
	Entity,
	component::{ComponentList, MultiComponentList, Light, Mesh, TextComponentList, Transform2DComponentList, Transform3DComponentList, mesh::Material, Text},
	Font,
	Geometry3D,
	math::{vector3, Vector3},
	pool::{Pool, Handle},
	vulkan::{Context, Buffer}
};
use ash::{vk, version::DeviceV1_0, extensions::khr};

mod creation;
use creation::*;

mod mesh_render_system;
use mesh_render_system::*;

mod text_render_system;
use text_render_system::*;

const IN_FLIGHT_FRAMES_COUNT: usize = 2;
const FRAME_DATA_MEMORY_SIZE: usize = 76 * 4;
const MATERIALS_COUNT: usize = 4;
const MAX_POINT_LIGHTS: usize = 5;
const MAX_FONTS: usize = 10;

pub struct RenderSystem {
	context: Context,
	render_pass: vk::RenderPass,
	swapchain: Swapchain,
	descriptor_pool: vk::DescriptorPool,
	command_pool: vk::CommandPool,
	frame_data_descriptor_set_layout: vk::DescriptorSetLayout,
	instance_data_descriptor_set_layout: vk::DescriptorSetLayout,
	in_flight_frames: [InFlightFrame; IN_FLIGHT_FRAMES_COUNT],
	current_in_flight_frame_index: usize,
	mesh_resources: MeshRenderSystem,
	text_resources: TextRenderSystem
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
	line_instance_data_resources: InstanceDataResources,
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
		line_instance_data_array_offset: usize,
		line_instance_data_array_size: usize,
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
		// Line
		let line_descriptor_buffer_info = vk::DescriptorBufferInfo::builder()
			.buffer(self.instance_data_buffer.handle)
			.offset(line_instance_data_array_offset as u64)
			.range(max(1, line_instance_data_array_size) as u64);
		let line_descriptor_buffer_infos = [line_descriptor_buffer_info.build()];

		let line_write_descriptor_set = vk::WriteDescriptorSet::builder()
			.dst_set(self.line_instance_data_resources.descriptor_set)
			.dst_binding(0)
			.dst_array_element(0)
			.descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
			.buffer_info(&line_descriptor_buffer_infos);
		
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
			line_write_descriptor_set.build(),
			basic_write_descriptor_set.build(),
			normal_write_descriptor_set.build(),
			lambert_write_descriptor_set.build(),
			text_write_descriptor_set.build()
		];
		
		unsafe { logical_device.update_descriptor_sets(&write_descriptor_sets, &[]) };

		// Set offsets and sizes
		self.line_instance_data_resources.array_offset = line_instance_data_array_offset;
		self.line_instance_data_resources.array_size = line_instance_data_array_size;

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

impl RenderSystem {
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
		let mesh_resources = MeshRenderSystem::new(&context.logical_device, frame_data_descriptor_set_layout, instance_data_descriptor_set_layout, swapchain.extent, render_pass, descriptor_pool);
		let text_renderer = TextRenderSystem::new(&context.logical_device, instance_data_descriptor_set_layout, swapchain.extent, render_pass, descriptor_pool);

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

	pub fn get_swapchain_extent(&self) -> (u32, u32) {
		let extent = &self.swapchain.extent;
		(extent.width, extent.height)
	}

	pub fn recreate_swapchain(&mut self, framebuffer_width: i32, framebuffer_height: i32) -> (u32, u32) {
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
		self.mesh_resources.handle_swapchain_recreation(&self.context.logical_device, self.swapchain.extent, self.render_pass);
		self.text_resources.handle_swapchain_recreation(&self.context.logical_device, self.swapchain.extent, self.render_pass);
		println!("Swapchain recreated");

		let extent = &self.swapchain.extent;
		(extent.width, extent.height)
	}

	pub fn submit_static_geometries(&mut self, geometries: &mut Pool<Geometry3D>, handles: &[Handle]) {
		self.mesh_resources.submit_static_geometries(&self.context, self.command_pool, geometries, handles);
		println!("Static meshes submitted");
	}

	pub fn submit_fonts(&mut self, fonts: &mut Pool<Font>) {
		self.text_resources.submit_fonts(&self.context, self.command_pool, fonts);
		println!("Fonts submitted");
	}

	pub fn render(&mut self,
		camera: &Camera,
		light_components: &ComponentList<Light>,
		geometries: &Pool<Geometry3D>,
		mesh_components: &MultiComponentList<Mesh>,
		transform3d_components: &Transform3DComponentList,
		fonts: &Pool<Font>,
		text_components: &TextComponentList,
		transform2d_components: &Transform2DComponentList) -> bool
	{
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
		let projection_matrix = &camera.projection_matrix.elements;
		let projection_matrix_dst_ptr = frame_data_buffer_ptr as *mut [f32; 4];
		unsafe { copy_nonoverlapping(projection_matrix.as_ptr(), projection_matrix_dst_ptr, 4) };

		let mut inverse_view_matrix = camera.transform.global_matrix;
		inverse_view_matrix.invert();
		unsafe {
			let inverse_view_matrix_dst_ptr = frame_data_buffer_ptr.add(16 * 4) as *mut [f32; 4];
			copy_nonoverlapping(inverse_view_matrix.elements.as_ptr(), inverse_view_matrix_dst_ptr, 4);
		}

		// Iterate over lights to
		// - Calculate the total ambient light color and intensity
		// - Copy the point light data into the frame data buffer
		let mut total_ambient_light_color = vector3::ZERO;
		let mut total_ambient_light_intensity = 0.0;

		let mut point_light_count = 0;
		let position_base_offest = 36 * 4;
		let color_base_offest = 40 * 4;
		let stride = 8 * 4;

		for (entity, light) in light_components.iter() {
			match light {
				Light::AmbientLight(ambient_light) => {
					total_ambient_light_color += ambient_light.color;
					total_ambient_light_intensity += ambient_light.intensity;
				},
				Light::PointLight(point_light) => {
					let intensified_color = point_light.color * point_light.intensity;
					let position = transform3d_components.borrow(entity).global_matrix.extract_position();

					unsafe {
						let position_dst_ptr = frame_data_buffer_ptr.add(position_base_offest + stride * point_light_count) as *mut Vector3;
						copy_nonoverlapping(&position as *const Vector3, position_dst_ptr, 1);

						let color_dst_ptr = frame_data_buffer_ptr.add(color_base_offest + stride * point_light_count) as *mut Vector3;
						copy_nonoverlapping(&intensified_color as *const Vector3, color_dst_ptr, 1);
					}

					point_light_count += 1;
				}
			}
		}

		// Copy point light count into frame data buffer
		assert!(point_light_count <= MAX_POINT_LIGHTS, "Cannot render scene because {} point lights is more than the limit {}", point_light_count, MAX_POINT_LIGHTS);
		unsafe {
			let point_light_count_dst_ptr = frame_data_buffer_ptr.add(35 * 4) as *mut u32;
			copy_nonoverlapping(&(point_light_count as u32) as *const u32, point_light_count_dst_ptr, 1);
		}

		// Copy total intensified ambient light color into frame data buffer
		let total_ambient_light_intensified_color = total_ambient_light_color * total_ambient_light_intensity;
		unsafe {
			let ambient_light_dst_ptr = frame_data_buffer_ptr.add(32 * 4) as *mut Vector3;
			copy_nonoverlapping(&total_ambient_light_intensified_color as *const Vector3, ambient_light_dst_ptr, 1);
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

		// Iterate over meshes to
		// - Calculate the offsets and size of the data
		// - Count the number of entities of each material to render
		struct InstanceGroupInfo<'a> {
			tuple: &'a (Vec<Entity>, Mesh),
			index_array_relative_offset: usize,
			attribute_array_relative_offset: usize
		}

		let mut instance_group_infos: Vec<InstanceGroupInfo> = Vec::new();
		let mut index_arrays_size = 0;
		let mut attribute_arrays_size = 0;
		let mut material_counts = [0; MATERIALS_COUNT];

		for tuple in mesh_components.iter() {
			instance_group_infos.push(InstanceGroupInfo {
				tuple,
				index_array_relative_offset: index_arrays_size,
				attribute_array_relative_offset: attribute_arrays_size
			});
			
			let (instances, mesh) = tuple;
			let geometry = geometries.borrow(mesh.geometry_handle);

			index_arrays_size += size_of_val(geometry.indices());
			attribute_arrays_size += size_of_val(geometry.attributes());
			material_counts[mesh.material as usize] += instances.len();
		}

		// Iterate over text to
		struct TextInfo<'a> {
			tuple: &'a (Entity, Text),
			index_array_relative_offset: usize,
			attribute_array_relative_offset: usize
		}

		let mut text_infos: Vec<TextInfo> = Vec::new();

		for tuple in text_components.iter() {
			let (_, text) = tuple;

			if text.string.is_empty() {
				continue;
			}

			let vertex_indices_size = size_of_val(text.indices());
			let vertex_attributes_size = size_of_val(text.attributes());

			text_infos.push(TextInfo {
				tuple,
				index_array_relative_offset: index_arrays_size,
				attribute_array_relative_offset: attribute_arrays_size
			});

			index_arrays_size += vertex_indices_size;
			attribute_arrays_size += vertex_attributes_size;
		}

		// Calculate offsets
		let alignment = self.context.physical_device.min_storage_buffer_offset_alignment as usize;

		let line_instance_data_array_offset = 0;
		let line_instance_data_array_size = 4 * 16 * material_counts[Material::Line as usize];

		let unaligned_basic_instance_data_array_offset = line_instance_data_array_offset + line_instance_data_array_size;
		let basic_instance_data_array_padding = (alignment - unaligned_basic_instance_data_array_offset % alignment) % alignment;
		let basic_instance_data_array_offset = unaligned_basic_instance_data_array_offset + basic_instance_data_array_padding;
		let basic_instance_data_array_size = 4 * 16 * material_counts[Material::Basic as usize];

		let unaligned_normal_instance_data_array_offset = basic_instance_data_array_offset + basic_instance_data_array_size;
		let normal_instance_data_array_padding = (alignment - unaligned_normal_instance_data_array_offset % alignment) % alignment;
		let normal_instance_data_array_offset = unaligned_normal_instance_data_array_offset + normal_instance_data_array_padding;
		let normal_instance_data_array_size = 4 * 16 * material_counts[Material::Normal as usize];
		
		let unaligned_lambert_instance_data_array_offset = normal_instance_data_array_offset + normal_instance_data_array_size;
		let lambert_instance_data_array_padding = (alignment - unaligned_lambert_instance_data_array_offset % alignment) % alignment;
		let lambert_instance_data_array_offset = unaligned_lambert_instance_data_array_offset + lambert_instance_data_array_padding;
		let lambert_instance_data_array_size = 4 * 16 * material_counts[Material::Lambert as usize];

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
				line_instance_data_array_offset,
				line_instance_data_array_size,
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
				line_instance_data_array_offset,
				line_instance_data_array_size,
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
		let line_instance_data_resources = &in_flight_frame.line_instance_data_resources;
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
			// Line
			logical_device.begin_command_buffer(line_instance_data_resources.secondary_command_buffer, &command_buffer_begin_info).unwrap();
			logical_device.cmd_bind_pipeline(line_instance_data_resources.secondary_command_buffer, vk::PipelineBindPoint::GRAPHICS, self.mesh_resources.line_pipeline);
			logical_device.cmd_bind_descriptor_sets(
				line_instance_data_resources.secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.mesh_resources.pipeline_layout,
				0,
				&[in_flight_frame.frame_data_descriptor_set],
				&[]);
			logical_device.cmd_bind_descriptor_sets(
				line_instance_data_resources.secondary_command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				self.mesh_resources.pipeline_layout,
				1,
				&[line_instance_data_resources.descriptor_set],
				&[]);
			
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

		for instance_group in &instance_group_infos {
			let index_array_offset = index_arrays_offset + instance_group.index_array_relative_offset;
			let attribute_array_offset = attribute_arrays_offset + instance_group.attribute_array_relative_offset;
			let (instances, mesh) = instance_group.tuple;
			let geometry = geometries.borrow(mesh.geometry_handle);

			// Copy geometry data
			let indices = geometry.indices();
			let attributes = geometry.attributes();

			unsafe {
				let index_array_dst_ptr = instance_data_buffer_ptr.add(index_array_offset) as *mut u16;
				copy_nonoverlapping(indices.as_ptr(), index_array_dst_ptr, indices.len());

				let attribute_array_dst_ptr = instance_data_buffer_ptr.add(attribute_array_offset) as *mut f32;
				copy_nonoverlapping(attributes.as_ptr(), attribute_array_dst_ptr, attributes.len());
			}

			// Copy instance data
			let instance_group_index = &mut instance_group_indices[mesh.material as usize];
			let secondary_command_buffer;

			match mesh.material {
				Material::Line => {
					for (instance_index, instance) in instances.iter().enumerate() {
						let transform_ptr = transform3d_components.borrow(instance).global_matrix.elements.as_ptr();
						let offset = line_instance_data_resources.array_offset + 4 * 16 * (*instance_group_index + instance_index);

						unsafe {
							let instance_data_dst_ptr = instance_data_buffer_ptr.add(offset) as *mut [f32; 4];
							copy_nonoverlapping(transform_ptr, instance_data_dst_ptr, 4);
						}
					}

					secondary_command_buffer = line_instance_data_resources.secondary_command_buffer;
				},
				Material::Basic => {
					for (instance_index, instance) in instances.iter().enumerate() {
						let transform_ptr = transform3d_components.borrow(instance).global_matrix.elements.as_ptr();
						let offset = basic_instance_data_resources.array_offset + 4 * 16 * (*instance_group_index + instance_index);

						unsafe {
							let instance_data_dst_ptr = instance_data_buffer_ptr.add(offset) as *mut [f32; 4];
							copy_nonoverlapping(transform_ptr, instance_data_dst_ptr, 4);
						}
					}

					secondary_command_buffer = basic_instance_data_resources.secondary_command_buffer;
				},
				Material::Normal => {
					for (instance_index, instance) in instances.iter().enumerate() {
						let transform_ptr = transform3d_components.borrow(instance).global_matrix.elements.as_ptr();
						let instance_data_offset = normal_instance_data_resources.array_offset + 4 * 16 * (*instance_group_index + instance_index);

						unsafe {
							let instance_data_dst_ptr = instance_data_buffer_ptr.add(instance_data_offset) as *mut [f32; 4];
							copy_nonoverlapping(transform_ptr, instance_data_dst_ptr, 4);
						}
					}

					secondary_command_buffer = normal_instance_data_resources.secondary_command_buffer;
				},
				Material::Lambert => {
					for (instance_index, instance) in instances.iter().enumerate() {
						let transform_ptr = transform3d_components.borrow(instance).global_matrix.elements.as_ptr();
						let instance_data_offset = lambert_instance_data_resources.array_offset + 4 * 16 * (*instance_group_index + instance_index);

						unsafe {
							let instance_data_dst_ptr = instance_data_buffer_ptr.add(instance_data_offset) as *mut [f32; 4];
							copy_nonoverlapping(transform_ptr, instance_data_dst_ptr, 4);
						}
					}

					secondary_command_buffer = lambert_instance_data_resources.secondary_command_buffer;
				}
			}

			// Record draw commands
			unsafe {
				logical_device.cmd_bind_index_buffer(secondary_command_buffer, in_flight_frame.instance_data_buffer.handle, index_array_offset as u64, vk::IndexType::UINT16);
				logical_device.cmd_bind_vertex_buffers(secondary_command_buffer, 0, &[in_flight_frame.instance_data_buffer.handle], &[attribute_array_offset as u64]);
				logical_device.cmd_draw_indexed(secondary_command_buffer, geometry.indices().len() as u32, instances.len() as u32, 0, 0, *instance_group_index as u32);
			}

			*instance_group_index += instances.len();
		}

		// End command buffers and add to submission list if there are meshes to draw
		unsafe {
			logical_device.end_command_buffer(line_instance_data_resources.secondary_command_buffer).unwrap();
			logical_device.end_command_buffer(basic_instance_data_resources.secondary_command_buffer).unwrap();
			logical_device.end_command_buffer(normal_instance_data_resources.secondary_command_buffer).unwrap();
			logical_device.end_command_buffer(lambert_instance_data_resources.secondary_command_buffer).unwrap();
		}

		let mut secondary_command_buffers = vec![];

		if material_counts[Material::Line as usize] != 0 || self.mesh_resources.static_material_counts[Material::Line as usize] != 0 {
			secondary_command_buffers.push(line_instance_data_resources.secondary_command_buffer);
		}

		if material_counts[Material::Basic as usize] != 0 || self.mesh_resources.static_material_counts[Material::Basic as usize] != 0 {
			secondary_command_buffers.push(basic_instance_data_resources.secondary_command_buffer);
		}

		if material_counts[Material::Normal as usize] != 0 || self.mesh_resources.static_material_counts[Material::Normal as usize] != 0 {
			secondary_command_buffers.push(normal_instance_data_resources.secondary_command_buffer);
		}

		if material_counts[Material::Lambert as usize] != 0 || self.mesh_resources.static_material_counts[Material::Lambert as usize] != 0 {
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

		// Copy text data into buffer and record draw commands
		for (index, text_info) in text_infos.iter().enumerate() {
			let (entity, text) = text_info.tuple;
			let font = fonts.borrow(text.font);
			let submission_info = font.submission_info.as_ref().unwrap(); // error message
			assert!(submission_info.generation == self.text_resources.submission_generation);

			let instance_data_offset = text_instance_data_resources.array_offset + 4 * 16 * index;
			let index_array_offset = index_arrays_offset + text_info.index_array_relative_offset;
			let attribute_array_offset = attribute_arrays_offset + text_info.attribute_array_relative_offset;

			let indices = text.indices();
			let attributes = text.attributes();

			let projection_matrix = &self.text_resources.projection_matrix;
			let transform_matrix = &transform2d_components.borrow(entity).matrix;
			let final_matrix = projection_matrix * transform_matrix;

			unsafe {
				// Copy data
				let final_matrix_dst_ptr = instance_data_buffer_ptr.add(instance_data_offset) as *mut [f32; 4];
				copy_nonoverlapping(final_matrix.to_padded_array().as_ptr(), final_matrix_dst_ptr, 3);

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

impl Drop for RenderSystem {
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