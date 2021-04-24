use std::ffi::CString;
use ash::{vk, version::DeviceV1_0};
use super::super::create_shader_module;

pub fn create_pipeline_layout(
	logical_device: &ash::Device,
	frame_data_descriptor_set_layout: vk::DescriptorSetLayout,
	instance_data_descriptor_set_layout: vk::DescriptorSetLayout)
	-> vk::PipelineLayout
{
	let descriptor_set_layouts = [frame_data_descriptor_set_layout, instance_data_descriptor_set_layout];

	let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
		.set_layouts(&descriptor_set_layouts);

	unsafe { logical_device.create_pipeline_layout(&pipeline_layout_create_info, None) }.unwrap()
}

pub fn create_pipelines(logical_device: &ash::Device, extent: vk::Extent2D, pipeline_layout: vk::PipelineLayout, render_pass: vk::RenderPass) -> Vec<vk::Pipeline> {
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
	
	let input_attribute_description_normal = vk::VertexInputAttributeDescription::builder()	
		.binding(0)
		.location(1)
		.format(vk::Format::R32G32B32_SFLOAT)
		.offset(12)
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
	let basic_vert_module = create_shader_module(logical_device, "basic.vert.spv");
	let basic_vert_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
		.stage(vk::ShaderStageFlags::VERTEX)
		.module(basic_vert_module)
		.name(entry_point_cstr);
	
	let basic_frag_module = create_shader_module(logical_device, "basic.frag.spv");
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
	
	// Normal
	let normal_vert_module = create_shader_module(logical_device, "normal.vert.spv");
	let normal_vert_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
		.stage(vk::ShaderStageFlags::VERTEX)
		.module(normal_vert_module)
		.name(entry_point_cstr);

	let normal_frag_module =  create_shader_module(logical_device, "normal.frag.spv");
	let normal_frag_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
		.stage(vk::ShaderStageFlags::FRAGMENT)
		.module(normal_frag_module)
		.name(entry_point_cstr);
	
	let normal_stage_create_infos = [normal_vert_stage_create_info.build(), normal_frag_stage_create_info.build()];
	let normal_input_attribute_descriptions = [input_attribute_description_position, input_attribute_description_normal];

	let normal_vert_input_state_create_info = vk::PipelineVertexInputStateCreateInfo::builder()
		.vertex_binding_descriptions(&input_binding_descriptions)
		.vertex_attribute_descriptions(&normal_input_attribute_descriptions);
	
	let normal_pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
		.stages(&normal_stage_create_infos)
		.vertex_input_state(&normal_vert_input_state_create_info)
		.input_assembly_state(&input_assembly_state_create_info)
		.viewport_state(&viewport_state_create_info)
		.rasterization_state(&rasterization_state_create_info)
		.multisample_state(&multisample_state_create_info)
		.depth_stencil_state(&depth_stencil_state_create_info)
		.color_blend_state(&color_blend_state_create_info)
		.layout(pipeline_layout)
		.render_pass(render_pass)
		.subpass(0);
	
	// Lambert
	let lambert_vert_module = create_shader_module(logical_device, "lambert.vert.spv");
	let lambert_vert_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
		.stage(vk::ShaderStageFlags::VERTEX)
		.module(lambert_vert_module)
		.name(entry_point_cstr);

	let lambert_frag_module =  create_shader_module(logical_device, "lambert.frag.spv");
	let lambert_frag_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
		.stage(vk::ShaderStageFlags::FRAGMENT)
		.module(lambert_frag_module)
		.name(entry_point_cstr);

	let lambert_stage_create_infos = [lambert_vert_stage_create_info.build(), lambert_frag_stage_create_info.build()];
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
		normal_pipeline_create_info.build(),
		lambert_pipeline_create_info.build()];
	
	let pipelines = unsafe { logical_device.create_graphics_pipelines(vk::PipelineCache::null(), &pipeline_create_infos, None) }.unwrap();

	// Destroy shader modules
	unsafe {
		logical_device.destroy_shader_module(basic_vert_module, None);
		logical_device.destroy_shader_module(basic_frag_module, None);

		logical_device.destroy_shader_module(normal_vert_module, None);
		logical_device.destroy_shader_module(normal_frag_module, None);

		logical_device.destroy_shader_module(lambert_vert_module, None);
		logical_device.destroy_shader_module(lambert_frag_module, None);
	}

	pipelines
}

pub fn create_static_descriptor_sets(logical_device: &ash::Device, descriptor_pool: vk::DescriptorPool, instance_data_descriptor_set_layout: vk::DescriptorSetLayout) -> Vec<vk::DescriptorSet> {
	let descriptor_set_layouts = [instance_data_descriptor_set_layout, instance_data_descriptor_set_layout, instance_data_descriptor_set_layout];
	let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
		.descriptor_pool(descriptor_pool)
		.set_layouts(&descriptor_set_layouts);
	
	unsafe { logical_device.allocate_descriptor_sets(&descriptor_set_allocate_info) }.unwrap()
}