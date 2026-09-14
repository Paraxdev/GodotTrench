//! End-to-end tests: launch the real editor with its MCP server and drive it with simulated user input.
//! They need a GPU and a desktop session, so they are ignored by default:
//!   cargo test -p gt_editor --test e2e -- --ignored --test-threads=1
//! Screenshots are written to target/e2e/.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use base64::Engine;
use serde_json::{Value, json};

struct Editor {
    child: Child,
    port: u16,
    name: &'static str,
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

fn artifacts() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/e2e");
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

impl Editor {
    fn launch(name: &'static str) -> Editor {
        Self::launch_with(name, &[])
    }

    fn launch_with(name: &'static str, args: &[&str]) -> Editor {
        let port = free_port();
        let child =
            Command::new(env!("CARGO_BIN_EXE_godottrench")).arg(format!("--mcp-http={port}")).arg("--default-prefs").args(args).spawn().expect("launch editor");
        let editor = Editor { child, port, name };
        let deadline = Instant::now() + Duration::from_secs(60);
        loop {
            if editor
                .try_rpc("initialize", json!({ "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "e2e", "version": "1" } }))
                .is_some()
            {
                break;
            }
            assert!(Instant::now() < deadline, "editor did not start");
            std::thread::sleep(Duration::from_millis(250));
        }
        // Let the first frames lay out the viewports.
        std::thread::sleep(Duration::from_millis(500));
        editor
    }

    fn try_rpc(&self, method: &str, params: Value) -> Option<Value> {
        let mut s = TcpStream::connect(("127.0.0.1", self.port)).ok()?;
        s.set_read_timeout(Some(Duration::from_secs(180))).ok()?;
        let body = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params }).to_string();
        write!(s, "POST /mcp HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}", body.len()).ok()?;
        let mut out = String::new();
        s.read_to_string(&mut out).ok()?;
        let (_, body) = out.split_once("\r\n\r\n")?;
        serde_json::from_str(body).ok()
    }

    /// Calls a tool and returns its structured result, panicking on tool errors.
    fn call(&self, tool: &str, args: Value) -> Value {
        let r = self.try_rpc("tools/call", json!({ "name": tool, "arguments": args })).unwrap_or_else(|| panic!("{tool}: no response"));
        let result = &r["result"];
        assert!(!result["isError"].as_bool().unwrap_or(true), "{tool} failed: {}", result["content"][0]["text"]);
        result["structuredContent"].clone()
    }

    fn call_err(&self, tool: &str, args: Value) -> String {
        let r = self.try_rpc("tools/call", json!({ "name": tool, "arguments": args })).unwrap();
        assert!(r["result"]["isError"].as_bool().unwrap_or(false), "{tool} unexpectedly succeeded");
        r["result"]["content"][0]["text"].as_str().unwrap_or_default().to_string()
    }

    fn input(&self, target: &str, events: Value) -> Value {
        self.call("simulate_input", json!({ "target": target, "events": events }))["state"].clone()
    }

    fn state(&self) -> Value {
        self.call("get_state", json!({}))
    }

    fn brushes(&self) -> u64 {
        self.state()["map"]["brushes"].as_u64().unwrap()
    }

    fn selection_bounds(&self) -> (Vec<f64>, Vec<f64>) {
        let b = &self.state()["selection"]["bounds"];
        let v = |k: &str| b[k].as_array().unwrap().iter().map(|x| x.as_f64().unwrap()).collect::<Vec<_>>();
        (v("min"), v("max"))
    }

    /// Saves a screenshot and returns (width, height, distinct colors sampled).
    fn screenshot(&self, target: &str, label: &str) -> (u32, u32, usize) {
        let r = self.try_rpc("tools/call", json!({ "name": "screenshot", "arguments": { "target": target } })).unwrap();
        let data = r["result"]["content"][0]["data"].as_str().expect("image content");
        let png = base64::engine::general_purpose::STANDARD.decode(data).unwrap();
        std::fs::write(artifacts().join(format!("{}_{label}.png", self.name)), &png).unwrap();
        let img = image::load_from_memory(&png).unwrap().to_rgba8();
        let mut colors = std::collections::HashSet::new();
        for (x, y, p) in img.enumerate_pixels() {
            if x % 7 == 0 && y % 7 == 0 {
                colors.insert(p.0);
            }
        }
        (img.width(), img.height(), colors.len())
    }

    fn box_brush(&self, min: [f64; 3], max: [f64; 3]) -> u64 {
        self.call("create_brush", json!({ "min": min, "max": max }))["ids"][0].as_u64().unwrap()
    }
}

fn approx(a: &[f64], b: &[f64]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-3)
}

#[test]
#[ignore]
fn draw_select_move_delete_undo() {
    let ed = Editor::launch("basic");
    // Draw a brush by dragging in the top view. Depth comes from the default 0..64 range.
    ed.input("top", json!([{ "type": "drag", "world": [-64, 0, -64], "to_world": [64, 0, 32] }]));
    assert_eq!(ed.brushes(), 1);
    let (min, max) = ed.selection_bounds();
    assert!(approx(&min, &[-64.0, 0.0, -64.0]) && approx(&max, &[64.0, 64.0, 32.0]), "{min:?} {max:?}");

    // Click empty space deselects, clicking the brush selects it again.
    ed.input("top", json!([{ "type": "click", "world": [400, 0, 200] }]));
    assert!(ed.state()["selection"]["nodes"].as_array().unwrap().is_empty());
    ed.input("front", json!([{ "type": "click", "world": [0, 32, 0] }]));
    assert_eq!(ed.state()["selection"]["nodes"].as_array().unwrap().len(), 1);

    // Move by dragging the selection in the front view.
    ed.input("front", json!([{ "type": "drag", "world": [0, 32, 0], "to_world": [96, 32, 0] }]));
    let (min, _) = ed.selection_bounds();
    assert!(approx(&min, &[32.0, 0.0, -64.0]), "moved min {min:?}");

    // Ctrl+drag duplicates.
    ed.input("front", json!([{ "type": "drag", "world": [96, 32, 0], "to_world": [96, 128, 0], "modifiers": ["ctrl"] }]));
    assert_eq!(ed.brushes(), 2);

    // Delete key removes the selection, Ctrl+Z brings it back.
    ed.input("top", json!([{ "type": "move", "world": [0, 0, 0] }, { "type": "key", "key": "Delete" }]));
    assert_eq!(ed.brushes(), 1);
    ed.input("window", json!([{ "type": "key", "key": "Z", "modifiers": ["ctrl"] }]));
    assert_eq!(ed.brushes(), 2);

    let (w, h, colors) = ed.screenshot("window", "final");
    assert!(w > 800 && h > 500 && colors > 20);
}

#[test]
#[ignore]
fn resize_edges_and_faces() {
    let ed = Editor::launch("resize");
    ed.box_brush([-32.0, 0.0, -32.0], [32.0, 64.0, 32.0]);
    ed.call("set_camera", json!({ "view": "top", "center": [0, 0, 0], "zoom": 1.5 }));
    // Drag the right edge (+X) in the top view.
    ed.input("top", json!([{ "type": "drag", "world": [32, 0, 0], "to_world": [96, 0, 0] }]));
    let (min, max) = ed.selection_bounds();
    assert!(approx(&min, &[-32.0, 0.0, -32.0]) && approx(&max, &[96.0, 64.0, 32.0]), "edge resize {min:?} {max:?}");

    // Shift+drag the top face upwards in the 3D view.
    ed.call("set_camera", json!({ "view": "3d", "position": [-200, 250, 220], "look_at": [32, 32, 0] }));
    ed.input("3d", json!([{ "type": "drag", "world": [32, 64, 0], "to_world": [32, 128, 0], "modifiers": ["shift"], "steps": 12 }]));
    let (_, max) = ed.selection_bounds();
    assert!(max[1] > 64.0, "face drag should raise the top, got {max:?}");
    ed.screenshot("3d", "after_face_drag");
}

#[test]
#[ignore]
fn clip_vertex_rotate_tools() {
    let ed = Editor::launch("tools");
    ed.box_brush([-64.0, 0.0, -32.0], [64.0, 64.0, 32.0]);
    ed.call("set_camera", json!({ "view": "front", "center": [0, 32, 0], "zoom": 1.5 }));

    // Clip tool: two points on the x=16 line in the front view, Enter keeps the front side.
    ed.input("window", json!([{ "type": "key", "key": "C" }]));
    assert_eq!(ed.state()["editor"]["tool"], "Clip");
    ed.input("front", json!([{ "type": "click", "world": [16, -16, 0] }, { "type": "click", "world": [16, 96, 0] }]));
    assert_eq!(ed.state()["editor"]["clip_points"].as_array().unwrap().len(), 2);
    ed.screenshot("front", "clip_preview");
    ed.input("front", json!([{ "type": "key", "key": "Enter" }]));
    assert_eq!(ed.brushes(), 1);
    let (min, max) = ed.selection_bounds();
    let width = max[0] - min[0];
    assert!((width - 48.0).abs() < 1e-3 || (width - 80.0).abs() < 1e-3, "clip width {width}");

    // Vertex tool: drag the top-right vertex up by 32 in the front view.
    ed.call("set_editor", json!({ "tool": "vertex" }));
    let (min, max) = ed.selection_bounds();
    let corner = [max[0], max[1], max[2]];
    ed.input("front", json!([{ "type": "drag", "world": corner, "to_world": [corner[0], corner[1] + 32.0, corner[2]] }]));
    let (_, new_max) = ed.selection_bounds();
    assert!((new_max[1] - (max[1] + 32.0)).abs() < 1e-3, "vertex move {new_max:?}");
    let node = ed.call("list_nodes", json!({ "type": "brush" }))["nodes"][0]["id"].as_u64().unwrap();
    let brush = ed.call("get_node", json!({ "id": node }));
    assert!(brush["faces"].as_array().unwrap().len() >= 6);
    let _ = min;

    // Rotate 90 degrees about Y with the menu action swaps the X and Z extents.
    ed.call("set_editor", json!({ "tool": "select" }));
    let (min, max) = ed.selection_bounds();
    let (sx, sz) = (max[0] - min[0], max[2] - min[2]);
    ed.call("run_action", json!({ "action": "rotate", "args": { "axis": "y", "degrees": 90 } }));
    let (min, max) = ed.selection_bounds();
    assert!(((max[0] - min[0]) - sz).abs() < 1e-3 && ((max[2] - min[2]) - sx).abs() < 1e-3);
    assert_eq!(ed.call("validate_map", json!({}))["count"], 0, "map should have no issues");
}

#[test]
#[ignore]
fn csg_entities_io_and_files() {
    let ed = Editor::launch("csg");
    ed.box_brush([-64.0, 0.0, -64.0], [64.0, 128.0, 64.0]);
    let cutter = ed.box_brush([-16.0, -16.0, -128.0], [16.0, 64.0, 128.0]);
    ed.call("select", json!({ "ids": [cutter] }));
    ed.call("run_action", json!({ "action": "csg_subtract" }));
    assert_eq!(ed.brushes(), 3, "door cut leaves left, right and top pieces");

    ed.call("run_action", json!({ "action": "select_all" }));
    ed.call("run_action", json!({ "action": "group" }));
    assert_eq!(ed.call("list_nodes", json!({ "type": "group" }))["total"], 1);
    ed.call("run_action", json!({ "action": "ungroup" }));
    assert_eq!(ed.call("list_nodes", json!({ "type": "group" }))["total"], 0);

    // Brush entity with I/O pointing at a light that does not exist yet.
    let door = ed.box_brush([-16.0, 0.0, -4.0], [16.0, 64.0, 4.0]);
    ed.call("select", json!({ "ids": [door] }));
    ed.call("run_action", json!({ "action": "create_brush_entity", "args": { "classname": "func_door" } }));
    let door_entity = ed.state()["selection"]["nodes"][0].as_u64().unwrap();
    ed.call("update_entity", json!({ "id": door_entity, "properties": { "targetname": "door1" }, "outputs": [{ "output": "opened", "target": "lamp", "input": "turn_on", "delay": 0.5 }] }));
    let issues = ed.call("validate_map", json!({}));
    assert!(issues["issues"].as_array().unwrap().iter().any(|i| i["code"] == "io_missing_target"));
    ed.call("create_entity", json!({ "classname": "light", "origin": [0, 96, 96], "properties": { "targetname": "lamp" } }));
    let issues = ed.call("validate_map", json!({}));
    assert!(!issues["issues"].as_array().unwrap().iter().any(|i| i["code"] == "io_missing_target"), "{issues}");

    // Save, clear, reopen.
    let path = artifacts().join("csg_roundtrip.gtm");
    ed.call("map_file", json!({ "op": "save", "path": path }));
    let before = ed.state()["map"].clone();
    ed.call("map_file", json!({ "op": "new" }));
    assert_eq!(ed.brushes(), 0);
    ed.call("map_file", json!({ "op": "open", "path": path }));
    let after = ed.state()["map"].clone();
    assert_eq!(before["brushes"], after["brushes"]);
    assert_eq!(before["entities"], after["entities"]);
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("\"format\": \"godottrench-map\"") && text.contains("\"opened\""));

