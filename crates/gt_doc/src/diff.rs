//! What changed between two versions of a map, node by node.

use std::collections::BTreeMap;

use gt_core::{DVec3, NodeId};
use gt_geom::FaceData;
use imbl::ordmap::DiffItem;

use crate::map::{Map, Node, NodeKind};

/// A key whose value changed, `None` where it is absent.
pub type KeyChange = (String, Option<String>, Option<String>);

#[derive(Debug, Default, PartialEq)]
pub struct MapDiff {
    /// Nodes only the new map has.
    pub added: Vec<NodeId>,
    /// Nodes only the old map has.
    pub removed: Vec<NodeId>,
    pub changed: Vec<NodeChange>,
    /// Worldspawn keys.
    pub properties: Vec<KeyChange>,
}

impl MapDiff {
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.changed.is_empty() && self.properties.is_empty()
    }
}

#[derive(Debug, PartialEq)]
pub struct NodeChange {
    pub id: NodeId,
    /// Such as moved, rotated, geometry, uv, materials, paint, properties, outputs, parent, name, hidden, locked, or
    /// settings for anything else.
    pub what: Vec<&'static str>,
    /// How far it moved, when it moved without changing shape.
    pub offset: Option<DVec3>,
    /// Entity keys.
    pub properties: Vec<KeyChange>,
}

pub fn diff(old: &Map, new: &Map) -> MapDiff {
    let mut out = MapDiff::default();
    for item in old.nodes.diff(&new.nodes) {
        match item {
            DiffItem::Add(id, _) => out.added.push(*id),
            DiffItem::Remove(id, _) => out.removed.push(*id),
            DiffItem::Update { old: (id, a), new: (_, b) } => {
                let change = compare(*id, a, b);
                if !change.what.is_empty() {
                    out.changed.push(change);
                }
            }
        }
    }

    out.properties = key_changes(&old.properties, &new.properties);
    out
}

fn key_changes(old: &BTreeMap<String, String>, new: &BTreeMap<String, String>) -> Vec<KeyChange> {
    let mut out: Vec<KeyChange> = old.iter().filter(|(k, v)| new.get(*k) != Some(v)).map(|(k, v)| (k.clone(), Some(v.clone()), new.get(k).cloned())).collect();
    out.extend(new.iter().filter(|(k, _)| !old.contains_key(*k)).map(|(k, v)| (k.clone(), None, Some(v.clone()))));
    out.sort();
    out
}

/// The offset between two point lists when one is the other moved, None when they differ in any other way.
fn uniform_offset(a: &[DVec3], b: &[DVec3]) -> Option<DVec3> {
    let d = *b.first()? - *a.first()?;
    (a.len() == b.len() && a.iter().zip(b).all(|(p, q)| (*q - *p - d).length() < 1e-6)).then_some(d)
}

/// Face indices and data, in face order.
type Faces<'a> = Vec<(&'a [u32], &'a FaceData)>;

/// Brush and mesh faces: whether the shape moved, changed, or kept still while its UVs changed.
fn surface(change: &mut NodeChange, verts: (&[DVec3], &[DVec3]), faces: (Faces, Faces)) {
    let (a, b) = faces;
    let same_topology = a.len() == b.len() && a.iter().zip(&b).all(|(x, y)| x.0 == y.0);
    let pairs = || a.iter().zip(&b).map(|(x, y)| (x.1, y.1));
    match uniform_offset(verts.0, verts.1).filter(|_| same_topology) {
        Some(d) if d.length() < 1e-6 => {
            if pairs().any(|(x, y)| x.disp != y.disp) {
                change.what.push("geometry");
            }

            // UV lock rewrites the UVs of anything that moves, so only unmoved faces report them.
            if pairs().any(|(x, y)| x.uv != y.uv) {
                change.what.push("uv");
            }
        }
        Some(d) => {
            change.what.push("moved");
            change.offset = Some(d);
        }
        None => change.what.push("geometry"),
    }

    let materials = |f: &Faces| f.iter().map(|x| x.1.material.clone()).collect::<Vec<_>>();
    if materials(&a) != materials(&b) {
        change.what.push("materials");
    }

    if same_topology && pairs().any(|(x, y)| x.colors != y.colors || x.props != y.props) {
        change.what.push("paint");
    }
}

