#version 460

layout(location = 0) in vec2 uv;

layout(location = 0) out vec4 outFragColor;

layout(set = 0, binding = 0) uniform sampler2D surfaceTexture;

void main() {
    float alpha = texture(surfaceTexture, uv).a;
    outFragColor = vec4(uv * alpha, 0.0, 1.0);
}