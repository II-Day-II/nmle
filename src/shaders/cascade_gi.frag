#version 460

layout(location = 0) in vec2 uv;

layout(location = 0) out vec4 outFragColor;

layout(set = 0, binding = 0) uniform texture2D sceneTexture;
layout(set = 0, binding = 1) uniform texture2D distanceTexture;
layout(set = 0, binding = 2) uniform texture2D lastTexture;
layout(set = 0, binding = 3) uniform sampler nearest_sampler;
layout(set = 0, binding = 4) uniform sampler linear_sampler;

layout(set = 1, binding = 0) uniform CascadeParams {
    float angle_offset;
    int cascade_count;
    int cascade_index;
    int base_ray_count;
} Uniforms;

const float PI = 3.14259265;
const float TAU = 2.0 * PI;
const int maxSteps = 32;

float rand(vec2 co) { // random function from jason's implementation
    return fract(sin(dot(co.xy, vec2(12.9898, 78.233))) * 43758.5453);
}

vec4 raymarch() {
    const float base = float(Uniforms.base_ray_count);
    const float sqrt_base = sqrt(base); 
    const float cascade_index = float(Uniforms.cascade_index);
    const float ray_count = pow(base, cascade_index + 1.0);
    
    const vec2 resolution = textureSize(sampler2D(sceneTexture, linear_sampler), 0);
    const float oneOverBaseRayCount = 1.0 / float(Uniforms.base_ray_count);
    const float oneOverRayCount = 1.0 / float(ray_count);
    const float angleStepSize = TAU * oneOverRayCount;
    
    vec2 coord = floor(uv * resolution); 

    vec2 one_over_resolution = 1.0 / resolution;
    float shortest_side = min(resolution.x, resolution.y);
    vec2 aspect = shortest_side * one_over_resolution; // ensure non-square resolutions work
    float min_step_size = min(one_over_resolution.x, one_over_resolution.y) * 0.5; // half a texel in uv units
    
    bool is_cascade_0 = Uniforms.cascade_index == 0;
    // bool is_cascade_0 = Uniforms.ray_count == Uniforms.base_ray_count;
    // float interval_start = is_cascade_0 ? 0.0 : Uniforms.interval_split;

    // hack
    float modifier_hack = base < 16.0 ? 1.0 : 4.0;

    float interval_start = is_cascade_0 ? 0.0 : (modifier_hack * pow(base, cascade_index - 1.0) / shortest_side);
    float interval_length = modifier_hack * pow(base, cascade_index) / shortest_side;
    // float interval_end = is_cascade_0 ? Uniforms.interval_split : sqrt(2.0);

    float spacing = pow(sqrt_base, cascade_index); // spacing between probes
    vec2 num_probes = floor(resolution / spacing); // number of probes in x/y directions
    vec2 probe_rel_pos = mod(coord, num_probes); // which probe are we doing this pass?
    vec2 ray_pos = floor(coord / num_probes); // which group of rays are we doing this pass?
    float base_idx = base * (ray_pos.x + (spacing * ray_pos.y)); // linear index of current ray set
    vec2 probe_center = (probe_rel_pos + 0.5) * spacing; 
    vec2 normalized_probe_center = probe_center * one_over_resolution;


    vec4 radiance = vec4(0.0);
    for (int i = 0; i < Uniforms.base_ray_count; ++i) { // shoot rays in base_rayCount directions, equally spaced
        float idx = base_idx + float(i);
        float angle = angleStepSize * (idx + Uniforms.angle_offset); 
        vec2 rayDiriection = vec2(cos(angle), -sin(angle));

        // start in our decided starting location
        vec2 sampleUV = normalized_probe_center + rayDiriection * interval_start * aspect;
        // track how far we've gone
        float traveled = 0.0;

        vec4 radDelta = vec4(0.0);
        bool hitSurface = false;
        for (int stp = 1; stp < maxSteps; ++stp) {
            // how far is nearest object?
            float dist = texture(sampler2D(distanceTexture, nearest_sampler), sampleUV).r;
            // go that far in our direction
            sampleUV += rayDiriection * dist * aspect; 
            // check if oob
            if (sampleUV.x > 1.0 || sampleUV.x < 0.0 || sampleUV.y > 1.0 || sampleUV.y < 0.0) {
                break;
            }
            // if we're close enough to the shape
            if (dist <= min_step_size) {
                vec4 sampleColor = texture(sampler2D(sceneTexture, nearest_sampler), sampleUV);
                if (sampleColor.a == 0.0) { // apparently this happens a lot... we miss the original shape
                    // radiance = vec4(1.0, 0.0, 0.0, 1.0);
                    sampleUV += 1.0 * min_step_size * rayDiriection; // taking an extra step works sometimes...
                    continue;
                }
                radDelta += sampleColor;
                hitSurface = true;
                break;
            }
            traveled += dist;
            if (traveled >= interval_length) { 
                break; 
            }
        }
        // merge cascades on non-opaque areas
        if (Uniforms.cascade_index < Uniforms.cascade_count - 1 && radDelta.a == 0.0) {
            float upper_spacing = pow(sqrt_base, Uniforms.cascade_index + 1.0); // spacing between probes in upper cascade
            vec2 upper_size = floor(resolution / upper_spacing); // number of probes in x/y of upper cascade
            vec2 upper_pos = vec2(mod(idx, upper_spacing), floor(idx / upper_spacing)) * upper_size; // position of this probe in upper cascade

            // offset by center of probe in current layer relative to upper probe
            vec2 offset = (probe_rel_pos + 0.5) / sqrt_base;
            vec2 clamped = clamp(offset, vec2(0.5), upper_size - 0.5);
            vec2 upper_uv = (upper_pos + clamped) / resolution;

            vec4 upper_sample = texture(sampler2D(lastTexture, linear_sampler), upper_uv);
            radDelta += upper_sample;
        }
        // accumulate total radiance
        radiance += radDelta;
    }
    // return vec4(is_cascade_0);
    return vec4((radiance * oneOverBaseRayCount).rgb, 1.0);
}

void main() {
    outFragColor = raymarch();
}