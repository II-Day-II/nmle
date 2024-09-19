#version 460

layout (location = 0) in vec4 in_uv;
layout (location = 1) in vec3 inColor;

layout (location = 0) out vec4 out_FragColor;

void main()
{
    vec2 uv = in_uv.xy * 2.0 - 1.0;
    if (dot(uv, uv) < 0.0065 ) { //|| abs(uv.x) - abs(uv.y) > 0.9) {
        out_FragColor = vec4(inColor, 1);
    }
    else {
        out_FragColor = vec4(0.0);
    }
}