    assert!(ed.call_err("get_node", json!({ "id": 999999 })).contains("no node"));
    let (_, _, colors) = ed.screenshot("3d", "csg");
    assert!(colors > 8, "3d view should show geometry");
}

#[test]
#[ignore]
fn prefabs_and_command_palette() {
    use gt_doc::{Entity, Map, NodeKind, format};
    let ed = Editor::launch("prefabs");
    let dir = artifacts().join("prefabs");
    std::fs::create_dir_all(&dir).unwrap();

    let mut prefab = Map::new();
    let layer = prefab.default_layer();
    prefab.insert(
        layer,
        NodeKind::Brush(gt_geom::Brush::from_aabb(&gt_core::Aabb::new(gt_core::DVec3::ZERO, gt_core::DVec3::new(64.0, 32.0, 16.0)), "dev/blue").unwrap()),
    );
    let mut light = Entity::new("light");
    light.origin = gt_core::DVec3::new(32.0, 48.0, 0.0);
    light.properties.insert("targetname".into(), "plight".into());
    prefab.insert(layer, NodeKind::Entity(light));
    format::save(&prefab, &dir.join("crate.gtm")).unwrap();

    ed.call("map_file", json!({ "op": "save", "path": dir.join("main.gtm") }));
    let inserted = ed.call(
        "run_action",
        json!({ "action": "insert_prefab", "args": { "path": dir.join("crate.gtm"), "origin": [100, 0, 0], "angles": [0, 90, 0], "fixup": "a" } }),
    );
    assert_eq!(inserted["path"], "crate.gtm", "prefab path stored relative to the map");
    // The scene cache resolves instance bounds on the next frame.
    ed.input("window", json!([{ "type": "move", "x": 10, "y": 10 }]));
    let inst = &ed.call("list_nodes", json!({ "type": "instance" }))["nodes"][0];
    let min: Vec<f64> = inst["bounds"]["min"].as_array().unwrap().iter().map(|v| v.as_f64().unwrap()).collect();
    let max: Vec<f64> = inst["bounds"]["max"].as_array().unwrap().iter().map(|v| v.as_f64().unwrap()).collect();
    // Brush rotated 90° spans x 100..116, z -64..0. The light's 16 unit box at (100, 48, -32) widens x and y.
    assert!(approx(&min, &[92.0, 0.0, -64.0]) && approx(&max, &[116.0, 56.0, 0.0]), "rotated instance bounds {min:?} {max:?}");
    ed.screenshot("3d", "instance");

    ed.call("run_action", json!({ "action": "explode_instances" }));
    assert_eq!(ed.brushes(), 1);
    let lights = ed.call("list_nodes", json!({ "classname": "light" }));
    assert_eq!(lights["nodes"][0]["name"], "light (a-plight)", "fixup applied to exploded names");

    // Command palette: F1, type, Enter.
    ed.input("window", json!([{ "type": "key", "key": "F1" }, { "type": "text", "text": "grid larger" }, { "type": "key", "key": "Enter" }]));
    assert_eq!(ed.state()["editor"]["grid"], 32.0);
}

