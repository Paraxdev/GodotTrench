struct Camera {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    eye: vec4<f32>,
    params: vec4<f32>,
};

@group(0) @binding(0) var<uniform> cam: Camera;

struct VIn {
    @location(0) pos: vec3<f32>,
    @location(1) color: vec4<f32>,
};

struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(v: VIn) -> VOut {
    var o: VOut;
    var clip = cam.view_proj * vec4<f32>(v.pos, 1.0);
    // Reverse-Z: scaling depth up pulls lines towards the camera by a fraction of their distance,
    // so edges win the depth test against their own faces at any range.
    clip.z = clip.z * 1.002;
    o.clip = clip;
    o.color = v.color;
    return o;
}

@fragment
fn fs_main(f: VOut) -> @location(0) vec4<f32> {
    return f.color;
}
