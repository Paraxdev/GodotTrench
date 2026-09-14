use gt_core::{DMat3, DMat4, DQuat, DVec2, DVec3};
use serde::{Deserialize, Serialize};

/// Valve 220 style texture projection. Texel coordinates are
/// `dot(p, u_axis) / scale.x + offset.x` (same for v).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FaceUv {
    pub u_axis: DVec3,
    pub v_axis: DVec3,
    pub offset: DVec2,
    pub scale: DVec2,
    /// Degrees. Informational, the axes already include the rotation.
    #[serde(default)]
    pub rotation: f64,
}

impl Default for FaceUv {
    fn default() -> Self {
        Self::paraxial(DVec3::Y, DVec2::ONE)
    }
}

impl FaceUv {
    /// Axis aligned projection picked from the dominant normal axis, without mirroring on opposite sides.
    pub fn paraxial(normal: DVec3, scale: DVec2) -> Self {
        let (u, v) = paraxial_axes(normal);
        Self { u_axis: u, v_axis: v, offset: DVec2::ZERO, scale, rotation: 0.0 }
    }

    /// Axes lying in the face plane, u follows the plane basis.
    pub fn face_aligned(normal: DVec3, scale: DVec2) -> Self {
        let (pu, _) = paraxial_axes(normal);
        let u = (pu - normal * pu.dot(normal)).normalize_or(gt_core::Plane::from_point_normal(DVec3::ZERO, normal).basis().0);
        let v = normal.cross(u).normalize();
        let v = if v.dot(paraxial_axes(normal).1) < 0.0 { -v } else { v };
        Self { u_axis: u, v_axis: v, offset: DVec2::ZERO, scale, rotation: 0.0 }
    }

    pub fn texel(&self, p: DVec3) -> DVec2 {
        let sx = if self.scale.x.abs() < 1e-9 { 1.0 } else { self.scale.x };
        let sy = if self.scale.y.abs() < 1e-9 { 1.0 } else { self.scale.y };
        DVec2::new(p.dot(self.u_axis) / sx + self.offset.x, p.dot(self.v_axis) / sy + self.offset.y)
    }

    /// Normalized UV for a texture of the given pixel size.
    pub fn uv(&self, p: DVec3, tex_size: DVec2) -> DVec2 {
        self.texel(p) / tex_size.max(DVec2::ONE)
    }

    /// True if the projection collapses on a face with this normal.
    pub fn is_degenerate_for(&self, normal: DVec3) -> bool {
        self.u_axis.cross(self.v_axis).normalize_or_zero().dot(normal).abs() < 0.02
    }

    pub fn texture_normal(&self) -> DVec3 {
        self.u_axis.cross(self.v_axis).normalize_or(DVec3::Y)
    }

    pub fn rotate(&mut self, degrees: f64) {
        let axis = self.texture_normal();
        let q = DQuat::from_axis_angle(axis, -degrees.to_radians());
        self.u_axis = q * self.u_axis;
        self.v_axis = q * self.v_axis;
        self.rotation = (self.rotation + degrees).rem_euclid(360.0);
    }

    /// Re-expresses the projection so that texels stay attached to the geometry under `m`.
    pub fn transformed(&self, m: &DMat4) -> Self {
        let linear = DMat3::from_mat4(*m);
        let translation = m.w_axis.truncate();
        let inv_t = linear.inverse().transpose();
        let map_axis = |axis: DVec3, scale: f64, offset: f64| -> (DVec3, f64, f64) {
            let a = inv_t * axis;
            let len = a.length();
            if len < 1e-12 {
                return (axis, scale, offset);
            }
            let new_offset = offset - a.dot(translation) / scale;
            (a / len, scale / len, new_offset)
        };
        let (u, su, ou) = map_axis(self.u_axis, self.scale.x, self.offset.x);
        let (v, sv, ov) = map_axis(self.v_axis, self.scale.y, self.offset.y);
        let mut out = Self { u_axis: u, v_axis: v, offset: DVec2::new(ou, ov), scale: DVec2::new(su, sv), rotation: self.rotation };
        out.snap();
        out
    }

    /// Applies translation only, keeping texels attached (cheap uv lock for moves).
    pub fn translated(&self, offset: DVec3) -> Self {
        let mut out = self.clone();
        out.offset.x -= offset.dot(self.u_axis) / self.scale.x;
        out.offset.y -= offset.dot(self.v_axis) / self.scale.y;
        out
    }

