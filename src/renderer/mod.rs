use ash::{vk, version::InstanceV1_0, version::DeviceV1_0};
use std::{ffi::CString, mem::size_of};
use crate::Mesh;
use crate::math::Matrix4;

mod vulkan;

const REQUIRED_LAYERS: &[&str] = &["VK_LAYER_KHRONOS_validation"];
const REQUIRED_INSTANCE_EXTENSIONS: &[&str] = &["VK_EXT_debug_utils"];
const REQUIRED_DEVICE_EXTENSIONS: &[&str] = &["VK_KHR_swapchain"];
const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct Renderer {
	instance: ash::Instance,
	debug_utils: vulkan::DebugUtils,
	surface: vulkan::Surface,
	physical_device: vulkan::PhysicalDevice,
	logical_device: ash::Device,
	graphics_queue_family: vulkan::QueueFamily,
	present_queue_family: vulkan::QueueFamily,
	command_pool: vk::CommandPool,
	render_pass: vk::RenderPass,
	descriptor_pool: vk::DescriptorPool,
	projection_matrix_descriptor_set: vulkan::DescriptorSet,
	model_matrix_descriptor_set: vulkan::DescriptorSet,
	pipeline: vulkan::Pipeline,
	in_flight_frames: Vec<vulkan::InFlightFrame>,
	current_in_flight_frame: usize,
	swapchain: vulkan::Swapchain,
	vertex_buffer: vulkan::Buffer
}

impl Renderer {
	pub fn new(glfw: &glfw::Glfw, window: &glfw::Window) -> Self {
		let entry = ash::Entry::new().unwrap();

		let layers_c_string: Vec<CString> = REQUIRED_LAYERS.iter().map(|&s| CString::new(s).unwrap()).collect();

		let glfw_instance_extensions_string = glfw.get_required_instance_extensions().unwrap();
		let mut glfw_instance_extensions_c_string: Vec<CString> = glfw_instance_extensions_string.iter().map(|s| CString::new(s.as_str()).unwrap()).collect();
		let mut instance_extensions_c_string: Vec<CString> = REQUIRED_INSTANCE_EXTENSIONS.iter().map(|&s| CString::new(s).unwrap()).collect();
		instance_extensions_c_string.append(&mut glfw_instance_extensions_c_string);
		
		let device_extensions_c_string: Vec<CString> = REQUIRED_DEVICE_EXTENSIONS.iter().map(|&s| CString::new(s).unwrap()).collect();

		let instance = vulkan::create_instance(&entry, &layers_c_string, &instance_extensions_c_string);
		let debug_utils = vulkan::create_debug_utils(&entry, &instance);
		let (surface_extension, surface_handle) = vulkan::create_surface(&entry, &instance, window);

		let (physical_device, graphics_queue_family_index, present_queue_family_index) = vulkan::choose_physical_device(
			&instance,
			&device_extensions_c_string,
			&surface_extension,
			&surface_handle);
		
		let surface_format = vulkan::create_surface_format(&surface_extension, &surface_handle, &physical_device.handle);

		let (logical_device, graphics_queue_handle, present_queue_handle) = vulkan::create_logical_device(
			&instance,
			&layers_c_string,
			&device_extensions_c_string,
			&physical_device.handle,
			graphics_queue_family_index,
			present_queue_family_index);

		let (framebuffer_width, framebuffer_height) = window.get_framebuffer_size();
		let (swapchain_extension, swapchain_handle, swapchain_extent, swapchain_image_views) = vulkan::create_swapchain(
			&instance,
			&physical_device.handle,
			&logical_device,
			&surface_extension,
			&surface_handle,
			&surface_format,
			graphics_queue_family_index,
			present_queue_family_index,
			framebuffer_width as u32,
			framebuffer_height as u32);
		
		let render_pass = vulkan::create_render_pass(&logical_device, &surface_format);
		let descriptor_pool = vulkan::create_descriptor_pool(&logical_device);
		let (projection_matrix_descriptor_set, model_matrix_descriptor_set) = vulkan::create_descriptor_sets(&logical_device, &descriptor_pool);
		let descriptor_set_layouts = [projection_matrix_descriptor_set.layout, model_matrix_descriptor_set.layout];
		let pipeline = vulkan::create_pipeline(&logical_device, &swapchain_extent, &render_pass, &descriptor_set_layouts);
		let framebuffers = vulkan::create_framebuffers(&logical_device, &swapchain_image_views, &swapchain_extent, &render_pass);
		let command_pool = vulkan::create_command_pool(&logical_device, graphics_queue_family_index);
		let command_buffers = vulkan::create_command_buffers(&logical_device, &command_pool, swapchain_image_views.len() as u32);
		let in_flight_frames = vulkan::create_in_flight_frames(&logical_device, MAX_FRAMES_IN_FLIGHT);

		let mut swapchain_frames: Vec<vulkan::Frame> = Vec::with_capacity(swapchain_image_views.len());
		for i in 0..swapchain_image_views.len() {
			swapchain_frames.push(vulkan::Frame {
				image_view: swapchain_image_views[i],
				framebuffer: framebuffers[i],
				command_buffer: command_buffers[i],
				fence: vk::Fence::null()
			});
		}

		Renderer {
			instance,
			debug_utils,
			surface: vulkan::Surface {
				extension: surface_extension,
				handle: surface_handle,
				format: surface_format
			},
			physical_device,
			logical_device,
			graphics_queue_family: vulkan::QueueFamily {
				index: graphics_queue_family_index,
				queue: graphics_queue_handle
			},
			present_queue_family: vulkan::QueueFamily {
				index: present_queue_family_index,
				queue: present_queue_handle
			},
			command_pool,
			render_pass,
			pipeline,
			descriptor_pool,
			projection_matrix_descriptor_set,
			model_matrix_descriptor_set,
			in_flight_frames,
			current_in_flight_frame: 0,
			swapchain: vulkan::Swapchain {
				extension: swapchain_extension,
				handle: swapchain_handle,
				extent: swapchain_extent,
				frames: swapchain_frames
			},
			vertex_buffer: vulkan::Buffer {
				handle: vk::Buffer::null(),
				memory: vk::DeviceMemory::null()
			}
		}
	}

