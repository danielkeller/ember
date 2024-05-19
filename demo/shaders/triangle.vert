#version 450

layout(location = 0) in vec4 i_Position;
layout(location = 1) in vec2 i_Uv;

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
} ubo;

out gl_PerVertex
{
    vec4 gl_Position;
};

layout(location = 0) out vec2 v_Uv;

void main() {
    gl_Position = ubo.proj * ubo.view * ubo.model * i_Position;
    v_Uv = i_Uv;
}