    /// Scales and offsets the projection so the texture repeats `repeats` times across the given face.
    pub fn fit(&mut self, points: &[DVec3], tex_size: DVec2, repeats: DVec2) {
        if points.is_empty() {
            return;
        }
        let (mut umin, mut umax, mut vmin, mut vmax) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
        for p in points {
            let (u, v) = (p.dot(self.u_axis), p.dot(self.v_axis));
            umin = umin.min(u);
            umax = umax.max(u);
            vmin = vmin.min(v);
            vmax = vmax.max(v);
        }
        let reps = repeats.max(DVec2::splat(1e-6));
        if umax - umin > 1e-9 {
            self.scale.x = (umax - umin) / (tex_size.x * reps.x);
            self.offset.x = -umin / self.scale.x;
        }
        if vmax - vmin > 1e-9 {
            self.scale.y = (vmax - vmin) / (tex_size.y * reps.y);
            self.offset.y = -vmin / self.scale.y;
        }
    }

    /// Hammer++ style hotspot fit: maps the face extent onto the pixel rectangle `[x, y, w, h]`, turning the projection
    /// by 90 degrees when that matches the rectangle's aspect ratio better. Returns how well the rectangle matched (lower is better).
    pub fn fit_to_rect(&mut self, points: &[DVec3], normal: DVec3, rect: [f64; 4], allow_rotate: bool) -> f64 {
        let base = Self::face_aligned(normal, DVec2::ONE);
        let mut best: Option<(f64, FaceUv)> = None;
        for rotated in [false, true] {
            if rotated && !allow_rotate {
                continue;
            }
            let (u, v) = if rotated { (base.v_axis, -base.u_axis) } else { (base.u_axis, base.v_axis) };
            let (mut umin, mut umax, mut vmin, mut vmax) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
            for p in points {
                umin = umin.min(p.dot(u));
                umax = umax.max(p.dot(u));
                vmin = vmin.min(p.dot(v));
                vmax = vmax.max(p.dot(v));
            }
            let (w, h) = ((umax - umin).max(1e-6), (vmax - vmin).max(1e-6));
            let (rw, rh) = (rect[2].max(1e-6), rect[3].max(1e-6));
            let score = ((w / h).ln() - (rw / rh).ln()).abs();
            let scale = DVec2::new(w / rw, h / rh);
            let uv = FaceUv {
                u_axis: u,
                v_axis: v,
                offset: DVec2::new(rect[0] - umin / scale.x, rect[1] - vmin / scale.y),
                scale,
                rotation: if rotated { 90.0 } else { 0.0 },
            };
            if best.as_ref().is_none_or(|(s, _)| score < *s) {
                best = Some((score, uv));
            }
        }
        let (score, uv) = best.expect("at least one orientation");
        *self = uv;
        score
    }

    /// Texel coordinates without the offset, as (min, max) over the points.
    pub fn texel_bounds(&self, points: &[DVec3]) -> (DVec2, DVec2) {
        let base = Self { offset: DVec2::ZERO, ..self.clone() };
        points.iter().map(|p| base.texel(*p)).fold((DVec2::MAX, DVec2::MIN), |(lo, hi), t| (lo.min(t), hi.max(t)))
    }

    /// Hammer style justification of the texture against the extent of `points` (all faces for "treat as one").
    pub fn justify(&mut self, points: &[DVec3], tex_size: DVec2, mode: Justify) {
        if points.is_empty() {
            return;
        }
        let tex = tex_size.max(DVec2::ONE);
        if matches!(mode, Justify::FitWidth | Justify::FitHeight) {
            let (lo, hi) = self.texel_bounds(points);
            let extent = (hi - lo) * self.scale.abs();
            let factor = if mode == Justify::FitWidth { extent.x / tex.x } else { extent.y / tex.y };
            if factor > 1e-9 {
                self.scale = DVec2::new(factor * self.scale.x.signum(), factor * self.scale.y.signum());
            }
        }
        if mode == Justify::Fit {
            self.fit(points, tex, DVec2::ONE);
            return;
        }
        let (lo, hi) = self.texel_bounds(points);
        match mode {
            Justify::Left => self.offset.x = -lo.x,
            Justify::FitWidth | Justify::FitHeight => self.offset = -lo,
            Justify::Right => self.offset.x = tex.x - hi.x,
            Justify::Top => self.offset.y = -lo.y,
            Justify::Bottom => self.offset.y = tex.y - hi.y,
            Justify::Center => self.offset = tex * 0.5 - (lo + hi) * 0.5,
            Justify::Fit => {}
        }
    }

    /// Projection straight along the view: u follows the camera's right vector and v its down vector.
    pub fn from_view(right: DVec3, up: DVec3, scale: DVec2) -> Self {
        Self { u_axis: right.normalize_or(DVec3::X), v_axis: -up.normalize_or(DVec3::Y), offset: DVec2::ZERO, scale, rotation: 0.0 }
    }

