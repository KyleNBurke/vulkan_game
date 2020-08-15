#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(set = 0, binding = 0, std140, row_major) uniform FrameData {
	mat4 projectionMatrix;
	mat4 viewMatrix;
};

layout(set = 1, binding = 0, std140, row_major) uniform MeshData {
	mat4 modelMatrix;
};

layout(location = 0) in vec3 inPosition;
layout(location = 0) out vec3 fragColor;

vec3 colors[3] = vec3[](
	vec3(1.0, 0.0, 0.0),
	vec3(0.0, 1.0, 0.0),
	vec3(0.0, 0.0, 1.0)
);

void main() {
	gl_Position = projectionMatrix * viewMatrix * modelMatrix * vec4(inPosition, 1.0);
	fragColor = colors[gl_VertexIndex % 3];
}