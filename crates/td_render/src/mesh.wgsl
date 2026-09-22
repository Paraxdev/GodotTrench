@group(1) @binding(0) var t_color: texture_2d<f32>;
@group(1) @binding(1) var s_color: sampler;
@group(1) @binding(2) var t_normal: texture_2d<f32>;
@group(1) @binding(3) var t_emission: texture_2d<f32>;

struct MaterialParams {
    tint: vec4<f32>,
    // rgb: linear emission color, w: energy
    emission: vec4<f32>,
    // x: has normal map, y: alpha mode (0 opaque, 1 blend, 2 scissor, 3 hash), z: scissor threshold, w: unshaded
    flags: vec4<f32>,
    // x: normal scale, y: 1 when t_emission holds a second albedo mixed in by vertex color alpha,
    // zw: UV multiplier of that second albedo
    extra: vec4<f32>,
    // x: 1 when t_emission holds an emission texture, y: 1 to multiply it with the color instead of adding it
    glow: vec4<f32>,
};
@group(1) @binding(4) var<uniform> mat: MaterialParams;

struct VIn {
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

@vertex
fn vs_main(v: VIn) -> VOut {
    var o: VOut;
    o.clip = cam.view_proj * vec4<f32>(v.pos, 1.0);
    o.world = v.pos;
    o.normal = v.normal;
    o.uv = v.uv;
    o.color = v.color;
    return o;
}

@fragment
fn fs_main(f: VOut, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    let blended = mat.extra.y > 0.5;
    let second = textureSample(t_emission, s_color, select(f.uv, f.uv * mat.extra.zw, blended));
    let tex = select(textureSample(t_color, s_color, f.uv), mix(textureSample(t_color, s_color, f.uv), second, clamp(f.color.a, 0.0, 1.0)), blended);
    let normal_sample = textureSample(t_normal, s_color, f.uv).xyz;
    let emission_sample = second.rgb;
    let dp1 = dpdx(f.world);
    let dp2 = dpdy(f.world);
    let duv1 = dpdx(f.uv);
    let duv2 = dpdy(f.uv);
    // No grid on alpha tested cards (foliage, decals): across hundreds of small leaf cards it reads as a white mesh.
    let grid = select(grid_amount(f.world, normalize(f.normal)), 0.0, mat.flags.y > 1.5);

    let flat_mode = step(0.5, cam.params.z) * step(cam.params.z, 1.5);
    let albedo = tex * mat.tint;
    let base = vec4<f32>(mix(albedo.rgb, vec3<f32>(1.0), flat_mode), mix(albedo.a, 1.0, flat_mode * step(mat.flags.y, 0.5)));
    let hash = fract(sin(dot(floor(f.clip.xy), vec2<f32>(12.9898, 78.233))) * 43758.5453);
    let cut = select(mat.flags.z, clamp(hash, 0.02, 0.98), mat.flags.y > 2.5);
    if (mat.flags.y > 1.5 && tex.a < cut && flat_mode < 0.5) {
        discard;
    }
    // Alpha tested texels that survive the cut are opaque. Passing the (mipmapped, partial) texture alpha
    // through leaves translucent pixels in the viewport target, which egui then composites as frosted edges.
    let alpha = select(base.a * select(f.color.a, 1.0, blended), 1.0, mat.flags.y > 1.5 && flat_mode < 0.5);
    if (alpha < 0.02) {
        discard;
    }
    var n = normalize(f.normal);
    if (!front) {
        n = -n;
    }
    if (mat.flags.x > 0.5 && flat_mode < 0.5) {
        n = perturb_normal(n, dp1, dp2, duv1, duv2, normal_sample, mat.extra.x);
    }
    // Godot's emission: (color + texture) or (color * texture), times energy, with a black texture when none is set.
    let emission_tex = select(vec3<f32>(0.0), emission_sample, mat.glow.x > 0.5);
    var glow = select(mat.emission.rgb + emission_tex, mat.emission.rgb * emission_tex, mat.glow.y > 0.5) * mat.emission.w;
    if (blended) {
        glow = glow * (1.0 - clamp(f.color.a, 0.0, 1.0));
    }
    var rgb: vec3<f32>;
    if (mat.flags.w > 0.5 && flat_mode < 0.5) {
        rgb = base.rgb * f.color.rgb;
        if (is_lit()) {
            rgb = rgb + glow * (vec3<f32>(1.0) - rgb);
        }
    } else if (is_lit()) {
        // Emission joins the light before the tonemap, so it glows at night and saturates like a real light.
        rgb = tonemap(base.rgb * f.color.rgb * light_at(f.world, n) + glow);
    } else {
        rgb = shade(base.rgb * f.color.rgb, f.world, n);
    }
    if (flat_mode < 0.5 && !is_lit()) {
        // Textured mode has no light values to add to, so emission only brightens towards the glow color.
        rgb = max(rgb, min(glow, vec3<f32>(1.0)) * 0.85);
    }
    rgb = apply_fog(rgb, f.world);
    let grid_strength = select(0.22, 0.06, is_lit());
    rgb = mix(rgb, vec3<f32>(1.0), grid * grid_strength);
    return vec4<f32>(rgb, alpha);
}
