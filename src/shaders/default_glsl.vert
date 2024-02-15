#version 460

layout (location = 0) out vec3 outColor;

const vec3 colors[3] = vec3[3](
    vec3(1,0,0),
    vec3(0,1,0),
    vec3(0,0,1)
);

void main() 
{
    float x = (1 - gl_VertexIndex) * 0.5;
    float y = ((gl_VertexIndex & 1) * 2 - 1) * 0.5;
    gl_Position = vec4(x, y, 0, 1);
    outColor = colors[gl_VertexIndex];
}

