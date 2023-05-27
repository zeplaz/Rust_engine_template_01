#version 450

#define MAX_LIGHTS 16

// Texture, tint color, and global ambient light level uniforms
layout(set = 0, binding = 0) uniform texture2D Texture;
layout(set = 1, binding = 0) uniform sampler Sampler;
layout(set = 2, binding = 0) uniform Tint {
    vec4 color;
};
layout(set = 3, binding = 0) uniform GlobalAmbientLight {
    vec4 ambient_color;
    float ambient_intensity;
};

// Local lights uniform (assuming a maximum of 16 local lights)

layout(set = 4, binding = 0) uniform LocalLights {
    vec4 light_positions[MAX_LIGHTS];
    vec4 light_colors[MAX_LIGHTS];
    vec4 light_params[MAX_LIGHTS]; // intensity and range stored in x and y, respectively
} lights;

layout(location = 0) in vec2 v_Uv;
layout(location = 1) in vec3 v_Normal;

layout(location = 0) out vec4 o_Target;

// Function to calculate the lighting contribution from a single local light
vec3 calculate_light(in vec3 normal, in vec3 light_direction, in vec3 light_color, in float light_intensity) {
    float diffuse = max(dot(normal, light_direction), 0.0);
    return light_color * diffuse * light_intensity;
}

void main() {
    vec4 base_color = texture(sampler2D(Texture, Sampler), v_Uv);
    vec3 normal = normalize(v_Normal);

    // Apply global ambient light
    vec3 final_color = base_color.rgb * ambient_color.rgb * ambient_intensity;

    // Apply local lights
    for (int i = 0; i < MAX_LIGHTS; i++) {
        vec3 light_pos = lights.light_positions[i].xyz;
        vec3 light_color = lights.light_colors[i].rgb;
        float light_intensity = lights.light_params[i].x;
        float light_range = lights.light_params[i].y;

        if (light_intensity > 0.0) {
            vec3 light_direction = normalize(light_pos - v_Normal);
            float distance = length(light_pos - v_Normal);
            float attenuation = 1.0 / (1.0 + 0.09 * distance + 0.032 * distance * distance);

            vec3 light_contribution = calculate_light(normal, light_direction, light_color, light_intensity);
            final_color += light_contribution * attenuation;
        }
    }

    // Apply tint color
    final_color *= color.rgb;

    o_Target = vec4(final_color, base_color.a);
}
