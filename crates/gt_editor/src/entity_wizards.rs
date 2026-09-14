//! One step setups for common gameplay: hinged and sliding doors, lifts, buttons, trigger volumes, spawners and I/O links.

use gt_core::{Aabb, DVec3, NodeId};
use gt_doc::{Entity, IoConnection, NodeKind, ops};
use gt_geom::Brush;
use serde::{Deserialize, Serialize};

use crate::gizmos::fmt_vec3;
use crate::state::EditorState;

pub const TRIGGER_MATERIAL: &str = "special/trigger";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HingeSide {
    /// The lower end of the door's wide axis.
    #[default]
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlideDirection {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DoorKind {
    Hinged {
        #[serde(default)]
        side: HingeSide,
        #[serde(default = "default_angle")]
        angle: f64,
    },
    Sliding {
        #[serde(default)]
        direction: SlideDirection,
        /// Map units left visible when open.
        #[serde(default)]
        lip: f64,
    },
}

fn default_angle() -> f64 {
    95.0
}

/// A targetname not used yet, `base` followed by a number.
pub fn unique_name(state: &EditorState, base: &str) -> String {
    let taken: std::collections::BTreeSet<String> = state.doc.map.entities().filter_map(|(_, e)| e.targetname().map(str::to_string)).collect();
    (1..).map(|i| format!("{base}_{i}")).find(|n| !taken.contains(n)).unwrap_or_else(|| base.to_string())
}

fn geometry_of(state: &EditorState, ids: &[NodeId]) -> Vec<NodeId> {
    let mut sel = gt_doc::Selection::default();
    sel.nodes.extend(ids.iter().copied());
    sel.geometry(&state.doc.map).into_iter().filter(|id| state.doc.map.terrain(*id).is_none()).collect()
}

/// Wide horizontal axis (0 = x, 2 = z) of a door leaf.
fn wide_axis(b: &Aabb) -> usize {
    if b.size().x >= b.size().z { 0 } else { 2 }
}

/// Hinge offset and signed angle for a leaf so it opens on the chosen side.
pub fn hinge_for(bounds: &Aabb, side: HingeSide, angle: f64) -> (DVec3, f64) {
    let axis = wide_axis(bounds);
    let half = bounds.size()[axis] * 0.5;
    let mut hinge = DVec3::ZERO;
    hinge[axis] = if side == HingeSide::Left { -half } else { half };
    let angle = if side == HingeSide::Left { angle } else { -angle };
    (hinge, angle)
}

/// Turns geometry into a door entity. Returns the new entity. With `trigger`, a volume on both sides opens it.
pub fn make_door(state: &mut EditorState, ids: &[NodeId], kind: &DoorKind, trigger: bool) -> Result<NodeId, String> {
    let geometry = geometry_of(state, ids);
    if geometry.is_empty() {
        return Err("Select the brushes or meshes of the door first".into());
    }
    let bounds = state.doc.map.bounds_of(geometry.iter().copied());
    let name = unique_name(state, "door");
    let (classname, props): (&str, Vec<(String, String)>) = match kind {
        DoorKind::Hinged { side, angle } => {
            let (hinge, angle) = hinge_for(&bounds, *side, *angle);
            ("func_door_rotating", vec![("hinge".into(), fmt_vec3(hinge)), ("open_angle".into(), format!("{angle}")), ("open_away".into(), "1".into())])
        }
        DoorKind::Sliding { direction, lip } => {
            let s = bounds.size();
            let axis = wide_axis(&bounds);
            let mut travel = DVec3::ZERO;
            match direction {
                SlideDirection::Up => travel.y = s.y - lip,
                SlideDirection::Down => travel.y = -(s.y - lip),
                SlideDirection::Left => travel[axis] = -(s[axis] - lip),
                SlideDirection::Right => travel[axis] = s[axis] - lip,
            }
            ("func_door", vec![("travel".into(), fmt_vec3(travel))])
        }
    };
    let parent = state.insert_parent();
    let door = state.doc.edit("Make Door", |m, s| {
        s.clear();
        s.nodes.extend(geometry.iter().copied());
        let id = ops::create_brush_entity(m, s, classname, parent)?;
        if let Some(e) = m.entity_mut(id) {
            e.properties.insert("targetname".into(), name.clone());
            e.properties.extend(props);
        }
        Some(id)
    });
    let door = door.ok_or("Could not create the door entity")?;
    if trigger {
        let axis = wide_axis(&bounds);
        let thin = 2 - axis;
        let mut volume = bounds;
        volume.min[thin] -= 64.0;
        volume.max[thin] += 64.0;
        volume.min[axis] -= 8.0;
        volume.max[axis] += 8.0;
        volume.max.y = volume.min.y + bounds.size().y.min(96.0);
        let outputs =
            vec![IoConnection { output: "entered".into(), target: name.clone(), input: "open".into(), parameter: String::new(), delay: 0.0, times: -1 }];
        make_volume(state, "trigger_multiple", &volume, &[], outputs)?;
        state.doc.select(|_, s| {
            s.clear();
            s.select_node(door);
        });
    }
    state.set_status(format!("{classname} '{name}' created, drag its gizmo handles to adjust"));
    Ok(door)
}

/// Turns geometry into a lift that travels by `travel` map units.
pub fn make_platform(state: &mut EditorState, ids: &[NodeId], travel: DVec3, mode: u8) -> Result<NodeId, String> {
    let geometry = geometry_of(state, ids);
    if geometry.is_empty() {
        return Err("Select the brushes of the platform first".into());
    }
    let name = unique_name(state, "lift");
    let parent = state.insert_parent();
    state
        .doc
        .edit("Make Platform", |m, s| {
            s.clear();
            s.nodes.extend(geometry.iter().copied());
            let id = ops::create_brush_entity(m, s, "func_platform", parent)?;
            if let Some(e) = m.entity_mut(id) {
                e.properties.insert("targetname".into(), name.clone());
                e.properties.insert("travel".into(), fmt_vec3(travel));
                e.properties.insert("mode".into(), mode.to_string());
            }
            Some(id)
        })
        .ok_or_else(|| "Could not create the platform".to_string())
}

/// Turns geometry into a button whose pressed output calls `input` on `target`.
pub fn make_button(state: &mut EditorState, ids: &[NodeId], target: &str, input: &str) -> Result<NodeId, String> {
    let geometry = geometry_of(state, ids);
    if geometry.is_empty() {
        return Err("Select the brushes of the button first".into());
    }
    let name = unique_name(state, "button");
    let parent = state.insert_parent();
    let (target, input) = (target.to_string(), input.to_string());
    state
        .doc
        .edit("Make Button", |m, s| {
            s.clear();
            s.nodes.extend(geometry.iter().copied());
            let id = ops::create_brush_entity(m, s, "func_button", parent)?;
            if let Some(e) = m.entity_mut(id) {
                e.properties.insert("targetname".into(), name.clone());
                if !target.is_empty() {
                    e.outputs.push(IoConnection { output: "pressed".into(), target, input, parameter: String::new(), delay: 0.0, times: -1 });
                }
            }
            Some(id)
        })
        .ok_or_else(|| "Could not create the button".to_string())
}

/// A brush entity volume with the trigger material.
pub fn make_volume(state: &mut EditorState, classname: &str, bounds: &Aabb, props: &[(String, String)], outputs: Vec<IoConnection>) -> Result<NodeId, String> {
    let brush = Brush::from_aabb(bounds, TRIGGER_MATERIAL).map_err(|e| format!("invalid volume: {e}"))?;
    let parent = state.insert_parent();
    let classname = classname.to_string();
    let props = props.to_vec();
    let id = state.doc.edit("Create Volume", |m, s| {
        let mut e = Entity::new(classname.clone());
        e.properties.extend(props);
        e.outputs = outputs;
        let id = m.insert(parent, NodeKind::Entity(e));
        m.insert(id, NodeKind::Brush(brush));
        s.clear();
        s.select_node(id);
        id
    });
    state.last_bounds = *bounds;
    Ok(id)
}

/// Gives an entity a targetname if it has none. Returns the name.
pub fn ensure_targetname(state: &mut EditorState, id: NodeId) -> Option<String> {
    let e = state.doc.map.entity(id)?;
    if let Some(n) = e.targetname() {
        return Some(n.to_string());
    }
    let base = e.classname.split_once('_').map(|(_, rest)| rest).unwrap_or(&e.classname).to_string();
    let name = unique_name(state, &base);
    state.doc.edit("Name Entity", |m, _| {
        if let Some(e) = m.entity_mut(id) {
            e.properties.insert("targetname".into(), name.clone());
        }
    });
    Some(name)
}

/// Adds an output on `from` that calls `input` on `to`, naming `to` when needed.
pub fn link(state: &mut EditorState, from: NodeId, to: NodeId, output: &str, input: &str, parameter: &str, delay: f64) -> Result<IoConnection, String> {
    if state.doc.map.entity(from).is_none() || state.doc.map.entity(to).is_none() {
        return Err("Both ends of a link must be entities".into());
    }
    let target = ensure_targetname(state, to).ok_or("target has no name")?;
    let conn = IoConnection { output: output.into(), target, input: input.into(), parameter: parameter.into(), delay, times: -1 };
    let c = conn.clone();
    state.doc.edit("Link Entities", |m, _| {
        if let Some(e) = m.entity_mut(from) {
            e.outputs.push(c);
        }
    });
    Ok(conn)
}

/// Outputs of `from` and inputs of `to` from their definitions, for link pickers.
pub fn link_options(state: &EditorState, from: NodeId, to: NodeId) -> (Vec<String>, Vec<String>) {
    let class = |id: NodeId| state.doc.map.entity(id).and_then(|e| state.game.entity(&e.classname));
    let outputs = class(from).map(|d| d.outputs.iter().map(|o| o.name.clone()).collect()).unwrap_or_default();
    let mut inputs: Vec<String> = class(to).map(|d| d.inputs.iter().map(|i| i.name.clone()).collect()).unwrap_or_default();
    for builtin in ["kill", "show", "hide", "enable", "disable"] {
        if !inputs.iter().any(|i| i == builtin) {
            inputs.push(builtin.into());
        }
    }
    (outputs, inputs)
}

/// Point entity placed on the ground at `at`, selected.
pub fn place_entity(state: &mut EditorState, classname: &str, at: DVec3, props: &[(String, String)]) -> NodeId {
    let parent = state.insert_parent();
    let classname = classname.to_string();
    let props = props.to_vec();
    state.doc.edit("Place Entity", |m, s| {
        let id = ops::create_point_entity(m, parent, &classname, at);
        if let Some(e) = m.entity_mut(id) {
            e.properties.extend(props);
        }
        s.clear();
        s.select_node(id);
        id
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaf(state: &mut EditorState, min: [f64; 3], max: [f64; 3]) -> NodeId {
        let layer = state.doc.map.default_layer();
        state.doc.edit("leaf", |m, _| m.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::from(min), DVec3::from(max)), "wood").unwrap())))
    }

    #[test]
    fn hinged_door_with_trigger_and_button_link() {
        let mut state = EditorState::new(Default::default());
        let b = leaf(&mut state, [0.0, 0.0, -4.0], [48.0, 96.0, 4.0]);
        let door = make_door(&mut state, &[b], &DoorKind::Hinged { side: HingeSide::Right, angle: 90.0 }, true).unwrap();
        let e = state.doc.map.entity(door).unwrap();
        assert_eq!(e.classname, "func_door_rotating");
        assert_eq!(e.property("hinge"), Some("24 0 0"));
        assert_eq!(e.property("open_angle"), Some("-90"));
        let trigger = state.doc.map.entities().find(|(_, e)| e.classname == "trigger_multiple").map(|(id, _)| id).unwrap();
        let tb = state.doc.map.bounds(trigger);
        assert!(tb.min.z < -60.0 && tb.max.z > 60.0, "volume covers both sides: {tb:?}");
        assert_eq!(state.doc.map.entity(trigger).unwrap().outputs[0].target, "door_1");

        let knob = leaf(&mut state, [60.0, 40.0, -8.0], [64.0, 48.0, -4.0]);
        let button = make_button(&mut state, &[knob], "door_1", "toggle").unwrap();
        assert_eq!(state.doc.map.entity(button).unwrap().outputs[0].input, "toggle");

        let lamp = place_entity(&mut state, "light", DVec3::new(0.0, 120.0, 0.0), &[]);
        let conn = link(&mut state, door, lamp, "opened", "turn_on", "", 0.5).unwrap();
        assert_eq!(state.doc.map.entity(lamp).unwrap().targetname(), Some(conn.target.as_str()));
        assert!(gt_doc::issues::check(&state.doc.map).iter().all(|i| i.code != "io_missing_target"));
    }

    #[test]
    fn sliding_door_and_platform_travel() {
        let mut state = EditorState::new(Default::default());
        let b = leaf(&mut state, [0.0, 0.0, 0.0], [8.0, 96.0, 64.0]);
        let door = make_door(&mut state, &[b], &DoorKind::Sliding { direction: SlideDirection::Left, lip: 4.0 }, false).unwrap();
        assert_eq!(state.doc.map.entity(door).unwrap().property("travel"), Some("0 0 -60"));
        let p = leaf(&mut state, [200.0, 0.0, 0.0], [264.0, 8.0, 64.0]);
        let lift = make_platform(&mut state, &[p], DVec3::new(0.0, 160.0, 0.0), 1).unwrap();
        assert_eq!(state.doc.map.entity(lift).unwrap().property("travel"), Some("0 160 0"));
    }
}
