use gt_core::{DVec3, NodeId, Plane};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolKind {
    /// Selection, moving, brush drawing, face dragging and entity gizmo handles (TrenchBroom default tool).
    Select,
    Clip,
    Vertex,
    Rotate,
    Scale,
    /// Displacement and terrain sculpting.
    Sculpt,
    /// Material blending on terrains, displacements and faces with a blend material.
    Blend,
    /// Vertex color painting on brush and mesh faces.
    Paint,
    /// Blender style editing of mesh vertices, edges and faces.
    Mesh,
    /// Radial painting and erasing of trees, rocks and foliage into scatter layers.
    Scatter,
    /// Draws gameplay volumes: triggers, spawn areas, hurt and teleport zones.
    Volume,
    /// Places chains of path_corner entities.
    Path,
    /// Measures distances between two points.
    Measure,
    /// Hammer style texture application: apply, wrap, pick and slide textures on faces.
    Texture,
}

impl ToolKind {
    pub fn label(&self) -> &'static str {
        match self {
            ToolKind::Select => "Select",
            ToolKind::Clip => "Clip",
            ToolKind::Vertex => "Vertex",
            ToolKind::Rotate => "Rotate",
            ToolKind::Scale => "Scale",
            ToolKind::Sculpt => "Sculpt",
            ToolKind::Blend => "Blend",
            ToolKind::Paint => "Paint",
            ToolKind::Mesh => "Mesh",
            ToolKind::Scatter => "Scatter",
            ToolKind::Volume => "Volume",
            ToolKind::Path => "Path",
            ToolKind::Measure => "Measure",
            ToolKind::Texture => "Texture",
        }
    }

    pub fn all() -> [ToolKind; 14] {
        [
            ToolKind::Select,
            ToolKind::Clip,
            ToolKind::Vertex,
            ToolKind::Rotate,
            ToolKind::Scale,
            ToolKind::Mesh,
            ToolKind::Sculpt,
            ToolKind::Blend,
            ToolKind::Paint,
            ToolKind::Scatter,
            ToolKind::Volume,
            ToolKind::Path,
            ToolKind::Measure,
            ToolKind::Texture,
        ]
    }

    /// Tools grouped the way the toolbar and the Tools menu show them: brush editing, meshes, surfaces, terrain, gameplay.
    pub const GROUPS: [&'static [ToolKind]; 5] = [
        &[ToolKind::Select, ToolKind::Clip, ToolKind::Vertex, ToolKind::Rotate, ToolKind::Scale],
        &[ToolKind::Mesh],
        &[ToolKind::Texture, ToolKind::Paint],
        &[ToolKind::Sculpt, ToolKind::Blend, ToolKind::Scatter],
        &[ToolKind::Volume, ToolKind::Path, ToolKind::Measure],
    ];

    /// Accepts labels case insensitively, and "sprinkle" as the old name of the scatter tool.
    pub fn from_name(name: &str) -> Option<ToolKind> {
        if name.eq_ignore_ascii_case("sprinkle") {
            return Some(ToolKind::Scatter);
        }
        ToolKind::all().into_iter().find(|t| t.label().eq_ignore_ascii_case(name))
    }
}

/// An in-progress mouse drag in a viewport.
#[derive(Clone, Debug)]
pub enum Drag {
    Move {
        start: DVec3,
        plane: Plane,
        axis: Option<DVec3>,
        duplicate: bool,
    },
    CreateBrush {
        start: DVec3,
        normal: DVec3,
        anchor: f64,
    },
    CreateBrush2d {
        start: DVec3,
    },
    FaceResize {
        faces: Vec<(NodeId, usize)>,
        origin: DVec3,
        normal: DVec3,
    },
    Look,
    Pan,
    Orbit {
        pivot: DVec3,
    },
    /// Dragging an entity gizmo handle.
    Gizmo {
        handle: crate::gizmos::GizmoHandle,
        start: DVec3,
    },
    /// Tool specific drags handled by the tool itself.
    Tool,
}

/// Closest point parameter on the line `origin + dir * s` to the ray.
pub fn line_ray_param(origin: DVec3, dir: DVec3, ray: &gt_core::Ray) -> Option<f64> {
    let w0 = origin - ray.origin;
    let b = dir.dot(ray.dir);
    let d = dir.dot(w0);
    let e = ray.dir.dot(w0);
    let denom = dir.dot(dir) * ray.dir.dot(ray.dir) - b * b;
    if denom.abs() < 1e-6 {
        return None;
    }
    Some((b * e - ray.dir.dot(ray.dir) * d) / denom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_list_every_tool_once() {
        let grouped: Vec<ToolKind> = ToolKind::GROUPS.iter().flat_map(|g| g.iter().copied()).collect();
        assert_eq!(grouped.len(), ToolKind::all().len());
        assert!(ToolKind::all().iter().all(|t| grouped.contains(t)));
    }
}
