@group(1) @binding(0) var t_layer0: texture_2d<f32>;
@group(1) @binding(1) var t_layer1: texture_2d<f32>;
@group(1) @binding(2) var t_layer2: texture_2d<f32>;
@group(1) @binding(3) var t_layer3: texture_2d<f32>;
@group(1) @binding(4) var s_terrain: sampler;
struct TerrainParams {
    // World units per texture repeat for each layer.
    tiles: vec4<f32>,
    // Per layer strength of the de-tiling, 0 leaves the texture repeating as authored.
    detiles: vec4<f32>,
    // Per layer crispness of the de-tiled blend, 0 mixes the cells evenly, 1 mixes only in a narrow band.
    sharpens: vec4<f32>,
};
@group(1) @binding(5) var<uniform> params: TerrainParams;

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

fn hash2(p: vec2<f32>) -> vec2<f32> {
    var q = fract(vec3<f32>(p.x, p.y, p.x) * vec3<f32>(0.1031, 0.1030, 0.0973));
    q = q + dot(q, vec3<f32>(q.y, q.z, q.x) + 33.33);
    return fract((vec2<f32>(q.x, q.x) + vec2<f32>(q.y, q.z)) * vec2<f32>(q.z, q.y));
}

/// One repeat of the texture, turned and shifted by an amount fixed per cell. Matches gt_terrain.gdshader.
fn cell_sample(t: texture_2d<f32>, uv: vec2<f32>, cell: vec2<f32>, strength: f32) -> vec3<f32> {
    let h = hash2(cell);
    let angle = (h.x - 0.5) * 6.2831853 * strength;
    let ca = cos(angle);
    let sa = sin(angle);
    let pivot = cell + 0.5;
    let d = uv - pivot;
    let turned = vec2<f32>(d.x * ca - d.y * sa, d.x * sa + d.y * ca);
    return textureSample(t, s_terrain, pivot + turned + (h - 0.5) * strength).rgb;
}

/// Blends the four nearest cells so the joins of the repeat are blurred away at the corners. `sharpen` raises
/// the weights to a power, pulling the mixing into a narrow band and leaving the rest of each cell crisp.
fn detiled(t: texture_2d<f32>, uv: vec2<f32>, strength: f32, sharpen: f32) -> vec3<f32> {
    if strength <= 0.0 {
        return textureSample(t, s_terrain, uv).rgb;
    }

    let g = uv - 0.5;
    let base = floor(g);
    let f = g - base;
    let w = f * f * (3.0 - 2.0 * f);
    let power = mix(1.0, 8.0, clamp(sharpen, 0.0, 1.0));
    var sum = vec3<f32>(0.0);
    var total = 0.0;
    for (var j = 0; j < 2; j = j + 1) {
        for (var i = 0; i < 2; i = i + 1) {
            let corner = base + vec2<f32>(f32(i), f32(j));
            let weight = pow(mix(1.0 - w.x, w.x, f32(i)) * mix(1.0 - w.y, w.y, f32(j)), power);
            sum = sum + cell_sample(t, uv, corner, strength) * weight;
            total = total + weight;
        }
    }

    return sum / max(total, 0.00001);
}

/// Triplanar sample so steep cliffs are not stretched by the top down projection.
fn layer(t: texture_2d<f32>, tile: f32, detile: f32, sharpen: f32, world: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
    let s = 1.0 / max(tile, 0.001);
    let top = detiled(t, world.xz * s, detile, sharpen);
    let side_x = detiled(t, vec2<f32>(world.z, -world.y) * s, detile, sharpen);
    let side_z = detiled(t, vec2<f32>(world.x, -world.y) * s, detile, sharpen);
    return top * blend.y + side_x * blend.x + side_z * blend.z;
}

@fragment
fn fs_main(f: VOut) -> @location(0) vec4<f32> {
    let nn = normalize(f.normal);
    var blend = pow(abs(nn), vec3<f32>(6.0));
    blend = blend / max(blend.x + blend.y + blend.z, 0.0001);
    let c0 = layer(t_layer0, params.tiles.x, params.detiles.x, params.sharpens.x, f.world, blend);
    let c1 = layer(t_layer1, params.tiles.y, params.detiles.y, params.sharpens.y, f.world, blend);
    let c2 = layer(t_layer2, params.tiles.z, params.detiles.z, params.sharpens.z, f.world, blend);
    let c3 = layer(t_layer3, params.tiles.w, params.detiles.w, params.sharpens.w, f.world, blend);
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
