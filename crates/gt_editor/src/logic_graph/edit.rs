//! The changes the graph makes to the map. Each one is a single undo step, and the wiring only ever lives in the
//! entities' outputs.

use std::collections::BTreeMap;

use egui::Pos2;
use gt_core::{DVec3, NodeId};
use gt_doc::map::GraphFrame;
use gt_doc::{IoConnection, Map, NodeKind, ops};

use super::model::{ConnectionId, NodeKey};
use crate::entity_wizards::{name_base, unique_name_in};
use crate::state::EditorState;

/// The layer new logic entities go into.
pub const LAYER: &str = "Logic";

/// The text an output targets to reach `to`, naming an entity that has no targetname yet inside `m`.
fn target_of(m: &mut Map, to: &NodeKey) -> Option<String> {
    match to {
        NodeKey::Entity(id) => {
            let e = m.entity(*id)?;
            if let Some(name) = e.targetname() {
                return Some(name.to_string());
            }

            let name = unique_name_in(m, name_base(&e.classname));
            m.entity_mut(*id)?.properties.insert("targetname".into(), name.clone());
            Some(name)
        }
        other => other.target(m).map(str::to_string),
    }
}

fn io(output: &str, target: String, input: &str) -> IoConnection {
    IoConnection { output: output.into(), target, input: input.into(), parameter: String::new(), delay: 0.0, times: -1 }
}

/// Stores the positions of nodes that have none yet.
fn pin_positions(m: &mut Map, pin: &[(NodeId, Pos2)]) {
    for (id, p) in pin {
        if let Some(n) = m.get_mut(*id).filter(|n| n.graph.is_none()) {
            n.set_graph(Some([p.x, p.y]));
        }
    }
}

/// Adds an output on `from` that calls `input` on `to`, and names `to` when it has no targetname. `pin` stores graph
/// positions along with it, so the picture holds still instead of being laid out again.
pub fn connect(state: &mut EditorState, from: NodeId, output: &str, to: &NodeKey, input: &str, pin: &[(NodeId, Pos2)]) -> Result<IoConnection, String> {
    let map = &state.doc.map;
    let source = map.entity(from).ok_or("The source is no longer an entity")?;
    let unnamed = match to {
        NodeKey::Entity(id) => map.entity(*id).ok_or("The target is no longer an entity")?.targetname().is_none(),
        _ => false,
    };
    if let Some(target) = to.target(map)
        && source.outputs.iter().any(|c| c.output == output && c.target == target && c.input == input)
    {
        return Err(format!("{output} already calls {input} on {target}"));
    }

    let conn = state.doc.edit(&format!("Connect {output} to {input}"), |m, _| {
        let conn = io(output, target_of(m, to)?, input);
        m.entity_mut(from)?.outputs.push(conn.clone());
        pin_positions(m, pin);
        Some(conn)
    });
    let conn = conn.ok_or("Could not connect")?;
    let named = if unnamed { format!(", the target is now named {}", conn.target) } else { String::new() };
    state.set_status(format!("Connected {output} to {}.{input}{named}", conn.target));
    Ok(conn)
}

/// Deletes output connections in one step. Returns how many were deleted.
pub fn disconnect(state: &mut EditorState, ids: &[ConnectionId]) -> usize {
    let mut by_entity: BTreeMap<NodeId, Vec<usize>> = BTreeMap::new();
    for (id, i) in ids {
        if state.doc.map.entity(*id).is_some_and(|e| *i < e.outputs.len()) {
            by_entity.entry(*id).or_default().push(*i);
        }
    }

    let count: usize = by_entity
        .values_mut()
        .map(|v| {
            v.sort_unstable();
            v.dedup();
            v.len()
        })
        .sum();
    if count == 0 {
        return 0;
    }

    let label = if count == 1 { "Delete Connection".to_string() } else { format!("Delete {count} Connections") };
    state.doc.edit(&label, |m, _| {
        for (id, indices) in &by_entity {
            if let Some(e) = m.entity_mut(*id) {
                for i in indices.iter().rev() {
                    e.outputs.remove(*i);
                }
            }
        }
    });
    count
}

