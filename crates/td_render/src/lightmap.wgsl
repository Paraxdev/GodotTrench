@group(2) @binding(0) var t_lightmap: texture_2d<f32>;
@group(2) @binding(1) var s_lightmap: sampler;
struct Baked {
    // x: 1 when a light map is bound, y: 1 to show the map itself instead of lighting the textures with it,
    // z: 1 when the map shown holds light values that need the tonemap
    params: vec4<f32>,
};
@group(2) @binding(2) var<uniform> baked: Baked;

fn is_baked() -> bool {
    return cam.params.z > 2.5;
}

/// Whether a surface with this light map coordinate draws from the bake. Surfaces the bake does not cover, like
/// entities or brushes changed since, fall back to the lit preview.
fn has_bake(uv2: vec2<f32>) -> bool {
    return is_baked() && baked.params.x > 0.5 && uv2.x >= 0.0;
}

fn baked_light(uv2: vec2<f32>) -> vec3<f32> {
    return textureSampleLevel(t_lightmap, s_lightmap, uv2, 0.0).rgb;
}

fn shows_map() -> bool {
    return baked.params.y > 0.5;
}

/// The map itself: light through the tonemap, shadow and occlusion as they are.
fn map_color(value: vec3<f32>) -> vec3<f32> {
    return select(value, tonemap(value), baked.params.z > 0.5);
}