#[test]
#[ignore]
fn mesh_tool_extrude_with_keys() {
    let ed = Editor::launch("mesh");
    let id = ed.call("create_mesh", json!({ "shape": "cuboid", "min": [-32, 0, -32], "max": [32, 64, 32] }))["id"].as_u64().unwrap();
    assert_eq!(ed.state()["map"]["meshes"], 1);
    ed.call("set_editor", json!({ "tool": "mesh", "mesh_component": "face" }));
    ed.call("set_camera", json!({ "view": "3d", "position": [-120, 200, 160], "look_at": [0, 32, 0] }));

    // Blender style: click the top face, E extrudes into a grab, Y constrains, typed distance, Enter confirms.
    ed.input("3d", json!([{ "type": "click", "world": [0, 64, 0] }]));
    ed.input("3d", json!([{ "type": "key", "key": "E" }, { "type": "key", "key": "Y" }, { "type": "text", "text": "32" }, { "type": "key", "key": "Enter" }]));
    let mesh = ed.call("get_node", json!({ "id": id }));
    assert_eq!(mesh["faces"].as_array().unwrap().len(), 10, "extruding one face of a cube adds four side faces");
    let (_, max) = ed.selection_bounds();
    assert!((max[1] - 96.0).abs() < 1e-3, "extruded top {max:?}");
    ed.screenshot("3d", "extruded");

    // Undo restores the cube, the topology API works on the same node.
    ed.input("window", json!([{ "type": "key", "key": "Z", "modifiers": ["ctrl"] }]));
    ed.input("window", json!([{ "type": "key", "key": "Z", "modifiers": ["ctrl"] }]));
    assert_eq!(ed.call("get_node", json!({ "id": id }))["faces"].as_array().unwrap().len(), 6);
    let cut = ed.call("mesh_edit", json!({ "id": id, "op": "bisect", "point": [0, 16, 0], "normal": [0, 1, 0], "delete": "back", "fill": true }));
    assert_eq!(cut["closed"], true, "bisect with fill keeps the mesh closed");
    let r = ed.call("mesh_edit", json!({ "id": id, "op": "solidify", "thickness": 4 }));
    assert!(r["face_count"].as_u64().unwrap() >= 12);
    assert!(ed.call_err("mesh_edit", json!({ "id": id, "op": "nope" })).contains("unknown"));

    let cube = ed.call("create_mesh", json!({ "shape": "cuboid", "min": [100, 0, 0], "max": [164, 64, 64] }))["id"].as_u64().unwrap();
    ed.call("select", json!({ "ids": [cube] }));
    let brushes = ed.brushes();
    ed.call("run_action", json!({ "action": "convert_to_brushes" }));
    assert_eq!(ed.brushes(), brushes + 1);
    assert_eq!(ed.state()["map"]["meshes"], 1, "only the convex cube was converted");
}

