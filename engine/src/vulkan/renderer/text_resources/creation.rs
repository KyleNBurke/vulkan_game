use std::ffi::CString;
use ash::{vk, version::DeviceV1_0};
use super::{super::create_shader_module, MAX_FONTS};

pub fn create_sampler_descriptor_set_layout(logical_device: &ash::Device) -> vk::DescriptorSetLayout {
	let layout_binding = vk::DescriptorSetLayoutBinding::builder()
		.binding(0)
		.descriptor_type(vk::DescriptorType::SAMPLER)
		.descriptor_count(1)
		.stage_flags(vk::ShaderStageFlags::FRAGMENT);
	let layout_bindings = [layout_binding.build()];

	let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
		.bindings(&layout_bindings);

	unsafe { logical_device.create_descriptor_set_layout(&create_info, None) }.unwrap()
}

pub fn create_atlases_descriptor_set_layout(logical_device: &ash::Device) -> vk::DescriptorSetLayout {
	let layout_binding = vk::DescriptorSetLayoutBinding::builder()
		.binding(0)
		.descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
		.descriptor_count(MAX_FONTS as u32)
		.stage_flags(vk::ShaderStageFlags::FRAGMENT);
	let layout_bindings = [layout_binding.build()];

	let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
		.bindings(&layout_bindings);

	unsafe { logical_device.create_descriptor_set_layout(&create_info, None) }.unwrap()
}

pub fn create_pipeline_layout(
	logical_device: &ash::Device,
	instance_data_descriptor_set_layout: vk::DescriptorSetLayout,
	sampler_descriptor_set_layout: vk::DescriptorSetLayout,
	atlases_descriptor_set_layout: vk::DescriptorSetLayout)
	-> vk::PipelineLayout
{
	let descriptor_set_layouts = [
		instance_data_descriptor_set_layout,
		sampler_descriptor_set_layout,
		atlases_descriptor_set_layout
	];

	let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
		.set_layouts(&descriptor_set_layouts);

	unsafe { logical_device.create_pipeline_layout(&pipeline_layout_create_info, None) }.unwrap()
}


pub fn create_pipeline(logical_device: &ash::Device, extent: vk::Extent2D, pipeline_layout: vk::PipelineLayout, render_pass: vk::RenderPass) -> vk::Pipeline {
	// Create entry point string
	let entry_point = CString::new("main").unwrap();
	let entry_point_cstr = entry_point.as_c_str();

	// Create shader stage create infos
	let vert_module = create_shader_module(logical_device, "text.vert.spv");
	let vert_stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
		.stage(vk::ShaderStageFlags::VERTEX)
		.module(vert_module)
		.name(entry_point_cstr);
	
	let frag_module = create_shader_module(logical_device, "text.frag.spv");
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

pub fn create_descriptor_sets(
	logical_device: &ash::Device,
	sampler_descriptor_set_layout: vk::DescriptorSetLayout,
	atlases_descriptor_set_layout: vk::DescriptorSetLayout,
	descriptor_pool: vk::DescriptorPool)
	-> Vec<vk::DescriptorSet>
{
	let descriptor_set_layouts = [sampler_descriptor_set_layout, atlases_descriptor_set_layout];
	let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
		.descriptor_pool(descriptor_pool)
		.set_layouts(&descriptor_set_layouts);
	
	unsafe { logical_device.allocate_descriptor_sets(&descriptor_set_allocate_info) }.unwrap()
}

pub fn create_sampler(logical_device: &ash::Device) -> vk::Sampler {
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
	
	unsafe { logical_device.create_sampler(&sampler_create_info, None) }.unwrap()
}

pub fn update_sampler(logical_device: &ash::Device, sampler: vk::Sampler, descriptor_set: vk::DescriptorSet) {
	let descriptor_image_info = vk::DescriptorImageInfo::builder()
		.sampler(sampler);
	let descriptor_image_infos = [descriptor_image_info.build()];
	
	let write_descriptor_set = vk::WriteDescriptorSet::builder()
		.dst_set(descriptor_set)
		.dst_binding(0)
		.dst_array_element(0)
		.descriptor_type(vk::DescriptorType::SAMPLER)
		.image_info(&descriptor_image_infos)
		.build();

	unsafe { logical_device.update_descriptor_sets(&[write_descriptor_set], &[]) };
}