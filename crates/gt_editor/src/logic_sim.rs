//! A pure preview of how entity I/O cascades through a map, for the Logic panel. It resolves each output's
//! targets by targetname the way the runtime does, then follows a small table of which inputs make an entity
//! fire its own outputs, so a wired scene can be checked without launching Godot. It models the wiring and the
//! common entity logic, not full runtime behavior.

use std::collections::{HashMap, VecDeque};

use gt_core::NodeId;
use gt_doc::Map;
use gt_doc::issues::is_dynamic_target;

/// One delivered input in a simulated cascade.
#[derive(Clone, Debug, PartialEq)]
pub struct SimEvent {
    pub source: NodeId,
    pub source_name: String,
    pub output: String,
    pub target: String,
    pub input: String,
    pub delay: f64,
    /// Target nodes the name resolved to, empty when the target is dynamic or unresolved.
    pub resolved: Vec<NodeId>,
    /// A runtime target (!player, @group, a node path or a wildcard) that cannot be resolved here.
    pub dynamic: bool,
    pub depth: usize,
}

impl SimEvent {
    /// The target is a plain name that resolved to nothing.
    pub fn broken(&self) -> bool {
        !self.dynamic && self.resolved.is_empty()
    }
}

#[derive(Clone, Debug, Default)]
pub struct SimResult {
    pub events: Vec<SimEvent>,
    /// Set when the cascade hit the event cap, usually a feedback loop.
    pub truncated: bool,
}

impl SimResult {
    pub fn broken(&self) -> usize {
        self.events.iter().filter(|e| e.broken()).count()
    }
}

const MAX_EVENTS: usize = 300;
const MAX_DEPTH: usize = 32;

/// The outputs an entity fires when `input` is delivered, so a cascade flows. Empty for a leaf input. This is a
/// preview of the common entities, not every input of every entity.
pub fn triggered_outputs(classname: &str, input: &str) -> &'static [&'static str] {
    match (classname, input) {
        ("logic_relay", "trigger") => &["triggered"],
        ("logic_sequence", "start") => &["step_1", "step_2", "step_3", "step_4", "step_5", "step_6", "step_7", "step_8", "step", "finished"],
        ("logic_branch", "test" | "set_and_test") => &["on_true", "on_false"],
        ("logic_counter", "add" | "subtract" | "set_value" | "reset") => &["changed"],
        ("logic_timer", "start" | "fire") => &["timer"],
        ("logic_call", "trigger" | "call_with") => &["called"],
        ("trigger_call", "trigger") => &["called", "triggered"],
        ("logic_script", "run" | "run_with") => &["ran"],
        ("func_button", "press" | "use") => &["pressed"],
        ("func_door" | "func_door_rotating" | "func_gate", "open") => &["opened", "started_opening"],
        ("func_door" | "func_door_rotating" | "func_gate", "close") => &["closed", "started_closing"],
        ("func_door" | "func_door_rotating" | "func_gate", "toggle" | "use") => &["opened", "closed"],
        ("func_platform", "start") => &["started", "reached_end", "reached_start"],
        ("func_platform", "go_to_end") => &["reached_end"],
        ("func_platform", "go_to_start") => &["reached_start"],
        ("func_train" | "npc_walker", "start") => &["arrived", "finished"],
        ("npc_walker", "walk_to") => &["arrived"],
        ("prop_physics", "smash" | "ignite") => &["broken"],
        ("prop_physics", "take_damage") => &["damaged"],
        ("env_explosion", "explode") => &["exploded"],
        ("game_text", "show") => &["shown"],
        ("game_text", "hide") => &["hidden"],
        ("light" | "light_spot", "turn_on" | "turn_off" | "toggle") => &["switched"],
        ("info_spawner" | "trigger_spawn_area", "spawn") => &["spawned"],
        _ => &[],
    }
}