fn compare(id: NodeId, a: &Node, b: &Node) -> NodeChange {
    let mut c = NodeChange { id, what: Vec::new(), offset: None, properties: Vec::new() };
    if a.parent != b.parent {
        c.what.push("parent");
    }

    if a.hidden != b.hidden {
        c.what.push("hidden");
    }

    if a.locked != b.locked {
        c.what.push("locked");
    }

    if a.label != b.label {
        c.what.push("name");
    }

    let before = c.what.len();
    match (&a.kind, &b.kind) {
        (NodeKind::Entity(x), NodeKind::Entity(y)) => {
            if x.classname != y.classname {
                c.what.push("classname");
            }

            if x.origin != y.origin {
                c.what.push("moved");
                c.offset = Some(y.origin - x.origin);
            }

            if x.angles != y.angles {
                c.what.push("rotated");
            }

            c.properties = key_changes(&x.properties, &y.properties);
            if !c.properties.is_empty() {
                c.what.push("properties");
            }

            if x.outputs != y.outputs {
                c.what.push("outputs");
            }
        }
        (NodeKind::Brush(x), NodeKind::Brush(y)) => surface(
            &mut c,
            (&x.vertices, &y.vertices),
            (x.faces.iter().map(|f| (f.indices.as_slice(), &f.data)).collect(), y.faces.iter().map(|f| (f.indices.as_slice(), &f.data)).collect()),
        ),
        (NodeKind::Mesh(x), NodeKind::Mesh(y)) => {
            surface(
                &mut c,
                (&x.vertices, &y.vertices),
                (x.faces.iter().map(|f| (f.indices.as_slice(), &f.data)).collect(), y.faces.iter().map(|f| (f.indices.as_slice(), &f.data)).collect()),
            );
            if !c.what.contains(&"uv") && x.faces.iter().zip(&y.faces).any(|(f, g)| f.uvs != g.uvs) {
                c.what.push("uv");
            }
        }
        (NodeKind::Terrain(x), NodeKind::Terrain(y)) => {
            if x.origin != y.origin {
                c.what.push("moved");
                c.offset = Some(y.origin - x.origin);
            }

            if x.heights != y.heights || x.resolution != y.resolution || x.cell_size != y.cell_size || x.holes != y.holes {
                c.what.push("geometry");
            }

            if x.layers != y.layers || x.splat != y.splat {
                c.what.push("materials");
            }
        }
        (NodeKind::Instance(x), NodeKind::Instance(y)) => {
            if x.origin != y.origin {
                c.what.push("moved");
                c.offset = Some(y.origin - x.origin);
            }

            if x.angles != y.angles {
                c.what.push("rotated");
            }
        }
        (NodeKind::Layer(x), NodeKind::Layer(y)) => {
            if x.name != y.name {
                c.what.push("name");
            }

            if x.omit_from_export != y.omit_from_export {
                c.what.push("omit_from_export");
            }

            // A layer's color is only how the editor draws it, and it loses precision in a saved file.
            return c;
        }
        (NodeKind::Group(x), NodeKind::Group(y)) => {
            if x.name != y.name {
                c.what.push("name");
            }

            if x.link_id != y.link_id {
                c.what.push("linked");
            }

            // A linked group's transform follows its children, which are reported themselves.
            return c;
        }
        (NodeKind::Scatter(x), NodeKind::Scatter(y)) if x.instances != y.instances => c.what.push("instances"),
        (x, y) if std::mem::discriminant(x) != std::mem::discriminant(y) => c.what.push("type"),
        _ => {}
    }

    // Children lists change whenever a child is added or removed, and those children are reported themselves.
    if c.what.len() == before && a.kind != b.kind {
        c.what.push("settings");
    }

    c
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Entity, ops};
    use gt_core::Aabb;
    use gt_geom::Brush;

    fn cube(map: &mut Map, at: DVec3) -> NodeId {
        let layer = map.default_layer();
        map.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(at, at + DVec3::splat(32.0)), "wood").unwrap()))
    }

    #[test]
    fn reports_added_removed_moved_and_retextured_nodes() {
        let mut old = Map::new();
        let kept = cube(&mut old, DVec3::ZERO);
        let gone = cube(&mut old, DVec3::X * 100.0);
        let painted = cube(&mut old, DVec3::X * 200.0);
        let layer = old.default_layer();
        let lamp = ops::create_point_entity(&mut old, layer, "light", DVec3::ZERO);
        old.entity_mut(lamp).unwrap().properties.insert("energy".into(), "1".into());

        let mut new = old.clone();
        new.remove(gone);
        let added = cube(&mut new, DVec3::Z * 100.0);
        for v in &mut new.brush_mut(kept).unwrap().vertices {
            *v += DVec3::new(0.0, 16.0, 0.0);
        }

        for f in &mut new.brush_mut(painted).unwrap().faces {
            f.data.material = "brick".into();
        }

        let e = new.entity_mut(lamp).unwrap();
        e.origin = DVec3::new(8.0, 0.0, 0.0);
        e.properties.insert("energy".into(), "2".into());
        e.properties.insert("color".into(), "red".into());
        new.properties.insert("message".into(), "night".into());

        let d = diff(&old, &new);
        assert_eq!(d.added, [added]);
        assert_eq!(d.removed, [gone]);
        let find = |id| d.changed.iter().find(|c| c.id == id).unwrap();
        assert_eq!(find(kept).what, ["moved"]);
        assert_eq!(find(kept).offset, Some(DVec3::new(0.0, 16.0, 0.0)));
        assert_eq!(find(painted).what, ["materials"]);
        assert_eq!(find(lamp).what, ["moved", "properties"]);
        assert_eq!(
            find(lamp).properties,
            [("color".to_string(), None, Some("red".to_string())), ("energy".to_string(), Some("1".to_string()), Some("2".to_string()))]
        );
        assert_eq!(d.changed.len(), 3, "the layer whose children changed is not reported: {:?}", d.changed);
        assert_eq!(d.properties, [("message".to_string(), None, Some("night".to_string()))]);
        assert!(diff(&new, &new).is_empty());
    }

    #[test]
    fn reshaped_brushes_and_renamed_entities() {
        let mut old = Map::new();
        let b = cube(&mut old, DVec3::ZERO);
        let layer = old.default_layer();
        let e = old.insert(layer, NodeKind::Entity(Entity::new("info_null")));
        let mut new = old.clone();
        new.brush_mut(b).unwrap().vertices[0].x -= 8.0;
        new.entity_mut(e).unwrap().classname = "info_target".into();
        new.get_mut(e).unwrap().hidden = true;
        if let NodeKind::Layer(l) = &mut new.get_mut(layer).unwrap().kind {
            l.color = gt_core::Color::from_seed(99);
        }

        let d = diff(&old, &new);
        assert_eq!(d.changed.len(), 2, "a layer color is not a change: {:?}", d.changed);
        assert_eq!(d.changed.iter().find(|c| c.id == b).unwrap().what, ["geometry"]);
        assert_eq!(d.changed.iter().find(|c| c.id == e).unwrap().what, ["hidden", "classname"]);
    }
}
