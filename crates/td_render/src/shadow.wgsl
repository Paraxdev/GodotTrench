@group(0) @binding(0) var<uniform> light_view_proj: mat4x4<f32>;

@vertex
fn vs_main(
    @location(0) pos: vec3<f32>,
    @location(5) row0: vec4<f32>,
    @location(6) row1: vec4<f32>,
    @location(7) row2: vec4<f32>,
) -> @builtin(position) vec4<f32> {
    let p = vec4<f32>(pos, 1.0);
    return light_view_proj * vec4<f32>(dot(row0, p), dot(row1, p), dot(row2, p), 1.0);
}
