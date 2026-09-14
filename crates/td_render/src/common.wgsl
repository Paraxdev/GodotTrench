struct Camera {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    eye: vec4<f32>,
    // x: grid size, y: grid alpha, z: mode (0 textured, 1 flat, 2 lit), w: 1 for orthographic
    params: vec4<f32>,
};

struct Light {
    pos_range: vec4<f32>,
    color_energy: vec4<f32>,
    // xyz: spot direction, w: cosine of the cone angle or -2 for omni lights
    dir_cone: vec4<f32>,
};

struct Lights {
    // xyz: direction the sun light travels, w: 1 when the shadow map is valid
    sun_dir: vec4<f32>,
    sun_color: vec4<f32>,
    // w: number of lights
    ambient: vec4<f32>,
    sky_top: vec4<f32>,
    sky_horizon: vec4<f32>,
    sky_ground: vec4<f32>,
    // w: density per world unit
    fog: vec4<f32>,
    shadow_view_proj: mat4x4<f32>,
    lights: array<Light, 64>,
};

@group(0) @binding(0) var<uniform> cam: Camera;
@group(0) @binding(1) var<uniform> lighting: Lights;
@group(0) @binding(2) var shadow_map: texture_depth_2d;
@group(0) @binding(3) var shadow_sampler: sampler_comparison;

fn plane_coords(world: vec3<f32>, n: vec3<f32>) -> vec2<f32> {
    let an = abs(n);
    if (an.x >= an.y && an.x >= an.z) {
        return world.zy;
    } else if (an.y >= an.z) {
        return world.xz;
    }
    return world.xy;
}

fn grid_amount(world: vec3<f32>, n: vec3<f32>) -> f32 {
    let g = plane_coords(world, n) / max(cam.params.x, 0.0001);
    let w = fwidth(g);
    let d = abs(fract(g - 0.5) - 0.5) / max(w, vec2<f32>(0.0001));
    let line = 1.0 - min(min(d.x, d.y), 1.0);
    let density = max(w.x, w.y);
    return line * clamp(1.0 - (density - 0.2) * 3.0, 0.0, 1.0) * cam.params.y * step(0.0001, cam.params.x);
}

fn sun_shadow(world: vec3<f32>, n: vec3<f32>) -> f32 {
    if (lighting.sun_dir.w < 0.5) {
        return 1.0;
    }
    let p = lighting.shadow_view_proj * vec4<f32>(world + n * 3.0, 1.0);
    let ndc = p.xyz / p.w;
    let uv = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0 || ndc.z > 1.0) {
        return 1.0;
    }
    let texel = 1.0 / vec2<f32>(textureDimensions(shadow_map));
    var sum = 0.0;
    for (var x = -1; x <= 1; x = x + 1) {
        for (var y = -1; y <= 1; y = y + 1) {
            sum = sum + textureSampleCompareLevel(shadow_map, shadow_sampler, uv + vec2<f32>(f32(x), f32(y)) * texel, ndc.z - 0.0003);
        }
    }
    return sum / 9.0;
}

/// Shades a surface. Mode 0 (textured) and 1 (flat) use fixed face shading, mode 2 uses the scene lights.
fn shade(base: vec3<f32>, world: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    let n = normalize(normal);
    if (cam.params.z < 1.5) {
        let s = 0.62 + 0.38 * (abs(n.x) * 0.72 + max(n.y, 0.0) + max(-n.y, 0.0) * 0.5 + abs(n.z) * 0.86);
        return base * s;
    }
    var light = lighting.ambient.rgb;
    let to_sun = -normalize(lighting.sun_dir.xyz);
    let ndl = max(dot(n, to_sun), 0.0);
    light = light + lighting.sun_color.rgb * lighting.sun_color.w * ndl * sun_shadow(world, n);
    let count = i32(lighting.ambient.w);
    for (var i = 0; i < count; i = i + 1) {
        let l = lighting.lights[i];
        let d = l.pos_range.xyz - world;
        let dist = length(d);
        let range = max(l.pos_range.w, 0.001);
        if (dist >= range) {
            continue;
        }
        let dir = d / max(dist, 0.0001);
        var att = pow(1.0 - dist / range, 2.0);
        if (l.dir_cone.w > -1.5) {
            let cos_angle = dot(-dir, normalize(l.dir_cone.xyz));
            att = att * smoothstep(l.dir_cone.w, min(l.dir_cone.w + 0.08, 1.0), cos_angle);
        }
        light = light + l.color_energy.rgb * l.color_energy.w * att * max(dot(n, dir), 0.0);
    }
    let lit = base * light;
    return vec3<f32>(1.0) - exp(-lit * 1.25);
}

fn is_lit() -> bool {
    return cam.params.z > 1.5;
}

/// Distance fog towards the fog color, lit mode only.
fn apply_fog(rgb: vec3<f32>, world: vec3<f32>) -> vec3<f32> {
    if (!is_lit() || lighting.fog.w <= 0.0) {
        return rgb;
    }
    let amount = 1.0 - exp(-length(world - cam.eye.xyz) * lighting.fog.w);
    return mix(rgb, lighting.fog.rgb, amount);
}

/// Normal from a tangent space normal map sample, using screen space derivatives instead of stored tangents.
fn perturb_normal(n: vec3<f32>, dp1: vec3<f32>, dp2: vec3<f32>, duv1: vec2<f32>, duv2: vec2<f32>, sample: vec3<f32>, scale: f32) -> vec3<f32> {
    let dp2perp = cross(dp2, n);
    let dp1perp = cross(n, dp1);
    let t = dp2perp * duv1.x + dp1perp * duv2.x;
    let b = dp2perp * duv1.y + dp1perp * duv2.y;
    let len = max(dot(t, t), dot(b, b));
    if (len < 1e-12) {
        return n;
    }
    let inv = inverseSqrt(len);
    // Godot normal maps point green up, texture v grows downwards.
    var ts = sample * 2.0 - 1.0;
    ts = vec3<f32>(ts.x * scale, -ts.y * scale, ts.z);
    return normalize(t * inv * ts.x + b * inv * ts.y + n * ts.z);
}
