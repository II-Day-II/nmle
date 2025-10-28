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
    const float logscale = 5.0;

    vec2 offset = (inverse(camera.view_proj) * vec4(uv * 2 - 1, 0.0, 1.0)).xy; // TODO: optimize
    vec2 derivative = fwidth(offset);
    
    float grid_level = log(pan_and_zoom.zoom) / log(logscale) + 3.0 + 1.20; // [0, 4] for zoom [0.001, 10]
    uint grid_floor = uint(floor(grid_level));
    uint grid_ceil = uint(ceil(grid_level));
    float t = grid_level - float(grid_floor);
    //t = t*t;//pow(t,t); // slightly more aggressive fadein
    
    float thickness0 = (1.0 - t) * 0.010;
    float thickness1 = (t) * 0.010;
    float scale0 = pow(logscale, 3.0 - grid_floor);
    float scale1 = scale0 / logscale;
    vec3 col0 = grid(offset, pan_and_zoom.zoom, thickness0, scale0);
    //if (col0 == vec3(1.0)){
    //	col0 = vec3(1,0,0);
    //}
    vec3 col1 = grid(offset, pan_and_zoom.zoom, thickness1, scale1);
    //if (col1 == vec3(1.0)) {
    //	col1 = vec3(0,1,0);
    //}
    vec3 col = mix(col0, col1, t);

    //vec3 col0 = grid(offset, pan_and_zoom.zoom, 0.005, 100); // grid valid for zoom ~0.001 (unused..?) 0
    //vec3 col1 = grid(offset, pan_and_zoom.zoom, 0.005, 10); // grid valid for zoom ~0.01               1
    //vec3 col2 = grid(offset, pan_and_zoom.zoom, 0.005, 1); // grid valid for zoom ~0.1                 2
    //vec3 col3 = grid(offset, pan_and_zoom.zoom, 0.005, 0.1); // grid valid for zoom ~1                 3
    //vec3 col4 = grid(offset, pan_and_zoom.zoom, 0.005, 0.01); // grid valid for zoom ~10               4
    //vec3 grids[5] = {col0, col1, col2, col3, col4};

    //vec3 col = mix(grids[grid_floor], grids[grid_ceil], t);


    out_FragColor = vec4(col, 1.0);
}