    /// Continues `src` from the face on `src_plane` across the edge where it meets `dst_plane`, as if the destination
    /// face were folded flat into the source plane (Hammer's apply with wrap).
    pub fn wrapped(src: &FaceUv, src_plane: &gt_core::Plane, dst_plane: &gt_core::Plane) -> Self {
        let (n1, n2) = (src_plane.normal, dst_plane.normal);
        let dir = n1.cross(n2);
        if dir.length_squared() < 1e-10 {
            return src.clone();
        }
        let Some(point) = gt_core::Plane::intersect_three(src_plane, dst_plane, &gt_core::Plane::new(dir.normalize(), 0.0)) else { return src.clone() };
        let q = DQuat::from_rotation_arc(n1, n2);
        let map_axis = |axis: DVec3, scale: f64, offset: f64| -> (DVec3, f64) {
            let rotated = q * axis;
            let s = if scale.abs() < 1e-9 { 1.0 } else { scale };
            (rotated, offset + (point.dot(axis) - point.dot(rotated)) / s)
        };
        let (u, ou) = map_axis(src.u_axis, src.scale.x, src.offset.x);
        let (v, ov) = map_axis(src.v_axis, src.scale.y, src.offset.y);
        let mut out = Self { u_axis: u, v_axis: v, offset: DVec2::new(ou, ov), scale: src.scale, rotation: src.rotation };
        out.snap();
        out
    }

    /// Keeps offsets small, they only matter modulo the texture size.
    pub fn wrap_offset(&mut self, tex_size: DVec2) {
        self.offset.x = self.offset.x.rem_euclid(tex_size.x.max(1.0));
        self.offset.y = self.offset.y.rem_euclid(tex_size.y.max(1.0));
    }

