use glam::{DMat4, DVec3};
use serde::{Deserialize, Serialize};

use crate::EPSILON;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaneSide {
    Front,
    Back,
    On,
}

/// Plane in the form `dot(normal, p) == dist`. Front is the side the normal points to.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Plane {
    pub normal: DVec3,
    pub dist: f64,
}

impl Default for Plane {
    fn default() -> Self {
        Self { normal: DVec3::Y, dist: 0.0 }
    }
}

impl Plane {
    pub fn new(normal: DVec3, dist: f64) -> Self {
        Self { normal, dist }
    }

    pub fn from_point_normal(point: DVec3, normal: DVec3) -> Self {
        let n = normal.normalize();
        Self { normal: n, dist: n.dot(point) }
    }

    /// Counter-clockwise winding (seen from the front) produces an outward normal.
    pub fn from_points(a: DVec3, b: DVec3, c: DVec3) -> Option<Self> {
        let n = (b - a).cross(c - a);
        let len = n.length();
        if len < 1e-12 {
            return None;
        }

        let n = n / len;
        Some(Self { normal: n, dist: n.dot(a) })
    }

    /// Newell's method, robust for slightly non-planar polygons.
    pub fn from_polygon(points: &[DVec3]) -> Option<Self> {
        if points.len() < 3 {
            return None;
        }

        let mut n = DVec3::ZERO;
        let mut centroid = DVec3::ZERO;
        for i in 0..points.len() {
            let cur = points[i];
            let next = points[(i + 1) % points.len()];
            n.x += (cur.y - next.y) * (cur.z + next.z);
            n.y += (cur.z - next.z) * (cur.x + next.x);
            n.z += (cur.x - next.x) * (cur.y + next.y);
            centroid += cur;
        }

        let len = n.length();
        if len < 1e-12 {
            return None;
        }

        centroid /= points.len() as f64;
        Some(Self::from_point_normal(centroid, n / len))
    }

    pub fn distance(&self, p: DVec3) -> f64 {
        self.normal.dot(p) - self.dist
    }

    pub fn side(&self, p: DVec3) -> PlaneSide {
        self.side_eps(p, EPSILON)
    }

    pub fn side_eps(&self, p: DVec3, eps: f64) -> PlaneSide {
        let d = self.distance(p);
        if d > eps {
            PlaneSide::Front
        } else if d < -eps {
            PlaneSide::Back
        } else {
            PlaneSide::On
        }
    }

    pub fn flipped(&self) -> Self {
        Self { normal: -self.normal, dist: -self.dist }
    }

    pub fn project_point(&self, p: DVec3) -> DVec3 {
        p - self.normal * self.distance(p)
    }

    pub fn point(&self) -> DVec3 {
        self.normal * self.dist
    }

    pub fn translated(&self, offset: DVec3) -> Self {
        Self { normal: self.normal, dist: self.dist + self.normal.dot(offset) }
    }

    pub fn offset(&self, amount: f64) -> Self {
        Self { normal: self.normal, dist: self.dist + amount }
    }

    pub fn transformed(&self, m: &DMat4) -> Self {
        let p = m.transform_point3(self.point());
        let n = m.inverse().transpose().transform_vector3(self.normal);
        Self::from_point_normal(p, n)
    }

    pub fn approx_eq(&self, other: &Plane, normal_eps: f64, dist_eps: f64) -> bool {
        (self.normal - other.normal).abs().max_element() <= normal_eps && (self.dist - other.dist).abs() <= dist_eps
    }

    /// Returns parameter t along `origin + dir * t` where the line meets the plane.
    pub fn intersect_line(&self, origin: DVec3, dir: DVec3) -> Option<f64> {
        let denom = self.normal.dot(dir);
        if denom.abs() < 1e-12 {
            return None;
        }

        Some((self.dist - self.normal.dot(origin)) / denom)
    }

    pub fn intersect_three(a: &Plane, b: &Plane, c: &Plane) -> Option<DVec3> {
        let n1 = a.normal;
        let n2 = b.normal;
        let n3 = c.normal;
        let denom = n1.dot(n2.cross(n3));
        if denom.abs() < 1e-12 {
            return None;
        }

        Some((n2.cross(n3) * a.dist + n3.cross(n1) * b.dist + n1.cross(n2) * c.dist) / denom)
    }

    /// Two orthonormal tangents spanning the plane.
    pub fn basis(&self) -> (DVec3, DVec3) {
        let n = self.normal;
        let helper = if n.y.abs() < 0.9 { DVec3::Y } else { DVec3::X };
        let u = helper.cross(n).normalize();
        let v = n.cross(u);
        (u, v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_plane_intersection() {
        let p = Plane::intersect_three(&Plane::new(DVec3::X, 1.0), &Plane::new(DVec3::Y, 2.0), &Plane::new(DVec3::Z, 3.0)).unwrap();
        assert!(crate::vec_approx_eq(p, DVec3::new(1.0, 2.0, 3.0)));
    }

    #[test]
    fn winding_gives_outward_normal() {
        let p = Plane::from_points(DVec3::ZERO, DVec3::X, DVec3::Y).unwrap();
        assert!(crate::vec_approx_eq(p.normal, DVec3::Z));
    }
}
