#version 460

layout(location = 0) in vec2 uv;

layout(location = 0) out vec4 outFragColor;

layout(set = 0, binding = 0) uniform Uniforms {
    float offset;
};
layout(set = 0, binding = 1) uniform texture2D inputTexture;
layout(set = 0, binding = 2) uniform sampler inputSampler;

bool inBounds(vec2 uvpos) {
    return uvpos.x >= 0.0 && uvpos.x <= 1.0 && uvpos.y >= 0.0 && uvpos.y <= 1.0;
}

void main() {
    vec4 nearestSeed = vec4(-2.0);
    float nearestDist = 99999999.9;
    vec2 resolution = textureSize(sampler2D(inputTexture, inputSampler), 0);
    for (float y = -1.0; y <= 1.0; y += 1.0) {
        for (float x = -1.0; x <= 1.0; x += 1.0) {
            vec2 sampleUV = uv + vec2(x, y) * offset / resolution;
            if (inBounds(sampleUV)) {
                vec4 sampleValue = texture(sampler2D(inputTexture, inputSampler), sampleUV);
                vec2 sampleSeed = sampleValue.xy;
                if (sampleSeed.x != 0.0 || sampleSeed.y != 0.0) {
                    vec2 diff = sampleSeed - uv;
                    float dist = dot(diff, diff);
                    if (dist < nearestDist) {
                        nearestDist = dist;
                        nearestSeed = sampleValue;
                    }
                }
            }
        }
    }
    // jfa output
    outFragColor = nearestSeed;
    // this was a mistake, but the output looks pretty cool sometimes
    // outFragColor = vec4(vec3(clamp(distance(uv, nearestSeed.xy), 0.0, 1.0)), 1.0);
}