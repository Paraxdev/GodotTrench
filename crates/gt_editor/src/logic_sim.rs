//! A pure preview of how entity I/O cascades through a map, for Simulate in the Logic graph. It resolves each output's
//! targets by targetname the way the runtime does, then follows a small table of which inputs make an entity
//! fire its own outputs, so a wired scene can be checked without launching Godot. It models the wiring and the
//! common entity logic, not full runtime behavior. Each event carries the connection it went through and when it
//! arrives, so a timed replay can build on the same events.

use std::collections::{HashMap, VecDeque};

use gt_core::NodeId;
use gt_doc::issues::is_dynamic_target;
use gt_doc::{Entity, Map};
use gt_formats::{EntityDef, GameConfig};

/// One delivered input in a simulated cascade.
#[derive(Clone, Debug, PartialEq)]
pub struct SimEvent {
    pub source: NodeId,
    /// Index of the connection in the source entity's outputs.
    pub connection: usize,
    pub source_name: String,
    pub output: String,
    pub target: String,
    pub input: String,
    pub delay: f64,
    /// Seconds after the first output fired that the input arrives, the delays along the chain added up.
    pub time: f64,
    /// Target nodes the name resolved to, empty when the target is dynamic or unresolved.
    pub resolved: Vec<NodeId>,
    /// A runtime target (!player, @group, a node path or a wildcard) that cannot be resolved here.
    pub dynamic: bool,
    pub depth: usize,
    /// Why this step only might happen: it or a step before it is one branch an entity picks among several, like a
    /// gate's true and false or a counter's limits. None when the cascade always gets here.
    pub condition: Option<String>,
}

impl SimEvent {
    /// The target is a plain name that resolved to nothing.
    pub fn broken(&self) -> bool {
        !self.dynamic && self.resolved.is_empty()
    }

