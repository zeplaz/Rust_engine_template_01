#version 450

layout(location = 0) in vec2 v_Uv;

layout(location = 0) out vec4 o_Target;

layout(set = 2, binding = 0) uniform ColorMaterial {
    vec4 Color;
    sampler2D Texture;
};

layout(set = 3, binding = 0) uniform Tint {
    vec4 TintColor;
};

layout(set = 4, binding = 0) uniform AmbientLight {
    vec3 AmbientColor;
    float AmbientIntensity;
};

void main() {
    vec4 texture_color = texture(Texture, v_Uv);
    vec3 ambient_light = AmbientColor * AmbientIntensity;
    vec4 tinted_color = texture_color * TintColor;

    o_Target = vec4(tinted_color.rgb * ambient_light, tinted_color.a);
}
