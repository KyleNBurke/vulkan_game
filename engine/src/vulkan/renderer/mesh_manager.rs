use std::{ffi::CString, mem::{size_of, size_of_val}, ptr};
use ash::{vk, version::DeviceV1_0};
use crate::{vulkan::{Buffer, Renderer}, Geometry3D, mesh::{Material, StaticMesh, StaticInstancedMesh}, pool::Pool};

pub struct StaticMeshData {
	pub index_offset: usize,
	pub attribute_offset: usize,
	pub uniform_offset: usize,
	pub index_count: usize,
	pub material: Material
}

pub struct StaticInstancedMeshData {
	pub index_offset: usize,
	pub attribute_offset: usize,
	pub matrix_offset: usize,
	pub index_count: usize,
	pub instance_count: usize,
	pub material: Material
}

pub struct MeshManager {
	pub frame_data_descriptor_set_layout: vk::DescriptorSetLayout,
	pub mesh_data_descriptor_set_layout: vk::DescriptorSetLayout,
	pub pipeline_layout: vk::PipelineLayout,
	pub instanced_pipeline_layout: vk::PipelineLayout,
	pub basic_pipeline: vk::Pipeline,
	pub basic_instanced_pipeline: vk::Pipeline,
	pub lambert_pipeline: vk::Pipeline,
	pub static_mesh_data_descriptor_set: vk::DescriptorSet,
	pub static_mesh_buffer: Buffer,
	pub static_mesh_data: Vec<StaticMeshData>,
	pub static_instanced_mesh_data: Vec<StaticInstancedMeshData>
}

impl MeshManager {
	pub fn new(logical_device: &ash::Device, extent: vk::Extent2D, render_pass: vk::RenderPass, descriptor_pool: vk::DescriptorPool) -> Self {
		let (frame_data_descriptor_set_layout, mesh_data_descriptor_set_layout) = Self::create_descriptor_set_layouts(logical_device);
		let pipeline_layout = Self::create_pipeline_layout(logical_device, frame_data_descriptor_set_layout, mesh_data_descriptor_set_layout);
		let instanced_pipeline_layout = Self::create_instanced_pipeline_layout(logical_device, frame_data_descriptor_set_layout);
		let (basic_pipeline, basic_instanced_pipeline, lambert_pipeline) = Self::create_pipelines(logical_device, extent, pipeline_layout, instanced_pipeline_layout, render_pass);
		let static_mesh_data_descriptor_set = Self::create_static_mesh_data_descriptor_set(logical_device, mesh_data_descriptor_set_layout, descriptor_pool);

		let static_mesh_buffer = Buffer::null(
			vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::UNIFORM_BUFFER,
			vk::MemoryPropertyFlags::DEVICE_LOCAL);

		Self {
			frame_data_descriptor_set_layout,
			mesh_data_descriptor_set_layout,
			pipeline_layout,
			instanced_pipeline_layout,
			basic_pipeline,
			basic_instanced_pipeline,
			lambert_pipeline,
			static_mesh_data_descriptor_set,
			static_mesh_buffer,
			static_mesh_data: vec![],
			static_instanced_mesh_data: vec![]
		}
	}

	fn create_descriptor_set_layouts(logical_device: &ash::Device) -> (vk::DescriptorSetLayout, vk::DescriptorSetLayout) {
		// Frame data
		let frame_data_descriptor_set_layout_binding = vk::DescriptorSetLayoutBinding::builder()
			.binding(0)
			.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
			.descriptor_count(1)
			.stage_flags(vk::ShaderStageFlags::VERTEX);

		let frame_data_descriptor_set_layout_bindings = [frame_data_descriptor_set_layout_binding.build()];
		let frame_data_descriptor_set_layout_create_info = vk::DescriptorSetLayoutCreateInfo::builder()
			.bindings(&frame_data_descriptor_set_layout_bindings);

		let frame_data_descriptor_set_layout = unsafe { logical_device.create_descriptor_set_layout(&frame_data_descriptor_set_layout_create_info, None) }.unwrap();

		// Mesh data
		let mesh_data_descriptor_set_layout_binding = vk::DescriptorSetLayoutBinding::builder()
			.binding(0)
			.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
			.descriptor_count(1)
			.stage_flags(vk::ShaderStageFlags::VERTEX);

		let mesh_data_descriptor_set_layout_bindings = [mesh_data_descriptor_set_layout_binding.build()];
		let mesh_data_descriptor_set_layout_create_info = vk::DescriptorSetLayoutCreateInfo::builder()
			.bindings(&mesh_data_descriptor_set_layout_bindings);

		let mesh_data_descriptor_set_layout = unsafe { logical_device.create_descriptor_set_layout(&mesh_data_descriptor_set_layout_create_info, None) }.unwrap();

		(frame_data_descriptor_set_layout, mesh_data_descriptor_set_layout)
	}