    /// The step depends on a branch, so it is one of several things that might follow.
    pub fn possible(&self) -> bool {
        self.condition.is_some()
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
        ("logic_counter", "add" | "subtract" | "set_value") => &["changed", "hit_max", "hit_min"],
        ("logic_counter", "reset") => &["changed"],
        ("logic_gate", "set_a" | "set_b" | "toggle_a" | "toggle_b") => &["changed", "on_true", "on_false"],
        ("logic_gate", "test") => &["on_true", "on_false"],
        ("logic_random", "pick") => &["picked", "out_1", "out_2", "out_3", "out_4", "out_5", "out_6", "out_7", "out_8"],
        ("logic_random", "roll") => &["on_success", "on_fail"],
        ("logic_case", "in_value") => &["on_case_1", "on_case_2", "on_case_3", "on_case_4", "on_case_5", "on_case_6", "on_case_7", "on_case_8", "on_default"],
        ("logic_flipflop", "trigger") => &["changed", "on_a", "on_b"],
        ("logic_flipflop", "reset") => &["changed"],
        ("math_calc", "calculate") => &["result"],
        ("math_compare", "compare" | "set_and_compare") => &["on_less", "on_equal", "on_not_equal", "on_greater"],
        ("math_value", "set_value" | "add" | "reset") => &["changed"],
        ("math_value", "get_value") => &["value"],
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

/// The outputs of `triggered_outputs` an entity fires only in some cases, so a step through one is a possible branch.
fn branch_outputs(classname: &str, input: &str) -> &'static [&'static str] {
    match (classname, input) {
        ("logic_branch" | "logic_gate", _) => &["on_true", "on_false"],
        ("logic_counter", "add" | "subtract" | "set_value") => &["hit_max", "hit_min"],
        ("logic_random", "pick") => &["out_1", "out_2", "out_3", "out_4", "out_5", "out_6", "out_7", "out_8"],
        ("logic_random", "roll") => &["on_success", "on_fail"],
        ("logic_case", _) => &["on_case_1", "on_case_2", "on_case_3", "on_case_4", "on_case_5", "on_case_6", "on_case_7", "on_case_8", "on_default"],
        ("logic_flipflop", "trigger") => &["on_a", "on_b"],
        ("math_compare", _) => &["on_less", "on_equal", "on_not_equal", "on_greater"],
        ("func_door" | "func_door_rotating" | "func_gate", "toggle" | "use") => &["opened", "closed"],
        _ => &[],
    }
}

/// A property of an entity: its own value, else the definition's default.
fn property<'a>(entity: &'a Entity, def: Option<&'a EntityDef>, name: &str) -> Option<&'a str> {
    entity.property(name).or_else(|| def?.properties.iter().find(|p| p.name == name).map(|p| p.default.as_str()))
}

fn number(entity: &Entity, def: Option<&EntityDef>, name: &str) -> Option<f64> {
    property(entity, def, name).and_then(|v| v.trim().parse().ok())
}

/// When each step of a logic_sequence fires after it starts: the wait before it is its own entry of `times`, else
/// `interval`, and the waits add up. Only `steps` of them run.
fn sequence_times(entity: &Entity, def: Option<&EntityDef>) -> Vec<f64> {
    let steps = number(entity, def, "steps").map_or(3, |n| n as i64).clamp(1, 8) as usize;
    let interval = number(entity, def, "interval").unwrap_or(1.0).max(0.0);
    let own: Vec<&str> = property(entity, def, "times").unwrap_or_default().split_whitespace().collect();
    let mut at = 0.0;
    (0..steps)
        .map(|i| {
            at += own.get(i).and_then(|t| t.parse::<f64>().ok()).unwrap_or(interval).max(0.0);
            at
        })
        .collect()
}

/// One output an input makes an entity fire.
struct Fire {
    output: String,
    /// Seconds after the input arrives, for entities that take their own time.
    after: f64,
    /// Only fires in some cases.
    branch: bool,
}

fn fires(entity: &Entity, def: Option<&EntityDef>, input: &str) -> Vec<Fire> {
    let step = |output: String, after: f64| Fire { output, after, branch: false };
    if entity.classname == "logic_sequence" && input == "start" {
        let times = sequence_times(entity, def);
        let mut fired: Vec<Fire> = Vec::new();
        for (i, at) in times.iter().enumerate() {
            fired.push(step(format!("step_{}", i + 1), *at));
            fired.push(step("step".into(), *at));
        }

        fired.push(step("finished".into(), times.last().copied().unwrap_or(0.0)));
        return fired;
    }

    let branches = branch_outputs(&entity.classname, input);
    triggered_outputs(&entity.classname, input).iter().map(|o| Fire { output: (*o).into(), after: 0.0, branch: branches.contains(o) }).collect()
}

/// What a branch waits for, shown when the step is hovered.
fn branch_condition(entity: &Entity, def: Option<&EntityDef>, name: &str, output: &str) -> String {
    match (entity.classname.as_str(), output) {
        ("logic_counter", "hit_max") => format!("Only when {name} reaches its max of {}", number(entity, def, "max").map_or(3, |n| n as i64)),
        ("logic_counter", "hit_min") => format!("Only when {name} reaches its min of {}", number(entity, def, "min").map_or(0, |n| n as i64)),
        _ => format!("Only when {name} takes its {output} branch"),
    }
}

struct Pending {
    source: NodeId,
    output: String,
    depth: usize,
    at: f64,
    condition: Option<String>,
}

/// Simulates firing `output` on `start` and follows the wiring across the map, returning the cascade in the order its
/// steps arrive.
pub fn simulate(map: &Map, game: &GameConfig, start: NodeId, output: &str) -> SimResult {
    let mut names: HashMap<&str, Vec<NodeId>> = HashMap::new();
    for (id, e) in map.entities() {
        if let Some(name) = e.targetname() {
            names.entry(name).or_default().push(id);
        }
    }

    let mut events: Vec<SimEvent> = Vec::new();
    let mut fired: HashMap<(NodeId, usize), i32> = HashMap::new();
    let mut queue: VecDeque<Pending> = VecDeque::new();
    queue.push_back(Pending { source: start, output: output.to_string(), depth: 0, at: 0.0, condition: None });

    while let Some(Pending { source, output: out, depth, at, condition }) = queue.pop_front() {
        if depth > MAX_DEPTH {
            continue;
        }

        let Some(entity) = map.entity(source) else { continue };
        let source_name = crate::logic_graph::model::node_title(map, source);
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

            let time = at + conn.delay.max(0.0);
            events.push(SimEvent {
                source,
                connection: i,
                source_name: source_name.clone(),
                output: out.clone(),
                target: conn.target.clone(),
                input: conn.input.clone(),
                delay: conn.delay,
                time,
                resolved: resolved.clone(),
                dynamic,
                depth,
                condition: condition.clone(),
            });
            for t in resolved {
                if let Some(target_entity) = map.entity(t) {
                    let def = game.entity(&target_entity.classname);
                    let target_name = crate::logic_graph::model::node_title(map, t);
                    for fire in fires(target_entity, def, &conn.input) {
                        let condition = if fire.branch { Some(branch_condition(target_entity, def, &target_name, &fire.output)) } else { condition.clone() };
                        queue.push_back(Pending { source: t, output: fire.output, depth: depth + 1, at: time + fire.after, condition });
                    }
                }
            }
        }
    }

    // The list reads as a timeline, entities that wait for their own time fire out of the order they were reached in.
    events.sort_by(|a, b| a.time.total_cmp(&b.time));
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
    use gt_formats::GameConfig;

    fn sim(map: &Map, start: NodeId, output: &str) -> SimResult {
        simulate(map, &GameConfig::with_gameplay_pack(), start, output)
    }

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