/// Replaces one connection, merged with the next edit of the same kind like typing in a field.
pub fn update(state: &mut EditorState, (id, i): ConnectionId, conn: IoConnection) {
    if state.doc.map.entity(id).and_then(|e| e.outputs.get(i)) == Some(&conn) {
        return;
    }

    state.doc.edit_coalesced("Edit Connection", |m, _| {
        if let Some(slot) = m.entity_mut(id).and_then(|e| e.outputs.get_mut(i)) {
            *slot = conn;
        }
    });
}

/// Stores graph positions, `None` giving a node back to the automatic layout.
pub fn set_positions(state: &mut EditorState, label: &str, positions: &[(NodeId, Option<Pos2>)]) {
    let changed: Vec<(NodeId, Option<[f32; 2]>)> = positions
        .iter()
        .map(|(id, p)| (*id, p.map(|p| [p.x, p.y])))
        .filter(|(id, p)| state.doc.map.get(*id).is_some_and(|n| n.graph != p.map(|p| p.map(f32::round))))
        .collect();
    if changed.is_empty() {
        return;
    }

    state.doc.edit(label, |m, _| {
        for (id, p) in changed {
            if let Some(n) = m.get_mut(id) {
                n.set_graph(p);
            }
        }
    });
}

/// Changes the Logic graph's frames in one step. `moved` gives nodes new positions along with it, as moving a frame
/// moves what is inside, and `pin` stores positions of nodes that have none yet.
pub fn edit_frames(state: &mut EditorState, label: &str, moved: &[(NodeId, Pos2)], pin: &[(NodeId, Pos2)], f: impl FnOnce(&mut Vec<GraphFrame>)) {
    state.doc.edit(label, |m, _| {
        f(&mut m.editor.graph_frames);
        for (id, p) in moved {
            if let Some(n) = m.get_mut(*id) {
                n.set_graph(Some([p.x, p.y]));
            }
        }

        pin_positions(m, pin);
    });
}

/// Which way a new node is wired to an existing one.
#[derive(Clone, Debug, PartialEq)]
pub enum NewLink {
    /// `output` on the existing entity calls the new entity.
    From { source: NodeId, output: String },
    /// The new entity calls `input` on the existing node.
    To { target: NodeKey, input: String },
}

/// Where a new entity goes, in the map and in the graph.
pub struct Placement<'a> {
    /// Graph position of the new node.
    pub at: Pos2,
    /// The entity it lands near in the map.
    pub near: Option<NodeId>,
    /// Map position when there is nothing to land near.
    pub fallback: DVec3,
    /// Positions stored along with it, so nodes of the automatic layout stay where they are once one node has its own.
    pub pin: &'a [(NodeId, Pos2)],
}