	fn create_pipeline_layout(logical_device: &ash::Device, frame_data_descriptor_set_layout: vk::DescriptorSetLayout, mesh_data_descriptor_set_layout: vk::DescriptorSetLayout) -> vk::PipelineLayout {
		let descriptor_set_layouts = [frame_data_descriptor_set_layout, mesh_data_descriptor_set_layout];

		let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
			.set_layouts(&descriptor_set_layouts);

		unsafe { logical_device.create_pipeline_layout(&pipeline_layout_create_info, None) }.unwrap()
	}

	fn create_instanced_pipeline_layout(logical_device: &ash::Device, frame_data_descriptor_set_layout: vk::DescriptorSetLayout) -> vk::PipelineLayout {
		let descriptor_set_layouts = [frame_data_descriptor_set_layout];

		let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
			.set_layouts(&descriptor_set_layouts);

		unsafe { logical_device.create_pipeline_layout(&pipeline_layout_create_info, None) }.unwrap()
	}

	fn create_pipelines(
		logical_device: &ash::Device,
		extent: vk::Extent2D,
		pipeline_layout: vk::PipelineLayout,
		instanced_pipeline_layout: vk::PipelineLayout,
		render_pass: vk::RenderPass,
	) -> (vk::Pipeline, vk::Pipeline, vk::Pipeline) {
		// Shared
		let entry_point = CString::new("main").unwrap();
		let entry_point_cstr = entry_point.as_c_str();

		let input_binding_description = vk::VertexInputBindingDescription::builder()
			.binding(0)
			.stride(24)
			.input_rate(vk::VertexInputRate::VERTEX);
		let input_binding_descriptions = [input_binding_description.build()];

		let input_attribute_description_position = vk::VertexInputAttributeDescription::builder()	
			.binding(0)
			.location(0)
			.format(vk::Format::R32G32B32_SFLOAT)
			.offset(0)
			.build();
		
		let input_assembly_state_create_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
			.topology(vk::PrimitiveTopology::TRIANGLE_LIST)
			.primitive_restart_enable(false);

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

		let rasterization_state_create_info = vk::PipelineRasterizationStateCreateInfo::builder()
			.depth_clamp_enable(false)
			.rasterizer_discard_enable(false)
			.polygon_mode(vk::PolygonMode::FILL)
			.line_width(1.0)
			.cull_mode(vk::CullModeFlags::BACK)
			.front_face(vk::FrontFace::CLOCKWISE)
			.depth_bias_enable(false);

		let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo::builder()
			.sample_shading_enable(false)
			.rasterization_samples(vk::SampleCountFlags::TYPE_1);

		let depth_stencil_state_create_info = vk::PipelineDepthStencilStateCreateInfo::builder()
			.depth_test_enable(true)
			.depth_write_enable(true)
			.depth_compare_op(vk::CompareOp::LESS)
			.depth_bounds_test_enable(false)
			.stencil_test_enable(false);

		let color_blend_attachment_state = vk::PipelineColorBlendAttachmentState::builder()
			.color_write_mask(vk::ColorComponentFlags::all())
			.blend_enable(false);
		let color_blend_attachment_states = [color_blend_attachment_state.build()];

		let color_blend_state_create_info = vk::PipelineColorBlendStateCreateInfo::builder()
			.logic_op_enable(false)
			.attachments(&color_blend_attachment_states);

		// Basic
		let basic_vert_module = Renderer::create_shader_module(logical_device, "basic.vert.spv");
		let basic_vert_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
			.stage(vk::ShaderStageFlags::VERTEX)
			.module(basic_vert_module)
			.name(entry_point_cstr);
		
		let basic_frag_module = Renderer::create_shader_module(logical_device, "basic.frag.spv");
		let basic_frag_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
			.stage(vk::ShaderStageFlags::FRAGMENT)
			.module(basic_frag_module)
			.name(entry_point_cstr);
		
		let basic_stage_create_infos = [basic_vert_stage_create_info.build(), basic_frag_stage_create_info.build()];
		let basic_input_attribute_descriptions = [input_attribute_description_position];

		let basic_vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo::builder()
			.vertex_binding_descriptions(&input_binding_descriptions)
			.vertex_attribute_descriptions(&basic_input_attribute_descriptions);

		let basic_pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
			.stages(&basic_stage_create_infos)
			.vertex_input_state(&basic_vertex_input_state_create_info)
			.input_assembly_state(&input_assembly_state_create_info)
			.viewport_state(&viewport_state_create_info)
			.rasterization_state(&rasterization_state_create_info)
			.multisample_state(&multisample_state_create_info)
			.depth_stencil_state(&depth_stencil_state_create_info)
			.color_blend_state(&color_blend_state_create_info)
			.layout(pipeline_layout)
			.render_pass(render_pass)
			.subpass(0);
		
		// Basic instanced
		let basic_instanced_vert_module = Renderer::create_shader_module(logical_device, "basic_instanced.vert.spv");
		let basic_instanced_vert_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
			.stage(vk::ShaderStageFlags::VERTEX)
			.module(basic_instanced_vert_module)
			.name(entry_point_cstr);
		
		let basic_instanced_frag_module = Renderer::create_shader_module(logical_device, "basic.frag.spv");
		let basic_instanced_frag_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
			.stage(vk::ShaderStageFlags::FRAGMENT)
			.module(basic_instanced_frag_module)
			.name(entry_point_cstr);
		
		let basic_instanced_stage_create_infos = [basic_instanced_vert_stage_create_info.build(), basic_instanced_frag_stage_create_info.build()];

		let input_binding_description_vertex = vk::VertexInputBindingDescription::builder()
			.binding(0)
			.stride(24)
			.input_rate(vk::VertexInputRate::VERTEX);

		let input_binding_description_instance = vk::VertexInputBindingDescription::builder()
			.binding(1)
			.stride(64)
			.input_rate(vk::VertexInputRate::INSTANCE);
		
		let basic_instanced_input_binding_descriptions = [input_binding_description_vertex.build(), input_binding_description_instance.build()];

		let input_attribute_description_model_matrix_1 = vk::VertexInputAttributeDescription::builder()
			.binding(1)
			.location(1)
			.format(vk::Format::R32G32B32A32_SFLOAT)
			.offset(0)
			.build();
		
		let input_attribute_description_model_matrix_2 = vk::VertexInputAttributeDescription::builder()
			.binding(1)
			.location(2)
			.format(vk::Format::R32G32B32A32_SFLOAT)
			.offset(16)
			.build();
		
		let input_attribute_description_model_matrix_3 = vk::VertexInputAttributeDescription::builder()
			.binding(1)
			.location(3)
			.format(vk::Format::R32G32B32A32_SFLOAT)
			.offset(32)
			.build();
		
		let input_attribute_description_model_matrix_4 = vk::VertexInputAttributeDescription::builder()
			.binding(1)
			.location(4)
			.format(vk::Format::R32G32B32A32_SFLOAT)
			.offset(48)
			.build();
		
		let basic_instanced_input_attribute_descriptions = [
			input_attribute_description_position,
			input_attribute_description_model_matrix_1,
			input_attribute_description_model_matrix_2,
			input_attribute_description_model_matrix_3,
			input_attribute_description_model_matrix_4];

		let basic_instanced_vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo::builder()
			.vertex_binding_descriptions(&basic_instanced_input_binding_descriptions)
			.vertex_attribute_descriptions(&basic_instanced_input_attribute_descriptions);

		let basic_instanced_pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
			.stages(&basic_instanced_stage_create_infos)
			.vertex_input_state(&basic_instanced_vertex_input_state_create_info)
			.input_assembly_state(&input_assembly_state_create_info)
			.viewport_state(&viewport_state_create_info)
			.rasterization_state(&rasterization_state_create_info)
			.multisample_state(&multisample_state_create_info)
			.depth_stencil_state(&depth_stencil_state_create_info)
			.color_blend_state(&color_blend_state_create_info)
			.layout(instanced_pipeline_layout)
			.render_pass(render_pass)
			.subpass(0);
		
		// Lambert
		let lambert_vert_module =  Renderer::create_shader_module(logical_device, "lambert.vert.spv");
		let lambert_vert_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
			.stage(vk::ShaderStageFlags::VERTEX)
			.module(lambert_vert_module)
			.name(entry_point_cstr);

		let lambert_frag_module =  Renderer::create_shader_module(logical_device, "lambert.frag.spv");
		let lambert_frag_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
			.stage(vk::ShaderStageFlags::FRAGMENT)
			.module(lambert_frag_module)
			.name(entry_point_cstr);

		let lambert_stage_create_infos = [lambert_vert_stage_create_info.build(), lambert_frag_stage_create_info.build()];

		let input_attribute_description_normal = vk::VertexInputAttributeDescription::builder()	
			.binding(0)
			.location(1)
			.format(vk::Format::R32G32B32_SFLOAT)
			.offset(12)
			.build();

		let lambert_input_attribute_descriptions = [input_attribute_description_position, input_attribute_description_normal];

		let lambert_vert_input_state_create_info = vk::PipelineVertexInputStateCreateInfo::builder()
			.vertex_binding_descriptions(&input_binding_descriptions)
			.vertex_attribute_descriptions(&lambert_input_attribute_descriptions);

		let lambert_pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
			.stages(&lambert_stage_create_infos)
			.vertex_input_state(&lambert_vert_input_state_create_info)
			.input_assembly_state(&input_assembly_state_create_info)
			.viewport_state(&viewport_state_create_info)
			.rasterization_state(&rasterization_state_create_info)
			.multisample_state(&multisample_state_create_info)
			.depth_stencil_state(&depth_stencil_state_create_info)
			.color_blend_state(&color_blend_state_create_info)
			.layout(pipeline_layout)
			.render_pass(render_pass)
			.subpass(0);
		
		// Create pipelines
		let pipeline_create_infos = [
			basic_pipeline_create_info.build(),
			basic_instanced_pipeline_create_info.build(),
			lambert_pipeline_create_info.build()];
		
		let pipelines = unsafe { logical_device.create_graphics_pipelines(vk::PipelineCache::null(), &pipeline_create_infos, None) }.unwrap();

		unsafe {
			logical_device.destroy_shader_module(basic_vert_module, None);
			logical_device.destroy_shader_module(basic_frag_module, None);

			logical_device.destroy_shader_module(basic_instanced_vert_module, None);
			logical_device.destroy_shader_module(basic_instanced_frag_module, None);

			logical_device.destroy_shader_module(lambert_vert_module, None);
			logical_device.destroy_shader_module(lambert_frag_module, None);
		}

		(pipelines[0], pipelines[1], pipelines[2])
	}