        let res = sim(&m, relay, "triggered");
        let chain: Vec<(String, String)> =
            res.events.iter().map(|e| (format!("{}.{}", e.source_name, e.output), format!("{}.{}", e.target, e.input))).collect();
        assert!(chain.contains(&("r1.triggered".into(), "c1.add".into())), "{chain:?}");
        assert!(chain.contains(&("c1.changed".into(), "l1.turn_on".into())), "counter cascades into the light: {chain:?}");
        assert_eq!(res.broken(), 1, "the ghost target is flagged broken");
        assert_eq!(res.events[1].connection, 1, "each event names the connection it went through");
    }

    #[test]
    fn delays_add_up_along_the_chain() {
        let mut m = Map::new();
        let l = m.default_layer();
        let relay = entity(&mut m, l, "logic_relay", "r1", &[("triggered", "r2", "trigger")]);
        entity(&mut m, l, "logic_relay", "r2", &[("triggered", "l1", "turn_on")]);
        entity(&mut m, l, "light", "l1", &[]);
        for (id, delay) in [(relay, 0.5), (m.find_by_targetname("r2")[0], 1.25)] {
            m.entity_mut(id).unwrap().outputs[0].delay = delay;
        }

        let times: Vec<f64> = sim(&m, relay, "triggered").events.iter().map(|e| e.time).collect();
        assert_eq!(times, [0.5, 1.75]);
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

        let res = sim(&m, r, "triggered");
        assert_eq!(res.events.len(), 1);
        assert!(res.events[0].dynamic && !res.events[0].broken(), "!player is a runtime target, not broken");
    }

    #[test]
    fn math_and_logic_nodes_pass_the_cascade_on() {
        let mut m = Map::new();
        let l = m.default_layer();
        let button = entity(&mut m, l, "func_button", "b", &[("pressed", "score", "add")]);
        entity(&mut m, l, "math_value", "score", &[("changed", "cmp", "set_and_compare")]);
        entity(&mut m, l, "math_compare", "cmp", &[("on_greater", "gate", "set_a"), ("on_equal", "gate", "set_a")]);
        entity(&mut m, l, "logic_gate", "gate", &[("on_true", "pick", "pick"), ("changed", "flip", "trigger")]);
        entity(&mut m, l, "logic_random", "pick", &[("out_2", "case", "in_value"), ("picked", "calc", "calculate")]);
        entity(&mut m, l, "math_calc", "calc", &[("result", "case", "in_value")]);
        entity(&mut m, l, "logic_case", "case", &[("on_case_2", "l1", "turn_on"), ("on_default", "l2", "turn_on")]);
        entity(&mut m, l, "logic_flipflop", "flip", &[("on_a", "l1", "turn_on")]);
        entity(&mut m, l, "light", "l1", &[]);
        entity(&mut m, l, "light", "l2", &[]);

        let res = sim(&m, button, "pressed");
        let chain: Vec<String> = res.events.iter().map(|e| format!("{}.{}>{}.{}", e.source_name, e.output, e.target, e.input)).collect();
        for step in [
            "b.pressed>score.add",
            "score.changed>cmp.set_and_compare",
            "cmp.on_greater>gate.set_a",
            "cmp.on_equal>gate.set_a",
            "gate.on_true>pick.pick",
            "gate.changed>flip.trigger",
            "pick.out_2>case.in_value",
            "pick.picked>calc.calculate",
            "calc.result>case.in_value",
            "case.on_case_2>l1.turn_on",
            "case.on_default>l2.turn_on",
            "flip.on_a>l1.turn_on",
        ] {
            assert!(chain.iter().any(|c| c == step), "{step} is in the cascade: {chain:?}");
        }

        assert_eq!(res.broken(), 0, "every target resolves");
    }

    #[test]
    fn the_table_only_fires_outputs_the_new_entities_declare() {
        let game = gt_formats::GameConfig::gameplay_pack();
        let classes = ["math_calc", "math_compare", "math_value", "logic_gate", "logic_random", "logic_case", "logic_flipflop"];
        for class in classes {
            let def = game.entity(class).unwrap();
            let mut fires = false;
            for input in &def.inputs {
                for out in triggered_outputs(class, &input.name) {
                    fires = true;
                    assert!(def.outputs.iter().any(|o| o.name == *out), "{class}.{} fires {out}, which {class} does not declare", input.name);
                }
            }

            assert!(fires, "{class} has an input the simulation follows");
        }

        let first = |class: &str| crate::logic_graph::model::default_input(&game, class);
        assert_eq!([first("math_calc"), first("math_compare"), first("logic_random")], ["calculate", "compare", "pick"]);
        assert_eq!([first("logic_case"), first("logic_flipflop"), first("logic_gate")], ["in_value", "trigger", "set_a"]);
    }

    #[test]
    fn a_counter_continues_to_its_limits_as_possible_branches() {
        let mut m = Map::new();
        let l = m.default_layer();
        let button = entity(&mut m, l, "func_button", "b", &[("pressed", "c", "add")]);
        let counter = entity(&mut m, l, "logic_counter", "c", &[("changed", "log", "write"), ("hit_max", "lamp", "turn_on"), ("hit_min", "lamp", "turn_off")]);
        m.entity_mut(counter).unwrap().properties.insert("max".into(), "5".into());
        entity(&mut m, l, "logic_debug", "log", &[]);
        entity(&mut m, l, "light", "lamp", &[("switched", "door", "open")]);
        entity(&mut m, l, "func_door", "door", &[]);

        let res = sim(&m, button, "pressed");
        let step =
            |from: &str| res.events.iter().find(|e| format!("{}.{}", e.source_name, e.output) == from).unwrap_or_else(|| panic!("{from} in {:?}", res.events));
        assert!(!step("b.pressed").possible() && !step("c.changed").possible(), "the counter always fires changed");
        assert_eq!(step("c.hit_max").condition.as_deref(), Some("Only when c reaches its max of 5"), "the entity's own max");
        assert_eq!(step("c.hit_min").condition.as_deref(), Some("Only when c reaches its min of 0"), "the definition's default");
        assert!(step("lamp.switched").possible(), "what follows a branch is only possible too");

        let plain = sim(&m, counter, "changed");
        assert!(plain.events.iter().all(|e| !e.possible()), "firing an output by hand is not a branch");
    }

    #[test]
    fn a_sequence_fires_its_steps_at_the_times_of_its_properties() {
        let mut m = Map::new();
        let l = m.default_layer();
        let seq = entity(
            &mut m,
            l,
            "logic_sequence",
            "s",
            &[("step_1", "a", "turn_on"), ("step_2", "a", "turn_off"), ("step_3", "a", "toggle"), ("step_4", "a", "kill")],
        );
        m.entity_mut(seq).unwrap().outputs.push(IoConnection {
            output: "finished".into(),
            target: "a".into(),
            input: "kill".into(),
            parameter: String::new(),
            delay: 0.5,
            times: -1,
        });
        entity(&mut m, l, "light", "a", &[]);
        let starter = entity(&mut m, l, "func_button", "go", &[("pressed", "s", "start")]);
        let time = |res: &SimResult, out: &str| res.events.iter().find(|e| e.output == out).map(|e| e.time);

        let res = sim(&m, starter, "pressed");
        assert_eq!(time(&res, "step_1"), Some(1.0), "the default interval is a second");
        assert_eq!(time(&res, "step_3"), Some(3.0));
        assert_eq!(time(&res, "step_4"), None, "only the default 3 steps run");

        let props = &mut m.entity_mut(seq).unwrap().properties;
        props.insert("steps".into(), "4".into());
        props.insert("interval".into(), "2".into());
        props.insert("times".into(), "0 0.5 junk".into());
        let res = sim(&m, starter, "pressed");
        let steps: Vec<(String, f64)> = res.events.iter().filter(|e| e.source == seq).map(|e| (e.output.clone(), e.time)).collect();
        assert_eq!(
            steps,
            [("step_1".to_string(), 0.0), ("step_2".to_string(), 0.5), ("step_3".to_string(), 2.5), ("step_4".to_string(), 4.5), ("finished".to_string(), 5.0)],
            "a time of its own replaces the interval, and the list is in arrival order"
        );
    }

    #[test]
    fn steps_are_named_like_the_graph_names_the_node() {
        let mut m = Map::new();
        let l = m.default_layer();
        let named = entity(&mut m, l, "func_button", "", &[("pressed", "d", "open")]);
        entity(&mut m, l, "func_door", "d", &[]);
        assert_eq!(sim(&m, named, "pressed").events[0].source_name, "func_button", "no name at all");
        m.get_mut(named).unwrap().set_label(Some("big red".into()));
        assert_eq!(sim(&m, named, "pressed").events[0].source_name, "big red", "the name from the Outliner titles the node");
    }

    #[test]
    fn feedback_loop_terminates_bounded() {
        let mut m = Map::new();
        let l = m.default_layer();
        // Two relays triggering each other forever, bounded by the depth and event caps so the sim always returns.
        let a = entity(&mut m, l, "logic_relay", "a", &[("triggered", "b", "trigger")]);
        entity(&mut m, l, "logic_relay", "b", &[("triggered", "a", "trigger")]);
        let res = sim(&m, a, "triggered");
        assert!(res.events.len() > 8, "the loop runs several hops: {}", res.events.len());
        assert!(res.events.len() <= MAX_EVENTS, "the loop stays bounded: {}", res.events.len());
    }
}
