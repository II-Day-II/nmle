#version 460

layout(location = 0) in vec2 uv;

// layout(location = 0) out vec4 outFragColor;
layout(location = 0) out vec2 outFragColor;

layout(set = 0, binding = 0) uniform texture2D surfaceTexture;
layout(set = 0, binding = 1) uniform sampler surfaceSampler;

void main() {
    float alpha = texture(sampler2D(surfaceTexture, surfaceSampler), uv).a;
    // outFragColor = vec4(uv * alpha, 0.0, 1.0);
    outFragColor = vec2(uv * alpha);
}