	fn create_static_mesh_data_descriptor_set(logical_device: &ash::Device, mesh_data_descriptor_set_layout: vk::DescriptorSetLayout, descriptor_pool: vk::DescriptorPool) -> vk::DescriptorSet {
		let descriptor_set_layouts = [mesh_data_descriptor_set_layout];

		let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
			.descriptor_pool(descriptor_pool)
			.set_layouts(&descriptor_set_layouts);
		
		unsafe { logical_device.allocate_descriptor_sets(&descriptor_set_allocate_info) }.unwrap()[0]
	}

	pub fn handle_resize(&mut self, logical_device: &ash::Device, extent: vk::Extent2D, render_pass: vk::RenderPass) {
		unsafe {
			logical_device.destroy_pipeline(self.basic_pipeline, None);
			logical_device.destroy_pipeline(self.basic_instanced_pipeline, None);
			logical_device.destroy_pipeline(self.lambert_pipeline, None);
		}

		let (basic_pipeline, basic_instanced_pipeline, lambert_pipeline) = Self::create_pipelines(logical_device, extent, self.pipeline_layout, self.instanced_pipeline_layout, render_pass);

		self.basic_pipeline = basic_pipeline;
		self.basic_instanced_pipeline = basic_instanced_pipeline;
		self.lambert_pipeline = lambert_pipeline;
	}

