//! Walkable area overlay: the navigation mesh the connected Godot editor bakes for the open map, drawn over the views
//! with the biggest connected area in one colour and every island cut off from it in another.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use gt_core::{Aabb, DVec3};
use gt_render::{LineVertex, MeshBatch, MeshVertex, WHITE_MATERIAL};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::state::EditorState;

/// Edits settle this long before the overlay is baked again.
const SETTLE: Duration = Duration::from_millis(1200);
/// Map units the overlay floats above the mesh, which Godot bakes close to the floor.
const LIFT: f64 = 1.0;
const FILL_ALPHA: f32 = 0.4;
/// Cut off islands listed one by one in a summary, the rest are only counted.
const LISTED: usize = 20;

/// Size of the agent the walkable area is baked for, in meters and degrees, see `GodotTrenchNav.profile`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Agent {
    pub radius: f64,
    pub height: f64,
    pub max_climb: f64,
    pub max_slope: f64,
}

impl Default for Agent {
    fn default() -> Self {
        Self { radius: 0.3, height: 1.8, max_climb: 0.3, max_slope: 45.0 }
    }
}

impl Agent {
    pub fn to_json(self) -> Value {
        json!({ "radius": self.radius, "height": self.height, "max_climb": self.max_climb, "max_slope": self.max_slope })
    }
}

/// Polygons connected to each other, with their area in square meters and bounds in map units.
#[derive(Clone, Debug)]
pub struct Island {
    pub polygons: Vec<usize>,
    pub area: f64,
    pub bounds: Aabb,
}

/// A navigation mesh in map units, as Godot baked it.
#[derive(Clone, Debug)]
pub struct Walkable {
    /// The agent Godot baked for, its sizes snapped to the bake's cells.
    pub agent: Value,
    pub vertices: Vec<DVec3>,
    pub polygons: Vec<Vec<usize>>,
    /// Largest area first.
    pub islands: Vec<Island>,
    pub msec: u64,
}

fn round(v: f64, decimals: i32) -> f64 {
    let f = 10f64.powi(decimals);
    (v * f).round() / f
}

fn point(v: DVec3) -> Value {
    json!([round(v.x, 1), round(v.y, 1), round(v.z, 1)])
}

fn bounds_json(b: &Aabb) -> Value {
    json!({ "min": point(b.min), "max": point(b.max) })
}

/// Normal scaled by the area, from Newell's method, so it works for any convex or concave outline.
fn area_normal(points: &[DVec3]) -> DVec3 {
    let mut n = DVec3::ZERO;
    for (i, a) in points.iter().enumerate() {
        n += a.cross(points[(i + 1) % points.len()]);
    }

    n * 0.5
}

fn color(c: egui::Color32, alpha: f32) -> [f32; 4] {
    let l = crate::theme::linear(c);
    [l[0] as f32, l[1] as f32, l[2] as f32, alpha]
}

impl Walkable {
    /// Reads a `walkable` reply of the live link.
    pub fn from_reply(reply: &Value) -> Result<Self, String> {
        let flat: Vec<f64> = reply["vertices"].as_array().ok_or("Godot sent no vertices")?.iter().map(|v| v.as_f64().unwrap_or(0.0)).collect();
        let vertices: Vec<DVec3> = flat.as_chunks::<3>().0.iter().map(|c| DVec3::from_array(*c)).collect();
        let index = |v: &Value| v.as_u64().map(|i| i as usize).filter(|i| *i < vertices.len());
        let polygons: Vec<Vec<usize>> = reply["polygons"]
            .as_array()
            .ok_or("Godot sent no polygons")?
            .iter()
            .map(|p| p.as_array().map(|p| p.iter().filter_map(index).collect()).unwrap_or_default())
            .collect();
        let per_meter = reply["units_per_meter"].as_f64().filter(|u| *u > 0.0).unwrap_or(32.0);
        let mut islands: Vec<Island> = reply["islands"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|island| {
                let members: Vec<usize> =
                    island.as_array().into_iter().flatten().filter_map(|p| p.as_u64().map(|p| p as usize)).filter(|p| *p < polygons.len()).collect();
                let mut area = 0.0;
                let mut bounds = Aabb::EMPTY;
                for p in &members {
                    let points: Vec<DVec3> = polygons[*p].iter().map(|v| vertices[*v]).collect();
                    area += area_normal(&points).length();
                    points.iter().for_each(|v| bounds.include_point(*v));
                }

                Island { polygons: members, area: area / (per_meter * per_meter), bounds }
            })
            .filter(|i| !i.polygons.is_empty())
            .collect();
        islands.sort_by(|a, b| b.area.total_cmp(&a.area));
        Ok(Self { agent: reply["agent"].clone(), vertices, polygons, islands, msec: reply["msec"].as_u64().unwrap_or(0) })
    }

