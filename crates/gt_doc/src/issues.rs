//! Map problems. The game configuration is optional: [`check_with`] also checks I/O names against entity definitions.

use std::collections::{BTreeMap, BTreeSet};

use gt_core::NodeId;
use serde::Serialize;

use crate::map::{Map, NodeKind};

/// Outputs and inputs an entity class declares. Empty lists mean the class does not describe that side of its I/O.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClassIo<'a> {
    pub outputs: Vec<&'a str>,
    pub inputs: Vec<&'a str>,
}

/// Inputs the Godot runtime handles on any node, see `godottrench_io.gd`.
pub const BUILTIN_INPUTS: [&str; 6] = ["kill", "show", "hide", "enable", "disable", "toggle"];

/// The runtime also finds `open` as `Open` or `open` as a C# or GDScript method, so names compare loosely.
fn io_name_matches(a: &str, b: &str) -> bool {
    let norm = |s: &str| s.chars().filter(|c| *c != '_').flat_map(char::to_lowercase).collect::<String>();
    norm(a) == norm(b)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Issue {
    pub node: Option<NodeId>,
    pub severity: Severity,
    pub code: &'static str,
    pub message: String,
}

pub fn check(map: &Map) -> Vec<Issue> {
    check_with(map, |_| None)
}

/// [`check`], plus warnings for outputs the source class does not declare and inputs the target class does not
/// declare. `classes` gives the declared I/O of a classname, `None` for classes without a definition, which are
/// skipped like dynamic targets.
pub fn check_with<'a>(map: &Map, classes: impl Fn(&str) -> Option<ClassIo<'a>>) -> Vec<Issue> {
    check_with_external(map, classes, &BTreeSet::new())
}

/// [`check_with`] where `external` holds targetnames that exist outside the map, like the nodes of a Godot overlay
/// built on top of it, so outputs and targets naming them are not reported as missing.
pub fn check_with_external<'a>(map: &Map, classes: impl Fn(&str) -> Option<ClassIo<'a>>, external: &BTreeSet<String>) -> Vec<Issue> {
    let mut out = check_map(map, external);
    check_io_names(map, &classes, &mut out);
    out.sort_by_key(|i| std::cmp::Reverse(i.severity));
    out
}

fn check_io_names<'a>(map: &Map, classes: &impl Fn(&str) -> Option<ClassIo<'a>>, out: &mut Vec<Issue>) {
    let mut cache: BTreeMap<&str, Option<ClassIo<'a>>> = BTreeMap::new();
    let mut by_name: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for (_, e) in map.entities() {
        if let Some(n) = e.targetname() {
            by_name.entry(n).or_default().insert(e.classname.as_str());
        }

        cache.entry(e.classname.as_str()).or_insert_with(|| classes(&e.classname));
    }

    for (id, e) in map.entities() {
        let source = cache.get(e.classname.as_str()).and_then(Option::as_ref);
        for o in &e.outputs {
            if let Some(io) = source
                && !o.output.is_empty()
                && !io.outputs.is_empty()
                && !io.outputs.iter().any(|d| io_name_matches(d, &o.output))
            {
                out.push(Issue {
                    node: Some(id),
                    severity: Severity::Warning,
                    code: "io_unknown_output",
                    message: format!("{} has no output '{}'", e.classname, o.output),
                });
            }

            if o.input.is_empty() || is_dynamic_target(&o.target) || BUILTIN_INPUTS.iter().any(|b| io_name_matches(b, &o.input)) {
                continue;
            }

            for class in by_name.get(o.target.as_str()).into_iter().flatten() {
                if let Some(Some(io)) = cache.get(class)
                    && !io.inputs.is_empty()
                    && !io.inputs.iter().any(|d| io_name_matches(d, &o.input))
                {
                    out.push(Issue {
                        node: Some(id),
                        severity: Severity::Warning,
                        code: "io_unknown_input",
                        message: format!("Output {} calls input '{}', which {class} '{}' does not have", o.output, o.input, o.target),
                    });
                }
            }
        }
    }
}

