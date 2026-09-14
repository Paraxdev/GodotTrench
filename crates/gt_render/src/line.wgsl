struct Camera {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    eye: vec4<f32>,
    params: vec4<f32>,
    viewport: vec4<f32>,
};

@group(0) @binding(0) var<uniform> cam: Camera;

struct VIn {
    @builtin(vertex_index) corner: u32,
    @location(0) a: vec3<f32>,
    @location(1) color_a: vec4<f32>,
    @location(2) b: vec3<f32>,
    @location(3) color_b: vec4<f32>,
};

struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec4<f32>,
};

const NEAR_W: f32 = 0.0001;

@vertex
fn vs_main(v: VIn) -> VOut {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0),
    );
    let corner = corners[v.corner];
    var o: VOut;
    var ca = cam.view_proj * vec4<f32>(v.a, 1.0);
    var cb = cam.view_proj * vec4<f32>(v.b, 1.0);
    if (ca.w < NEAR_W && cb.w < NEAR_W) {
        o.clip = vec4<f32>(2.0, 2.0, 2.0, 1.0);
        return o;
    }
    // Clip against the camera plane before the perspective divide, an endpoint behind the eye would flip the quad.
    if (ca.w < NEAR_W) {
        ca = mix(ca, cb, (NEAR_W - ca.w) / (cb.w - ca.w));
    } else if (cb.w < NEAR_W) {
        cb = mix(cb, ca, (NEAR_W - cb.w) / (ca.w - cb.w));
    }
    let size = max(cam.viewport.xy, vec2<f32>(1.0));
    let sa = ca.xy / ca.w * 0.5 * size;
    let sb = cb.xy / cb.w * 0.5 * size;
    var dir = sb - sa;
    let len = length(dir);
    dir = select(vec2<f32>(1.0, 0.0), dir / len, len > 0.00001);
    let half_width = cam.viewport.z * 0.5;
    let offset = (vec2<f32>(-dir.y, dir.x) * corner.y + dir * (corner.x * 2.0 - 1.0)) * half_width;
    var clip = mix(ca, cb, corner.x);
    clip = vec4<f32>(clip.xy + offset / (0.5 * size) * clip.w, clip.z, clip.w);
    // Reverse-Z: scaling depth up pulls lines towards the camera by a fraction of their distance,
    // so edges win the depth test against their own faces at any range.
    clip.z = clip.z * 1.002;
    o.clip = clip;
    o.color = mix(v.color_a, v.color_b, corner.x);
    return o;
}

@fragment
fn fs_main(f: VOut) -> @location(0) vec4<f32> {
    return f.color;
}