#[test]
#[ignore]
fn terrain_sprinkle_cordon_tabs_bookmarks() {
    let ed = Editor::launch("terrain");
    let t = ed.call("create_terrain", json!({ "origin": [-512, 0, -512], "resolution": 33, "cell_size": 32, "shape": "flat" }));
    let id = t["id"].as_u64().unwrap();
    assert_eq!(ed.state()["map"]["terrains"], 1);
    let sculpt = ed.call("run_action", json!({ "action": "sculpt", "args": { "center": [0, 0, 0], "mode": "raise", "radius": 128, "strength": 64 } }));
    assert_eq!(sculpt["changed"], true);
    let (_, max) = ed.selection_bounds();
    assert!(max[1] > 32.0, "raised terrain {max:?}");

    // Sculpt tool with real mouse drags over the terrain.
    ed.call("set_editor", json!({ "tool": "sculpt", "sculpt_mode": "raise", "brush_radius": 96 }));
    ed.call("set_camera", json!({ "view": "3d", "position": [-600, 500, 600], "look_at": [200, 0, -200] }));
    let rev = ed.state()["map"]["revision"].as_u64().unwrap();
    ed.input("3d", json!([{ "type": "drag", "world": [200, 0, -200], "to_world": [300, 0, -300], "steps": 10 }]));
    assert!(ed.state()["map"]["revision"].as_u64().unwrap() > rev, "sculpt drag edits the terrain");
    ed.call("set_editor", json!({ "shade": "lit", "tool": "select" }));
    let (_, _, colors) = ed.screenshot("3d", "terrain_lit");
    assert!(colors > 8);

    let placed = ed.call("run_action", json!({ "action": "sprinkle", "args": { "center": [-300, 0, 300], "items": ["info_player_start"], "radius": 160, "density": 2, "min_spacing": 24, "seed": 3 } }));
    let count = placed["ids"].as_array().unwrap().len();
    assert!(count >= 3, "sprinkle placed {count}");
    let first = ed.call("get_node", json!({ "id": placed["ids"][0] }));
    let y = first["origin"][1].as_f64().unwrap();
    assert!(y.abs() < 1.0, "sprinkled onto the flat part of the terrain, got {y}");

    ed.call("select", json!({ "ids": [id] }));
    ed.call("run_action", json!({ "action": "set_cordon" }));
    assert_eq!(ed.state()["map"]["cordon_enabled"], true);
    ed.call("run_action", json!({ "action": "clear_cordon" }));
    assert!(ed.state()["map"]["cordon"].is_null());

    ed.call("run_action", json!({ "action": "store_camera", "args": { "slot": 2 } }));
    assert_eq!(ed.state()["map"]["cameras"], json!([2]));
    ed.call("set_camera", json!({ "view": "3d", "position": [0, 50, 0], "look_at": [10, 50, 0] }));
    ed.call("run_action", json!({ "action": "recall_camera", "args": { "slot": 2 } }));
    let cameras = ed.state()["cameras"].clone();
    let cam = cameras.as_array().unwrap().iter().find(|v| v["view"] == "3d").unwrap().clone();
    assert!((cam["position"][0].as_f64().unwrap() + 600.0).abs() < 1.0, "recalled camera {cam}");

    ed.call("run_action", json!({ "action": "new_tab" }));
    let tabs = ed.state()["tabs"].clone();
    assert_eq!(tabs["titles"].as_array().unwrap().len(), 2);
    assert_eq!(ed.state()["map"]["terrains"], 0, "new tab starts empty");
    ed.call("run_action", json!({ "action": "next_tab" }));
    assert_eq!(ed.state()["map"]["terrains"], 1, "switching back keeps the first map");
}

