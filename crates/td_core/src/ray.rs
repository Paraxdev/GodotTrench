use glam::DVec3;

use crate::{Aabb, Plane};

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: DVec3,
    pub dir: DVec3,
}

impl Ray {
    pub fn new(origin: DVec3, dir: DVec3) -> Self {
        Self { origin, dir: dir.normalize() }
    }

    pub fn at(&self, t: f64) -> DVec3 {
        self.origin + self.dir * t
    }

    pub fn intersect_plane(&self, plane: &Plane) -> Option<f64> {
        plane.intersect_line(self.origin, self.dir)
    }

    pub fn intersect_aabb(&self, b: &Aabb) -> Option<f64> {
        let inv = 1.0 / self.dir;
        let t1 = (b.min - self.origin) * inv;
        let t2 = (b.max - self.origin) * inv;
        let tmin = t1.min(t2).max_element();
        let tmax = t1.max(t2).min_element();
        if tmax >= tmin.max(0.0) { Some(tmin.max(0.0)) } else { None }
    }

    /// Möller-Trumbore, double sided.
    pub fn intersect_triangle(&self, a: DVec3, b: DVec3, c: DVec3) -> Option<f64> {
        let e1 = b - a;
        let e2 = c - a;
        let p = self.dir.cross(e2);
        let det = e1.dot(p);
        if det.abs() < 1e-12 {
            return None;
        }

        let inv = 1.0 / det;
        let s = self.origin - a;
        let u = s.dot(p) * inv;
        if !(0.0..=1.0).contains(&u) {
            return None;
        }

        let q = s.cross(e1);
        let v = self.dir.dot(q) * inv;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = e2.dot(q) * inv;
        (t >= 0.0).then_some(t)
    }

    /// Distance from the ray to a point, and the ray parameter of the closest point.
    pub fn distance_to_point(&self, p: DVec3) -> (f64, f64) {
        let t = (p - self.origin).dot(self.dir).max(0.0);
        ((self.at(t) - p).length(), t)
    }

    /// Closest approach between the ray and a segment: (distance, ray t, segment s in 0..1).
    pub fn distance_to_segment(&self, a: DVec3, b: DVec3) -> (f64, f64, f64) {
        let d1 = self.dir;
        let d2 = b - a;
        let r = self.origin - a;
        let aa = d1.dot(d1);
        let e = d2.dot(d2);
        let f = d2.dot(r);
        if e < 1e-12 {
            let (d, t) = self.distance_to_point(a);
            return (d, t, 0.0);
        }

        let c = d1.dot(r);
        let bb = d1.dot(d2);
        let denom = aa * e - bb * bb;
        let mut s = if denom.abs() > 1e-12 { ((bb * f - c * e) / denom).max(0.0) } else { 0.0 };
        let mut u = (bb * s + f) / e;
        if u < 0.0 {
            u = 0.0;
            s = (-c / aa).max(0.0);
        } else if u > 1.0 {
            u = 1.0;
            s = ((bb - c) / aa).max(0.0);
        }

        let p1 = self.origin + d1 * s;
        let p2 = a + d2 * u;
        ((p1 - p2).length(), s, u)
    }
}
