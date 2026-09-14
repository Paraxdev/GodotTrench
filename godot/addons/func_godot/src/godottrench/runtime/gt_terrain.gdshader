shader_type spatial;
// GodotTrench terrain: four texture layers blended by vertex color weights (r, g, b, a), projected triplanar so
// cliffs are not stretched. The same projection as the editor preview.

uniform sampler2D layer0 : source_color, filter_linear_mipmap_anisotropic, repeat_enable;
uniform sampler2D layer1 : source_color, filter_linear_mipmap_anisotropic, repeat_enable;
uniform sampler2D layer2 : source_color, filter_linear_mipmap_anisotropic, repeat_enable;
uniform sampler2D layer3 : source_color, filter_linear_mipmap_anisotropic, repeat_enable;
// Meters covered by one repeat of each layer.
uniform vec4 tiles = vec4(8.0);
// Terrain origin in map space, so textures line up with the editor.
uniform vec3 map_offset = vec3(0.0);
uniform float roughness_value : hint_range(0.0, 1.0) = 0.9;

varying vec3 map_position;
varying vec3 map_normal;

void vertex() {
	map_position = VERTEX + map_offset;
	map_normal = NORMAL;
}

vec3 layer(sampler2D t, float tile, vec3 blend) {
	float s = 1.0 / max(tile, 0.001);
	vec3 top = texture(t, map_position.xz * s).rgb;
	vec3 side_x = texture(t, vec2(map_position.z, -map_position.y) * s).rgb;
	vec3 side_z = texture(t, vec2(map_position.x, -map_position.y) * s).rgb;
	return top * blend.y + side_x * blend.x + side_z * blend.z;
}

void fragment() {
	vec3 blend = pow(abs(normalize(map_normal)), vec3(6.0));
	blend /= max(blend.x + blend.y + blend.z, 0.0001);
	vec3 c0 = layer(layer0, tiles.x, blend);
	vec3 c1 = layer(layer1, tiles.y, blend);
	vec3 c2 = layer(layer2, tiles.z, blend);
	vec3 c3 = layer(layer3, tiles.w, blend);
	vec4 w = COLOR;
	float sum = max(w.r + w.g + w.b + w.a, 0.0001);
	ALBEDO = (c0 * w.r + c1 * w.g + c2 * w.b + c3 * w.a) / sum;
	ROUGHNESS = roughness_value;
}