fn check_map(map: &Map, external: &BTreeSet<String>) -> Vec<Issue> {
    let mut out = Vec::new();
    let mut names: BTreeSet<&str> = map.entities().filter_map(|(_, e)| e.targetname()).collect();
    names.extend(external.iter().map(String::as_str));
    let mut brush_keys: BTreeMap<Vec<[i64; 3]>, NodeId> = BTreeMap::new();

    for (id, node) in map.nodes.iter() {
        match &node.kind {
            NodeKind::Brush(b) => {
                if let Err(e) = b.validate() {
                    out.push(Issue { node: Some(*id), severity: Severity::Error, code: "invalid_brush", message: format!("Invalid brush: {e}") });
                }

                if b.volume() < 1e-3 {
                    out.push(Issue { node: Some(*id), severity: Severity::Error, code: "degenerate_brush", message: "Brush has no volume".into() });
                }

                for (fi, f) in b.faces.iter().enumerate() {
                    if f.data.material.is_empty() {
                        out.push(Issue { node: Some(*id), severity: Severity::Warning, code: "no_material", message: format!("Face {fi} has no material") });
                    }
                }

                let mut key: Vec<[i64; 3]> =
                    b.vertices.iter().map(|v| [(v.x * 64.0).round() as i64, (v.y * 64.0).round() as i64, (v.z * 64.0).round() as i64]).collect();
                key.sort();
                if let Some(other) = brush_keys.insert(key, *id) {
                    out.push(Issue {
                        node: Some(*id),
                        severity: Severity::Warning,
                        code: "duplicate_brush",
                        message: format!("Brush occupies the same space as {other}"),
                    });
                }
            }
            NodeKind::Entity(e) => {
                if e.classname.trim().is_empty() {
                    out.push(Issue { node: Some(*id), severity: Severity::Error, code: "missing_classname", message: "Entity has no classname".into() });
                }

                for (i, o) in e.outputs.iter().enumerate() {
                    let wildcard = is_dynamic_target(&o.target);
                    if o.target.is_empty() {
                        out.push(Issue {
                            node: Some(*id),
                            severity: Severity::Warning,
                            code: "io_no_target",
                            message: format!("Output {i} ({}) has no target", o.output),
                        });
                    } else if !wildcard && !names.contains(o.target.as_str()) {
                        out.push(Issue {
                            node: Some(*id),
                            severity: Severity::Error,
                            code: "io_missing_target",
                            message: format!("Output {} targets unknown entity '{}'", o.output, o.target),
                        });
                    }

                    if o.input.is_empty() {
                        out.push(Issue {
                            node: Some(*id),
                            severity: Severity::Warning,
                            code: "io_no_input",
                            message: format!("Output {i} ({}) has no input", o.output),
                        });
                    }
                }

                if let Some(t) = e.property("target")
                    && !t.is_empty()
                    && !is_dynamic_target(t)
                    && !names.contains(t)
                {
                    out.push(Issue { node: Some(*id), severity: Severity::Warning, code: "missing_target", message: format!("target '{t}' does not exist") });
                }
            }
            NodeKind::Mesh(m) => {
                if let Err(e) = m.validate() {
                    out.push(Issue { node: Some(*id), severity: Severity::Error, code: "invalid_mesh", message: format!("Invalid mesh: {e}") });
                }

                if m.faces.iter().any(|f| f.data.material.is_empty()) {
                    out.push(Issue { node: Some(*id), severity: Severity::Warning, code: "no_material", message: "Mesh has faces without material".into() });
                }
            }
            NodeKind::Terrain(t) => {
                if !t.is_valid() {
                    out.push(Issue {
                        node: Some(*id),
                        severity: Severity::Error,
                        code: "invalid_terrain",
                        message: "Terrain height data does not match its resolution".into(),
                    });
                }

                if t.layers.is_empty() {
                    out.push(Issue { node: Some(*id), severity: Severity::Warning, code: "no_material", message: "Terrain has no layers".into() });
                }
            }
            NodeKind::Scatter(s) => {
                if s.items.iter().any(|i| i.source.trim().is_empty()) {
                    out.push(Issue { node: Some(*id), severity: Severity::Warning, code: "scatter_no_source", message: "Scatter item has no model".into() });
                }

                if s.instances.iter().any(|i| i.item as usize >= s.items.len()) {
                    out.push(Issue {
                        node: Some(*id),
                        severity: Severity::Error,
                        code: "scatter_bad_item",
                        message: "Scatter instances refer to a missing palette entry".into(),
                    });
                }

                if s.targets.iter().any(|t| !map.contains(*t)) {
                    out.push(Issue {
                        node: Some(*id),
                        severity: Severity::Info,
                        code: "scatter_missing_target",
                        message: "Scatter set targets a surface that no longer exists".into(),
                    });
                }
            }
            NodeKind::Group(_) if node.children.is_empty() => {
                out.push(Issue { node: Some(*id), severity: Severity::Info, code: "empty_group", message: "Group is empty".into() });
            }
            _ => {}
        }
    }

    let mut seen: BTreeMap<&str, NodeId> = BTreeMap::new();
    for (id, e) in map.entities() {
        if let Some(n) = e.targetname()
            && let Some(first) = seen.insert(n, id)
        {
            out.push(Issue {
                node: Some(id),
                severity: Severity::Info,
                code: "shared_targetname",
                message: format!("targetname '{n}' is also used by {first}"),
            });
        }
    }

    out.sort_by_key(|i| std::cmp::Reverse(i.severity));
    out
}

