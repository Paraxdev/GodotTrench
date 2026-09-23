shader_type spatial;
// GodotTrench terrain: four texture layers blended by vertex color weights (r, g, b, a), projected triplanar so
// cliffs are not stretched. The same projection as the editor preview.

uniform sampler2D layer0 : source_color, filter_linear_mipmap_anisotropic, repeat_enable;
uniform sampler2D layer1 : source_color, filter_linear_mipmap_anisotropic, repeat_enable;
uniform sampler2D layer2 : source_color, filter_linear_mipmap_anisotropic, repeat_enable;
uniform sampler2D layer3 : source_color, filter_linear_mipmap_anisotropic, repeat_enable;
// Meters covered by one repeat of each layer.
uniform vec4 tiles = vec4(8.0);
// Per layer strength of the de-tiling, 0 leaves the texture repeating as authored.
uniform vec4 detiles = vec4(0.0);
// Per layer crispness of the de-tiled blend: 0 mixes the cells evenly and looks softest, 1 keeps one cell over
// almost all of its area and only mixes in a narrow band, so the texture stays as sharp as it was authored.
uniform vec4 sharpens = vec4(0.5);
// Terrain origin in map space, so textures line up with the editor.
uniform vec3 map_offset = vec3(0.0);
uniform float roughness_value : hint_range(0.0, 1.0) = 0.9;
// Emission of each layer's material, like BaseMaterial3D: (color + texture) or (color * texture), times energy,
// weighted by the layer. The same as the editor preview.
uniform vec4 glow_color0 : source_color = vec4(0.0, 0.0, 0.0, 1.0);
uniform vec4 glow_color1 : source_color = vec4(0.0, 0.0, 0.0, 1.0);
uniform vec4 glow_color2 : source_color = vec4(0.0, 0.0, 0.0, 1.0);
uniform vec4 glow_color3 : source_color = vec4(0.0, 0.0, 0.0, 1.0);
uniform sampler2D glow_texture0 : source_color, hint_default_black, filter_linear_mipmap_anisotropic, repeat_enable;
uniform sampler2D glow_texture1 : source_color, hint_default_black, filter_linear_mipmap_anisotropic, repeat_enable;
uniform sampler2D glow_texture2 : source_color, hint_default_black, filter_linear_mipmap_anisotropic, repeat_enable;
uniform sampler2D glow_texture3 : source_color, hint_default_black, filter_linear_mipmap_anisotropic, repeat_enable;
uniform vec4 glow_energy = vec4(0.0);
// Per layer 1 when its emission texture is set, so layers without one skip the samples.
uniform vec4 glow_textured = vec4(0.0);
// Per layer 1 for BaseMaterial3D's multiply emission operator.
uniform vec4 glow_multiply = vec4(0.0);

varying vec3 map_position;
varying vec3 map_normal;

void vertex() {
	map_position = VERTEX + map_offset;
	map_normal = NORMAL;
}

vec2 hash2(vec2 p) {
	vec3 q = fract(vec3(p.xyx) * vec3(0.1031, 0.1030, 0.0973));
	q += dot(q, q.yzx + 33.33);
	return fract((q.xx + q.yz) * q.zy);
}

// One repeat of the texture, turned and shifted by an amount fixed per cell. A tileable texture stays seamless
// however it is turned, so this breaks up the grid the repeat would otherwise show.
vec3 cell_sample(sampler2D t, vec2 uv, vec2 cell, float strength) {
	vec2 h = hash2(cell);
	float angle = (h.x - 0.5) * 6.2831853 * strength;
	float ca = cos(angle);
	float sa = sin(angle);
	vec2 pivot = cell + 0.5;
	vec2 d = uv - pivot;
	vec2 turned = vec2(d.x * ca - d.y * sa, d.x * sa + d.y * ca);
	return texture(t, pivot + turned + (h - 0.5) * strength).rgb;
}

// Blends the four cells nearest to the sample. In the middle of a cell one of them carries all the weight, at
// the corners where the seams would be they mix, so the joins are blurred out instead of lining up.
// [param sharpen] raises the weights to a power, which pulls the mixing into a narrow band around the corners
// and leaves the rest of each cell a single crisp read, undoing the softness an even mix would give.
vec3 detiled(sampler2D t, vec2 uv, float strength, float sharpen) {
	if (strength <= 0.0) {
		return texture(t, uv).rgb;
	}
	vec2 g = uv - 0.5;
	vec2 base = floor(g);
	vec2 f = g - base;
	vec2 w = f * f * (3.0 - 2.0 * f);
	float power = mix(1.0, 8.0, clamp(sharpen, 0.0, 1.0));
	vec3 sum = vec3(0.0);
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

vec3 layer(sampler2D t, float tile, float detile, float sharpen, vec3 blend) {
	float s = 1.0 / max(tile, 0.001);
	vec3 top = detiled(t, map_position.xz * s, detile, sharpen);
	vec3 side_x = detiled(t, vec2(map_position.z, -map_position.y) * s, detile, sharpen);
	vec3 side_z = detiled(t, vec2(map_position.x, -map_position.y) * s, detile, sharpen);
	return top * blend.y + side_x * blend.x + side_z * blend.z;
}

vec3 layer_glow(sampler2D t, vec3 color, float energy, float textured, float multiply, float tile, float detile, float sharpen, vec3 blend) {
	vec3 tex = vec3(0.0);
	if (textured > 0.5) {
		tex = layer(t, tile, detile, sharpen, blend);
	}
	return (multiply > 0.5 ? color * tex : color + tex) * energy;
}

void fragment() {
	vec3 blend = pow(abs(normalize(map_normal)), vec3(6.0));
	blend /= max(blend.x + blend.y + blend.z, 0.0001);
	vec3 c0 = layer(layer0, tiles.x, detiles.x, sharpens.x, blend);
	vec3 c1 = layer(layer1, tiles.y, detiles.y, sharpens.y, blend);
	vec3 c2 = layer(layer2, tiles.z, detiles.z, sharpens.z, blend);
	vec3 c3 = layer(layer3, tiles.w, detiles.w, sharpens.w, blend);
	vec4 w = COLOR;
	float sum = w.r + w.g + w.b + w.a;
	// An all zero splat means the first layer, matching Terrain::weights.
	w = sum > 0.0001 ? w / sum : vec4(1.0, 0.0, 0.0, 0.0);
	ALBEDO = c0 * w.r + c1 * w.g + c2 * w.b + c3 * w.a;
	vec3 g0 = layer_glow(glow_texture0, glow_color0.rgb, glow_energy.x, glow_textured.x, glow_multiply.x, tiles.x, detiles.x, sharpens.x, blend);
	vec3 g1 = layer_glow(glow_texture1, glow_color1.rgb, glow_energy.y, glow_textured.y, glow_multiply.y, tiles.y, detiles.y, sharpens.y, blend);
	vec3 g2 = layer_glow(glow_texture2, glow_color2.rgb, glow_energy.z, glow_textured.z, glow_multiply.z, tiles.z, detiles.z, sharpens.z, blend);
	vec3 g3 = layer_glow(glow_texture3, glow_color3.rgb, glow_energy.w, glow_textured.w, glow_multiply.w, tiles.w, detiles.w, sharpens.w, blend);
	EMISSION = g0 * w.r + g1 * w.g + g2 * w.b + g3 * w.a;
	ROUGHNESS = roughness_value;
}
