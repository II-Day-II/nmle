#version 460

layout (location = 0) in vec4 in_uv;
layout (location = 1) in vec3 inColor;

layout (location = 0) out vec4 out_FragColor;

void main()
{
    out_FragColor = vec4(inColor, 1);
}
