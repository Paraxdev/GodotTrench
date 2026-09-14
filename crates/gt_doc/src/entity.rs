use std::collections::BTreeMap;

use gt_core::{DMat4, DQuat, DVec3};
use serde::{Deserialize, Serialize};

/// Hammer style output: when `output` fires on this entity, call `input` on every entity named `target`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IoConnection {
    pub output: String,
    pub target: String,
    pub input: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub parameter: String,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub delay: f64,
    /// -1 fires every time, otherwise the number of times it may fire.
    #[serde(default = "default_times", skip_serializing_if = "is_default_times")]
    pub times: i32,
}

fn is_zero(v: &f64) -> bool {
    *v == 0.0
}

fn default_times() -> i32 {
    -1
}

fn is_default_times(v: &i32) -> bool {
    *v == -1
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub classname: String,
    /// Map units, Y-up. Point entities only, brush entities derive their position from brushes.
    #[serde(default)]
    pub origin: DVec3,
    /// Degrees: pitch (X), yaw (Y), roll (Z), applied in Godot's YXZ order.
    #[serde(default)]
    pub angles: DVec3,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<IoConnection>,
}

impl Entity {
    pub fn new(classname: impl Into<String>) -> Self {
        Self { classname: classname.into(), origin: DVec3::ZERO, angles: DVec3::ZERO, properties: BTreeMap::new(), outputs: Vec::new() }
    }

    pub fn property(&self, key: &str) -> Option<&str> {
        self.properties.get(key).map(|s| s.as_str())
    }

    pub fn targetname(&self) -> Option<&str> {
        self.property("targetname").filter(|s| !s.is_empty())
    }

    pub fn rotation(&self) -> DQuat {
        DQuat::from_euler(gt_core::EulerRot::YXZ, self.angles.y.to_radians(), self.angles.x.to_radians(), self.angles.z.to_radians())
    }

    pub fn transform(&self) -> DMat4 {
        DMat4::from_rotation_translation(self.rotation(), self.origin)
    }

    /// Applies a transform to origin and angles.
    pub fn transform_by(&mut self, m: &DMat4) {
        self.origin = gt_core::snap_vec(m.transform_point3(self.origin));
        let (_, rot, _) = m.to_scale_rotation_translation();
        let q = (rot * self.rotation()).normalize();
        let (y, x, z) = q.to_euler(gt_core::EulerRot::YXZ);
        let clean = |d: f64| {
            let r = (d * 1e4).round() / 1e4;
            if r == -0.0 { 0.0 } else { r }
        };
        self.angles = DVec3::new(clean(x.to_degrees()), clean(y.to_degrees()), clean(z.to_degrees()));
    }
}
