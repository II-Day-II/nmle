#version 460

layout (location = 0) in vec2 uv;

layout (location = 0) out vec4 out_FragColor;

layout (set = 0, binding = 2) uniform PanAndZoom {
    vec2 position;
    float zoom;
    float aspect;
} pan_and_zoom;

void main() {
    vec2 offset = ((uv * 2 - 1) - pan_and_zoom.position) / pan_and_zoom.zoom;
    offset.x *= pan_and_zoom.aspect;
    offset = fract(offset);
    offset -= 0.5;
    float dist = smoothstep(0.05, 0.1, length(offset));
    vec3 col = vec3(1.0 - dist);
    out_FragColor = vec4(col, 1.0);
}