fn texel(uv: &Value, p: [f64; 3]) -> [f64; 2] {
    let axis = |k: &str| -> [f64; 3] {
        let a = uv[k].as_array().unwrap();
        [a[0].as_f64().unwrap(), a[1].as_f64().unwrap(), a[2].as_f64().unwrap()]
    };
    let (u, v) = (axis("u_axis"), axis("v_axis"));
    let s = |i: usize| uv["scale"][i].as_f64().unwrap();
    let o = |i: usize| uv["offset"][i].as_f64().unwrap();
    let dot = |a: [f64; 3]| a[0] * p[0] + a[1] * p[1] + a[2] * p[2];
    [dot(u) / s(0) + o(0), dot(v) / s(1) + o(1)]
}

#[test]
#[ignore]
fn texture_tool_apply_wrap_slide_scale() {
    let ed = Editor::launch("texture");
    let brush = ed.box_brush([0.0, 0.0, 0.0], [64.0, 64.0, 64.0]);
    let all = ed.call("texture", json!({ "op": "get", "faces": (0..6).map(|f| json!([brush, f])).collect::<Vec<_>>() }));
    let face_with = |n: [f64; 3]| {
        let found = all["faces"].as_array().unwrap().iter().find(|f| (0..3).all(|i| (f["normal"][i].as_f64().unwrap() - n[i]).abs() < 1e-6)).unwrap();
        found["face"][1].as_u64().unwrap()
    };
    let (front, left) = (face_with([0.0, 0.0, -1.0]), face_with([-1.0, 0.0, 0.0]));
    ed.call("set_camera", json!({ "view": "3d", "position": [-110, 110, -150], "look_at": [32, 32, 32] }));
    ed.call("set_editor", json!({ "tool": "texture", "material": "dev/orange" }));
    assert_eq!(ed.state()["editor"]["tool"], "Texture");

    // Click selects the front face, right click on the left face applies the current material there.
    ed.input("3d", json!([{ "type": "click", "world": [32, 32, 0] }]));
    assert_eq!(ed.state()["selection"]["faces"], json!([[brush, front]]));
    ed.input("3d", json!([{ "type": "click", "world": [0, 32, 32], "button": "right" }]));
    let get = |f: u64| ed.call("texture", json!({ "op": "get", "faces": [[brush, f]] }))["faces"][0].clone();
    assert_eq!(get(left)["material"], "dev/orange");
    assert_ne!(get(front)["material"], "dev/orange", "right click does not touch the selection");

    // Dragging the selected face slides its texture by the dragged distance.
    let before = get(front)["uv"]["offset"][0].as_f64().unwrap();
    ed.input("3d", json!([{ "type": "drag", "world": [32, 32, 0], "to_world": [48, 32, 0], "steps": 10 }]));
    let after = get(front)["uv"]["offset"][0].as_f64().unwrap();
    assert!(((after - before).abs() - 16.0).abs() <= 1.0, "slid by {}", after - before);

    // Alt+right click wraps the selected face's alignment around the corner.
    ed.input("3d", json!([{ "type": "click", "world": [0, 32, 32], "button": "right", "modifiers": ["alt"] }]));
    let (f_uv, l_uv) = (get(front)["uv"].clone(), get(left)["uv"].clone());
    for y in [0.0, 20.0, 64.0] {
        let (a, b) = (texel(&f_uv, [0.0, y, 0.0]), texel(&l_uv, [0.0, y, 0.0]));
        assert!((a[0] - b[0]).abs() < 1e-6 && (a[1] - b[1]).abs() < 1e-6, "seamless at the corner: {a:?} {b:?}");
    }

    // Ctrl+wheel doubles the texture size of the selected face.
    let scale = f_uv["scale"][0].as_f64().unwrap().abs();
    ed.input("3d", json!([{ "type": "scroll", "world": [32, 32, 0], "delta": 120, "modifiers": ["ctrl"] }]));
    assert!((get(front)["uv"]["scale"][0].as_f64().unwrap().abs() - scale * 2.0).abs() < 1e-9);

    // Alt+click picks material and alignment, justify aligns the texture to the face edge.
    ed.input("3d", json!([{ "type": "click", "world": [0, 32, 32], "modifiers": ["alt"] }]));
    assert_eq!(ed.state()["editor"]["material"], "dev/orange");
    let justified = ed.call("texture", json!({ "op": "justify", "mode": "left", "faces": [[brush, front]] }));
    assert_eq!(justified["changed"], 1);
    let uv = get(front)["uv"].clone();
    let min_u = [[0.0, 0.0, 0.0], [64.0, 0.0, 0.0]].iter().map(|p| texel(&uv, *p)[0]).fold(f64::MAX, f64::min);
    assert!(min_u.abs() < 1e-6, "left justified, min u {min_u}");
    ed.screenshot("3d", "texture_tool");

    // Mesh UV projections write explicit UVs, hotspots need a texture file.
    let mesh = ed.call("create_mesh", json!({ "shape": "cylinder", "min": [128, 0, 0], "max": [192, 64, 64] }))["id"].as_u64().unwrap();
    let projected = ed.call("texture", json!({ "op": "mesh_uv", "kind": "cylinder" }));
    assert!(projected["changed"].as_u64().unwrap() >= 16);
    assert!(ed.call("texture", json!({ "op": "get", "faces": [[mesh, 2]] }))["faces"][0]["explicit"].is_array());
    assert!(ed.call_err("texture", json!({ "op": "set_hotspots", "material": "dev/grey", "rects": [[0, 0, 64, 64]] })).contains("no texture file"));
    ed.call("set_editor", json!({ "shade": "lit" }));
    let (_, _, colors) = ed.screenshot("3d", "lit_sky");
    assert!(colors > 20, "lit view with sky");
}

