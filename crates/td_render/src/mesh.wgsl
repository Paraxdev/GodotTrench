@group(1) @binding(0) var t_color: texture_2d<f32>;
@group(1) @binding(1) var s_color: sampler;
@group(1) @binding(2) var t_normal: texture_2d<f32>;
@group(1) @binding(3) var t_emission: texture_2d<f32>;

struct MaterialParams {
    tint: vec4<f32>,
    // rgb: emission times energy, w: 1 when an emission texture is bound
    emission: vec4<f32>,
    // x: has normal map, y: alpha mode (0 opaque, 1 blend, 2 scissor), z: scissor threshold, w: unshaded
    flags: vec4<f32>,
    // x: normal scale, y: 1 when t_emission holds a second albedo mixed in by vertex color alpha
    extra: vec4<f32>,
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
    let second = textureSample(t_emission, s_color, f.uv);
    let tex = select(textureSample(t_color, s_color, f.uv), mix(textureSample(t_color, s_color, f.uv), second, clamp(f.color.a, 0.0, 1.0)), blended);
    let normal_sample = textureSample(t_normal, s_color, f.uv).xyz;
    let emission_sample = second.rgb;
    let dp1 = dpdx(f.world);
    let dp2 = dpdy(f.world);
    let duv1 = dpdx(f.uv);
    let duv2 = dpdy(f.uv);
    let grid = grid_amount(f.world, normalize(f.normal));

    let flat_mode = step(0.5, cam.params.z) * step(cam.params.z, 1.5);
    let albedo = tex * mat.tint;
    let base = vec4<f32>(mix(albedo.rgb, vec3<f32>(1.0), flat_mode), mix(albedo.a, 1.0, flat_mode * step(mat.flags.y, 0.5)));
    if (mat.flags.y > 1.5 && tex.a < mat.flags.z && flat_mode < 0.5) {
        discard;
    }
    let alpha = base.a * select(f.color.a, 1.0, blended);
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
    var rgb: vec3<f32>;
    if (mat.flags.w > 0.5 && flat_mode < 0.5) {
        rgb = base.rgb * f.color.rgb;
    } else {
        rgb = shade(base.rgb * f.color.rgb, f.world, n);
    }
    if (flat_mode < 0.5) {
        let glow = mat.emission.rgb * mix(vec3<f32>(1.0), emission_sample, mat.emission.w);
        // Textured mode has no light values to add to, so emission only brightens towards the glow color.
        rgb = select(max(rgb, min(glow, vec3<f32>(1.0)) * 0.85), rgb + glow * (vec3<f32>(1.0) - rgb), is_lit());
    }
    rgb = apply_fog(rgb, f.world);
    let grid_strength = select(0.22, 0.06, is_lit());
    rgb = mix(rgb, vec3<f32>(1.0), grid * grid_strength);
    return vec4<f32>(rgb, alpha);
}
