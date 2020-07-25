#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec2 inPosition;
layout(location = 1) in vec2 inTexPos;

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec2 fragTexPos;

void main() {
	gl_Position = vec4(inPosition, 0.5, 1.0);
	fragColor = vec3(1.0, 0.0, 0.0);
	fragTexPos = inTexPos;
}