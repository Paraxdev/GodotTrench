// The GPU side of trace.rs: direct light and the bounce gather for a range of samples. The math and the random
// sequences follow trace.rs and sampling.rs line by line, so both backends bake the same map up to rounding.

struct Params {
    sky_top: vec3f,
    sky_energy: f32,
    sky_horizon: vec3f,
    sky_share: f32,
    sky_ground: vec3f,
    bias: f32,
    sky_ambient: vec3f,
    far: f32,
    offset: u32,
    count: u32,
    lights: u32,
    rays: u32,
    shadow_samples: u32,
    pass_index: u32,
    width: u32,
    height: u32,
    upm: f32,
    ao_distance: f32,
    inside: f32,
    tris: u32,
}

struct Node {
    min: vec3f,
    start: u32,
    max: vec3f,
    count: u32,
}

struct Tri {
    a: vec3f,
    id: u32,
    e1: vec3f,
    surface: u32,
    e2: vec3f,
    pad: u32,
}

struct Surface {
    albedo: vec3f,
    chart: u32,
    emission: vec3f,
    double_sided: u32,
}

struct Chart {
    axis0: vec3f,
    texel: f32,
    axis1: vec3f,
    pad: f32,
    min: vec2f,
    corner: vec2f,
}

struct Sample {
    pos: vec3f,
    pad0: f32,
    normal: vec3f,
    pad1: f32,
    face_normal: vec3f,
    pad2: f32,
}

