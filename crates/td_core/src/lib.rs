pub mod aabb;
pub mod color;
pub mod plane;
pub mod ray;

pub use aabb::Aabb;
pub use color::Color;
pub use glam::{DMat3, DMat4, DQuat, DVec2, DVec3, DVec4, EulerRot, Mat4, Vec2, Vec3, Vec4};
pub use plane::{Plane, PlaneSide};
pub use ray::Ray;

/// Tolerance for point-on-plane classification, in map units.
pub const EPSILON: f64 = 1e-6;
/// Coordinates closer than this to a whole number are snapped to it after plane intersection.
pub const SNAP_EPSILON: f64 = 1e-5;

pub fn snap_coord(v: f64) -> f64 {
    let r = v.round();
    if (v - r).abs() < SNAP_EPSILON { r } else { v }
}

pub fn snap_vec(v: DVec3) -> DVec3 {
    DVec3::new(snap_coord(v.x), snap_coord(v.y), snap_coord(v.z))
}

pub fn snap_to_grid(v: f64, grid: f64) -> f64 {
    if grid <= 0.0 { v } else { (v / grid).round() * grid }
}

pub fn snap_vec_to_grid(v: DVec3, grid: f64) -> DVec3 {
    DVec3::new(snap_to_grid(v.x, grid), snap_to_grid(v.y, grid), snap_to_grid(v.z, grid))
}

pub fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() <= EPSILON
}

pub fn vec_approx_eq(a: DVec3, b: DVec3) -> bool {
    (a - b).abs().max_element() <= EPSILON
}

/// Index (0 = x, 1 = y, 2 = z) of the largest absolute component.
pub fn major_axis(v: DVec3) -> usize {
    let a = v.abs();
    if a.x >= a.y && a.x >= a.z {
        0
    } else if a.y >= a.z {
        1
    } else {
        2
    }
}

/// Stable identifier for any node in a map document.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct NodeId(pub u64);

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}