/// Places a point entity for the graph on the Logic layer, wires it and selects it.
pub fn create(state: &mut EditorState, classname: &str, place: Placement, link: Option<NewLink>) -> Result<NodeId, String> {
    let Placement { at, near, fallback, pin } = place;
    let game = &state.game;
    let input = super::model::default_input(game, classname);
    let output = super::model::default_output(game, classname);
    let map = &state.doc.map;
    let anchor = near.filter(|id| map.contains(*id)).map(|id| map.bounds(id).center() + DVec3::new(0.0, 32.0, 0.0));
    let mut origin = state.snap(anchor.unwrap_or(fallback));
    let step = state.grid.max(8.0) * 2.0;
    for _ in 0..64 {
        if !map.entities().any(|(id, e)| map.is_point_entity(id) && (e.origin - origin).length() < 8.0) {
            break;
        }

        origin.x += step;
    }

    let classname = classname.to_string();
    let id = state.doc.edit(&format!("Add {classname}"), |m, s| {
        let named = |l: &NodeId| m.get(*l).is_some_and(|n| matches!(&n.kind, NodeKind::Layer(layer) if layer.name == LAYER));
        let layer = match m.layers.iter().copied().find(named) {
            Some(layer) => layer,
            None => m.add_layer(LAYER),
        };
        let name = unique_name_in(m, name_base(&classname));
        let id = ops::create_point_entity(m, layer, &classname, origin);
        m.entity_mut(id)?.properties.insert("targetname".into(), name.clone());
        m.get_mut(id)?.set_graph(Some([at.x, at.y]));
        pin_positions(m, pin);

        match &link {
            Some(NewLink::From { source, output: out }) => m.entity_mut(*source)?.outputs.push(io(out, name, &input)),
            Some(NewLink::To { target, input: into }) => {
                let target = target_of(m, target)?;
                m.entity_mut(id)?.outputs.push(io(&output, target, into));
            }
            None => {}
        }

        s.clear();
        s.select_node(id);
        Some(id)
    });
    id.ok_or_else(|| "Could not add the entity".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic_graph::model::{GraphModel, Inputs};
    use crate::state::Prefs;

    fn state() -> EditorState {
        let mut state = EditorState::new(Prefs::default());
        state.game = gt_formats::GameConfig::with_gameplay_pack();
        state
    }

    fn add(state: &mut EditorState, classname: &str, name: &str) -> NodeId {
        let layer = state.doc.map.default_layer();
        state.doc.edit("add", |m, _| {
            let id = ops::create_point_entity(m, layer, classname, DVec3::ZERO);
            if !name.is_empty() {
                m.entity_mut(id).unwrap().properties.insert("targetname".into(), name.into());
            }

            id
        })
    }

    #[test]
    fn connecting_names_the_target_in_the_same_step_and_deleting_undoes_it() {
        let mut state = state();
        let button = add(&mut state, "func_button", "b");
        let taken = add(&mut state, "func_door", "door_1");
        let door = add(&mut state, "func_door", "");
        let steps = state.doc.history.undo_labels().count();

        let conn = connect(&mut state, button, "pressed", &NodeKey::Entity(door), "open", &[]).unwrap();
        assert_eq!(conn.target, "door_2", "a readable name no other entity has");
        assert_eq!(state.doc.map.entity(door).unwrap().targetname(), Some("door_2"));
        assert_eq!(state.doc.map.entity(button).unwrap().outputs, vec![conn.clone()]);
        assert_eq!(state.doc.history.undo_labels().count(), steps + 1, "naming and wiring are one undo step");
        assert_eq!(state.doc.history.undo_labels().next(), Some("Connect pressed to open"));
        assert!(connect(&mut state, button, "pressed", &NodeKey::Entity(door), "open", &[]).is_err(), "the same connection twice is refused");

        let game = state.game.clone();
        let g = GraphModel::build(&state.doc.map, &Inputs { game: &game, selected: &Default::default(), external: &Default::default() });
        assert_eq!(g.edges.len(), 1);
        assert_eq!(g.nodes[g.edges[0].to].key, NodeKey::Entity(door));
        assert!(g.node(&NodeKey::Entity(taken)).is_none());

        assert_eq!(disconnect(&mut state, &[g.edges[0].id()]), 1);
        assert!(state.doc.map.entity(button).unwrap().outputs.is_empty());
        state.undo();
        assert_eq!(state.doc.map.entity(button).unwrap().outputs, vec![conn]);
        state.undo();
        assert_eq!(state.doc.map.entity(door).unwrap().targetname(), None, "one undo takes the name back with the wire");
    }

    #[test]
    fn connecting_to_a_marker_uses_its_text() {
        let mut state = state();
        let relay = add(&mut state, "logic_relay", "r");
        let conn = connect(&mut state, relay, "triggered", &NodeKey::Dynamic("!player".into()), "kill", &[]).unwrap();
        assert_eq!(conn.target, "!player");
    }

    #[test]
    fn editing_a_connection_is_one_step() {
        let mut state = state();
        let relay = add(&mut state, "logic_relay", "r");
        let conn = connect(&mut state, relay, "triggered", &NodeKey::Missing("ghost".into()), "open", &[]).unwrap();
        let steps = state.doc.history.undo_labels().count();
        update(&mut state, (relay, 0), IoConnection { delay: 1.5, ..conn.clone() });
        update(&mut state, (relay, 0), IoConnection { delay: 2.0, ..conn });
        assert_eq!(state.doc.map.entity(relay).unwrap().outputs[0].delay, 2.0);
        assert_eq!(state.doc.history.undo_labels().count(), steps + 1);
    }

    #[test]
    fn a_new_logic_entity_lands_on_the_logic_layer_near_its_partner() {
        let mut state = state();
        let button = add(&mut state, "func_button", "b");
        state.doc.edit("move", |m, _| m.entity_mut(button).unwrap().origin = DVec3::new(64.0, 0.0, 0.0));
        let link = NewLink::From { source: button, output: "pressed".into() };
        let place = Placement { at: egui::pos2(300.0, 40.0), near: Some(button), fallback: DVec3::ZERO, pin: &[(button, egui::pos2(10.0, 10.0))] };
        let relay = create(&mut state, "logic_relay", place, Some(link)).unwrap();

        let map = &state.doc.map;
        let node = map.get(relay).unwrap();
        assert_eq!(node.graph, Some([300.0, 40.0]));
        assert_eq!(map.get(button).unwrap().graph, Some([10.0, 10.0]), "the rest of the layout is kept in the same step");
        let layer = map.layer_of(relay);
        assert!(matches!(&map.get(layer).unwrap().kind, NodeKind::Layer(l) if l.name == LAYER));
        let e = map.entity(relay).unwrap();
        assert_eq!(e.targetname(), Some("relay_1"));
        assert!((e.origin - DVec3::new(64.0, 32.0, 0.0)).length() < 1e-9, "{:?}", e.origin);
        assert_eq!(map.entity(button).unwrap().outputs, vec![io("pressed", "relay_1".into(), "trigger")]);
        assert_eq!(state.doc.selection.nodes.iter().copied().collect::<Vec<_>>(), [relay]);

        let before = state.doc.map.layers.len();
        let place = Placement { at: egui::pos2(0.0, 0.0), near: Some(button), fallback: DVec3::ZERO, pin: &[] };
        let timer = create(&mut state, "logic_timer", place, Some(NewLink::To { target: NodeKey::Entity(relay), input: "trigger".into() })).unwrap();
        assert_eq!(state.doc.map.layers.len(), before, "the Logic layer is reused");
        assert_eq!(state.doc.map.entity(timer).unwrap().outputs, vec![io("timer", "relay_1".into(), "trigger")]);
        assert_ne!(state.doc.map.entity(timer).unwrap().origin, state.doc.map.entity(relay).unwrap().origin, "not stacked on the first");
        state.undo();
        assert!(!state.doc.map.contains(timer));
    }

    #[test]
    fn moving_a_frame_moves_its_nodes_in_one_step() {
        let mut state = state();
        let relay = add(&mut state, "logic_relay", "r");
        let frame = GraphFrame { title: "Doors".into(), rect: [0.0, 0.0, 200.0, 100.0], color: gt_core::Color::rgb(0.2, 0.4, 0.8) };
        edit_frames(&mut state, "Add Frame", &[], &[(relay, egui::pos2(10.0, 30.0))], |frames| frames.push(frame.clone()));
        assert_eq!(state.doc.map.get(relay).unwrap().graph, Some([10.0, 30.0]), "the automatic layout is kept once a frame exists");
        let steps = state.doc.history.undo_labels().count();
        edit_frames(&mut state, "Move Frame", &[(relay, egui::pos2(60.0, 30.0))], &[], |frames| frames[0].rect[0] += 50.0);
        assert_eq!(state.doc.map.editor.graph_frames[0].rect[0], 50.0);
        assert_eq!(state.doc.map.get(relay).unwrap().graph, Some([60.0, 30.0]));
        assert_eq!(state.doc.history.undo_labels().count(), steps + 1);
        state.undo();
        assert_eq!((state.doc.map.editor.graph_frames[0].rect[0], state.doc.map.get(relay).unwrap().graph), (0.0, Some([10.0, 30.0])));
    }

    #[test]
    fn positions_are_one_step_and_skip_unchanged_nodes() {
        let mut state = state();
        let relay = add(&mut state, "logic_relay", "r");
        let steps = state.doc.history.undo_labels().count();
        set_positions(&mut state, "Move", &[(relay, Some(egui::pos2(10.2, 20.0)))]);
        set_positions(&mut state, "Move", &[(relay, Some(egui::pos2(10.0, 20.0)))]);
        assert_eq!(state.doc.history.undo_labels().count(), steps + 1);
        assert_eq!(state.doc.map.get(relay).unwrap().graph, Some([10.0, 20.0]));
        set_positions(&mut state, "Arrange", &[(relay, None)]);
        assert_eq!(state.doc.map.get(relay).unwrap().graph, None);
    }
}
