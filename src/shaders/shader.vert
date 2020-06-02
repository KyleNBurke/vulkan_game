#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(set = 0, binding = 0, row_major) uniform ProjectionViewMatrix {
	mat4 projectionMatrix;
	mat4 viewMatrix;
};

layout(set = 1, binding = 0, row_major) uniform ModelMatrix {
	mat4 modelMatrix;
};

layout(location = 0) in vec3 inPosition;
layout(location = 0) out vec3 fragColor;

void main() {
	gl_Position = projectionMatrix * viewMatrix * modelMatrix * vec4(inPosition, 1.0);
	fragColor = vec3(0.0, 1.0, 0.0);
}