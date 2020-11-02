#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(set = 0, binding = 0) uniform sampler2D texSampler;

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec2 fragTexPosition;

layout(location = 0) out vec4 outColor;

void main() {
	float alpha = texture(texSampler, fragTexPosition).r;
	outColor = vec4(1, 1, 1, alpha);
}