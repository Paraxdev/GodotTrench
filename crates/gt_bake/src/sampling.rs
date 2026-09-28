//! Sample patterns. Each texel walks the R2 low discrepancy sequence from its own random start, which spreads few
//! rays evenly and leaves no pattern between neighbouring texels.

use glam::{Vec2, Vec3};

pub fn hash(mut x: u32) -> u32 {
    x = x.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    let w = ((x >> ((x >> 28) + 4)) ^ x).wrapping_mul(277_803_737);
    (w >> 22) ^ w
}

fn unit(x: u32) -> f32 {
    (x >> 8) as f32 / (1u32 << 24) as f32
}

/// Point `k` of a sequence seeded by `seed`, in the unit square.
pub fn r2(seed: u32, k: u32) -> Vec2 {
    const A1: f32 = 0.754_877_7;
    const A2: f32 = 0.569_840_3;
    let start = Vec2::new(unit(hash(seed)), unit(hash(seed ^ 0x9e37_79b9)));
    let k = k as f32;
    Vec2::new((start.x + A1 * k).fract(), (start.y + A2 * k).fract())
}

/// Two axes perpendicular to `n` and each other.
pub fn basis(n: Vec3) -> (Vec3, Vec3) {
    let sign = 1.0f32.copysign(n.z);
    let a = -1.0 / (sign + n.z);
    let b = n.x * n.y * a;
    (Vec3::new(1.0 + sign * n.x * n.x * a, sign * b, -sign * n.x), Vec3::new(b, sign + n.y * n.y * a, -n.y))
}

/// A direction around `n`, denser towards `n` in proportion to the cosine.
pub fn cosine(n: Vec3, u: Vec2) -> Vec3 {
    let (t, b) = basis(n);
    let r = u.x.sqrt();
    let phi = std::f32::consts::TAU * u.y;
    (t * (r * phi.cos()) + b * (r * phi.sin()) + n * (1.0 - u.x).max(0.0).sqrt()).normalize()
}

/// A point in the unit disk.
pub fn disk(u: Vec2) -> Vec2 {
    let r = u.x.sqrt();
    let phi = std::f32::consts::TAU * u.y;
    Vec2::new(r * phi.cos(), r * phi.sin())
}

/// A direction within `angle` radians of `dir`.
pub fn cone(dir: Vec3, angle: f32, u: Vec2) -> Vec3 {
    if angle <= 0.0 {
        return dir;
    }

    let (t, b) = basis(dir);
    let d = disk(u) * angle.tan();
    (dir + t * d.x + b * d.y).normalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosine_directions_stay_in_the_hemisphere_and_average_to_the_normal() {
        let n = Vec3::new(0.3, 0.8, -0.5).normalize();
        let mut sum = Vec3::ZERO;
        for k in 0..4096 {
            let d = cosine(n, r2(11, k));
            assert!(d.dot(n) >= -1e-4);
            sum += d;
        }

        // The mean of a cosine lobe points along the normal with length 2/3.
        let mean = sum / 4096.0;
        assert!((mean.length() - 2.0 / 3.0).abs() < 0.02, "{}", mean.length());
        assert!(mean.normalize().dot(n) > 0.999);
    }

    #[test]
    fn basis_is_orthonormal_even_facing_down() {
        for n in [Vec3::Z, Vec3::NEG_Z, Vec3::X, Vec3::new(1.0, -2.0, 0.3).normalize()] {
            let (t, b) = basis(n);
            assert!(t.dot(b).abs() < 1e-5 && t.dot(n).abs() < 1e-5 && b.dot(n).abs() < 1e-5);
            assert!((t.length() - 1.0).abs() < 1e-5 && (b.length() - 1.0).abs() < 1e-5);
        }
    }
}
