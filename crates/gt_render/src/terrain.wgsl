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
    // Per layer emission of the layer's material, rgb: linear color, w: energy.
    glow: array<vec4<f32>, 4>,
    // Per layer 1 when t_glowN holds the layer's emission texture.
    glow_textured: vec4<f32>,
    // Per layer 1 to multiply the emission color with the texture instead of adding them.
    glow_multiply: vec4<f32>,
    // Per layer 1 when t_normalN holds the layer's normal map.
    normal_mapped: vec4<f32>,
    normal_scales: vec4<f32>,
};
@group(1) @binding(5) var<uniform> params: TerrainParams;
@group(1) @binding(6) var t_glow0: texture_2d<f32>;
@group(1) @binding(7) var t_glow1: texture_2d<f32>;
@group(1) @binding(8) var t_glow2: texture_2d<f32>;
@group(1) @binding(9) var t_glow3: texture_2d<f32>;
@group(1) @binding(10) var t_normal0: texture_2d<f32>;
@group(1) @binding(11) var t_normal1: texture_2d<f32>;
@group(1) @binding(12) var t_normal2: texture_2d<f32>;
@group(1) @binding(13) var t_normal3: texture_2d<f32>;

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

/// One repeat of the texture, turned and shifted by an amount fixed per cell. A normal map comes back unpacked
/// to -1..1 with its xy turned by the same angle. Matches gt_terrain.gdshader.
fn cell_sample(t: texture_2d<f32>, uv: vec2<f32>, cell: vec2<f32>, strength: f32, normal_map: bool) -> vec3<f32> {
    let h = hash2(cell);
    let angle = (h.x - 0.5) * 6.2831853 * strength;
    let ca = cos(angle);
    let sa = sin(angle);
    let pivot = cell + 0.5;
    let d = uv - pivot;
    let turned = vec2<f32>(d.x * ca - d.y * sa, d.x * sa + d.y * ca);
    let c = textureSample(t, s_terrain, pivot + turned + (h - 0.5) * strength).rgb;
    if normal_map {
        let n = c.xy * 2.0 - 1.0;
        return vec3<f32>(n.x * ca - n.y * sa, n.x * sa + n.y * ca, 0.0);
    }

    return c;
}

/// Blends the four nearest cells so the joins of the repeat are blurred away at the corners. `sharpen` raises
/// the weights to a power, pulling the mixing into a narrow band and leaving the rest of each cell crisp.
fn detiled(t: texture_2d<f32>, uv: vec2<f32>, strength: f32, sharpen: f32, normal_map: bool) -> vec3<f32> {
    if strength <= 0.0 {
        let c = textureSample(t, s_terrain, uv).rgb;
        return select(c, vec3<f32>(c.xy * 2.0 - 1.0, 0.0), normal_map);
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
            sum = sum + cell_sample(t, uv, corner, strength, normal_map) * weight;
            total = total + weight;
        }
    }

    return sum / max(total, 0.00001);
}

/// Triplanar sample so steep cliffs are not stretched by the top down projection.
fn layer(t: texture_2d<f32>, tile: f32, detile: f32, sharpen: f32, world: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
    let s = 1.0 / max(tile, 0.001);
    let top = detiled(t, world.xz * s, detile, sharpen, false);
    let side_x = detiled(t, vec2<f32>(world.z, -world.y) * s, detile, sharpen, false);
    let side_z = detiled(t, vec2<f32>(world.x, -world.y) * s, detile, sharpen, false);
    return top * blend.y + side_x * blend.x + side_z * blend.z;
}

fn normal_z(n: vec2<f32>) -> f32 {
    return sqrt(max(0.0, 1.0 - dot(n, n)));
}

/// How far the layer's normal map turns `normal`, through the same projections as `layer`, each with its tangent
/// along u and its binormal against v. Matches gt_terrain.gdshader.
fn layer_bend(t: texture_2d<f32>, tile: f32, detile: f32, sharpen: f32, world: vec3<f32>, blend: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    let s = 1.0 / max(tile, 0.001);
    let top = detiled(t, world.xz * s, detile, sharpen, true).xy;
    let side_x = detiled(t, vec2<f32>(world.z, -world.y) * s, detile, sharpen, true).xy;
    let side_z = detiled(t, vec2<f32>(world.x, -world.y) * s, detile, sharpen, true).xy;
    return (vec3<f32>(top.x, 0.0, -top.y) + normal * (normal_z(top) - 1.0)) * blend.y
        + (vec3<f32>(0.0, side_x.y, side_x.x) + normal * (normal_z(side_x) - 1.0)) * blend.x
        + (vec3<f32>(side_z.x, side_z.y, 0.0) + normal * (normal_z(side_z) - 1.0)) * blend.z;
}

