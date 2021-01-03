#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(set = 0, binding = 0) uniform sampler samp;
layout(set = 0, binding = 1) uniform texture2D atlases[10];

layout(set = 1, binding = 1, std140) uniform TextData {
	uint atlasIndex;
};

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec2 fragTexPosition;

layout(location = 0) out vec4 outColor;

void main() {
	float alpha = texture(sampler2D(atlases[atlasIndex], samp), fragTexPosition).r;
	outColor = vec4(1, 1, 1, alpha);
}