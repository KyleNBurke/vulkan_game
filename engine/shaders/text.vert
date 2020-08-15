#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(set = 1, binding = 0, std140, row_major) uniform UIElementData {
	mat3 matrix;
};

layout(location = 0) in vec2 inPosition;
layout(location = 1) in vec2 inTexPos;

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec2 fragTexPos;

void main() {
	gl_Position = vec4(matrix * vec3(inPosition, 1.0), 1.0);
	fragColor = vec3(1.0, 0.0, 0.0);
	fragTexPos = inTexPos;
}