	pub fn drop(&mut self, logical_device: &ash::Device) {
		unsafe {
			logical_device.destroy_descriptor_set_layout(self.frame_data_descriptor_set_layout, None);
			logical_device.destroy_descriptor_set_layout(self.mesh_data_descriptor_set_layout, None);

			logical_device.destroy_pipeline_layout(self.pipeline_layout, None);
			logical_device.destroy_pipeline_layout(self.instanced_pipeline_layout, None);
			
			logical_device.destroy_pipeline(self.basic_pipeline, None);
			logical_device.destroy_pipeline(self.basic_instanced_pipeline, None);
			logical_device.destroy_pipeline(self.lambert_pipeline, None);
		}

		self.static_mesh_buffer.drop(logical_device);
	}
}

impl Renderer {
	pub fn submit_static_meshes(&mut self, geometries: &Pool<Geometry3D>, meshes: &mut [StaticMesh], instanced_meshes: &mut [StaticInstancedMesh]) {
		let context = &self.context;
		let logical_device = &context.logical_device;
		let static_mesh_data = &mut self.mesh_manager.static_mesh_data;
		let static_instanced_mesh_data = &mut self.mesh_manager.static_instanced_mesh_data;
		let static_mesh_buffer = &mut self.mesh_manager.static_mesh_buffer;

		static_mesh_data.clear();
		static_instanced_mesh_data.clear();

		// Calculate total buffer size and offsets
		let mut offset = 0;
		let uniform_alignment = self.context.physical_device.min_uniform_buffer_offset_alignment as usize;

		for mesh in meshes.iter() {
			let geometry = geometries.get(&mesh.geometry_handle).unwrap();
			let indices = &geometry.indices[..];
			let attributes = &geometry.attributes[..];

			let index_size = size_of_val(indices);
			let index_padding_size = (size_of::<f32>() - (offset + index_size) % size_of::<f32>()) % size_of::<f32>();
			let attribute_offset = offset + index_size + index_padding_size;
			let attribute_size = size_of_val(attributes);
			let attribute_padding_size = (uniform_alignment - (attribute_offset + attribute_size) % uniform_alignment) % uniform_alignment;
			let uniform_offset = attribute_offset + attribute_size + attribute_padding_size;
			let uniform_size = 16 * size_of::<f32>();

			static_mesh_data.push(StaticMeshData {
				index_offset: offset,
				attribute_offset,
				uniform_offset,
				index_count: indices.len(),
				material: mesh.material
			});
			
			offset = uniform_offset + uniform_size;
		}

		for mesh in instanced_meshes.iter() {
			let geometry = geometries.get(&mesh.geometry_handle).unwrap();
			let indices = &geometry.indices[..];
			let attributes = &geometry.attributes[..];

			let index_size = size_of_val(indices);
			let index_padding_size = (size_of::<f32>() - (offset + index_size) % size_of::<f32>()) % size_of::<f32>();
			let attribute_offset = offset + index_size + index_padding_size;
			let attribute_size = size_of_val(attributes);
			let matrix_offset = attribute_offset + attribute_size;
			let matrix_size = 16 * size_of::<f32>() * mesh.transforms.len();

			static_instanced_mesh_data.push(StaticInstancedMeshData {
				index_offset: offset,
				attribute_offset,
				matrix_offset,
				index_count: indices.len(),
				instance_count: mesh.transforms.len(),
				material: mesh.material
			});

			offset = matrix_offset + matrix_size;
		}

		// Create a host visible staging buffer
		let buffer_size = offset as u64;
		let mut staging_buffer = Buffer::new(&self.context, buffer_size, vk::BufferUsageFlags::TRANSFER_SRC, vk::MemoryPropertyFlags::HOST_VISIBLE);

		// Copy mesh data into staging buffer
		let buffer_ptr = unsafe { logical_device.map_memory(staging_buffer.memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty()).unwrap() };

		for (index, mesh) in meshes.iter_mut().enumerate() {
			mesh.transform.update_matrix();

			let mesh_data = &static_mesh_data[index];

			let geometry = geometries.get(&mesh.geometry_handle).unwrap();
			let indices = &geometry.indices[..];
			let attributes = &geometry.attributes[..];

			let matrix = &mesh.transform.matrix.elements;

			unsafe {
				let index_dst_ptr = buffer_ptr.add(mesh_data.index_offset) as *mut u16;
				ptr::copy_nonoverlapping(indices.as_ptr(), index_dst_ptr, indices.len());

				let attribute_dst_ptr = buffer_ptr.add(mesh_data.attribute_offset) as *mut f32;
				ptr::copy_nonoverlapping(attributes.as_ptr(), attribute_dst_ptr, attributes.len());

				let uniform_dst_ptr = buffer_ptr.add(mesh_data.uniform_offset) as *mut [f32; 4];
				ptr::copy_nonoverlapping(matrix.as_ptr(), uniform_dst_ptr, matrix.len());
			}
		}

		for (mesh_index, mesh) in instanced_meshes.iter_mut().enumerate() {
			let mesh_data = &static_instanced_mesh_data[mesh_index];

			let geometry = geometries.get(&mesh.geometry_handle).unwrap();
			let indices = &geometry.indices[..];
			let attributes = &geometry.attributes[..];

			unsafe {
				let index_dst_ptr = buffer_ptr.add(mesh_data.index_offset) as *mut u16;
				ptr::copy_nonoverlapping(indices.as_ptr(), index_dst_ptr, indices.len());

				let attribute_dst_ptr = buffer_ptr.add(mesh_data.attribute_offset) as *mut f32;
				ptr::copy_nonoverlapping(attributes.as_ptr(), attribute_dst_ptr, attributes.len());
			}

			for (transform_index, transform) in mesh.transforms.iter_mut().enumerate() {
				transform.update_matrix();
				transform.matrix.transpose();

				let matrix = &transform.matrix.elements;

				unsafe {
					let matrix_dst_ptr = buffer_ptr.add(mesh_data.matrix_offset + 16 * size_of::<f32>() * transform_index) as *mut [f32; 4];
					ptr::copy_nonoverlapping(matrix.as_ptr(), matrix_dst_ptr, matrix.len());
				}
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

		// Allocate larger device local memory buffer if necessary and update descriptor sets to reference new buffer
		if buffer_size > static_mesh_buffer.capacity {
			unsafe { logical_device.queue_wait_idle(self.context.graphics_queue).unwrap() };

			static_mesh_buffer.reallocate(&self.context, buffer_size);

			let model_matrix_buffer_info = vk::DescriptorBufferInfo::builder()
				.buffer(static_mesh_buffer.handle)
				.offset(0)
				.range(16 * size_of::<f32>() as u64);
			let model_matrix_buffer_infos = [model_matrix_buffer_info.build()];

			let model_matrix_write_descriptor_set = vk::WriteDescriptorSet::builder()
				.dst_set(self.mesh_manager.static_mesh_data_descriptor_set)
				.dst_binding(0)
				.dst_array_element(0)
				.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
				.buffer_info(&model_matrix_buffer_infos);
			
			unsafe { logical_device.update_descriptor_sets(&[model_matrix_write_descriptor_set.build()], &[]) };
		}

		// Record a command buffer to copy the data from the staging buffer to the device local buffer
		let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
			.level(vk::CommandBufferLevel::PRIMARY)
			.command_pool(self.command_pool)
			.command_buffer_count(1);

		let command_buffer = unsafe { logical_device.allocate_command_buffers(&command_buffer_allocate_info).unwrap()[0] };

		let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
			.flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
		
		let region = vk::BufferCopy::builder()
			.size(buffer_size);
		
		unsafe {
			logical_device.begin_command_buffer(command_buffer, &command_buffer_begin_info).unwrap();
			logical_device.cmd_copy_buffer(command_buffer, staging_buffer.handle, static_mesh_buffer.handle, &[region.build()]);
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
			logical_device.free_command_buffers(self.command_pool, &command_buffers);
		}

		staging_buffer.drop(logical_device);
	}
}