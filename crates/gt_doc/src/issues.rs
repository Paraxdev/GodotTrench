//! Map problems that do not depend on a game configuration.

use std::collections::{BTreeMap, BTreeSet};

use gt_core::NodeId;
use serde::Serialize;

use crate::map::{Map, NodeKind};

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
    let mut out = Vec::new();
    let names: BTreeSet<&str> = map.entities().filter_map(|(_, e)| e.targetname()).collect();
    let mut brush_keys: BTreeMap<Vec<i64>, NodeId> = BTreeMap::new();

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
                let mut key: Vec<i64> =
                    b.vertices.iter().flat_map(|v| [(v.x * 64.0).round() as i64, (v.y * 64.0).round() as i64, (v.z * 64.0).round() as i64]).collect();
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
        "missing_target" => map.entity_mut(id).is_some_and(|e| e.properties.remove("target").is_some()),
        "no_material" => {
            match map.get_mut(id).map(|n| &mut n.kind) {
                Some(NodeKind::Brush(b)) => {
                    b.faces.iter_mut().filter(|f| f.data.material.is_empty()).for_each(|f| f.data.material = default_material.to_string())
                }
                Some(NodeKind::Mesh(m)) => {
                    m.faces.iter_mut().filter(|f| f.data.material.is_empty()).for_each(|f| f.data.material = default_material.to_string())
                }
                Some(NodeKind::Terrain(t)) if t.layers.is_empty() => {
                    t.layers.push(gt_geom::TerrainLayer { material: default_material.to_string(), tile: 256.0 })
                }
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
}
