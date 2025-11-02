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


vec3 grid(vec2 position, float scale, float thickness, float level) {
    const float base_line_width = thickness;
    position += base_line_width / scale / 2.0;
    vec2 grid_pos = mod(position, level);
    vec2 derivative = fwidth(grid_pos);
    vec2 grid_point = smoothstep(grid_pos - derivative, grid_pos, vec2(base_line_width / scale)); // ensures constant line size
    float grid = min(1.0, grid_point.x + grid_point.y);
    vec3 col = vec3(grid);
    return col;
}


void main() {
    const float LOGSCALE = 5.0;
    const float MAX_LINE_THICKNESS = 0.010;

    vec2 offset = (inverse(camera.view_proj) * vec4(uv * 2 - 1, 0.0, 1.0)).xy; // TODO: optimize
    
    float grid_level = log(pan_and_zoom.zoom) / log(LOGSCALE) + 3.0 + 1.20; // [0, 4] for zoom [0.001, 10] + 1.2 for styling
    uint grid_floor = uint(floor(grid_level));
    uint grid_ceil = uint(ceil(grid_level));
    float t = grid_level - float(grid_floor);
    
    float thickness0 = (1.0 - t) * MAX_LINE_THICKNESS;
    float thickness1 = (t) * MAX_LINE_THICKNESS;
    float scale0 = pow(LOGSCALE, 3.0 - grid_floor);
    float scale1 = scale0 / LOGSCALE;
    vec3 col0 = grid(offset, pan_and_zoom.zoom, thickness0, scale0);
    vec3 col1 = grid(offset, pan_and_zoom.zoom, thickness1, scale1);
    vec3 col = mix(col0, col1, t);
    float thickness = max(thickness0, thickness1);
    if (abs(offset.x) < thickness / pan_and_zoom.zoom) {
        col.rb = vec2(0.0);
    }
    if (abs(offset.y) < thickness / pan_and_zoom.zoom) {
        col.gb = vec2(0.0);
    }
    out_FragColor = vec4(col, 1.0);
}
