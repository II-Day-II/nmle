#version 460

layout(location = 0) in vec2 uv;

layout(location = 0) out vec4 outFragColor;

layout(set = 0, binding = 0) uniform texture2D sceneTexture;
layout(set = 0, binding = 1) uniform texture2D distanceTexture;
layout(set = 0, binding = 2) uniform texture2D lastTexture;
layout(set = 0, binding = 3) uniform sampler texSampler;

layout(set = 1, binding = 0) uniform CascadeParams {
    float start_interval;
    float interval_size;
    float angle_offset;
    int cascade_count;
    int base_ray_count;
    int ray_count;
} Uniforms;

const float PI = 3.14259265;
const float TAU = 2.0 * PI;
const int maxSteps = 32;

float rand(vec2 co) { // random function from jason's implementation
    return fract(sin(dot(co.xy, vec2(12.9898, 78.233))) * 43758.5453);
}

vec4 raymarch() {
    vec2 resolution = textureSize(sampler2D(distanceTexture, texSampler), 0);
    vec2 coord = floor(uv * resolution); // floor?

    bool is_last_layer = Uniforms.ray_count == Uniforms.base_ray_count;
    vec2 effectiveUV = is_last_layer ? uv : floor(coord / 2.0) * 2.0 / resolution;
    float interval_start = is_last_layer ? 0.0 : Uniforms.interval_size;
    float interval_end = is_last_layer ? Uniforms.interval_size : sqrt(2.0);

    vec2 scale = min(resolution.x, resolution.y) / resolution;
    vec2 one_over_size = 1.0 / resolution;
    float min_step_size = min(one_over_size.x, one_over_size.y) * 0.5; // why 0.5?

    vec4 light = texture(sampler2D(sceneTexture, texSampler), uv);
    
    const float oneOverRayCount = 1.0 / float(Uniforms.ray_count);
    const float angleStepSize = TAU * oneOverRayCount;
    vec4 radiance = vec4(0.0);
    for (int i = 0; i < Uniforms.ray_count; ++i) { // shoot rays in rayCount directions, equally spaced
        float angle = angleStepSize * (float(i) + Uniforms.angle_offset); // add angle_offset (0.5) to avoid vertical angles..? why?
        vec2 rayDiriection = vec2(cos(angle), -sin(angle));

        // start in our decided starting location
        vec2 sampleUV = effectiveUV + rayDiriection * interval_start * scale;
        // track how far we've gone
        float traveled = interval_start;

        vec4 radDelta = vec4(0.0);
        bool hitSurface = false;
        for (int stp = 0; stp < maxSteps; ++stp) {
            // how far is nearest object?
            float dist = texture(sampler2D(distanceTexture, texSampler), sampleUV).r;
            // go that far in our direction
            sampleUV += rayDiriection * dist * scale; 
            if (sampleUV.x > 1.0 || sampleUV.x < 0.0 || sampleUV.y > 1.0 || sampleUV.y < 0.0) {
                break;
            }
            if (dist < min_step_size) {
                vec4 sampleColor = texture(sampler2D(sceneTexture, texSampler), sampleUV);
                if (sampleColor.a == 0.0) { // apparently this happens a lot...
                    radiance = vec4(1.0, 0.0, 0.0, 1.0);
                    sampleUV += min_step_size;
                    continue;
                    break;
                }
                radDelta += sampleColor;
                hitSurface = true;
                break;
            }
            traveled += dist;
            if (traveled >= interval_end) { 
                break; 
            }
        }
        // merge cascades on non-opaque areas
        if (is_last_layer && radDelta.a == 0.0) {
            vec4 upperSample = texture(sampler2D(lastTexture, texSampler), uv);
            radDelta += upperSample;
        }
        // accumulate total radiance
        radiance += radDelta;
    }
    // return vec4(is_last_layer);
    return vec4((radiance * oneOverRayCount).rgb, 1.0);
}

void main() {
    outFragColor = raymarch();
}