/// Replays the showcase maps in examples/mcp against the demo project, the same way a user or an LLM would build them.
#[test]
#[ignore]
fn example_mcp_scripts_replay() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let project = root.join("godot").canonicalize().unwrap();
    let ed = Editor::launch_with("examples", &["--project", project.to_str().unwrap()]);
    let out = artifacts().join("examples");
    std::fs::create_dir_all(&out).unwrap();

    // Array duplicate around a pivot places each copy one step further.
    let id = ed.box_brush([96.0, 0.0, -8.0], [112.0, 16.0, 8.0]);
    let copies = ed.call("duplicate", json!({ "ids": [id], "offset": [0, 0, 0], "rotate_y": 90, "pivot": [0, 0, 0], "count": 3 }));
    let brushes = ed.call("list_nodes", json!({ "type": "brush" }));
    let centers: Vec<[i64; 2]> = copies["copies"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            let node = brushes["nodes"].as_array().unwrap().iter().find(|n| n["id"] == c[0]).expect("copy listed");
            let b = node["bounds"].clone();
            let mid = |i: usize| ((b["min"][i].as_f64().unwrap() + b["max"][i].as_f64().unwrap()) * 0.5).round() as i64;
            [mid(0), mid(2)]
        })
        .collect();
    assert_eq!(centers, vec![[0, -104], [-104, 0], [0, 104]], "copies rotate 90, 180 and 270 degrees");

    for (script, min_brushes, min_entities) in [("mountain_house", 60, 10), ("church_school", 250, 20), ("lighthouse_forest", 100, 20)] {
        let path = root.join("examples/mcp").join(format!("{script}.json")).canonicalize().unwrap();
        let summary = ed.call("run_script", json!({ "path": path, "vars": { "save_dir": out } }));
        assert_eq!(summary["errors"], json!([]), "{script}");
        assert_eq!(summary["ran"], summary["steps"], "{script}");
        let map = ed.state()["map"].clone();
        assert!(map["brushes"].as_u64().unwrap() >= min_brushes, "{script}: {map}");
        assert!(map["entities"].as_u64().unwrap() >= min_entities, "{script}: {map}");
        assert_eq!(map["cameras"], json!([1, 2, 3]), "{script}");

        let scatters = ed.call("list_nodes", json!({ "type": "scatter" }));
        assert!(scatters["total"].as_u64().unwrap() >= 2, "{script}: scatter sets");
        let issues = ed.call("validate_map", json!({}));
        let errors: Vec<&Value> = issues["issues"].as_array().unwrap().iter().filter(|i| i["severity"] == "error").collect();
        assert!(errors.is_empty(), "{script}: {errors:?}");

        let saved = out.join(format!("{script}.gtm"));
        let reloaded = gt_doc::format::load(&saved).unwrap_or_else(|e| panic!("{script}: {e}"));
        assert_eq!(reloaded.scatters().count() as u64, scatters["total"].as_u64().unwrap());

        ed.call("run_action", json!({ "action": "recall_camera", "args": { "slot": 1 } }));
        ed.call("set_editor", json!({ "shade": "lit" }));
        ed.input("window", json!([{ "type": "move", "x": 10, "y": 10 }]));
        let (_, _, colors) = ed.screenshot("3d", script);
        assert!(colors > 40, "{script}: lit overview has {colors} colors");
    }
}

