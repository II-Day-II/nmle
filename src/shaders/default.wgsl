
struct VertexOutput {
    @builtin(position)
    clip_pos: vec4f,
    @location(0)
    color: vec3f,
};



@vertex 
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    let colors: array<vec3f, 3> = array(vec3f(1,0,0), vec3f(0,1,0), vec3f(0,0,1));
    var out: VertexOutput;
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    out.clip_pos = vec4f(x, y, 0, 1);
    out.color = colors[in_vertex_index];
    return out;
}

struct FragmentOutput {
    @location(0)
    out_fragColor: vec4f,
};

@fragment
fn fs_main(i: VertexOutput) -> FragmentOutput {
    var o: FragmentOutput;
    o.out_fragColor = vec4f(i.color.x, i.color.y, i.color.z, 1);
    return o;
}