struct Light {
    position: vec3f,
    kind: u32,
    direction: vec3f,
    range: f32,
    color: vec3f,
    attenuation: f32,
    spot_cos: f32,
    spot_attenuation: f32,
    size: f32,
    indirect_energy: f32,
    flags: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> nodes: array<Node>;
@group(0) @binding(2) var<storage, read> tris: array<Tri>;
@group(0) @binding(3) var<storage, read> surfaces: array<Surface>;
@group(0) @binding(4) var<storage, read> charts: array<Chart>;
@group(0) @binding(5) var<storage, read> samples: array<Sample>;
@group(0) @binding(6) var<storage, read> lights: array<Light>;
// The light each texel sends on, as two packed pairs of half floats.
@group(0) @binding(7) var<storage, read> prev: array<vec2u>;
@group(0) @binding(8) var<storage, read_write> out: array<vec4f>;

const NO_CHART: u32 = 0xffffffffu;
const MISS: f32 = 1e30;
const STACK: u32 = 64u;
const TAU: f32 = 6.2831853;

fn hash(v: u32) -> u32 {
    let x = v * 747796405u + 2891336453u;
    let w = ((x >> ((x >> 28u) + 4u)) ^ x) * 277803737u;
    return (w >> 22u) ^ w;
}

fn unit(x: u32) -> f32 {
    return f32(x >> 8u) / 16777216.0;
}

fn r2(seed: u32, k: u32) -> vec2f {
    let start = vec2f(unit(hash(seed)), unit(hash(seed ^ 0x9e3779b9u)));
    let kf = f32(k);
    return fract(vec2f(start.x + 0.7548777 * kf, start.y + 0.5698403 * kf));
}

fn basis_t(n: vec3f) -> vec3f {
    let sign = select(-1.0, 1.0, n.z >= 0.0);
    let a = -1.0 / (sign + n.z);
    let b = n.x * n.y * a;
    return vec3f(1.0 + sign * n.x * n.x * a, sign * b, -sign * n.x);
}

fn basis_b(n: vec3f) -> vec3f {
    let sign = select(-1.0, 1.0, n.z >= 0.0);
    let a = -1.0 / (sign + n.z);
    let b = n.x * n.y * a;
    return vec3f(b, sign + n.y * n.y * a, -n.y);
}

fn cosine(n: vec3f, u: vec2f) -> vec3f {
    let r = sqrt(u.x);
    let phi = TAU * u.y;
    return normalize(basis_t(n) * (r * cos(phi)) + basis_b(n) * (r * sin(phi)) + n * sqrt(max(1.0 - u.x, 0.0)));
}

fn cone(dir: vec3f, angle: f32, u: vec2f) -> vec3f {
    if angle <= 0.0 {
        return dir;
    }

    let r = sqrt(u.x);
    let phi = TAU * u.y;
    let d = vec2f(r * cos(phi), r * sin(phi)) * tan(angle);
    return normalize(dir + basis_t(dir) * d.x + basis_b(dir) * d.y);
}

fn sphere(u: vec2f, seed: u32) -> vec3f {
    let z = u.x * 2.0 - 1.0;
    let r = sqrt(max(1.0 - z * z, 0.0));
    let phi = TAU * u.y;
    let radius = pow(f32(hash(seed) >> 8u) / 16777216.0, 1.0 / 3.0);
    return vec3f(r * cos(phi), r * sin(phi), z) * radius;
}

fn sky(dir: vec3f) -> vec3f {
    let angle = acos(clamp(dir.y, -1.0, 1.0));
    let half = 1.5707964;
    var gradient: vec3f;
    if angle <= half {
        let c = 1.0 - angle / half;
        gradient = mix(params.sky_horizon, params.sky_top, clamp(1.0 - pow(max(1.0 - c, 0.0), 1.0 / 0.15), 0.0, 1.0));
    } else {
        let c = (angle - half) / half;
        gradient = mix(params.sky_horizon, params.sky_ground, clamp(1.0 - pow(max(1.0 - c, 0.0), 1.0 / 0.02), 0.0, 1.0));
    }

    return (gradient * params.sky_share + params.sky_ambient * (1.0 - params.sky_share)) * params.sky_energy;
}

fn safe_inv(x: f32) -> f32 {
    if abs(x) < 1e-20 {
        return select(-1e20, 1e20, x >= 0.0);
    }

    return 1.0 / x;
}

fn slab(n: Node, o: vec3f, inv: vec3f, t_min: f32, t_max: f32) -> f32 {
    let t0 = (n.min - o) * inv;
    let t1 = (n.max - o) * inv;
    let lo = min(t0, t1);
    let hi = max(t0, t1);
    let near = max(max(max(lo.x, lo.y), lo.z), t_min);
    let far = min(min(min(hi.x, hi.y), hi.z), t_max);
    return select(MISS, near, near <= far);
}

struct Hit {
    t: f32,
    surface: u32,
    front: bool,
    found: bool,
}

// Bvh::closest when `any` is false, Bvh::occluded when it is true.
fn trace(o: vec3f, d: vec3f, t_min: f32, t_max: f32, any: bool) -> Hit {
    var hit = Hit(0.0, 0u, false, false);
    if params.tris == 0u {
        return hit;
    }

    var t_far = t_max;
    let inv = vec3f(safe_inv(d.x), safe_inv(d.y), safe_inv(d.z));
    var stack: array<u32, 64>;
    var top = 0u;
    var node = 0u;
    loop {
        let n = nodes[node];
        if n.count > 0u {
            for (var k = n.start; k < n.start + n.count; k++) {
                let tri = tris[k];
                let p = cross(d, tri.e2);
                let det = dot(tri.e1, p);
                if abs(det) < 1e-12 {
                    continue;
                }

                let inv_det = 1.0 / det;
                let s = o - tri.a;
                let u = dot(s, p) * inv_det;
                if u < 0.0 || u > 1.0 {
                    continue;
                }

                let q = cross(s, tri.e1);
                let v = dot(d, q) * inv_det;
                if v < 0.0 || u + v > 1.0 {
                    continue;
                }

                let t = dot(tri.e2, q) * inv_det;
                if t > t_min && t < t_far {
                    t_far = t;
                    hit = Hit(t, tri.surface, det > 0.0, true);
                    if any {
                        return hit;
                    }
                }
            }
        } else {
            let l = node + 1u;
            let r = n.start;
            let tl = slab(nodes[l], o, inv, t_min, t_far);
            let tr = slab(nodes[r], o, inv, t_min, t_far);
            let hl = tl < MISS;
            let hr = tr < MISS;
            if hl && hr {
                var near = l;
                var far = r;
                if tr < tl {
                    near = r;
                    far = l;
                }

                if top < STACK {
                    stack[top] = far;
                    top += 1u;
                }

                node = near;
                continue;
            }

            if hl {
                node = l;
                continue;
            }

            if hr {
                node = r;
                continue;
            }
        }

        if top == 0u {
            break;
        }

        top -= 1u;
        node = stack[top];
    }

    return hit;
}

fn falloff(distance: f32, range: f32, decay: f32) -> f32 {
    let x = distance / max(range, 1e-4);
    let window = max(1.0 - x * x * x * x, 0.0);
    return window * window * pow(max(distance, 1e-4), -decay);
}

fn prev_at(x: f32, y: f32) -> vec3f {
    let xi = u32(clamp(i32(x), 0, i32(params.width) - 1));
    let yi = u32(clamp(i32(y), 0, i32(params.height) - 1));
    let v = prev[yi * params.width + xi];
    return vec3f(unpack2x16float(v.x), unpack2x16float(v.y).x);
}

fn bilinear(px: vec2f) -> vec3f {
    let p = px - vec2f(0.5);
    let x0 = floor(p.x);
    let y0 = floor(p.y);
    let fx = p.x - x0;
    let fy = p.y - y0;
    let top = mix(prev_at(x0, y0), prev_at(x0 + 1.0, y0), fx);
    let bottom = mix(prev_at(x0, y0 + 1.0), prev_at(x0 + 1.0, y0 + 1.0), fx);
    return mix(top, bottom, fy);
}

fn pixel(c: Chart, p: vec3f) -> vec2f {
    return c.corner + (vec2f(dot(p, c.axis0), dot(p, c.axis1)) - c.min) / c.texel;
}

@compute @workgroup_size(64)
fn direct(@builtin(global_invocation_id) gid: vec3u) {
    let local = gid.x;
    if local >= params.count {
        return;
    }

    let index = params.offset + local;
    let s = samples[index];
    let upm = max(params.upm, 1e-3);
    let origin = s.pos + s.face_normal * params.bias;
    var bake = vec3f(0.0);
    var bounce = vec3f(0.0);
    var sun = 1.0;
    for (var li = 0u; li < params.lights; li++) {
        let l = lights[li];
        let bake_direct = (l.flags & 1u) != 0u;
        let mask = (l.flags & 2u) != 0u;
        if !bake_direct && l.indirect_energy <= 0.0 && !mask {
            continue;
        }

        var reaches = false;
        var light = vec3f(0.0);
        var to = vec3f(0.0);
        if l.kind == 0u {
            to = -l.direction;
            let ndl = dot(s.normal, to);
            if ndl > 0.0 {
                reaches = true;
                light = l.color * ndl;
            }
        } else {
            let d = l.position - s.pos;
            let dist = length(d);
            if dist < l.range && dist > 1e-4 {
                to = d / dist;
                let ndl = dot(s.normal, to);
                if ndl > 0.0 {
                    var att = falloff(dist / upm, l.range / upm, l.attenuation);
                    if l.kind == 2u {
                        let c = clamp(l.spot_cos, -1.0, 0.9999);
                        let scos = max(dot(-to, l.direction), c);
                        let rim = max((1.0 - scos) / (1.0 - c), 1e-4);
                        att *= 1.0 - pow(rim, l.spot_attenuation);
                    }

                    if att > 0.0 {
                        reaches = true;
                        light = l.color * att * ndl;
                    }
                }
            }
        }

        if !reaches {
            if mask {
                sun = 0.0;
            }

            continue;
        }

        var count = 1u;
        if l.size > 0.0 {
            count = max(params.shadow_samples, 1u);
        }

        let seed = hash(index * 31u + li * 7919u);
        var lit = 0u;
        for (var k = 0u; k < count; k++) {
            var u = vec2f(0.5);
            if count > 1u {
                u = r2(seed, k);
            }

            var blocked = false;
            if l.kind == 0u {
                blocked = trace(origin, cone(to, l.size, u), 0.0, params.far, true).found;
            } else {
                var target_point = l.position;
                if l.size > 0.0 {
                    target_point = l.position + sphere(u, seed ^ k) * l.size;
                }

                let d = target_point - origin;
                let len = length(d);
                blocked = len > 1e-3 && trace(origin, d / len, 0.0, len - params.bias, true).found;
            }

            if !blocked {
                lit += 1u;
            }
        }

        let visible = f32(lit) / f32(count);
        if bake_direct {
            bake += light * visible;
        }

        bounce += light * visible * l.indirect_energy;
        if mask {
            sun = visible;
        }
    }

    out[local * 2u] = vec4f(bake, sun);
    out[local * 2u + 1u] = vec4f(bounce, 0.0);
}

@compute @workgroup_size(64)
fn gather(@builtin(global_invocation_id) gid: vec3u) {
    let local = gid.x;
    if local >= params.count {
        return;
    }

    let index = params.offset + local;
    let s = samples[index];
    let rays = max(params.rays, 1u);
    let seed = hash(index ^ (params.pass_index * 0x68e31da4u));
    let origin = s.pos + s.face_normal * params.bias;
    let ao_distance = max(params.ao_distance, 1e-3);
    var sum = vec3f(0.0);
    var open = 0.0;
    var backs = 0u;
    for (var k = 0u; k < rays; k++) {
        var dir = cosine(s.normal, r2(seed, k));
        let below = dot(dir, s.face_normal);
        if below <= 1e-3 {
            dir = normalize(dir + s.face_normal * (2e-3 - below));
        }

        let hit = trace(origin, dir, 0.0, params.far, false);
        if !hit.found {
            sum += sky(dir);
            open += 1.0;
            continue;
        }

        let surface = surfaces[hit.surface];
        open += clamp(hit.t / ao_distance, 0.0, 1.0);
        if !hit.front && surface.double_sided == 0u {
            backs += 1u;
            continue;
        }

        var arriving = vec3f(0.0);
        if surface.chart != NO_CHART {
            arriving = bilinear(pixel(charts[surface.chart], origin + dir * hit.t));
        }

        sum += surface.emission + surface.albedo * arriving;
    }

    let n = f32(rays);
    let ao = open / n;
    // Samples inside a solid carry their ambient occlusion negated and less one.
    out[local] = vec4f(sum / n, select(ao, -(ao + 1.0), f32(backs) > n * params.inside));
}
