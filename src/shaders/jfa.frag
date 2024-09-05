#version 460

layout(location = 0) in vec2 uv;

layout(location = 0) out vec4 outFragColor;

layout(set = 0, binding = 0) uniform Uniforms {
    float offset;
};
layout(set = 1, binding = 0) uniform sampler2D inputTexture;

bool inBounds(vec2 uvpos) {
    return uvpos.x >= 0.0 && uvpos.x <= 1.0 && uvpos.y >= 0.0 && uvpos.y <= 1.0;
}

void main() {
    vec4 nearestSeed = vec4(-2.0);
    float nearestDist = 99999999.9;
    vec2 resolution = textureSize(inputTexture, 0);
    for (float y = -1.0; y <= 1.0; y += 1.0) {
        for (float x = -1.0; x <= 1.0; x += 1.0) {
            vec2 sampleUV = uv + vec2(x, y) * offset / resolution;
            if (inBounds(sampleUV)) {
                vec4 sampleValue = texture(inputTexture, sampleUV);
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
    outFragColor = nearestSeed;
}