    /// Walkable area in square meters.
    pub fn area(&self) -> f64 {
        self.islands.iter().map(|i| i.area).sum()
    }

    /// What an agent needs to act without a screenshot: the areas, the main island and where the cut off ones are.
    pub fn summary(&self) -> Value {
        let cut_off: Vec<Value> = self
            .islands
            .iter()
            .skip(1)
            .take(LISTED)
            .map(|i| json!({ "area_m2": round(i.area, 2), "polygons": i.polygons.len(), "center": point(i.bounds.center()), "bounds": bounds_json(&i.bounds) }))
            .collect();
        let mut out = json!({
            "agent": self.agent,
            "area_m2": round(self.area(), 1),
            "polygons": self.polygons.len(),
            "islands": self.islands.len(),
            "main": self.islands.first().map(|i| json!({ "area_m2": round(i.area, 1), "polygons": i.polygons.len(), "bounds": bounds_json(&i.bounds) })),
            "cut_off": cut_off,
            "bake_ms": self.msec,
        });
        if self.islands.len() > LISTED + 1 {
            out["cut_off_unlisted"] = json!(self.islands.len() - LISTED - 1);
        }

        out["hint"] = json!(match self.islands.len() {
            0 => "Nothing is walkable for this agent. Check that the floors have collision and the agent fits under the ceilings.",
            1 => "Everything walkable is connected.",
            _ =>
                "Cut off islands sit behind a step higher than max_climb, a slope steeper than max_slope or a gap narrower than twice the radius. Wall, roof and furniture tops show up as small islands too.",
        });
        out
    }

    fn island_colors(&self, alpha: f32) -> Vec<[f32; 4]> {
        let mut out = vec![color(crate::theme::WALKABLE_CUT_OFF, alpha); self.polygons.len()];
        for p in self.islands.first().map(|i| i.polygons.as_slice()).unwrap_or_default() {
            out[*p] = color(crate::theme::WALKABLE, alpha);
        }

        out
    }

    /// Translucent fill of every polygon, facing up.
    pub fn mesh(&self) -> MeshBatch {
        let colors = self.island_colors(FILL_ALPHA);
        let mut batch = MeshBatch::default();
        for (polygon, color) in self.polygons.iter().zip(colors) {
            let mut points: Vec<DVec3> = polygon.iter().map(|v| self.vertices[*v] + DVec3::Y * LIFT).collect();
            let mut normal = area_normal(&points);
            if normal.y < 0.0 {
                points.reverse();
                normal = -normal;
            }

            let normal = crate::scene::v3(normal.normalize_or(DVec3::Y));
            let verts: Vec<MeshVertex> = points.iter().map(|p| MeshVertex { pos: crate::scene::v3(*p), normal, uv: [0.0, 0.0], color }).collect();
            batch.add_polygon(WHITE_MATERIAL, &verts);
        }

        batch
    }

    /// Outline of each island: the polygon edges no other polygon shares.
    pub fn outlines(&self) -> Vec<LineVertex> {
        let mut uses: HashMap<(usize, usize), usize> = HashMap::new();
        for polygon in &self.polygons {
            for (i, a) in polygon.iter().enumerate() {
                let b = polygon[(i + 1) % polygon.len()];
                *uses.entry((*a.min(&b), *a.max(&b))).or_default() += 1;
            }
        }

        let colors = self.island_colors(0.9);
        let mut out = Vec::new();
        for (polygon, color) in self.polygons.iter().zip(colors) {
            for (i, a) in polygon.iter().enumerate() {
                let b = polygon[(i + 1) % polygon.len()];
                if uses.get(&(*a.min(&b), *a.max(&b))) == Some(&1) {
                    for v in [*a, b] {
                        out.push(LineVertex { pos: crate::scene::v3(self.vertices[v] + DVec3::Y * LIFT), color });
                    }
                }
            }
        }

        out
    }
}