	pub fn submit_static_content(&mut self, projection_matrix: &Matrix4, meshes: &Vec<Mesh>) {
		unsafe {
			self.logical_device.device_wait_idle().unwrap();
			self.logical_device.destroy_buffer(self.vertex_buffer.handle, None);
			self.logical_device.free_memory(self.vertex_buffer.memory, None);
		}

		// Calculate total required memory size and calculate chunk sizes to help with offset calculations
		let mut total_size = 16 * size_of::<f32>();
		let mut mesh_chunk_sizes: Vec<[usize; 4]> = Vec::with_capacity(meshes.len());

		for mesh in meshes {
			let indices = mesh.geometry.get_vertex_indices();
			let attributes = mesh.geometry.get_vertex_attributes();
			let index_size = indices.len() * size_of::<u16>();
			let index_padding_size = size_of::<f32>() - (total_size + index_size) % size_of::<f32>();
			let attribute_size = attributes.len() * size_of::<f32>();
			let uniform_alignment = self.physical_device.min_uniform_buffer_offset_alignment as usize;
			let attribute_padding = uniform_alignment - (total_size + index_size + index_padding_size + attribute_size) % uniform_alignment;
			let uniform_size = 16 * size_of::<f32>();
			mesh_chunk_sizes.push([index_size, index_padding_size, attribute_size, attribute_padding]);
			total_size += index_size + index_padding_size + attribute_size + attribute_padding + uniform_size;
		}
		let total_size = total_size as u64;

		// Create a host visible staging buffer and copy the required data into it
		let staging_buffer = vulkan::create_buffer(
			&self.instance,
			&self.physical_device.handle,
			&self.logical_device,
			total_size,
			vk::BufferUsageFlags::TRANSFER_SRC,
			vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
		
		let dst_ptr = unsafe { self.logical_device.map_memory(staging_buffer.memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty()).unwrap() as *mut i8 };
		let mut mesh_offset = 16 * size_of::<f32>();

		for (i, mesh) in meshes.iter().enumerate() {
			let indices = mesh.geometry.get_vertex_indices();
			let attributes = mesh.geometry.get_vertex_attributes();
			let index_size = mesh_chunk_sizes[i][0];
			let index_padding_size = mesh_chunk_sizes[i][1];
			let attribute_size = mesh_chunk_sizes[i][2];
			let attribute_padding_size = mesh_chunk_sizes[i][3];
			let uniform_size = 16 * size_of::<f32>();

			unsafe {
				let projection_matrix_dst_ptr = dst_ptr as *mut [f32; 4];
				let mut projection_matrix = *projection_matrix;
				projection_matrix.transpose();
				std::ptr::copy_nonoverlapping(projection_matrix.elements.as_ptr(), projection_matrix_dst_ptr, projection_matrix.elements.len());

				let index_offset = mesh_offset;
				let index_dst_ptr = dst_ptr.offset(index_offset as isize) as *mut u16;
				std::ptr::copy_nonoverlapping(indices.as_ptr(), index_dst_ptr, indices.len());

				let attribute_offset = index_offset + index_size + index_padding_size;
				let attribute_dst_ptr = dst_ptr.offset(attribute_offset as isize) as *mut f32;
				std::ptr::copy_nonoverlapping(attributes.as_ptr(), attribute_dst_ptr, attributes.len());

				let model_matrix_offset = attribute_offset + attribute_size + attribute_padding_size;
				let model_matrix_dst_ptr = dst_ptr.offset(model_matrix_offset as isize) as *mut [f32; 4];
				let mut model_matrix = mesh.model_matrix;
				model_matrix.transpose();
				std::ptr::copy_nonoverlapping(model_matrix.elements.as_ptr(), model_matrix_dst_ptr, model_matrix.elements.len());
			}

			mesh_offset += index_size + index_padding_size + attribute_size + attribute_padding_size + uniform_size;
		}

		unsafe { self.logical_device.unmap_memory(staging_buffer.memory) };
		
		// Create the device local vertex buffer and copy the data from the staging buffer into it using a command buffer
		self.vertex_buffer = vulkan::create_buffer(
			&self.instance,
			&self.physical_device.handle,
			&self.logical_device,
			total_size,
			vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::UNIFORM_BUFFER,
			vk::MemoryPropertyFlags::DEVICE_LOCAL);

		let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
			.level(vk::CommandBufferLevel::PRIMARY)
			.command_pool(self.command_pool)
			.command_buffer_count(1);

		let command_buffer = unsafe { self.logical_device.allocate_command_buffers(&command_buffer_allocate_info).unwrap()[0] };

		let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
			.flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
		
		let region = vk::BufferCopy::builder()
			.size(total_size);
		let regions = [region.build()];
		
		unsafe {
			self.logical_device.begin_command_buffer(command_buffer, &command_buffer_begin_info).unwrap();
			self.logical_device.cmd_copy_buffer(command_buffer, staging_buffer.handle, self.vertex_buffer.handle, &regions);
			self.logical_device.end_command_buffer(command_buffer).unwrap();
		}

		let command_buffers = [command_buffer];
		let submit_info = vk::SubmitInfo::builder()
			.command_buffers(&command_buffers);
		let submit_infos = [submit_info.build()];
		
		unsafe {
			self.logical_device.queue_submit(self.graphics_queue_family.queue, &submit_infos, vk::Fence::null()).unwrap();
			self.logical_device.queue_wait_idle(self.graphics_queue_family.queue).unwrap();
			self.logical_device.free_command_buffers(self.command_pool, &command_buffers);
			self.logical_device.destroy_buffer(staging_buffer.handle, None);
			self.logical_device.free_memory(staging_buffer.memory, None);
		}

		// Update the descriptor sets to reference the vertex buffer
		let projection_matrix_buffer_info = vk::DescriptorBufferInfo::builder()
			.buffer(self.vertex_buffer.handle)
			.offset(0)
			.range(16 * size_of::<f32>() as u64);
		let projection_matrix_buffer_infos = [projection_matrix_buffer_info.build()];

		let projection_matrix_write_descriptor_set = vk::WriteDescriptorSet::builder()
			.dst_set(self.projection_matrix_descriptor_set.handle)
			.dst_binding(0)
			.dst_array_element(0)
			.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
			.buffer_info(&projection_matrix_buffer_infos);

		let model_matrix_buffer_info = vk::DescriptorBufferInfo::builder()
			.buffer(self.vertex_buffer.handle)
			.offset(0)
			.range(16 * size_of::<f32>() as u64);
		let model_matrix_buffer_infos = [model_matrix_buffer_info.build()];

		let model_matrix_write_descriptor_set = vk::WriteDescriptorSet::builder()
			.dst_set(self.model_matrix_descriptor_set.handle)
			.dst_binding(0)
			.dst_array_element(0)
			.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
			.buffer_info(&model_matrix_buffer_infos);
		
		let write_descriptor_sets = [projection_matrix_write_descriptor_set.build(), model_matrix_write_descriptor_set.build()];
		let copy_descriptor_sets = [];
		
		unsafe { self.logical_device.update_descriptor_sets(&write_descriptor_sets, &copy_descriptor_sets) };

		// Record the command buffers for drawing
		let clear_color = vk::ClearValue {
			color: vk::ClearColorValue {
				float32: [0.0, 0.0, 0.0, 1.0]
			}
		};
		let clear_colors = [clear_color];

		let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder();

		let mut render_pass_begin_info = vk::RenderPassBeginInfo::builder()
			.render_pass(self.render_pass)
			.render_area(vk::Rect2D::builder()
				.offset(vk::Offset2D::builder().x(0).y(0).build())
				.extent(self.swapchain.extent)
				.build())
			.clear_values(&clear_colors);

		for frame in &self.swapchain.frames {
			render_pass_begin_info = render_pass_begin_info.framebuffer(frame.framebuffer);
			
			unsafe {
				self.logical_device.begin_command_buffer(frame.command_buffer, &command_buffer_begin_info).unwrap();
				self.logical_device.cmd_begin_render_pass(frame.command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
				self.logical_device.cmd_bind_pipeline(frame.command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline.handle);
				
				let descriptor_sets = [self.projection_matrix_descriptor_set.handle];
				let dynamic_offsets = [];
				self.logical_device.cmd_bind_descriptor_sets(
					frame.command_buffer,
					vk::PipelineBindPoint::GRAPHICS,
					self.pipeline.layout,
					0,
					&descriptor_sets,
					&dynamic_offsets);
			}
			
			let mut mesh_offset = 16 * size_of::<f32>();
			for (i, mesh) in meshes.iter().enumerate() {
				let indices = mesh.geometry.get_vertex_indices();
				let index_size = mesh_chunk_sizes[i][0];
				let index_padding_size = mesh_chunk_sizes[i][1];
				let attribute_size = mesh_chunk_sizes[i][2];
				let attribute_padding_size = mesh_chunk_sizes[i][3];
				let uniform_size = 16 * size_of::<f32>();
				
				let buffers = [self.vertex_buffer.handle];
				let offsets = [(mesh_offset + index_size + index_padding_size) as u64];

				unsafe {
					self.logical_device.cmd_bind_index_buffer(frame.command_buffer, self.vertex_buffer.handle, mesh_offset as u64, vk::IndexType::UINT16);
					self.logical_device.cmd_bind_vertex_buffers(frame.command_buffer, 0, &buffers, &offsets);
					
					let descriptor_sets = [self.model_matrix_descriptor_set.handle];
					let dynamic_offsets = [(mesh_offset + index_size + index_padding_size + attribute_size + attribute_padding_size) as u32];
					self.logical_device.cmd_bind_descriptor_sets(
						frame.command_buffer,
						vk::PipelineBindPoint::GRAPHICS,
						self.pipeline.layout,
						1,
						&descriptor_sets,
						&dynamic_offsets);
					
					self.logical_device.cmd_draw_indexed(frame.command_buffer, indices.len() as u32, 1, 0, 0, 0);
				}

				mesh_offset += index_size + index_padding_size + attribute_size + attribute_padding_size + uniform_size;
			}

			unsafe {
				self.logical_device.cmd_end_render_pass(frame.command_buffer);
				self.logical_device.end_command_buffer(frame.command_buffer).unwrap();
			}
		}
		
	}
	
	pub fn render(&mut self, window: &glfw::Window) {
		let in_flight_frame = &self.in_flight_frames[self.current_in_flight_frame];
		
		let fences = [in_flight_frame.fence];
		unsafe { self.logical_device.wait_for_fences(&fences, true, std::u64::MAX).unwrap() };
		
		let result = unsafe {
			self.swapchain.extension.acquire_next_image(self.swapchain.handle,
				std::u64::MAX,
				in_flight_frame.image_available,
				vk::Fence::null())
		};

		if result.is_err() {
			if result.unwrap_err() == vk::Result::ERROR_OUT_OF_DATE_KHR {
				let (width, height) = window.get_framebuffer_size();
				self.recreate_swapchain(width as u32, height as u32);
				return;
			}

			panic!("Could not aquire a swapchain image");
		}

		let image_index = result.unwrap().0;
		let swapchain_frame = &mut self.swapchain.frames[image_index as usize];

		if swapchain_frame.fence != vk::Fence::null() {
			let fences = [swapchain_frame.fence];
			unsafe { self.logical_device.wait_for_fences(&fences, true, std::u64::MAX).unwrap() };
		}

		swapchain_frame.fence = in_flight_frame.fence;
		let image_available_semaphores = [in_flight_frame.image_available];
		let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
		let command_buffers = [swapchain_frame.command_buffer];
		let render_finished_semaphores = [in_flight_frame.render_finished];
		let submit_info = vk::SubmitInfo::builder()
			.wait_semaphores(&image_available_semaphores)
			.wait_dst_stage_mask(&wait_stages)
			.command_buffers(&command_buffers)
			.signal_semaphores(&render_finished_semaphores);
		let submit_infos = [submit_info.build()];

		unsafe {
			self.logical_device.reset_fences(&fences).unwrap();
			self.logical_device.queue_submit(self.graphics_queue_family.queue, &submit_infos, in_flight_frame.fence).unwrap();
		}

		let swapchains = [self.swapchain.handle];
		let image_indices = [image_index];
		let present_info = vk::PresentInfoKHR::builder()
			.wait_semaphores(&render_finished_semaphores)
			.swapchains(&swapchains)
			.image_indices(&image_indices);
		
		let result = unsafe { self.swapchain.extension.queue_present(self.graphics_queue_family.queue, &present_info) };

		if result.is_err() {
			if result.unwrap_err() == vk::Result::ERROR_OUT_OF_DATE_KHR {
				let (width, height) = window.get_framebuffer_size();
				self.recreate_swapchain(width as u32, height as u32);
			}
			else {
				panic!("Could not present swapchain image");
			}
		}
		else if result.unwrap() {
			let (width, height) = window.get_framebuffer_size();
			self.recreate_swapchain(width as u32, height as u32);
		}

		self.current_in_flight_frame = (self.current_in_flight_frame + 1) % MAX_FRAMES_IN_FLIGHT;
	}

	pub fn recreate_swapchain(&mut self, width: u32, height: u32) {
		unsafe {
			self.logical_device.device_wait_idle().unwrap();
			self.cleanup_swapchain();
		}

		let (swapchain_extension, swapchain_handle, swapchain_extent, swapchain_image_views) = vulkan::create_swapchain(
			&self.instance,
			&self.physical_device.handle,
			&self.logical_device,
			&self.surface.extension,
			&self.surface.handle,
			&self.surface.format,
			self.graphics_queue_family.index,
			self.present_queue_family.index,
			width,
			height);
		
		let descriptor_set_layouts = [self.projection_matrix_descriptor_set.layout, self.model_matrix_descriptor_set.layout];
		self.pipeline = vulkan::create_pipeline(&self.logical_device, &swapchain_extent, &self.render_pass, &descriptor_set_layouts);
		let framebuffers = vulkan::create_framebuffers(&self.logical_device, &swapchain_image_views, &swapchain_extent, &self.render_pass);
		let command_buffers = vulkan::create_command_buffers(&self.logical_device, &self.command_pool, swapchain_image_views.len() as u32);

		let mut swapchain_frames: Vec<vulkan::Frame> = Vec::with_capacity(swapchain_image_views.len());
		for i in 0..swapchain_image_views.len() {
			swapchain_frames.push(vulkan::Frame {
				image_view: swapchain_image_views[i],
				framebuffer: framebuffers[i],
				command_buffer: command_buffers[i],
				fence: vk::Fence::null()
			});
		}

		self.swapchain =vulkan:: Swapchain {
			extension: swapchain_extension,
			handle: swapchain_handle,
			extent: swapchain_extent,
			frames: swapchain_frames
		};
	}

	unsafe fn cleanup_swapchain(&mut self) {
		let mut command_buffers: Vec<vk::CommandBuffer> = Vec::with_capacity(self.swapchain.frames.len());

		for frame in &self.swapchain.frames {
			self.logical_device.destroy_image_view(frame.image_view, None);
			self.logical_device.destroy_framebuffer(frame.framebuffer, None);
			command_buffers.push(frame.command_buffer);
		}
		
		self.logical_device.free_command_buffers(self.command_pool, &command_buffers);
		self.logical_device.destroy_pipeline_layout(self.pipeline.layout, None);
		self.logical_device.destroy_pipeline(self.pipeline.handle, None);
		self.swapchain.extension.destroy_swapchain(self.swapchain.handle, None);
	}
}

impl Drop for Renderer {
	fn drop(&mut self) {
		unsafe {
			self.logical_device.device_wait_idle().unwrap();
			self.cleanup_swapchain();

			for frame in &self.in_flight_frames {
				self.logical_device.destroy_semaphore(frame.image_available, None);
				self.logical_device.destroy_semaphore(frame.render_finished, None);
				self.logical_device.destroy_fence(frame.fence, None);
			}

			self.logical_device.destroy_descriptor_pool(self.descriptor_pool, None);
			self.logical_device.destroy_descriptor_set_layout(self.model_matrix_descriptor_set.layout, None);
			self.logical_device.destroy_descriptor_set_layout(self.projection_matrix_descriptor_set.layout, None);
			self.logical_device.destroy_buffer(self.vertex_buffer.handle, None);
			self.logical_device.free_memory(self.vertex_buffer.memory, None);
			self.logical_device.destroy_render_pass(self.render_pass, None);
			self.logical_device.destroy_command_pool(self.command_pool, None);
			self.logical_device.destroy_device(None);
			self.surface.extension.destroy_surface(self.surface.handle, None);
			self.debug_utils.extension.destroy_debug_utils_messenger(self.debug_utils.messenger_handle, None);
			self.instance.destroy_instance(None);
		}
	}
}