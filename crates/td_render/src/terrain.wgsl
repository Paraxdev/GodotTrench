@group(1) @binding(0) var t_layer0: texture_2d<f32>;
@group(1) @binding(1) var t_layer1: texture_2d<f32>;
@group(1) @binding(2) var t_layer2: texture_2d<f32>;
@group(1) @binding(3) var t_layer3: texture_2d<f32>;
@group(1) @binding(4) var s_terrain: sampler;
// World units per texture repeat for each layer.
@group(1) @binding(5) var<uniform> tiles: vec4<f32>;

struct VIn {
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    // x: 1 when selected
    @location(2) uv: vec2<f32>,
    // blend weights of the four layers
    @location(3) color: vec4<f32>,
};

struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) flags: vec2<f32>,
    @location(3) weights: vec4<f32>,
};

@vertex
fn vs_main(v: VIn) -> VOut {
    var o: VOut;
    o.clip = cam.view_proj * vec4<f32>(v.pos, 1.0);
    o.world = v.pos;
    o.normal = v.normal;
    o.flags = v.uv;
    o.weights = v.color;
    return o;
}

/// Triplanar sample so steep cliffs are not stretched by the top down projection.
fn layer(t: texture_2d<f32>, tile: f32, world: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
    let s = 1.0 / max(tile, 0.001);
    let top = textureSample(t, s_terrain, world.xz * s).rgb;
    let side_x = textureSample(t, s_terrain, vec2<f32>(world.z, -world.y) * s).rgb;
    let side_z = textureSample(t, s_terrain, vec2<f32>(world.x, -world.y) * s).rgb;
    return top * blend.y + side_x * blend.x + side_z * blend.z;
}

@fragment
fn fs_main(f: VOut) -> @location(0) vec4<f32> {
    let nn = normalize(f.normal);
    var blend = pow(abs(nn), vec3<f32>(6.0));
    blend = blend / max(blend.x + blend.y + blend.z, 0.0001);
    let c0 = layer(t_layer0, tiles.x, f.world, blend);
    let c1 = layer(t_layer1, tiles.y, f.world, blend);
    let c2 = layer(t_layer2, tiles.z, f.world, blend);
    let c3 = layer(t_layer3, tiles.w, f.world, blend);
    let grid = grid_amount(f.world, nn);
    let w = f.weights / max(f.weights.x + f.weights.y + f.weights.z + f.weights.w, 0.0001);
    var base = c0 * w.x + c1 * w.y + c2 * w.z + c3 * w.w;
    let flat_mode = step(0.5, cam.params.z) * step(cam.params.z, 1.5);
    base = mix(base, vec3<f32>(0.8, 0.8, 0.8), flat_mode);
    base = mix(base, base * vec3<f32>(1.0, 0.62, 0.58), f.flags.x);
    var rgb = shade(base, f.world, f.normal);
    rgb = apply_fog(rgb, f.world);
    let grid_strength = select(0.12, 0.03, is_lit());
    rgb = mix(rgb, vec3<f32>(1.0), grid * grid_strength);
    return vec4<f32>(rgb, 1.0);
}
