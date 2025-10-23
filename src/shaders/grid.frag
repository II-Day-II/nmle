#version 460

layout (location = 0) in vec2 uv;

layout (location = 0) out vec4 out_FragColor;


layout (set = 0, binding = 0) uniform Camera {
    mat4 view_proj;
} camera;

layout (set = 0, binding = 2) uniform PanAndZoom {
    vec2 position;
    float zoom;
    float aspect;
} pan_and_zoom;


vec3 grid(vec2 position, float scale, float thickness) {
    const float base_line_width = thickness;
    position -= base_line_width / scale;
    vec2 grid_pos = fract(position);
    vec2 grid_point = step(grid_pos, vec2(base_line_width / scale)); // ensures constant line size, which I don't want...
    float grid = min(1.0, grid_point.x + grid_point.y); // float(grid_point.x != 0.0 || grid_point.y != 0.0)
    vec3 col = vec3(grid);
    if (abs(position.x) < base_line_width / scale) {
        col = vec3(0.0, 1.0, 0.0); // green at x=0
    }
    if (abs(position.y) < base_line_width / scale) {
        col = vec3(1.0, 0.0, 0.0); // red at y=0
    }
    return col;
}


void main() {
    vec2 offset = (inverse(camera.view_proj) * vec4(uv * 2 - 1, 0.0, 1.0)).xy; // TODO: optimize
    vec3 col0 = grid(offset, pan_and_zoom.zoom, 0.005);
    vec3 col1 = grid(offset, pan_and_zoom.zoom * 10, 0.01);
    vec3 col = mix(col0, col1, (log(pan_and_zoom.zoom * 10) / log(10)) / 2);
    out_FragColor = vec4(col, 1.0);
}