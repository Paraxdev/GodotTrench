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

    /// The angles Godot builds the entity with. An `angles`, `mangle` or `angle` property wins over [`Self::angles`] and
    /// is read the Quake way, like entity_assembler.gd does.
    pub fn build_angles(&self) -> DVec3 {
        self.quake_angles().unwrap_or(self.angles)
    }

    fn quake_angles(&self) -> Option<DVec3> {
        let vector = |key: &str| -> Option<Option<DVec3>> {
            let v: Vec<f64> = self.property(key)?.split_whitespace().map(|s| s.parse().unwrap_or(0.0)).collect();
            Some((v.len() >= 3).then(|| DVec3::new(v[0], v[1], v[2])))
        };
        // A value too short to read leaves Godot's yaw turn alone.
        if let Some(q) = vector("angles") {
            return Some(q.map_or(angles_from_quake(DVec3::ZERO), angles_from_quake));
        }

        if let Some(q) = vector("mangle") {
            return Some(q.map_or(angles_from_quake(DVec3::ZERO), |q| mangle_from_quake(&self.classname, q)));
        }

        let angle: f64 = self.property("angle")?.trim().parse().unwrap_or(0.0);
        Some(if (angle + 1.0).abs() < 1e-6 {
            DVec3::new(90.0, 180.0, 0.0)
        } else if (angle + 2.0).abs() < 1e-6 {
            DVec3::new(-90.0, 180.0, 0.0)
        } else {
            angles_from_quake(DVec3::new(0.0, angle, 0.0))
        })
    }

    /// The orientation Godot builds the entity with, see [`Self::build_angles`].
    pub fn rotation(&self) -> DQuat {
        let a = self.build_angles();
        DQuat::from_euler(gt_core::EulerRot::YXZ, a.y.to_radians(), a.x.to_radians(), a.z.to_radians())
    }

    pub fn transform(&self) -> DMat4 {
        DMat4::from_rotation_translation(self.rotation(), self.origin)
    }

    /// Applies a transform to origin and angles. A turn bakes an `angles`, `mangle` or `angle` property into
    /// [`Self::angles`] and drops it, since the property would otherwise keep the old orientation.
    pub fn transform_by(&mut self, m: &DMat4) {
        self.origin = gt_core::snap_vec(m.transform_point3(self.origin));
        if self.quake_angles().is_some() {
            if DMat3::from_mat4(*m).abs_diff_eq(DMat3::IDENTITY, 1e-12) {
                return;
            }

            self.angles = self.build_angles();
            for key in ["angles", "mangle", "angle"] {
                self.properties.remove(key);
            }
        }

        let q = transform_rotation(m, self.rotation());
        let (y, x, z) = q.to_euler(gt_core::EulerRot::YXZ);
        let clean = |d: f64| {
            let r = (d * 1e4).round() / 1e4;
            if r == -0.0 { 0.0 } else { r }
        };
        self.angles = DVec3::new(clean(x.to_degrees()), clean(y.to_degrees()), clean(z.to_degrees()));
    }
}

fn wrap_deg(d: f64) -> f64 {
    let r = (d + 180.0).rem_euclid(360.0) - 180.0;
    if (r + 180.0).abs() < 1e-9 { 180.0 } else { r }
}

/// GodotTrench angles from Quake `angles` as FuncGodot reads them: pitch and roll negated, yaw turned by 180.
pub fn angles_from_quake(q: DVec3) -> DVec3 {
    DVec3::new(wrap_deg(-q.x), wrap_deg(q.y + 180.0), wrap_deg(-q.z))
}

