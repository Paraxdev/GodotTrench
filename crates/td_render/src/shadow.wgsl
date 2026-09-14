@group(0) @binding(0) var<uniform> light_view_proj: mat4x4<f32>;

@vertex
fn vs_main(@location(0) pos: vec3<f32>, @location(1) normal: vec3<f32>, @location(2) uv: vec2<f32>, @location(3) color: vec4<f32>) -> @builtin(position) vec4<f32> {
    return light_view_proj * vec4<f32>(pos, 1.0);
}
