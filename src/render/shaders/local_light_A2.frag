// fragment.glsl
#version 450
#define MAX_LIGHTS 16
layout(location = 0) in vec2 v_Uv;
layout(location = 1) in vec3 v_Position;

layout(location = 0) out vec4 o_Target;

layout(set = 0, binding = 1) uniform texture2D Texture;
layout(set = 0, binding = 2) uniform sampler Sampler;

layout(set = 1, binding = 1) uniform LightParams {
    vec4 light_count;
    vec4 light_positions[MAX_LIGHTS];
    vec4 light_colors[MAX_LIGHTS];
    vec4 light_params[MAX_LIGHTS];
};

void main() {
    vec4 base_color = texture(sampler2D(Texture, Sampler), v_Uv);

    vec3 final_color = base_color.rgb * 0.1; // Base ambient color

    for (int i = 0; i < int(light_count.x); ++i) {
        vec3 light_pos = light_positions[i].xyz;
        vec3 light_color = light_colors[i].rgb;
        float light_intensity = light_params[i].x;
        float light_range = light_params[i].y;

        vec3 light_dir = normalize(light_pos - v_Position);
        float distance = length(light_pos - v_Position);
        float attenuation = 1.0 / (1.0 + 0.09 * distance + 0.032 * distance * distance);
        attenuation = min(attenuation, 1.0);

        vec3 diffuse = max(dot(vec3(0.0, 0.0, 1.0), light_dir), 0.0) * base_color.rgb * light_color * light_intensity * attenuation;
        vec3 specular = vec3(0.0); // You can add specular lighting here if needed.

        final_color += diffuse + specular;
    }

    o_Target = vec4(final_color, base_color.a);
}