/// `mangle` is yaw/pitch/roll on `light*` classes and unnegated pitch on `info_intermission`,
/// matching entity_assembler.gd's special cases; everything else reads it the same as `angles`.
pub fn mangle_from_quake(classname: &str, q: DVec3) -> DVec3 {
    if classname.starts_with("light") {
        DVec3::new(wrap_deg(q.y), wrap_deg(q.x + 180.0), wrap_deg(-q.z))
    } else if classname == "info_intermission" {
        DVec3::new(wrap_deg(q.x), wrap_deg(q.y + 180.0), wrap_deg(-q.z))
    } else {
        angles_from_quake(q)
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

/// Whether a light's `fixture` value names `targetname`: exactly, or by prefix with a trailing `*`, so `*` alone
/// names every named entity. Group (`@lamps`), node path (`/root/...`) and special (`!self`) targets name no entity
/// by its targetname.
pub fn fixture_matches(fixture: &str, targetname: &str) -> bool {
    if fixture.starts_with(['@', '/', '!']) {
        return false;
    }

    match fixture.strip_suffix('*') {
        Some(prefix) => targetname.starts_with(prefix),
        None => fixture == targetname,
    }
}

/// Whether the light's `fixture` names entity `e` the way GodotTrenchIO.find_targets in the addon finds it. `in_group`
/// answers whether Godot puts an entity in a node group, which the entity definitions decide. A node path depends on
/// where the map sits in the Godot scene, so it matches nothing here.
fn fixture_names(fixture: &str, e: &Entity, in_group: &dyn Fn(&Entity, &str) -> bool) -> bool {
    match fixture.strip_prefix('@') {
        Some(group) => in_group(e, group),
        None => e.targetname().is_some_and(|name| fixture_matches(fixture, name)),
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

pub fn off_fixtures(map: &crate::map::Map, in_group: &dyn Fn(&Entity, &str) -> bool) -> OffFixtures {
    let mut hide: Vec<(gt_core::NodeId, &str)> = Vec::new();
    let mut dark: Vec<(gt_core::NodeId, &str)> = Vec::new();
    for (id, e) in map.entities().filter(|(_, e)| e.property("start_on") == Some("0")) {
        let Some(fixture) = e.property("fixture").map(str::trim).filter(|f| !f.is_empty()) else { continue };
        if e.property("fixture_off") == Some("dark") { dark.push((id, fixture)) } else { hide.push((id, fixture)) }
    }

    let mut out = OffFixtures::default();
    if hide.is_empty() && dark.is_empty() {
        return out;
    }

    // A light never switches itself, the way apply_fixture skips it.
    let names = |list: &[(gt_core::NodeId, &str)], id, e: &Entity| list.iter().any(|(light, f)| *light != id && fixture_names(f, e, in_group));
    for (id, e) in map.entities() {
        if names(&hide, id, e) {
            out.hidden.insert(id);
        } else if names(&dark, id, e) {
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
        let no_groups = |_: &Entity, _: &str| false;
        let off = off_fixtures(&map, &no_groups);
        assert!(off.hidden.contains(&tube), "a prefix fixture of a light that starts off");
        assert!(!off.hidden.contains(&street) && !off.dark.contains(&street), "lights that start on show their fixture");
        assert!(!off.hidden.contains(&sign), "dark fixtures stay visible");
        assert_eq!(off.dark, [sign].into(), "and lose their glow");

        add(&mut map, "light", &[("start_on", "0"), ("fixture", "sign"), ("fixture_off", "hide")]);
        let off = off_fixtures(&map, &no_groups);
        assert!(off.hidden.contains(&sign) && off.dark.is_empty(), "a fixture that another light hides is not drawn at all");
        assert!(fixture_matches("bulb", "bulb") && !fixture_matches("bulb", "bulb_2") && fixture_matches("bulb*", "bulb_2"));
        assert!(fixture_matches("*", "bulb") && !fixture_matches("@bulb", "bulb") && !fixture_matches("/root/bulb", "bulb"));
    }

    #[test]
    fn fixtures_name_every_entity_groups_but_not_node_paths() {
        let mut map = crate::map::Map::new();
        let layer = map.default_layer();
        let add = |map: &mut crate::map::Map, class: &str, props: &[(&str, &str)]| {
            let mut e = Entity::new(class);
            e.properties.extend(props.iter().map(|(k, v)| (k.to_string(), v.to_string())));
            map.insert(layer, crate::map::NodeKind::Entity(e))
        };
        let lamp = add(&mut map, "func_lamp", &[]);
        let named = add(&mut map, "func_illusionary", &[("targetname", "sign")]);
        let all = add(&mut map, "light", &[("start_on", "0"), ("fixture", "*"), ("targetname", "master")]);
        let lamps = |e: &Entity, group: &str| e.classname == "func_lamp" && group == "lamps";
        let off = off_fixtures(&map, &lamps);
        assert_eq!(off.hidden, [named].into(), "* names every named entity but the light itself");

        map.remove(all);
        add(&mut map, "light", &[("start_on", "0"), ("fixture", "@lamps")]);
        assert_eq!(off_fixtures(&map, &lamps).hidden, [lamp].into(), "a group names its members, named or not");
        add(&mut map, "light", &[("start_on", "0"), ("fixture", "/root/Main/sign")]);
        assert_eq!(off_fixtures(&map, &lamps).hidden, [lamp].into(), "a node path is left alone");
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
    fn angle_properties_win_like_in_godot() {
        let with = |class: &str, key: &str, value: &str| {
            let mut e = Entity::new(class);
            e.angles = DVec3::new(10.0, 20.0, 30.0);
            e.properties.insert(key.into(), value.into());
            e.build_angles()
        };
        assert_eq!(Entity::new("light").build_angles(), DVec3::ZERO, "without a property the node's angles");
        assert_eq!(with("info_null", "angles", "10 20 30"), DVec3::new(-10.0, -160.0, -30.0), "pitch negated, yaw plus 180");
        assert_eq!(with("info_null", "mangle", "10 20 30"), DVec3::new(-10.0, -160.0, -30.0), "mangle reads like angles");
        assert_eq!(with("light", "mangle", "10 20 30"), DVec3::new(20.0, -170.0, -30.0), "light mangle is yaw pitch roll");
        assert_eq!(with("info_intermission", "mangle", "10 20 30"), DVec3::new(10.0, -160.0, -30.0), "pitch kept");
        assert_eq!(with("info_null", "angle", "90"), DVec3::new(0.0, -90.0, 0.0));
        assert_eq!(with("info_null", "angle", "-1"), DVec3::new(90.0, 180.0, 0.0), "-1 looks up");
        assert_eq!(with("info_null", "angle", "-2"), DVec3::new(-90.0, 180.0, 0.0), "-2 looks down");
        assert_eq!(with("info_null", "angles", "bad"), DVec3::new(0.0, 180.0, 0.0), "an unreadable value keeps only the yaw turn");

        let mut e = Entity::new("info_null");
        e.properties.insert("angle".into(), "90".into());
        e.properties.insert("angles".into(), "0 45 0".into());
        assert_eq!(e.build_angles(), DVec3::new(0.0, -135.0, 0.0), "angles before angle");
        let facing = e.rotation() * DVec3::NEG_Z;
        e.transform_by(&DMat4::from_translation(DVec3::X));
        assert!(e.properties.contains_key("angles") && (e.rotation() * DVec3::NEG_Z - facing).length() < 1e-9, "a move keeps the key");
        e.transform_by(&DMat4::from_rotation_y(90f64.to_radians()));
        assert!(!e.properties.contains_key("angles") && !e.properties.contains_key("angle"), "a turn bakes the key into angles");
        assert!((e.rotation() * DVec3::NEG_Z - DQuat::from_rotation_y(90f64.to_radians()) * facing).length() < 1e-6);
    }

    #[test]
    fn rotations_are_unchanged() {
        let mut e = Entity::new("light");
        e.angles = DVec3::new(0.0, 45.0, 0.0);
        e.transform_by(&DMat4::from_rotation_y(90f64.to_radians()));
        assert_eq!(e.angles, DVec3::new(0.0, 135.0, 0.0));
    }
}
