#version 450
#extension GL_ARB_separate_shader_objects : enable

#define MAX_POINT_LIGHTS 5

struct PointLight {
	vec3 position;
	vec3 color;
};

layout(set = 0, binding = 0, std140, row_major) uniform FrameData {
	mat4 projectionMatrix;
	mat4 viewMatrix;
	vec3 ambientLight;
	uint pointLightCount;
	PointLight pointLights[MAX_POINT_LIGHTS];
};

layout(set = 1, binding = 0, std140, row_major) uniform MeshData {
	mat4 modelMatrix;
};

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;

layout(location = 0) out vec3 fragColor;

void main() {
	vec4 vertexPositionObjectSpaceVec4 = modelMatrix * vec4(inPosition, 1.0);
	vec3 vertexPositionObjectSpaceVec3 = vec3(vertexPositionObjectSpaceVec4);
	vec3 vertexNormalObjectSpace = mat3(transpose(inverse(modelMatrix))) * inNormal;
	
	gl_Position = projectionMatrix * viewMatrix * vertexPositionObjectSpaceVec4;

	fragColor = ambientLight;

	for (int i = 0; i < pointLightCount; i++) {
		vec3 lightDirection = normalize(pointLights[i].position - vertexPositionObjectSpaceVec3);
		float diffuse = max(dot(vertexNormalObjectSpace, lightDirection), 0.0f);
		fragColor += pointLights[i].color * diffuse;
	}
}