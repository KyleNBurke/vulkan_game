#version 450
#extension GL_ARB_separate_shader_objects : enable

struct InstanceData {
	mat3 matrix;
	uint atlasIndex;
};

layout(set = 0, binding = 0, std140, row_major) buffer InstanceDataBlock {
	InstanceData instanceData[];
};

layout(location = 0) in vec2 inPosition;
layout(location = 1) in vec2 inTexPosition;

layout(location = 0) out vec2 fragTexPosition;
layout(location = 1) out flat uint outAtlasIndex;

void main() {
	InstanceData currentInstanceData = instanceData[gl_InstanceIndex];

	vec3 normalized_position = currentInstanceData.matrix * vec3(inPosition, 1.0);
	gl_Position = vec4(normalized_position.xy, 0.0, 1.0);

	outAtlasIndex = currentInstanceData.atlasIndex;
	fragTexPosition = inTexPosition;
}