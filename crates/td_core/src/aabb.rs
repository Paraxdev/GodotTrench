use glam::DVec3;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Aabb {
    pub min: DVec3,
    pub max: DVec3,
}

impl Default for Aabb {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl Aabb {
    pub const EMPTY: Aabb = Aabb { min: DVec3::splat(f64::INFINITY), max: DVec3::splat(f64::NEG_INFINITY) };

    pub fn new(a: DVec3, b: DVec3) -> Self {
        Self { min: a.min(b), max: a.max(b) }
    }

    pub fn from_points(points: impl IntoIterator<Item = DVec3>) -> Self {
        let mut b = Self::EMPTY;
        for p in points {
            b.include_point(p);
        }
        b
    }

    pub fn from_center_size(center: DVec3, size: DVec3) -> Self {
        Self::new(center - size * 0.5, center + size * 0.5)
    }

    pub fn is_empty(&self) -> bool {
        self.min.x > self.max.x || self.min.y > self.max.y || self.min.z > self.max.z
    }

    pub fn include_point(&mut self, p: DVec3) {
        self.min = self.min.min(p);
        self.max = self.max.max(p);
    }

    pub fn include(&mut self, other: &Aabb) {
        if other.is_empty() {
            return;
        }
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }

    pub fn union(mut self, other: &Aabb) -> Aabb {
        self.include(other);
        self
    }

    pub fn center(&self) -> DVec3 {
        (self.min + self.max) * 0.5
    }

    pub fn size(&self) -> DVec3 {
        self.max - self.min
    }

    pub fn expanded(&self, amount: f64) -> Aabb {
        Aabb { min: self.min - DVec3::splat(amount), max: self.max + DVec3::splat(amount) }
    }

    pub fn translated(&self, offset: DVec3) -> Aabb {
        Aabb { min: self.min + offset, max: self.max + offset }
    }

    pub fn contains_point(&self, p: DVec3) -> bool {
        p.cmpge(self.min).all() && p.cmple(self.max).all()
    }

    pub fn contains(&self, other: &Aabb) -> bool {
        other.min.cmpge(self.min).all() && other.max.cmple(self.max).all()
    }

    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min.cmple(other.max).all() && self.max.cmpge(other.min).all()
    }

    /// Strict overlap, touching boxes do not count.
    pub fn overlaps(&self, other: &Aabb, eps: f64) -> bool {
        (self.min + eps).cmplt(other.max).all() && (self.max - eps).cmpgt(other.min).all()
    }

    pub fn corners(&self) -> [DVec3; 8] {
        let (a, b) = (self.min, self.max);
        [
            DVec3::new(a.x, a.y, a.z),
            DVec3::new(b.x, a.y, a.z),
            DVec3::new(b.x, b.y, a.z),
            DVec3::new(a.x, b.y, a.z),
            DVec3::new(a.x, a.y, b.z),
            DVec3::new(b.x, a.y, b.z),
            DVec3::new(b.x, b.y, b.z),
            DVec3::new(a.x, b.y, b.z),
        ]
    }

    pub const EDGES: [(usize, usize); 12] = [(0, 1), (1, 2), (2, 3), (3, 0), (4, 5), (5, 6), (6, 7), (7, 4), (0, 4), (1, 5), (2, 6), (3, 7)];
}
