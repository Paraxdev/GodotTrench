struct SkyOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) ndc: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> SkyOut {
    let p = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u)) * 2.0 - 1.0;
    var o: SkyOut;
    o.clip = vec4<f32>(p, 0.0, 1.0);
    o.ndc = p;
    return o;
}

@fragment
fn fs_main(f: SkyOut) -> @location(0) vec4<f32> {
    let far = cam.inv_view_proj * vec4<f32>(f.ndc, 0.5, 1.0);
    let dir = normalize(far.xyz / far.w - cam.eye.xyz);
    let up = dir.y;
    var color: vec3<f32>;
    if (up >= 0.0) {
        color = mix(lighting.sky_horizon.rgb, lighting.sky_top.rgb, pow(up, 0.55));
    } else {
        color = mix(lighting.sky_horizon.rgb, lighting.sky_ground.rgb, pow(min(-up * 3.0, 1.0), 0.7));
    }
    let to_sun = -normalize(lighting.sun_dir.xyz);
    let d = max(dot(dir, to_sun), 0.0);
    let sun = lighting.sun_color.rgb * (pow(d, 12000.0) * 4.0 + pow(d, 64.0) * 0.12) * min(lighting.sun_color.w, 2.0);
    color = color + sun;
    if (lighting.fog.w > 0.0) {
        color = mix(color, lighting.fog.rgb, clamp(1.0 - up * 4.0, 0.0, 1.0) * 0.3);
    }
    return vec4<f32>(color, 1.0);
}