/// Called on the live link thread with the parsed bake.
pub type Then = Box<dyn FnOnce(&Result<Walkable, String>) + Send>;

/// The overlay as shown: on or off, the last bake and the one Godot is working on.
#[derive(Default)]
pub struct WalkableView {
    pub on: bool,
    pub result: Option<Walkable>,
    pub error: Option<String>,
    /// Goes up whenever `result` changes.
    pub generation: u64,
    pending: Option<Receiver<Result<Walkable, String>>>,
    /// Map, document revision and agent of the last bake asked for.
    asked: Option<(Option<PathBuf>, u64, Agent)>,
    changed: Option<Instant>,
}

impl WalkableView {
    pub fn busy(&self) -> bool {
        self.pending.is_some()
    }

    /// One line for the menu.
    pub fn describe(&self) -> String {
        match (&self.result, &self.error) {
            _ if self.busy() => "Baking in Godot…".into(),
            (_, Some(e)) => e.clone(),
            (Some(w), _) => match w.islands.len() {
                0 => "Nothing is walkable".into(),
                1 => format!("{:.0} m² walkable, all connected", w.area()),
                n => format!("{:.0} m² walkable, {} cut off islands", w.area(), n - 1),
            },
            _ => "Not baked yet".into(),
        }
    }

    /// Turning it on bakes on the next frame, turning it off drops the bake.
    pub fn set_on(&mut self, on: bool) {
        *self = Self { on, generation: self.generation + 1, ..Default::default() };
    }
}

/// Why Godot cannot bake the current map, checked before asking it.
pub fn unavailable(state: &EditorState) -> Option<String> {
    if state.doc.path.is_none() {
        Some("save the map into the Godot project first, Godot bakes the scene that builds it".into())
    } else if state.game.project_root.is_none() {
        Some("open the Godot project first".into())
    } else if state.link.is_none() || !state.prefs.live_link {
        Some("the Godot live link is off, turn it on in Preferences".into())
    } else {
        None
    }
}

/// Asks Godot to bake the walkable area of the current map. Without live mode Godot first builds the map as shown.
/// `then` runs with the result as soon as it arrives, on the live link thread.
pub fn request(state: &mut EditorState, repaint: Option<egui::Context>, then: Option<Then>) -> Result<(), String> {
    if let Some(why) = unavailable(state) {
        return Err(why);
    }

    let (Some(path), Some(link)) = (state.doc.path.clone(), state.link.as_ref()) else { return Err("no map or live link".into()) };
    let (tx, rx) = mpsc::channel();
    let agent = state.prefs.walk_agent;
    link.request(crate::live_link::Request::Walkable {
        path: crate::live_link::godot_path(&path),
        agent: agent.to_json(),
        build: (!state.live_active()).then(|| state.doc.map.clone()),
        done: Box::new(move |reply| {
            let result = reply.and_then(|r| Walkable::from_reply(&r));
            // The view gets the bake before an MCP caller hears back, so its next call already sees the overlay.
            let _ = tx.send(result.clone());
            if let Some(ctx) = repaint {
                ctx.request_repaint();
            }

            if let Some(then) = then {
                then(&result);
            }
        }),
    });
    let view = &mut state.walkable;
    view.pending = Some(rx);
    view.asked = Some((Some(path), state.doc.revision, agent));
    view.changed = None;
    Ok(())
}

/// Per frame: takes in a finished bake, and while the overlay is on bakes again once edits or the agent settle. A failed
/// bake turns the overlay off. Returns whether the overlay changed.
pub fn poll(state: &mut EditorState, ctx: &egui::Context) -> bool {
    let generation = state.walkable.generation;
    poll_inner(state, ctx);
    state.walkable.generation != generation
}

