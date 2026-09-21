use egui::{Pos2, Rect, Vec2};
use gt_core::{Aabb, DMat4, DVec3, Ray};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewKind {
    Perspective,
    /// Looking down -Y. Screen right = +X, screen up = -Z.
    Top,
    /// Looking along -Z. Screen right = +X, screen up = +Y.
    Front,
    /// Looking along -X. Screen right = -Z, screen up = +Y.
    Side,
}

impl ViewKind {
    pub fn label(&self) -> &'static str {
        match self {
            ViewKind::Perspective => "3D",
            ViewKind::Top => "Top (X/Z)",
            ViewKind::Front => "Front (X/Y)",
            ViewKind::Side => "Side (Z/Y)",
        }
    }

    pub fn is_2d(&self) -> bool {
        !matches!(self, ViewKind::Perspective)
    }

    /// (right, up, forward) for 2D views.
    pub fn axes(&self) -> (DVec3, DVec3, DVec3) {
        match self {
            ViewKind::Top => (DVec3::X, DVec3::NEG_Z, DVec3::NEG_Y),
            ViewKind::Front => (DVec3::X, DVec3::Y, DVec3::NEG_Z),
            ViewKind::Side => (DVec3::NEG_Z, DVec3::Y, DVec3::NEG_X),
            ViewKind::Perspective => (DVec3::X, DVec3::Y, DVec3::NEG_Z),
        }
    }

    /// World axis index the 2D view looks along.
    pub fn depth_axis(&self) -> usize {
        match self {
            ViewKind::Top => 1,
            ViewKind::Front | ViewKind::Perspective => 2,
            ViewKind::Side => 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Camera {
    pub kind: ViewKind,
    pub position: DVec3,
    pub yaw: f64,
    pub pitch: f64,
    pub fov: f64,
    /// 2D: view center.
    pub center: DVec3,
    /// 2D: pixels per map unit.
    pub zoom: f64,
}

const NEAR: f64 = 1.0;
const ORTHO_DEPTH: f64 = 131_072.0;

impl Camera {
    pub fn new(kind: ViewKind) -> Self {
        Self {
            kind,
            position: DVec3::new(-256.0, 192.0, 256.0),
            yaw: (-45.0f64).to_radians(),
            pitch: (-25.0f64).to_radians(),
            fov: 90.0,
            center: DVec3::ZERO,
            zoom: 0.5,
        }
    }

    pub fn forward(&self) -> DVec3 {
        match self.kind {
            ViewKind::Perspective => DVec3::new(-self.yaw.sin() * self.pitch.cos(), self.pitch.sin(), -self.yaw.cos() * self.pitch.cos()),
            k => k.axes().2,
        }
    }

    pub fn right(&self) -> DVec3 {
        match self.kind {
            ViewKind::Perspective => DVec3::new(self.yaw.cos(), 0.0, -self.yaw.sin()),
            k => k.axes().0,
        }
    }

    pub fn up(&self) -> DVec3 {
        match self.kind {
            ViewKind::Perspective => self.right().cross(self.forward()).normalize(),
            k => k.axes().1,
        }
    }

    pub fn eye(&self) -> DVec3 {
        match self.kind {
            ViewKind::Perspective => self.position,
            _ => self.center - self.forward() * ORTHO_DEPTH * 0.5,
        }
    }

    pub fn view_proj(&self, size: Vec2) -> DMat4 {
        let w = size.x.max(1.0) as f64;
        let h = size.y.max(1.0) as f64;
        use glam::dcamera::rh::{proj::directx, view};
        let view = view::look_to_mat4(self.eye(), self.forward(), self.up());
        let proj = match self.kind {
            ViewKind::Perspective => {
                let vfov = 2.0 * ((self.fov.to_radians() * 0.5).tan() * h / w).atan();
                directx::perspective_infinite_reverse(vfov, w / h, NEAR)
            }
            _ => {
                let hw = w * 0.5 / self.zoom;
                let hh = h * 0.5 / self.zoom;
                // Swapped near and far give reverse depth, matching the perspective projection.
                directx::orthographic(-hw, hw, -hh, hh, ORTHO_DEPTH, 0.0)
            }
        };
        proj * view
    }

    /// Ray through a point in the viewport rect.
    pub fn ray(&self, rect: Rect, pos: Pos2) -> Ray {
        let ndc_x = ((pos.x - rect.min.x) / rect.width()) as f64 * 2.0 - 1.0;
        let ndc_y = 1.0 - ((pos.y - rect.min.y) / rect.height()) as f64 * 2.0;
        match self.kind {
            ViewKind::Perspective => {
                let aspect = rect.width() as f64 / rect.height().max(1.0) as f64;
                let t = (self.fov.to_radians() * 0.5).tan();
                let dir = self.forward() + self.right() * ndc_x * t + self.up() * ndc_y * t / aspect;
                Ray::new(self.position, dir)
            }
            _ => {
                let world = self.screen_to_plane(rect, pos);
                Ray::new(world - self.forward() * ORTHO_DEPTH * 0.5, self.forward())
            }
        }
    }

    /// 2D views: the world point under the cursor on the plane through `center`.
    pub fn screen_to_plane(&self, rect: Rect, pos: Pos2) -> DVec3 {
        let (r, u, _) = self.kind.axes();
        let d = pos - rect.center();
        self.center + r * (d.x as f64 / self.zoom) - u * (d.y as f64 / self.zoom)
    }

    pub fn project(&self, rect: Rect, world: DVec3) -> Option<Pos2> {
        let clip = self.view_proj(rect.size()) * world.extend(1.0);
        if clip.w <= 1e-6 {
            return None;
        }

        let ndc = clip.truncate() / clip.w;
        Some(Pos2::new(rect.min.x + ((ndc.x + 1.0) * 0.5) as f32 * rect.width(), rect.min.y + ((1.0 - ndc.y) * 0.5) as f32 * rect.height()))
    }

    pub fn look_at(&mut self, target: DVec3) {
        let d = (target - self.position).normalize_or(DVec3::NEG_Z);
        self.pitch = d.y.clamp(-1.0, 1.0).asin();
        self.yaw = (-d.x).atan2(-d.z);
    }

    pub fn rotate(&mut self, delta: Vec2, sensitivity: f64) {
        self.yaw -= delta.x as f64 * sensitivity;
        self.pitch = (self.pitch - delta.y as f64 * sensitivity).clamp(-1.55, 1.55);
    }

    /// Orbits the 3D camera around a pivot.
    pub fn orbit(&mut self, pivot: DVec3, delta: Vec2, sensitivity: f64) {
        let offset = self.position - pivot;
        let dist = offset.length();
        self.rotate(delta, sensitivity);
        self.position = pivot - self.forward() * dist;
    }

    pub fn pan(&mut self, delta: Vec2, rect: Rect) {
        match self.kind {
            ViewKind::Perspective => {
                let speed = 1.0;
                self.position += (-self.right() * delta.x as f64 + self.up() * delta.y as f64) * speed;
            }
            _ => {
                let _ = rect;
                let (r, u, _) = self.kind.axes();
                self.center += (-r * delta.x as f64 + u * delta.y as f64) / self.zoom;
            }
        }
    }

    pub fn zoom_at(&mut self, rect: Rect, pos: Pos2, factor: f64) {
        let before = self.screen_to_plane(rect, pos);
        self.zoom = (self.zoom * factor).clamp(0.005, 64.0);
        let after = self.screen_to_plane(rect, pos);
        self.center += before - after;
    }

    pub fn focus(&mut self, bounds: &Aabb, rect: Rect) {
        if bounds.is_empty() {
            return;
        }

        let c = bounds.center();
        let radius = bounds.size().length().max(32.0) * 0.5;
        match self.kind {
            ViewKind::Perspective => {
                let dist = radius / (self.fov.to_radians() * 0.5).tan() * 1.3;
                self.position = c - self.forward() * dist;
            }
            _ => {
                self.center = c;
                let (r, u, _) = self.kind.axes();
                let size = bounds.size();
                let w = (size * r.abs()).element_sum().max(32.0);
                let h = (size * u.abs()).element_sum().max(32.0);
                self.zoom = ((rect.width() as f64 / w).min(rect.height() as f64 / h) * 0.7).clamp(0.005, 64.0);
            }
        }
    }

    /// Visible world rectangle of a 2D view along its right/up axes.
    pub fn visible_range(&self, rect: Rect) -> (f64, f64, f64, f64) {
        let (r, u, _) = self.kind.axes();
        let hw = rect.width() as f64 * 0.5 / self.zoom;
        let hh = rect.height() as f64 * 0.5 / self.zoom;
        let cr = self.center.dot(r);
        let cu = self.center.dot(u);
        (cr - hw, cr + hw, cu - hh, cu + hh)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_round_trip() {
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));
        for kind in [ViewKind::Perspective, ViewKind::Top, ViewKind::Front, ViewKind::Side] {
            let cam = Camera::new(kind);
            let pos = Pos2::new(300.0, 200.0);
            let ray = cam.ray(rect, pos);
            let p = ray.at(if kind.is_2d() { ORTHO_DEPTH * 0.5 } else { 500.0 });
            let back = cam.project(rect, p).unwrap();
            assert!((back - pos).length() < 0.5, "{kind:?}: {back:?}");
        }
    }

    #[test]
    fn default_forward_is_minus_z() {
        let mut c = Camera::new(ViewKind::Perspective);
        c.yaw = 0.0;
        c.pitch = 0.0;
        assert!(gt_core::vec_approx_eq(c.forward(), DVec3::NEG_Z));
        assert!(gt_core::vec_approx_eq(c.right(), DVec3::X));
        assert!(gt_core::vec_approx_eq(c.up(), DVec3::Y));
    }
}
