#version 460

layout (location = 0) in vec4 in_position;
layout (location = 1) in vec4 in_uv;

layout (location = 0) out vec4 out_uv;
layout (location = 1) out vec3 out_color;

layout (set = 0, binding = 0) uniform Camera {
    mat4 view_proj;
} camera;

layout (set = 0, binding = 1) uniform Model {
    mat4 transform;
} model;


const vec3 colors[4] = vec3[4](
    vec3(1,0,0),
    vec3(0,1,0),
    vec3(0,0,1),
    vec3(1,1,0)
);

void main() 
{
    gl_Position = camera.view_proj * model.transform * in_position;
    out_uv = in_uv;
    out_color = colors[gl_VertexIndex % 4];
}

