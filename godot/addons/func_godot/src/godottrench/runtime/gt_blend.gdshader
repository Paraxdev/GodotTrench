shader_type spatial;
// Two texture blend for GodotTrench displacements and vertex paint.
// COLOR.a is the blend weight painted in the editor (0 = texture_a, 1 = texture_b), COLOR.rgb tints the result.

uniform sampler2D texture_a : source_color, filter_linear_mipmap, repeat_enable;
uniform sampler2D texture_b : source_color, filter_linear_mipmap, repeat_enable;
uniform float blend_sharpness : hint_range(1.0, 16.0) = 1.0;
uniform bool use_vertex_tint = true;
uniform float roughness : hint_range(0.0, 1.0) = 0.9;
// Extra repeats of each texture across the face, so a painted path can be finer than the face's own UVs.
uniform vec2 uv_scale_a = vec2(1.0);
uniform vec2 uv_scale_b = vec2(1.0);
// Breaks up the repeat of each texture, 0 leaves it tiling as authored. See gt_terrain.gdshader.
uniform float detile_a : hint_range(0.0, 1.0) = 0.0;
uniform float detile_b : hint_range(0.0, 1.0) = 0.0;
// Crispness of the de-tiled blend: 0 mixes the cells evenly and looks softest, 1 mixes only in a narrow band.
uniform float detile_sharpen_a : hint_range(0.0, 1.0) = 0.5;
uniform float detile_sharpen_b : hint_range(0.0, 1.0) = 0.5;
// Each side's emission like BaseMaterial3D: (color + texture) or (color * texture) with the multiply operator, times
// energy. The textures default to black, as in BaseMaterial3D.
uniform vec3 emission_a : source_color = vec3(0.0);
uniform vec3 emission_b : source_color = vec3(0.0);
uniform float emission_energy_a = 0.0;
uniform float emission_energy_b = 0.0;
uniform bool emission_multiply_a = false;
uniform bool emission_multiply_b = false;
uniform sampler2D emission_texture_a : source_color, hint_default_black, filter_linear_mipmap, repeat_enable;
uniform sampler2D emission_texture_b : source_color, hint_default_black, filter_linear_mipmap, repeat_enable;

vec2 hash2(vec2 p) {
	vec3 q = fract(vec3(p.xyx) * vec3(0.1031, 0.1030, 0.0973));
	q += dot(q, q.yzx + 33.33);
	return fract((q.xx + q.yz) * q.zy);
}

// One repeat of the texture, turned and shifted by an amount fixed per cell.
vec4 cell_sample(sampler2D t, vec2 uv, vec2 cell, float strength) {
	vec2 h = hash2(cell);
	float angle = (h.x - 0.5) * 6.2831853 * strength;
	float ca = cos(angle);
	float sa = sin(angle);
	vec2 pivot = cell + 0.5;
	vec2 d = uv - pivot;
	vec2 turned = vec2(d.x * ca - d.y * sa, d.x * sa + d.y * ca);
	return texture(t, pivot + turned + (h - 0.5) * strength);
}

// Blends the four nearest cells, so the seams the repeat would line up are blurred away at the corners.
// [param sharpen] raises the weights to a power, narrowing the band where the cells mix so the rest stays crisp.
vec4 detiled(sampler2D t, vec2 uv, float strength, float sharpen) {
	if (strength <= 0.0) {
		return texture(t, uv);
	}
	vec2 g = uv - 0.5;
	vec2 base = floor(g);
	vec2 f = g - base;
	vec2 w = f * f * (3.0 - 2.0 * f);
	float power = mix(1.0, 8.0, clamp(sharpen, 0.0, 1.0));
	vec4 sum = vec4(0.0);
	float total = 0.0;
	for (int j = 0; j < 2; j++) {
		for (int i = 0; i < 2; i++) {
			vec2 corner = base + vec2(float(i), float(j));
			float weight = pow(mix(1.0 - w.x, w.x, float(i)) * mix(1.0 - w.y, w.y, float(j)), power);
			sum += cell_sample(t, uv, corner, strength) * weight;
			total += weight;
		}
	}
	return sum / max(total, 0.00001);
}

void fragment() {
	vec4 a = detiled(texture_a, UV * uv_scale_a, detile_a, detile_sharpen_a);
	vec4 b = detiled(texture_b, UV * uv_scale_b, detile_b, detile_sharpen_b);
	float w = clamp((COLOR.a - 0.5) * blend_sharpness + 0.5, 0.0, 1.0);
	vec3 albedo = mix(a.rgb, b.rgb, w);
	if (use_vertex_tint) {
		albedo *= COLOR.rgb;
	}
	ALBEDO = albedo;
	ROUGHNESS = roughness;
	if (emission_energy_a > 0.0 || emission_energy_b > 0.0) {
		vec3 ta = texture(emission_texture_a, UV * uv_scale_a).rgb;
		vec3 tb = texture(emission_texture_b, UV * uv_scale_b).rgb;
		vec3 glow_a = (emission_multiply_a ? emission_a * ta : emission_a + ta) * emission_energy_a;
		vec3 glow_b = (emission_multiply_b ? emission_b * tb : emission_b + tb) * emission_energy_b;
		EMISSION = mix(glow_a, glow_b, w);
	}
}
