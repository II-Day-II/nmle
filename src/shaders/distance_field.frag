#version 460

layout(location = 0) in vec2 uv;

layout(location = 0) out float outFragColor;

layout(set = 0, binding = 0) uniform texture2D jfaTexture;
layout(set = 0, binding = 1) uniform sampler jfaSampler;

void main() {
    vec2 nearestSeed = texture(sampler2D(jfaTexture, jfaSampler), uv).xy;
    float dist = clamp(distance(uv, nearestSeed), 0.0, 1.0);
    if (dist < 0.00030) // This is such bs, but it somehow sort of works
        dist = 0.0;

    outFragColor = dist; //vec4(vec3(dist, nearestSeed), 1.0);
}