    fn snap(&mut self) {
        let s = |v: f64| {
            let r = (v * 1e6).round() / 1e6;
            if (v - r).abs() < 1e-9 { r } else { v }
        };
        self.u_axis = DVec3::new(s(self.u_axis.x), s(self.u_axis.y), s(self.u_axis.z));
        self.v_axis = DVec3::new(s(self.v_axis.x), s(self.v_axis.y), s(self.v_axis.z));
        self.offset = DVec2::new(s(self.offset.x), s(self.offset.y));
        self.scale = DVec2::new(s(self.scale.x), s(self.scale.y));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Justify {
    Left,
    Right,
    Top,
    Bottom,
    Center,
    /// One repeat across the extent on both axes.
    Fit,
    /// Uniform scale so one repeat spans the width, then aligned to the top left.
    FitWidth,
    FitHeight,
}

impl Justify {
    pub const ALL: [Justify; 8] =
        [Justify::Left, Justify::Right, Justify::Top, Justify::Bottom, Justify::Center, Justify::Fit, Justify::FitWidth, Justify::FitHeight];

    pub fn label(&self) -> &'static str {
        match self {
            Justify::Left => "Left",
            Justify::Right => "Right",
            Justify::Top => "Top",
            Justify::Bottom => "Bottom",
            Justify::Center => "Center",
            Justify::Fit => "Fit",
            Justify::FitWidth => "Fit W",
            Justify::FitHeight => "Fit H",
        }
    }
}

pub fn paraxial_axes(normal: DVec3) -> (DVec3, DVec3) {
    match gt_core::major_axis(normal) {
        0 => {
            if normal.x >= 0.0 {
                (DVec3::NEG_Z, DVec3::NEG_Y)
            } else {
                (DVec3::Z, DVec3::NEG_Y)
            }
        }
        1 => (DVec3::X, DVec3::Z),
        _ => {
            if normal.z >= 0.0 {
                (DVec3::X, DVec3::NEG_Y)
            } else {
                (DVec3::NEG_X, DVec3::NEG_Y)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uv_lock_under_rigid_transform() {
        let uv = FaceUv::paraxial(DVec3::Z, DVec2::new(0.5, 2.0));
        let m = DMat4::from_rotation_translation(DQuat::from_rotation_y(0.7), DVec3::new(13.0, -4.0, 9.5));
        let moved = uv.transformed(&m);
        for p in [DVec3::new(1.0, 2.0, 3.0), DVec3::new(-40.0, 8.0, 16.0)] {
            let a = uv.texel(p);
            let b = moved.texel(m.transform_point3(p));
            assert!((a - b).length() < 1e-6, "{a} vs {b}");
        }
    }

    #[test]
    fn hotspot_fit_maps_face_onto_rect() {
        let pts = [DVec3::new(0.0, 0.0, 0.0), DVec3::new(128.0, 0.0, 0.0), DVec3::new(128.0, 32.0, 0.0), DVec3::new(0.0, 32.0, 0.0)];
        let mut uv = FaceUv::default();
        uv.fit_to_rect(&pts, DVec3::Z, [0.0, 64.0, 256.0, 64.0], true);
        let texels: Vec<DVec2> = pts.iter().map(|p| uv.texel(*p)).collect();
        let (lo, hi) = texels.iter().fold((DVec2::MAX, DVec2::MIN), |(lo, hi), t| (lo.min(*t), hi.max(*t)));
        assert!((lo - DVec2::new(0.0, 64.0)).length() < 1e-6 && (hi - DVec2::new(256.0, 128.0)).length() < 1e-6, "{lo} {hi}");
        let mut tall = FaceUv::default();
        tall.fit_to_rect(&pts, DVec3::Z, [0.0, 0.0, 32.0, 128.0], true);
        assert_eq!(tall.rotation, 90.0);
    }

    #[test]
    fn justify_aligns_texture_edges() {
        let pts = [DVec3::new(10.0, 0.0, 0.0), DVec3::new(74.0, 0.0, 0.0), DVec3::new(74.0, 40.0, 0.0), DVec3::new(10.0, 40.0, 0.0)];
        let tex = DVec2::new(64.0, 64.0);
        let mut uv = FaceUv::paraxial(DVec3::Z, DVec2::ONE);
        uv.justify(&pts, tex, Justify::Left);
        uv.justify(&pts, tex, Justify::Top);
        let (lo, _) = pts.iter().map(|p| uv.texel(*p)).fold((DVec2::MAX, DVec2::MIN), |(lo, hi), t| (lo.min(t), hi.max(t)));
        assert!((lo.x).abs() < 1e-9 && lo.y.abs() < 1e-9, "{lo}");
        uv.justify(&pts, tex, Justify::Bottom);
        let hi_y = pts.iter().map(|p| uv.texel(*p).y).fold(f64::MIN, f64::max);
        assert!((hi_y - 64.0).abs() < 1e-9);
        uv.justify(&pts, tex, Justify::FitHeight);
        let (lo, hi) = pts.iter().map(|p| uv.texel(*p)).fold((DVec2::MAX, DVec2::MIN), |(lo, hi), t| (lo.min(t), hi.max(t)));
        assert!((hi.y - lo.y - 64.0).abs() < 1e-9 && (uv.scale.x - uv.scale.y).abs() < 1e-12, "uniform fit {lo} {hi} {}", uv.scale);
    }

    #[test]
    fn wrap_is_seamless_across_the_shared_edge() {
        // A wall facing -Z and a wall facing +X meeting at the corner x = 64, z = 0.
        let src_plane = gt_core::Plane::from_point_normal(DVec3::ZERO, DVec3::NEG_Z);
        let dst_plane = gt_core::Plane::from_point_normal(DVec3::new(64.0, 0.0, 0.0), DVec3::X);
        let mut src = FaceUv::paraxial(DVec3::NEG_Z, DVec2::new(0.5, 0.75));
        src.offset = DVec2::new(13.0, 7.0);
        let wrapped = FaceUv::wrapped(&src, &src_plane, &dst_plane);
        for y in [0.0, 17.0, 96.0] {
            let corner = DVec3::new(64.0, y, 0.0);
            assert!((src.texel(corner) - wrapped.texel(corner)).length() < 1e-6, "edge texels match at y {y}");
        }
        // Moving 10 units away from the edge continues in the same direction on both faces.
        let a = src.texel(DVec3::new(54.0, 5.0, 0.0)) - src.texel(DVec3::new(64.0, 5.0, 0.0));
        let b = wrapped.texel(DVec3::new(64.0, 5.0, 10.0)) - wrapped.texel(DVec3::new(64.0, 5.0, 0.0));
        assert!((a + b).length() < 1e-6, "unfolded continuation {a} {b}");
    }

    #[test]
    fn uv_lock_under_scale() {
        let uv = FaceUv::paraxial(DVec3::Y, DVec2::ONE);
        let m = DMat4::from_scale_rotation_translation(DVec3::new(2.0, 1.0, 3.0), DQuat::IDENTITY, DVec3::new(1.0, 2.0, 3.0));
        let moved = uv.transformed(&m);
        let p = DVec3::new(5.0, 0.0, 7.0);
        assert!((uv.texel(p) - moved.texel(m.transform_point3(p))).length() < 1e-6);
    }
}