fn poll_inner(state: &mut EditorState, ctx: &egui::Context) {
    let view = &mut state.walkable;
    if let Some(rx) = &view.pending {
        match rx.try_recv() {
            Ok(Ok(w)) => {
                view.result = Some(w);
                view.error = None;
            }
            Ok(Err(e)) => {
                view.on = false;
                view.result = None;
                view.error = Some(e.clone());
                state.set_status(format!("Walkable area: {e}"));
            }
            Err(mpsc::TryRecvError::Disconnected) => {}
            Err(mpsc::TryRecvError::Empty) => return,
        }

        state.walkable.pending = None;
        state.walkable.generation += 1;
    }

    let view = &mut state.walkable;
    if !view.on {
        return;
    }

    let now = (state.doc.path.clone(), state.doc.revision, state.prefs.walk_agent);
    if view.asked.as_ref() == Some(&now) {
        return;
    }

    let switched = view.asked.as_ref().is_some_and(|a| a.0 != now.0);
    if switched && view.result.take().is_some() {
        view.generation += 1;
    }

    if view.asked.is_some() && !switched {
        let settled = view.changed.get_or_insert_with(Instant::now).elapsed();
        if settled < SETTLE || state.drag_preview.is_some() {
            ctx.request_repaint_after(SETTLE.saturating_sub(settled));
            return;
        }
    }

    if let Err(e) = request(state, Some(ctx.clone()), None) {
        state.walkable.on = false;
        state.walkable.error = Some(e.clone());
        state.walkable.generation += 1;
        state.set_status(format!("Walkable area: {e}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two floor squares 2 m wide at 32 units per meter, the second one split into two triangles, and a lone
    /// 1 m square up on a wall.
    fn sample() -> Value {
        json!({
            "ok": true, "units_per_meter": 32.0, "msec": 12,
            "agent": { "radius": 0.3, "height": 1.8, "max_climb": 0.3, "max_slope": 45.0 },
            "vertices": [
                0, 0, 0, 64, 0, 0, 64, 0, 64, 0, 0, 64,
                128, 0, 0, 128, 0, 64,
                300, 96, 0, 332, 96, 0, 332, 96, 32, 300, 96, 32
            ],
            "polygons": [[0, 3, 2, 1], [1, 2, 5], [1, 5, 4], [6, 9, 8, 7]],
            "islands": [[0, 1, 2], [3]]
        })
    }

    #[test]
    fn islands_come_with_area_and_bounds() {
        let w = Walkable::from_reply(&sample()).unwrap();
        assert_eq!(w.islands.len(), 2);
        assert!((w.islands[0].area - 8.0).abs() < 1e-9, "4 m² of floor plus 4 m² of triangles, got {}", w.islands[0].area);
        assert!((w.islands[1].area - 1.0).abs() < 1e-9);
        assert_eq!(w.islands[1].bounds.min, DVec3::new(300.0, 96.0, 0.0));
        let s = w.summary();
        assert_eq!(s["islands"], 2);
        assert_eq!(s["area_m2"], 9.0);
        assert_eq!(s["cut_off"][0]["center"], json!([316.0, 96.0, 16.0]));
        assert_eq!(s["main"]["bounds"]["max"], json!([128.0, 0.0, 64.0]));
    }

    #[test]
    fn islands_sort_by_area_not_polygon_count() {
        let mut reply = sample();
        reply["islands"] = json!([[3], [0, 1, 2]]);
        let w = Walkable::from_reply(&reply).unwrap();
        assert_eq!(w.islands[0].polygons, vec![0, 1, 2]);
    }

    #[test]
    fn the_overlay_faces_up_and_outlines_island_edges() {
        let w = Walkable::from_reply(&sample()).unwrap();
        let mesh = w.mesh();
        assert!(mesh.vertices.iter().all(|v| v.normal[1] > 0.99 && v.pos[1] >= LIFT as f32), "every polygon faces up and floats over the floor");
        let main = color(crate::theme::WALKABLE, FILL_ALPHA);
        let cut = color(crate::theme::WALKABLE_CUT_OFF, FILL_ALPHA);
        assert_eq!(mesh.vertices.iter().filter(|v| v.color == main).count(), 10);
        assert_eq!(mesh.vertices.iter().filter(|v| v.color == cut).count(), 4);
        let lines = w.outlines();
        assert_eq!(lines.len() / 2, 6 + 4, "the shared edges inside the main island are left out");
    }

    #[test]
    fn an_empty_bake_says_nothing_is_walkable() {
        let w = Walkable::from_reply(&json!({ "ok": true, "vertices": [], "polygons": [], "islands": [] })).unwrap();
        let s = w.summary();
        assert_eq!(s["islands"], 0);
        assert!(s["main"].is_null());
        assert!(s["hint"].as_str().unwrap().starts_with("Nothing"));
        assert!(Walkable::from_reply(&json!({ "ok": true })).is_err());
    }
}
