//! Per texture settings kept with the map, keyed by material name: how Bake Lighting treats faces with the texture
//! and how faces lay it out. Saved under the top-level `textures` key, see docs/format/container.md.

use std::borrow::Cow;

use gt_core::{DVec2, DVec3};
use gt_geom::FaceUv;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Projection {
    /// Each face keeps its own alignment.
    #[default]
    Face,
    /// Every face projects the texture along the world axis it faces most, ignoring its own offset, scale and
    /// rotation, so the texture runs on seamlessly across faces and brushes.
    World,
}

impl Projection {
    pub const ALL: [Projection; 2] = [Projection::Face, Projection::World];

    pub fn name(self) -> &'static str {
        match self {
            Projection::Face => "face",
            Projection::World => "world",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextureSettings {
    /// Faces with the texture get baked light. Off leaves them out of the light map, Godot then gives them the
    /// average baked light.
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub bake: bool,
    /// Faces with the texture block light in the bake.
    #[serde(default = "yes", skip_serializing_if = "is_true")]
    pub casts: bool,
    /// Multiplies the light map texel size on these faces: below 1 for sharper baked shadows, above 1 to save space.
    #[serde(default = "one", skip_serializing_if = "is_one")]
    pub texel_scale: f64,
    /// Map units one repeat of the texture covers, in place of the material's texture size.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<[f64; 2]>,
    #[serde(default, skip_serializing_if = "is_face")]
    pub projection: Projection,
}

fn yes() -> bool {
    true
}

fn one() -> f64 {
    1.0
}

fn is_true(b: &bool) -> bool {
    *b
}

fn is_one(v: &f64) -> bool {
    *v == 1.0
}

fn is_face(p: &Projection) -> bool {
    *p == Projection::Face
}

impl Default for TextureSettings {
    fn default() -> Self {
        Self { bake: true, casts: true, texel_scale: 1.0, size: None, projection: Projection::Face }
    }
}

/// Shortest and longest repeat size, in map units.
pub const SIZE_RANGE: std::ops::RangeInclusive<f64> = 1.0..=65536.0;
pub const TEXEL_SCALE_RANGE: std::ops::RangeInclusive<f64> = 0.125..=16.0;

impl TextureSettings {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    /// The same settings with every value in range, for values from files and scripts.
    pub fn sanitized(mut self) -> Self {
        self.texel_scale = if self.texel_scale.is_finite() { self.texel_scale.clamp(*TEXEL_SCALE_RANGE.start(), *TEXEL_SCALE_RANGE.end()) } else { 1.0 };
        self.size = self.size.filter(|s| s.iter().all(|v| v.is_finite() && *v > 0.0)).map(|s| s.map(|v| v.clamp(*SIZE_RANGE.start(), *SIZE_RANGE.end())));
        self
    }

    /// The projection a face with this texture uses: its own, or the world aligned one.
    pub fn face_uv<'a>(&self, uv: &'a FaceUv, normal: DVec3) -> Cow<'a, FaceUv> {
        match self.projection {
            Projection::Face => Cow::Borrowed(uv),
            Projection::World => Cow::Owned(FaceUv::paraxial(normal, DVec2::ONE)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_stay_out_of_the_file() {
        assert_eq!(serde_json::to_value(TextureSettings::default()).unwrap(), serde_json::json!({}));
        let set = TextureSettings { bake: false, texel_scale: 0.5, size: Some([256.0, 256.0]), projection: Projection::World, ..Default::default() };
        let text = serde_json::to_value(&set).unwrap();
        assert_eq!(text, serde_json::json!({ "bake": false, "texel_scale": 0.5, "size": [256.0, 256.0], "projection": "world" }));
        assert_eq!(serde_json::from_value::<TextureSettings>(text).unwrap(), set);
    }

    #[test]
    fn world_projection_runs_across_faces() {
        let set = TextureSettings { projection: Projection::World, ..Default::default() };
        let mut shifted = FaceUv::paraxial(DVec3::X, DVec2::splat(2.0));
        shifted.offset = DVec2::new(13.0, 7.0);
        let a = set.face_uv(&shifted, DVec3::X);
        let plain = FaceUv::default();
        let b = set.face_uv(&plain, DVec3::X);
        let p = DVec3::new(64.0, 10.0, 20.0);
        assert_eq!(a.texel(p), b.texel(p), "a face's own alignment is ignored");
        assert_eq!(TextureSettings::default().face_uv(&shifted, DVec3::X).texel(p), shifted.texel(p));
    }

    #[test]
    fn out_of_range_values_are_clamped() {
        let s = TextureSettings { texel_scale: f64::NAN, size: Some([0.0, 64.0]), ..Default::default() }.sanitized();
        assert_eq!((s.texel_scale, s.size), (1.0, None));
        let s = TextureSettings { texel_scale: 100.0, size: Some([1e9, 2.0]), ..Default::default() }.sanitized();
        assert_eq!((s.texel_scale, s.size), (16.0, Some([65536.0, 2.0])));
    }
}