#[test]
#[ignore]
fn keyboard_shortcuts_and_views() {
    let ed = Editor::launch("keys");
    ed.input("window", json!([{ "type": "key", "key": "CloseBracket" }]));
    assert_eq!(ed.state()["editor"]["grid"], 32.0);
    ed.input("window", json!([{ "type": "key", "key": "OpenBracket" }, { "type": "key", "key": "OpenBracket" }]));
    assert_eq!(ed.state()["editor"]["grid"], 8.0);
    ed.box_brush([0.0, 0.0, 0.0], [32.0, 32.0, 32.0]);
    ed.box_brush([64.0, 0.0, 0.0], [96.0, 32.0, 32.0]);
    ed.input("top", json!([{ "type": "move" }, { "type": "key", "key": "A", "modifiers": ["ctrl"] }]));
    assert_eq!(ed.state()["selection"]["nodes"].as_array().unwrap().len(), 2);
    ed.input("window", json!([{ "type": "key", "key": "H", "modifiers": ["ctrl"] }]));
    assert!(ed.call("list_nodes", json!({ "type": "brush" }))["nodes"].as_array().unwrap().iter().all(|n| n["hidden"] == true));
    ed.input("window", json!([{ "type": "key", "key": "H", "modifiers": ["ctrl", "shift"] }]));
    assert!(ed.call("list_nodes", json!({ "type": "brush" }))["nodes"].as_array().unwrap().iter().all(|n| n["hidden"] == false));
    for view in ["3d", "top", "front", "side"] {
        let (w, h, _) = ed.screenshot(view, view);
        assert!(w > 50 && h > 50);
    }

    let ppp = |ed: &Editor| ed.state()["ui"]["pixels_per_point"].as_f64().unwrap();
    let base = ppp(&ed);
    ed.input("window", json!([{ "type": "key", "key": "Equals", "modifiers": ["ctrl"] }, { "type": "key", "key": "Equals", "modifiers": ["ctrl"] }]));
    ed.input("window", json!([{ "type": "move", "x": 5, "y": 5 }]));
    assert_eq!(ed.state()["ui"]["scale"], 1.2);
    assert!((ppp(&ed) / base - 1.2).abs() < 0.01, "pixels per point {} from {base}", ppp(&ed));
    ed.input("window", json!([{ "type": "key", "key": "0", "modifiers": ["ctrl"] }]));
    assert_eq!(ed.state()["ui"]["scale"], 1.0);
}