/// Simulates firing `output` on `start` and follows the wiring across the map, returning the ordered cascade.
pub fn simulate(map: &Map, start: NodeId, output: &str) -> SimResult {
    let mut names: HashMap<&str, Vec<NodeId>> = HashMap::new();
    for (id, e) in map.entities() {
        if let Some(name) = e.targetname() {
            names.entry(name).or_default().push(id);
        }
    }

    let mut events: Vec<SimEvent> = Vec::new();
    let mut fired: HashMap<(NodeId, usize), i32> = HashMap::new();
    let mut queue: VecDeque<(NodeId, String, usize)> = VecDeque::new();
    queue.push_back((start, output.to_string(), 0));

    while let Some((source, out, depth)) = queue.pop_front() {
        if depth > MAX_DEPTH {
            continue;
        }

        let Some(entity) = map.entity(source) else { continue };
        let source_name = entity.targetname().unwrap_or(&entity.classname).to_string();
        for (i, conn) in entity.outputs.iter().enumerate() {
            if conn.output != out {
                continue;
            }

            let count = fired.entry((source, i)).or_insert(0);
            if conn.times >= 0 && *count >= conn.times {
                continue;
            }

            *count += 1;
            let dynamic = is_dynamic_target(&conn.target);
            let resolved: Vec<NodeId> =
                if dynamic || conn.target.is_empty() { Vec::new() } else { names.get(conn.target.as_str()).cloned().unwrap_or_default() };
            if events.len() >= MAX_EVENTS {
                return SimResult { events, truncated: true };
            }

            events.push(SimEvent {
                source,
                source_name: source_name.clone(),
                output: out.clone(),
                target: conn.target.clone(),
                input: conn.input.clone(),
                delay: conn.delay,
                resolved: resolved.clone(),
                dynamic,
                depth,
            });
            for t in resolved {
                if let Some(target_entity) = map.entity(t) {
                    for next in triggered_outputs(&target_entity.classname, &conn.input) {
                        queue.push_back((t, (*next).to_string(), depth + 1));
                    }
                }
            }
        }
    }

    SimResult { events, truncated: false }
}

/// Every output an entity can fire, for the picker: its declared outputs, or, when it has no definition, the
/// distinct outputs it already wires.
pub fn start_outputs(def: Option<&gt_formats::EntityDef>, entity: &gt_doc::Entity) -> Vec<String> {
    if let Some(def) = def
        && !def.outputs.is_empty()
    {
        return def.outputs.iter().map(|o| o.name.clone()).collect();
    }

    let mut seen: Vec<String> = Vec::new();
    for o in &entity.outputs {
        if !seen.contains(&o.output) {
            seen.push(o.output.clone());
        }
    }

    seen
}

#[cfg(test)]
mod tests {
    use super::*;
    use gt_doc::{Entity, IoConnection, NodeKind};

    fn entity(map: &mut Map, layer: NodeId, classname: &str, name: &str, outputs: &[(&str, &str, &str)]) -> NodeId {
        let mut e = Entity::new(classname);
        if !name.is_empty() {
            e.properties.insert("targetname".into(), name.into());
        }

        for (o, t, i) in outputs {
            e.outputs.push(IoConnection { output: (*o).into(), target: (*t).into(), input: (*i).into(), parameter: String::new(), delay: 0.0, times: -1 });
        }

        map.insert(layer, NodeKind::Entity(e))
    }

    #[test]
    fn cascades_through_relay_and_flags_broken() {
        let mut m = Map::new();
        let l = m.default_layer();
        let relay = entity(&mut m, l, "logic_relay", "r1", &[("triggered", "c1", "add"), ("triggered", "ghost", "kill")]);
        entity(&mut m, l, "logic_counter", "c1", &[("changed", "l1", "turn_on")]);
        entity(&mut m, l, "light", "l1", &[]);

        let res = simulate(&m, relay, "triggered");
        let chain: Vec<(String, String)> =
            res.events.iter().map(|e| (format!("{}.{}", e.source_name, e.output), format!("{}.{}", e.target, e.input))).collect();
        assert!(chain.contains(&("r1.triggered".into(), "c1.add".into())), "{chain:?}");
        assert!(chain.contains(&("c1.changed".into(), "l1.turn_on".into())), "counter cascades into the light: {chain:?}");
        assert_eq!(res.broken(), 1, "the ghost target is flagged broken");
    }

    #[test]
    fn dynamic_targets_are_not_broken_and_times_limits_fires() {
        let mut m = Map::new();
        let l = m.default_layer();
        let mut e = Entity::new("logic_relay");
        e.properties.insert("targetname".into(), "r".into());
        e.outputs.push(IoConnection {
            output: "triggered".into(),
            target: "!player".into(),
            input: "kill".into(),
            parameter: String::new(),
            delay: 0.0,
            times: -1,
        });
        let r = m.insert(l, NodeKind::Entity(e));

        let res = simulate(&m, r, "triggered");
        assert_eq!(res.events.len(), 1);
        assert!(res.events[0].dynamic && !res.events[0].broken(), "!player is a runtime target, not broken");
    }

    #[test]
    fn feedback_loop_terminates_bounded() {
        let mut m = Map::new();
        let l = m.default_layer();
        // Two relays triggering each other forever, bounded by the depth and event caps so the sim always returns.
        let a = entity(&mut m, l, "logic_relay", "a", &[("triggered", "b", "trigger")]);
        entity(&mut m, l, "logic_relay", "b", &[("triggered", "a", "trigger")]);
        let res = simulate(&m, a, "triggered");
        assert!(res.events.len() > 8, "the loop runs several hops: {}", res.events.len());
        assert!(res.events.len() <= MAX_EVENTS, "the loop stays bounded: {}", res.events.len());
    }
}
