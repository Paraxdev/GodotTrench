shader_type spatial;
// Two texture blend for GodotTrench displacements and vertex paint.
// COLOR.a is the blend weight painted in the editor (0 = texture_a, 1 = texture_b), COLOR.rgb tints the result.

uniform sampler2D texture_a : source_color, filter_linear_mipmap, repeat_enable;
uniform sampler2D texture_b : source_color, filter_linear_mipmap, repeat_enable;
uniform float blend_sharpness : hint_range(1.0, 16.0) = 1.0;
uniform bool use_vertex_tint = true;
uniform float roughness : hint_range(0.0, 1.0) = 0.9;

void fragment() {
	vec4 a = texture(texture_a, UV);
	vec4 b = texture(texture_b, UV);
	float w = clamp((COLOR.a - 0.5) * blend_sharpness + 0.5, 0.0, 1.0);
	vec3 albedo = mix(a.rgb, b.rgb, w);
	if (use_vertex_tint) {
		albedo *= COLOR.rgb;
	}
	ALBEDO = albedo;
	ROUGHNESS = roughness;
}