/// Targets resolved at runtime rather than by targetname: wildcards, `!self` style names, `@group` and node paths.
pub fn is_dynamic_target(target: &str) -> bool {
    target.contains('*') || target.starts_with('!') || target.starts_with('@') || target.starts_with('/') || target.starts_with('%')
}

/// Whether `fix` knows how to repair an issue with this code.
pub fn fixable(code: &str) -> bool {
    matches!(
        code,
        "scatter_bad_item"
            | "scatter_missing_target"
            | "invalid_brush"
            | "degenerate_brush"
            | "duplicate_brush"
            | "empty_group"
            | "io_missing_target"
            | "io_no_target"
            | "missing_target"
            | "no_material"
            | "invalid_mesh"
            | "invalid_terrain"
    )
}

/// Applies the standard repair for an issue. Returns true if the map changed.
pub fn fix(map: &mut Map, issue: &Issue, default_material: &str) -> bool {
    let Some(id) = issue.node else { return false };
    if !map.contains(id) {
        return false;
    }

    match issue.code {
        "invalid_brush" | "degenerate_brush" | "duplicate_brush" | "empty_group" | "invalid_terrain" => {
            map.remove(id);
            true
        }
        "invalid_mesh" => {
            let Some(m) = map.mesh_mut(id) else { return false };
            let len = m.vertices.len() as u32;
            m.faces.retain(|f| f.indices.iter().all(|i| *i < len));
            m.cleanup();
            if m.faces.is_empty() {
                map.remove(id);
            }

            true
        }
        "io_missing_target" | "io_no_target" => {
            let names: BTreeSet<String> = map.entities().filter_map(|(_, e)| e.targetname().map(str::to_string)).collect();
            let Some(e) = map.entity_mut(id) else { return false };
            let before = e.outputs.len();
            e.outputs.retain(|o| is_dynamic_target(&o.target) || names.contains(&o.target));
            e.outputs.len() != before
        }
        "scatter_bad_item" => {
            let Some(s) = map.scatter_mut(id) else { return false };
            let n = s.items.len() as u32;
            s.instances.retain(|i| i.item < n);
            true
        }
        "scatter_missing_target" => {
            let existing: BTreeSet<NodeId> = map.nodes.keys().copied().collect();
            let Some(s) = map.scatter_mut(id) else { return false };
            s.targets.retain(|t| existing.contains(t));
            true
        }
        "missing_target" => {
            let Some(t) = map.entity(id).and_then(|e| e.property("target")) else { return false };
            if t.is_empty() || is_dynamic_target(t) || !map.find_by_targetname(t).is_empty() {
                return false;
            }

            map.entity_mut(id).is_some_and(|e| e.properties.remove("target").is_some())
        }
        "no_material" => {
            match map.get_mut(id).map(|n| &mut n.kind) {
                Some(NodeKind::Brush(b)) => {
                    b.faces.iter_mut().filter(|f| f.data.material.is_empty()).for_each(|f| f.data.material = default_material.to_string())
                }
                Some(NodeKind::Mesh(m)) => {
                    m.faces.iter_mut().filter(|f| f.data.material.is_empty()).for_each(|f| f.data.material = default_material.to_string())
                }
                Some(NodeKind::Terrain(t)) if t.layers.is_empty() => t.layers.push(gt_geom::TerrainLayer::new(default_material, 256.0)),
                _ => return false,
            }

            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Entity, IoConnection};
    use gt_core::{Aabb, DVec3};
    use gt_geom::Brush;

    #[test]
    fn finds_broken_io_and_duplicates() {
        let mut m = Map::new();
        let l = m.default_layer();
        let b = Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::splat(16.0)), "m").unwrap();
        m.insert(l, NodeKind::Brush(b.clone()));
        m.insert(l, NodeKind::Brush(b));
        let mut e = Entity::new("func_button");
        e.outputs.push(IoConnection { output: "pressed".into(), target: "door".into(), input: "open".into(), parameter: String::new(), delay: 0.0, times: -1 });
        m.insert(l, NodeKind::Entity(e));
        let issues = check(&m);
        let codes: Vec<&str> = issues.iter().map(|i| i.code).collect();
        assert!(codes.contains(&"duplicate_brush"));
        assert!(codes.contains(&"io_missing_target"));
        assert_eq!(issues[0].severity, Severity::Error);

        for issue in issues.iter().filter(|i| fixable(i.code)) {
            fix(&mut m, issue, "dev/grey");
        }

        assert!(check(&m).iter().all(|i| !fixable(i.code)), "{:?}", check(&m));
    }

    #[test]
    fn overlay_targetnames_count_as_existing_targets() {
        let mut m = Map::new();
        let l = m.default_layer();
        let mut e = Entity::new("trigger_once");
        e.outputs.push(IoConnection {
            output: "triggered".into(),
            target: "court_fireflies".into(),
            input: "emitting".into(),
            parameter: "true".into(),
            delay: 0.0,
            times: -1,
        });
        e.properties.insert("target".into(), "court_string_lights".into());
        m.insert(l, NodeKind::Entity(e));
        let missing = |issues: Vec<Issue>| issues.iter().filter(|i| i.code == "io_missing_target" || i.code == "missing_target").count();
        assert_eq!(missing(check(&m)), 2);
        let external: BTreeSet<String> = ["court_fireflies".to_string(), "court_string_lights".to_string()].into();
        assert_eq!(missing(check_with_external(&m, |_| None, &external)), 0);
    }

    #[test]
    fn perpendicular_walls_are_not_duplicates() {
        let mut m = Map::new();
        let l = m.default_layer();
        let wall = |max: DVec3| NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, max), "m").unwrap());
        m.insert(l, wall(DVec3::new(256.0, 128.0, 16.0)));
        m.insert(l, wall(DVec3::new(16.0, 128.0, 256.0)));
        assert!(check(&m).iter().all(|i| i.code != "duplicate_brush"), "{:?}", check(&m));
    }

    #[test]
    fn dynamic_target_properties_are_kept() {
        let mut m = Map::new();
        let l = m.default_layer();
        for t in ["!player", "@doors", "door*", "/root/Game", "nobody"] {
            let mut e = Entity::new("trigger_once");
            e.properties.insert("target".into(), t.into());
            m.insert(l, NodeKind::Entity(e));
        }

        let issues: Vec<Issue> = check(&m).into_iter().filter(|i| i.code == "missing_target").collect();
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("nobody"));
        for (id, e) in m.entities() {
            let issue = Issue { node: Some(id), ..issues[0].clone() };
            let mut copy = m.clone();
            assert_eq!(fix(&mut copy, &issue, "m"), e.property("target") == Some("nobody"));
        }
    }

    #[test]
    fn io_names_are_checked_against_definitions() {
        let mut m = Map::new();
        let l = m.default_layer();
        let conn = |output: &str, target: &str, input: &str| IoConnection {
            output: output.into(),
            target: target.into(),
            input: input.into(),
            parameter: String::new(),
            delay: 0.0,
            times: -1,
        };
        let mut relay = Entity::new("logic_relay");
        relay.outputs.push(conn("OnBogus", "door", "open"));
        relay.outputs.push(conn("on_trigger", "door", "BogusInput"));
        relay.outputs.push(conn("OnTrigger", "door", "Open"));
        relay.outputs.push(conn("on_trigger", "door", "kill"));
        relay.outputs.push(conn("on_trigger", "@doors", "whatever"));
        relay.outputs.push(conn("on_trigger", "thing", "whatever"));
        m.insert(l, NodeKind::Entity(relay));
        let mut door = Entity::new("func_door");
        door.properties.insert("targetname".into(), "door".into());
        m.insert(l, NodeKind::Entity(door));
        let mut thing = Entity::new("custom_thing");
        thing.properties.insert("targetname".into(), "thing".into());
        m.insert(l, NodeKind::Entity(thing));

        let issues = check_with(&m, |class| match class {
            "logic_relay" => Some(ClassIo { outputs: vec!["on_trigger"], inputs: vec!["trigger"] }),
            "func_door" => Some(ClassIo { outputs: vec!["opened"], inputs: vec!["open", "close"] }),
            _ => None,
        });
        let io: Vec<(&str, &str)> = issues.iter().filter(|i| i.code.starts_with("io_unknown")).map(|i| (i.code, i.message.as_str())).collect();
        assert_eq!(io.len(), 2, "{io:?}");
        assert!(io.iter().any(|(c, msg)| *c == "io_unknown_output" && msg.contains("OnBogus")));
        assert!(io.iter().any(|(c, msg)| *c == "io_unknown_input" && msg.contains("BogusInput")));
        assert!(check(&m).iter().all(|i| !i.code.starts_with("io_unknown")), "no definitions, no name checks");
    }
}