fn layer_normal_bend(t: texture_2d<f32>, l: i32, world: vec3<f32>, blend: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    if params.normal_mapped[l] < 0.5 {
        return vec3<f32>(0.0);
    }

    return layer_bend(t, params.tiles[l], params.detiles[l], params.sharpens[l], world, blend, normal) * params.normal_scales[l];
}

/// Godot's emission of one layer: (color + texture) or (color * texture), times energy, black without a texture.
/// Matches gt_terrain.gdshader.
fn layer_glow(t: texture_2d<f32>, glow: vec4<f32>, textured: f32, multiply: f32, tile: f32, detile: f32, sharpen: f32, world: vec3<f32>, blend: vec3<f32>) -> vec3<f32> {
    var tex = vec3<f32>(0.0);
    if textured > 0.5 {
        tex = layer(t, tile, detile, sharpen, world, blend);
    }

    return select(glow.rgb + tex, glow.rgb * tex, multiply > 0.5) * glow.w;
}

@fragment
fn fs_main(f: VOut) -> @location(0) vec4<f32> {
    let nn = normalize(f.normal);
    var blend = pow(abs(nn), vec3<f32>(6.0));
    blend = blend / max(blend.x + blend.y + blend.z, 0.0001);
    // The Godot shader skips projections this small, drop them here too so both blend the same.
    blend = blend * step(vec3<f32>(0.002), blend);
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
    let g0 = layer_glow(t_glow0, params.glow[0], params.glow_textured.x, params.glow_multiply.x, params.tiles.x, params.detiles.x, params.sharpens.x, f.world, blend);
    let g1 = layer_glow(t_glow1, params.glow[1], params.glow_textured.y, params.glow_multiply.y, params.tiles.y, params.detiles.y, params.sharpens.y, f.world, blend);
    let g2 = layer_glow(t_glow2, params.glow[2], params.glow_textured.z, params.glow_multiply.z, params.tiles.z, params.detiles.z, params.sharpens.z, f.world, blend);
    let g3 = layer_glow(t_glow3, params.glow[3], params.glow_textured.w, params.glow_multiply.w, params.tiles.w, params.detiles.w, params.sharpens.w, f.world, blend);
    let glow = g0 * w.x + g1 * w.y + g2 * w.z + g3 * w.w;
    var n = nn;
    if (flat_mode < 0.5) {
        let bend = layer_normal_bend(t_normal0, 0, f.world, blend, nn) * w.x
            + layer_normal_bend(t_normal1, 1, f.world, blend, nn) * w.y
            + layer_normal_bend(t_normal2, 2, f.world, blend, nn) * w.z
            + layer_normal_bend(t_normal3, 3, f.world, blend, nn) * w.w;
        n = normalize(nn + bend);
    }
    var rgb: vec3<f32>;
    if (is_lit()) {
        // Like brushes, emission joins the light before the tonemap.
        rgb = tonemap(base * light_at(f.world, n) + glow);
    } else {
        // The fixed face shading suits brush sides but barely changes over rolling ground, so terrains take light from
        // a fixed direction instead. Flat ground keeps full brightness and slopes turned away darken, the relief shows.
        let to_light = normalize(vec3<f32>(0.6, 1.0, 0.4));
        let relief = clamp(1.0 - 0.9 * (1.0 - dot(n, to_light) / to_light.y), 0.35, 1.05);
        rgb = base * relief;
        if (flat_mode < 0.5) {
            rgb = max(rgb, min(glow, vec3<f32>(1.0)) * 0.85);
        }
    }
    rgb = apply_fog(rgb, f.world);
    let grid_strength = select(0.12, 0.03, is_lit());
    rgb = mix(rgb, vec3<f32>(1.0), grid * grid_strength);
    return vec4<f32>(rgb, 1.0);
}
