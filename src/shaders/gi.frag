#version 460

layout(location = 0) in vec2 uv;

layout(location = 0) out vec4 outFragColor;

layout(set = 0, binding = 0) uniform texture2D sceneTexture;
layout(set = 0, binding = 1) uniform texture2D distanceTexture;
layout(set = 0, binding = 2) uniform sampler texSampler;

layout(set = 1, binding = 0) uniform Group1 {
    int maxSteps;
    int rayCount;
} Uniforms;

const float PI = 3.14259265;
const float TAU = 2.0 * PI;
const float EPSILON = 0.00000001;
const bool ADD_NOISE = true;

float rand(vec2 co) { // random function from jason's implementation
    return fract(sin(dot(co.xy, vec2(12.9898, 78.233))) * 43758.5453);
}

vec4 raymarch() {
    vec4 light = texture(sampler2D(sceneTexture, texSampler), uv);
    
    const int rayCount = 32; // this is all temp anyway
    const int maxSteps = 64;

    const float oneOverRayCount = 1.0 / float(rayCount);
    const float angleStepSize = TAU * oneOverRayCount;
    float noise = ADD_NOISE ? rand(uv) : 0.0;
    vec4 radiance = vec4(0.0);
    if (light.a < EPSILON) { // not light source or occluder
        for (int i = 0; i < rayCount; ++i) { // shoot rays in rayCount directions, equally spaced
            float angle = angleStepSize * (float(i) + noise);
            vec2 rayDiriection = vec2(cos(angle), -sin(angle));

            vec2 sampleUV = uv;
            vec4 radDelta = vec4(0.0);
            bool hitSurface = false;
            for (int stp = 1; stp < maxSteps; ++stp) {
                // how far is nearest object?
                float dist = texture(sampler2D(distanceTexture, texSampler), sampleUV).r;
                sampleUV += rayDiriection * dist; // go that far in our direction
                if (sampleUV.x > 1.0 || sampleUV.x < 0.0 || sampleUV.y > 1.0 || sampleUV.y < 0.0) break;
                if (dist < EPSILON) {
                    vec4 sampleColor = texture(sampler2D(sceneTexture, texSampler), sampleUV);
                    radDelta += sampleColor;
                    hitSurface = true;
                    break;
                }
            }
            radiance += radDelta;
        }
    }
    else {
        radiance = light;
    }
    return vec4(max(light, radiance * oneOverRayCount).rgb, 1.0);
}

void main() {
    outFragColor = raymarch();
}