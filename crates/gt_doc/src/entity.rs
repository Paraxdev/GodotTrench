use std::collections::BTreeMap;

use gt_core::{DMat3, DMat4, DQuat, DVec3};
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
        let q = transform_rotation(m, self.rotation());
        let (y, x, z) = q.to_euler(gt_core::EulerRot::YXZ);
        let clean = |d: f64| {
            let r = (d * 1e4).round() / 1e4;
            if r == -0.0 { 0.0 } else { r }
        };
        self.angles = DVec3::new(clean(x.to_degrees()), clean(y.to_degrees()), clean(z.to_degrees()));
    }
}

/// Orientation `rot` after the transform `m`. A mirroring transform cannot be a rotation, so the result keeps the
/// transformed forward (-Z) and up (+Y) directions and gives up the handedness of the side axis, the way a mirrored
/// copy of a left-right symmetric object faces.
pub fn transform_rotation(m: &DMat4, rot: DQuat) -> DQuat {
    let linear = DMat3::from_mat4(*m);
    let back = (linear * (rot * DVec3::Z)).normalize_or_zero();
    let up = (linear * (rot * DVec3::Y)).normalize_or_zero();
    let side = up.cross(back).normalize_or_zero();
    if side == DVec3::ZERO || back == DVec3::ZERO {
        let (_, r, _) = m.to_scale_rotation_translation();
        return (r * rot).normalize();
    }

    DQuat::from_mat3(&DMat3::from_cols(side, back.cross(side), back)).normalize()
}

/// Whether a light's `fixture` value names `targetname`: exactly, or by prefix with a trailing `*`.
pub fn fixture_matches(fixture: &str, targetname: &str) -> bool {
    match fixture.strip_suffix('*') {
        Some(prefix) => targetname.starts_with(prefix),
        None => fixture == targetname,
    }
}

/// Fixtures of the lights that start off (`start_on 0`), the way the Godot addon shows them when the map starts.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OffFixtures {
    /// Fixtures of lights with `fixture_off hide` (the default), not shown at all.
    pub hidden: std::collections::BTreeSet<gt_core::NodeId>,
    /// Fixtures of lights with `fixture_off dark`, shown with the emission of their materials off.
    pub dark: std::collections::BTreeSet<gt_core::NodeId>,
}

pub fn off_fixtures(map: &crate::map::Map) -> OffFixtures {
    let mut hide: Vec<&str> = Vec::new();
    let mut dark: Vec<&str> = Vec::new();
    for (_, e) in map.entities().filter(|(_, e)| e.property("start_on") == Some("0")) {
        let Some(fixture) = e.property("fixture").map(str::trim).filter(|f| !f.is_empty() && *f != "*") else { continue };
        if e.property("fixture_off") == Some("dark") { dark.push(fixture) } else { hide.push(fixture) }
    }

    let mut out = OffFixtures::default();
    if hide.is_empty() && dark.is_empty() {
        return out;
    }

    for (id, e) in map.entities() {
        let Some(name) = e.targetname() else { continue };
        if hide.iter().any(|f| fixture_matches(f, name)) {
            out.hidden.insert(id);
        } else if dark.iter().any(|f| fixture_matches(f, name)) {
            out.dark.insert(id);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixtures_of_lights_that_start_off_are_hidden_or_dark() {
        let mut map = crate::map::Map::new();
        let layer = map.default_layer();
        let add = |map: &mut crate::map::Map, class: &str, props: &[(&str, &str)]| {
            let mut e = Entity::new(class);
            e.properties.extend(props.iter().map(|(k, v)| (k.to_string(), v.to_string())));
            map.insert(layer, crate::map::NodeKind::Entity(e))
        };
        add(&mut map, "light", &[("start_on", "0"), ("fixture", "hall_tube_*")]);
        add(&mut map, "light", &[("fixture", "street_bulb")]);
        add(&mut map, "light_spot", &[("start_on", "0"), ("fixture", "sign"), ("fixture_off", "dark")]);
        let tube = add(&mut map, "func_illusionary", &[("targetname", "hall_tube_1")]);
        let street = add(&mut map, "func_illusionary", &[("targetname", "street_bulb")]);
        let sign = add(&mut map, "func_illusionary", &[("targetname", "sign")]);
        let off = off_fixtures(&map);
        assert!(off.hidden.contains(&tube), "a prefix fixture of a light that starts off");
        assert!(!off.hidden.contains(&street) && !off.dark.contains(&street), "lights that start on show their fixture");
        assert!(!off.hidden.contains(&sign), "dark fixtures stay visible");
        assert_eq!(off.dark, [sign].into(), "and lose their glow");

        add(&mut map, "light", &[("start_on", "0"), ("fixture", "sign"), ("fixture_off", "hide")]);
        let off = off_fixtures(&map);
        assert!(off.hidden.contains(&sign) && off.dark.is_empty(), "a fixture that another light hides is not drawn at all");
        assert!(fixture_matches("bulb", "bulb") && !fixture_matches("bulb", "bulb_2") && fixture_matches("bulb*", "bulb_2"));
    }

    fn flipped(yaw: f64, axis: usize) -> DVec3 {
        let mut e = Entity::new("light");
        e.angles = DVec3::new(0.0, yaw, 0.0);
        let mut s = DVec3::ONE;
        s[axis] = -1.0;
        e.transform_by(&DMat4::from_scale(s));
        e.angles
    }

    fn facing(angles: DVec3) -> DVec3 {
        let mut e = Entity::new("light");
        e.angles = angles;
        e.rotation() * DVec3::NEG_Z
    }

    #[test]
    fn flips_mirror_the_facing() {
        assert_eq!(flipped(90.0, 0), DVec3::new(0.0, -90.0, 0.0));
        assert_eq!(flipped(90.0, 2), DVec3::new(0.0, 90.0, 0.0));
        assert_eq!(flipped(0.0, 0), DVec3::ZERO);
        assert!((facing(flipped(0.0, 2)) - DVec3::Z).length() < 1e-9);
        assert!((facing(flipped(30.0, 1)) - facing(DVec3::new(0.0, 30.0, 0.0))).length() < 1e-9, "a vertical flip keeps the yaw");

        for axis in 0..3 {
            let angles = DVec3::new(20.0, 70.0, 10.0);
            let mut s = DVec3::ONE;
            s[axis] = -1.0;
            let mut want = facing(angles);
            want[axis] = -want[axis];
            let mut e = Entity::new("light");
            e.angles = angles;
            e.transform_by(&DMat4::from_scale(s));
            assert!((facing(e.angles) - want).length() < 1e-6, "axis {axis}: {:?}", e.angles);
        }
    }

    #[test]
    fn rotations_are_unchanged() {
        let mut e = Entity::new("light");
        e.angles = DVec3::new(0.0, 45.0, 0.0);
        e.transform_by(&DMat4::from_rotation_y(90f64.to_radians()));
        assert_eq!(e.angles, DVec3::new(0.0, 135.0, 0.0));
    }
}
