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

    // The grabbed edge moves in every view, whichever way its axes point on screen.
    ed.input("top", json!([{ "type": "drag", "world": [32, 0, -32], "to_world": [32, 0, -64] }]));
    ed.call("set_camera", json!({ "view": "front", "center": [32, 32, 0], "zoom": 1.5 }));
    ed.input("front", json!([{ "type": "drag", "world": [32, 64, 0], "to_world": [32, 32, 0] }]));
    ed.input("front", json!([{ "type": "drag", "world": [32, 0, 0], "to_world": [32, 16, 0] }]));
    ed.call("set_camera", json!({ "view": "side", "center": [0, 24, 0], "zoom": 1.5 }));
    ed.input("side", json!([{ "type": "drag", "world": [0, 24, 32], "to_world": [0, 24, 64] }]));
    let (min, max) = ed.selection_bounds();
    assert!(approx(&min, &[-32.0, 16.0, -64.0]) && approx(&max, &[96.0, 32.0, 64.0]), "edge resize in all views {min:?} {max:?}");

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
    let opened = ed.call("map_file", json!({ "op": "open", "path": path }));
    assert_eq!(opened["problems"], json!([]));
    let after = ed.state()["map"].clone();
    assert_eq!(before["brushes"], after["brushes"]);
    assert_eq!(before["entities"], after["entities"]);
    let bytes = std::fs::read(&path).unwrap();
    assert!(gt_doc::binary::is_binary(&bytes), "maps are saved in the binary container");
    let (text, _) = gt_doc::format::file_to_json(&bytes).unwrap();
    assert!(text.contains("\"format\": \"godottrench-map\"") && text.contains("\"opened\""));

    // A damaged save still opens, with what was lost reported.
    let damaged = artifacts().join("csg_damaged.gtm");
    std::fs::write(&damaged, &bytes[..bytes.len() - 8]).unwrap();
    let opened = ed.call("map_file", json!({ "op": "open", "path": damaged }));
    assert!(opened["problems"].as_array().is_some_and(|p| !p.is_empty()), "{opened}");
    assert_eq!(ed.state()["map"]["brushes"], after["brushes"]);

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

    let r = ed.call(
        "run_action",
        json!({ "action": "create_decal", "args": { "material": "base/wall", "at": [100, 32, 32], "normal": [-1, 0, 0], "size": [48, 24] } }),
    );
    let decal = ed.call("get_node", json!({ "id": r["selection"][0] }));
    assert_eq!(decal["decal"], true);
    let (min, max) = ed.selection_bounds();
    assert!(approx(&[max[1] - min[1], max[2] - min[2]], &[24.0, 48.0]), "decal spans its size on the wall {min:?} {max:?}");
    assert!(ed.call_err("run_action", json!({ "action": "create_decal", "args": { "at": [0, 0, 0] } })).contains("material"));
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
    ed.call("run_action", json!({ "action": "select_none" }));
    assert_eq!(ed.state()["selection"]["nodes"], json!([]), "select_none empties the selection with the Sculpt tool active");
    assert_eq!(ed.state()["editor"]["tool"], "Sculpt", "and leaves the tool alone");
    ed.call("terrain_edit", json!({ "id": id, "op": "set_layers", "layers": [["dev/green", 256], ["dev/grey", 256], ["dev/blue", 256], ["dev/orange", 256]] }));
    let low_weight = |sea_level: Value| {
        ed.call("terrain_edit", json!({ "id": id, "op": "auto_paint", "sea_level": sea_level }));
        ed.call("blend", json!({ "op": "weights", "id": id, "center": [-500, 0, -500] }))["weights"][3].as_f64().unwrap()
    };
    let (from_floor, from_sea) = (low_weight(Value::Null), low_weight(json!(40)));
    assert!(from_floor < 0.5 && from_sea > 0.8, "flat ground below sea level joins the low band only when bands start at sea level: {from_floor} {from_sea}");

    // clear_layer wipes a layer's paint everywhere, so a script that repaints an area gives the same result however
    // many times it runs.
    let weight_at = |id: u64, p: [f64; 3]| ed.call("blend", json!({ "op": "weights", "id": id, "center": p }))["weights"][3].as_f64().unwrap();
    assert_eq!(ed.call("terrain_edit", json!({ "id": id, "op": "clear_layer", "layer": 3 }))["changed"], true);
    assert_eq!(weight_at(id, [-500.0, 0.0, -500.0]), 0.0, "layer 3 cleared everywhere");

    // A rectangular hole for scripts that need a precise cut rather than a circular brush.
    let holed = ed.call("terrain_edit", json!({ "id": id, "op": "holes", "min": [-40, -40], "max": [40, 40], "hole": true }));
    assert_eq!(holed["changed"], true);
    ed.call("terrain_edit", json!({ "id": id, "op": "holes", "min": [-40, -40], "max": [40, 40], "hole": false }));

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

    for (script, min_brushes, min_entities) in [
        ("mountain_house", 60, 10),
        ("church_school", 250, 20),
        ("lighthouse_forest", 100, 20),
        ("night_district", 150, 30),
        ("withered_city", 600, 40),
        ("sea_island", 16, 6),
    ] {
        // Outputs may target Godot overlay nodes, which the editor only knows from the sidecar next to the saved map.
        let sidecar = root.join("godot/demo/maps/showcase").join(format!("{script}.overlay.json"));
        if sidecar.is_file() {
            std::fs::copy(&sidecar, out.join(format!("{script}.overlay.json"))).unwrap();
        }

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
        assert!(reloaded.problems.is_empty(), "{script}: {:?}", reloaded.problems);
        assert_eq!(reloaded.map.scatters().count() as u64, scatters["total"].as_u64().unwrap());

        ed.call("run_action", json!({ "action": "recall_camera", "args": { "slot": 1 } }));
        ed.call("set_editor", json!({ "shade": "lit" }));
        ed.input("window", json!([{ "type": "move", "x": 10, "y": 10 }]));
        let (_, _, colors) = ed.screenshot("3d", script);
        assert!(colors > 40, "{script}: lit overview has {colors} colors");
    }

    // The night map's roller door keeps the beacon outputs passed to make_door next to the ones the wizard wires.
    let night = gt_doc::format::load(&out.join("night_district.gtm")).unwrap().map;
    let (_, garage) = night.entities().find(|(_, e)| e.targetname() == Some("garage_door")).expect("garage door in the night map");
    assert_eq!(garage.outputs.iter().filter(|o| o.target == "garage_beacon").count(), 4, "{:?}", garage.outputs);

    // A bad scatter item is reported instead of leaving an empty palette.
    let e = ed.call_err("scatter", json!({ "op": "new_set", "name": "bad", "items": [{ "source": "res://x.glb", "align": true }] }));
    assert!(e.contains("items[0]"), "{e}");

    // Screenshots can render the 3D camera larger than its docked pane.
    let r = ed.try_rpc("tools/call", json!({ "name": "screenshot", "arguments": { "target": "3d", "width": 640, "height": 360 } })).unwrap();
    let png = base64::engine::general_purpose::STANDARD.decode(r["result"]["content"][0]["data"].as_str().expect("image content")).unwrap();
    assert_eq!(image::load_from_memory(&png).unwrap().to_rgba8().dimensions(), (640, 360));
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

#[test]
#[ignore]
fn vertex_gizmo_moves_a_vertex_along_one_axis() {
    let ed = Editor::launch("vertex_gizmo");
    let id = ed.box_brush([-32.0, -32.0, -32.0], [32.0, 32.0, 32.0]);
    ed.call("select", json!({ "ids": [id] }));
    ed.call("set_editor", json!({ "tool": "vertex" }));
    ed.call("set_camera", json!({ "view": "3d", "position": [190.0, 150.0, 210.0], "look_at": [0.0, 0.0, 0.0] }));

    // Double click the +x+y+z corner to attach the precise-move gizmo, then drag its x arrow by +32.
    ed.input("3d", json!([{ "type": "double_click", "world": [32.0, 32.0, 32.0] }]));
    ed.screenshot("3d", "vertex_gizmo");
    let (_, before) = ed.selection_bounds();
    ed.input("3d", json!([{ "type": "drag", "world": [50.0, 32.0, 32.0], "to_world": [82.0, 32.0, 32.0], "steps": 10 }]));
    let (_, after) = ed.selection_bounds();
    assert!((after[0] - 64.0).abs() < 2.0, "x corner should move +32 to 64 via the gizmo, before {before:?} after {after:?}");
    assert!((after[1] - before[1]).abs() < 1e-3 && (after[2] - before[2]).abs() < 1e-3, "only x should change, after {after:?}");
}

#[test]
#[ignore]
fn scatter_materials_and_scattering_onto_a_scatter() {
    let ed = Editor::launch("scatter_materials");
    ed.box_brush([-512.0, -32.0, -512.0], [512.0, 0.0, 512.0]);
    let rocks = ed.call("scatter", json!({ "op": "paint", "preset": "rocks", "center": [0, 0], "radius": 420, "density": 3, "seed": 4 }));
    let rocks_id = rocks["set"]["id"].as_u64().unwrap();
    assert!(rocks["placed"].as_u64().unwrap() > 0, "rocks were painted");

    // A material on the set retextures every model in it.
    ed.call("scatter", json!({ "op": "material", "id": rocks_id, "material": "showcase/snow" }));
    let node = ed.call("get_node", json!({ "id": rocks_id }));
    assert_eq!(node["material"], "showcase/snow");
    assert!(node["items"][0].get("material").is_none(), "no entry override yet");

    // A material on one palette entry wins for that entry only.
    ed.call("scatter", json!({ "op": "material", "id": rocks_id, "item": 0, "material": "showcase/gold" }));
    let node = ed.call("get_node", json!({ "id": rocks_id }));
    assert_eq!(node["items"][0]["material"], "showcase/gold", "entry 0 is retextured");
    assert!(node["items"][1].get("material").is_none(), "entry 1 still follows the set");
    assert_eq!(node["material"], "showcase/snow", "the set keeps its own material");
    let entries = rocks["set"]["items"].as_array().unwrap().len();
    assert_eq!(ed.call_err("scatter", json!({ "op": "material", "id": rocks_id, "item": 9 })), format!("the set has no palette entry 9, it has {entries}"));

    // Grass that targets the rock set lands on the rocks, above the floor they stand on.
    let grass = ed.call("scatter", json!({ "op": "new_set", "preset": "grass", "name": "on_rocks", "targets": [rocks_id] }));
    let grass_id = grass["id"].as_u64().unwrap();
    let painted = ed.call("scatter", json!({ "op": "paint", "id": grass_id, "center": [0, 0], "radius": 420, "density": 12, "seed": 7 }));
    assert!(painted["placed"].as_u64().unwrap() > 0, "grass landed on the rocks");
    let blades = ed.call("get_node", json!({ "id": grass_id }));
    let above = blades["instances"].as_array().unwrap().iter().filter(|i| i[2].as_f64().unwrap() > 0.0).count();
    assert_eq!(above, blades["instances"].as_array().unwrap().len(), "every blade sits on a rock, not on the floor");
}

#[test]
#[ignore]
fn the_scatter_tool_paints_grass_onto_scattered_boulders_under_the_mouse() {
    let ed = Editor::launch("scatter_on_scatter_mouse");
    let floor = ed.box_brush([-1024.0, -32.0, -1024.0], [1024.0, 0.0, 1024.0]);
    ed.call("scatter", json!({ "op": "new_set", "name": "boulders", "preset": "boulders", "targets": [floor] }));
    let boulders = ed.call("scatter", json!({ "op": "paint", "center": [0, 0], "radius": 600, "density": 2, "seed": 3 }));
    let boulders_id = boulders["set"]["id"].as_u64().unwrap();
    let rock = ed.call("get_node", json!({ "id": boulders_id }))["instances"][0].clone();
    let (x, z) = (rock[1].as_f64().unwrap(), rock[3].as_f64().unwrap());

    // What a user does: a new grass set from the preset, the scatter tool, and a drag across a boulder.
    let grass = ed.call("scatter", json!({ "op": "new_set", "name": "moss", "preset": "grass" }));
    let grass_id = grass["id"].as_u64().unwrap();
    ed.call("set_editor", json!({ "tool": "scatter" }));
    ed.call("set_camera", json!({ "view": "3d", "position": [x, 420.0, z + 260.0], "look_at": [x, 20.0, z] }));
    ed.input("3d", json!([{ "type": "drag", "world": [x - 20.0, 30.0, z], "to_world": [x + 20.0, 30.0, z], "steps": 10 }]));
    ed.screenshot("3d", "grass_on_boulders");

    let node = ed.call("get_node", json!({ "id": grass_id }));
    assert_eq!(node["targets"], json!([boulders_id]), "the stroke started on a boulder, so the boulders are the target");
    let blades = node["instances"].as_array().unwrap();
    assert!(!blades.is_empty(), "grass was painted");
    assert!(blades.iter().all(|i| i[2].as_f64().unwrap() > 0.0), "every blade sits on a boulder, above the floor");
}

#[test]
#[ignore]
fn mesh_uv_projections_adjustments_and_the_mesh_vertex_gizmo() {
    let ed = Editor::launch("mesh_uv_adjust");
    let mesh = ed.call("create_mesh", json!({ "shape": "cuboid", "min": [0, 0, 0], "max": [64, 64, 64] }))["id"].as_u64().unwrap();
    let uvs = |face: u64| ed.call("texture", json!({ "op": "get", "faces": [[mesh, face]] }))["faces"][0]["explicit"].clone();
    let floats = |v: &Value| v.as_array().unwrap().iter().flat_map(|c| c.as_array().unwrap().iter().map(|x| x.as_f64().unwrap())).collect::<Vec<_>>();
    assert!(uvs(0).is_null(), "a new mesh uses planar projections");
    let err = ed.call_err("texture", json!({ "op": "uv_adjust", "adjust": "flip_u" }));
    assert!(err.contains("no explicit UV corners"), "{err}");

    // Bake writes the planar look as corner UVs, every projection then relays them.
    assert_eq!(ed.call("texture", json!({ "op": "mesh_uv", "kind": "bake" }))["changed"], 6);
    assert!(uvs(0).is_array());
    for kind in ["planar", "planar_x", "planar_y", "planar_z", "box", "cylinder_x", "cylinder", "cylinder_z", "sphere", "view", "unfold", "world"] {
        assert_eq!(ed.call("texture", json!({ "op": "mesh_uv", "kind": kind }))["changed"], 6, "{kind}");
    }

    let start = floats(&uvs(0));
    for _ in 0..2 {
        ed.call("texture", json!({ "op": "uv_adjust", "adjust": "flip_u", "faces": [[mesh, 0]] }));
    }

    assert!(approx(&floats(&uvs(0)), &start), "flipping twice is the identity");
    for _ in 0..4 {
        ed.call("texture", json!({ "op": "uv_adjust", "adjust": "rotate", "degrees": 90, "faces": [[mesh, 0]] }));
    }

    assert!(approx(&floats(&uvs(0)), &start), "four quarter turns are the identity");

    ed.call("texture", json!({ "op": "uv_adjust", "adjust": "fit", "faces": [[mesh, 0]] }));
    let fitted = floats(&uvs(0));
    assert!(fitted.iter().all(|x| (-1e-4..=1.0001).contains(x)), "fit spans the texture once: {fitted:?}");
    assert!(fitted.iter().any(|x| *x < 1e-4) && fitted.iter().any(|x| *x > 0.9999));

    let moved = ed.call("texture", json!({ "op": "uv_adjust", "adjust": "move", "texels": [8, 0], "corners": [[mesh, 0, 0]] }));
    assert_eq!(moved["changed"], 1, "a corner stitched to nothing moves alone");
    ed.call("texture", json!({ "op": "uv_adjust", "adjust": "align_horizontal", "corners": [[mesh, 0, 0], [mesh, 0, 1]] }));
    let aligned = floats(&uvs(0));
    assert!((aligned[1] - aligned[3]).abs() < 1e-5, "corners 0 and 1 share v: {aligned:?}");
    ed.call("texture", json!({ "op": "uv_adjust", "adjust": "snap", "faces": [[mesh, 0]] }));
    assert!(ed.call_err("texture", json!({ "op": "uv_adjust", "adjust": "wobble", "faces": [[mesh, 0]] })).contains("unknown adjust"));
    assert_eq!(ed.call("texture", json!({ "op": "mesh_uv", "kind": "clear" }))["changed"], 6);
    assert!(uvs(0).is_null());

    // In the mesh tool a double click puts the transform gizmo on a vertex, its X arrow moves along X only.
    ed.call("select", json!({ "ids": [mesh] }));
    ed.call("set_editor", json!({ "tool": "mesh" }));
    ed.call("set_camera", json!({ "view": "front", "center": [64, 64, 0], "zoom": 2.0 }));
    ed.input("front", json!([{ "type": "double_click", "world": [64.0, 64.0, 64.0] }]));
    ed.screenshot("front", "mesh_vertex_gizmo");
    let before = ed.call("get_node", json!({ "id": mesh }))["vertices"].clone();
    ed.input("front", json!([{ "type": "drag", "world": [80.0, 64.0, 64.0], "to_world": [112.0, 72.0, 64.0], "steps": 10 }]));
    let after = ed.call("get_node", json!({ "id": mesh }))["vertices"].clone();
    let changed: Vec<(Vec<f64>, Vec<f64>)> = before
        .as_array()
        .unwrap()
        .iter()
        .zip(after.as_array().unwrap())
        .map(|(a, b)| (floats(&json!([a])), floats(&json!([b]))))
        .filter(|(a, b)| a != b)
        .collect();
    assert_eq!(changed.len(), 1, "one vertex moved: {changed:?}");
    let (a, b) = &changed[0];
    assert_eq!((b[0] - a[0], b[1] - a[1], b[2] - a[2]), (32.0, 0.0, 0.0), "moved +32 along X only");
    assert_eq!(ed.state()["undo"][0], "Move Vertices");
}

/// Scripted building blocks the withered city showcase needs: block letters, window grids, plane clips, vertex moves and
/// named wizard entities, all without mouse input.
#[test]
#[ignore]
fn block_text_window_grids_clips_and_named_wizards() {
    let ed = Editor::launch("block_text");
    let text = ed.call("create_brush", json!({ "shape": "text", "text": "HI", "min": [0, 0, 0], "max": [300, 4, 140], "material": "dev/orange" }));
    let letters = text["letters"].as_array().unwrap();
    assert_eq!(letters.len(), 2);
    assert_eq!(letters[0].as_array().unwrap().len(), 3, "H is two posts and a bar");
    assert_eq!(text["ids"].as_array().unwrap().len(), 3 + letters[1].as_array().unwrap().len());
    assert!(ed.call_err("create_brush", json!({ "shape": "text", "text": "Ö", "min": [0, 0, 0], "max": [64, 8, 64] })).contains("no block letter"));

    let solid = ed.call("create_brush", json!({ "min": [0, 0, 200], "max": [512, 128, 216] }))["ids"].as_array().unwrap().len();
    let wall = ed.call(
        "create_brush",
        json!({ "min": [0, 0, 300], "max": [512, 128, 316], "openings": [{ "min": [32, 32, 296], "max": [96, 96, 320], "count": 4, "step": [128, 0, 0] }] }),
    );
    assert!(wall["ids"].as_array().unwrap().len() > solid + 3, "four windows cut the wall into pieces: {wall}");

    ed.call("select", json!({ "ids": wall["ids"] }));
    let clipped = ed.call("run_action", json!({ "action": "clip_apply", "args": { "point": [0, 64, 0], "normal": [0, 1, 0], "keep": "back" } }));
    assert!(!clipped["ids"].as_array().unwrap().is_empty());
    assert_eq!(ed.selection_bounds().1[1], 64.0, "only the part below the plane is left");
    assert!(ed.call_err("run_action", json!({ "action": "clip_apply", "args": { "point": [0, 0, 0] } })).contains("normal"));

    let block = ed.call("create_brush", json!({ "min": [600, 0, 0], "max": [664, 64, 64] }))["ids"][0].clone();
    ed.call("select", json!({ "ids": [block] }));
    ed.call("run_action", json!({ "action": "move_vertices", "args": { "vertices": [[664, 64, 64]], "offset": [0, -32, 0] } }));
    let node = ed.call("get_node", json!({ "id": block }));
    assert!(node.to_string().contains("32"), "the corner sags: {node}");
    assert!(ed.call_err("run_action", json!({ "action": "move_vertices", "args": { "vertices": [[1, 2, 3]], "offset": [0, 8, 0] } })).contains("vertices"));

    let lift = ed.call("create_brush", json!({ "min": [800, 0, 0], "max": [864, 8, 64] }))["ids"].clone();
    let platform =
        ed.call("gameplay", json!({ "op": "make_platform", "ids": lift, "travel": [0, 256, 0], "properties": { "targetname": "lift", "speed": "3" } }));
    assert_eq!(platform["properties"]["targetname"], "lift");
    assert_eq!(platform["properties"]["speed"], "3");
    let button = ed.call("create_brush", json!({ "min": [900, 0, 0], "max": [908, 8, 8] }))["ids"].clone();
    let made = ed.call("gameplay", json!({ "op": "make_button", "ids": button, "target": "lift", "input": "toggle", "properties": { "targetname": "call" } }));
    assert_eq!(made["properties"]["targetname"], "call");
    assert_eq!(made["outputs"][0]["target"], "lift");
}

fn shot(ed: &Editor, args: Value, label: &str) -> image::RgbaImage {
    let r = ed.try_rpc("tools/call", json!({ "name": "screenshot", "arguments": args })).unwrap();
    let data = r["result"]["content"][0]["data"].as_str().unwrap_or_else(|| panic!("{label}: no image in {r}"));
    let png = base64::engine::general_purpose::STANDARD.decode(data).unwrap();
    std::fs::write(artifacts().join(format!("{}_{label}.png", ed.name)), &png).unwrap();
    image::load_from_memory(&png).unwrap().to_rgba8()
}

/// Mean absolute difference per channel, 0 for identical pictures.
fn image_difference(a: &image::RgbaImage, b: &image::RgbaImage) -> f64 {
    assert_eq!(a.dimensions(), b.dimensions());
    let sum: u64 = a.as_raw().iter().zip(b.as_raw()).map(|(x, y)| x.abs_diff(*y) as u64).sum();
    sum as f64 / a.as_raw().len() as f64
}

#[test]
#[ignore]
fn offscreen_beauty_shots_leave_out_editor_overlays() {
    let ed = Editor::launch("beauty");
    let floor = ed.box_brush([-256.0, -16.0, -256.0], [256.0, 0.0, 256.0]);
    ed.call("set_camera", json!({ "view": "3d", "position": [0, 96, 192], "look_at": [0, 16, 0] }));
    ed.call("run_action", json!({ "action": "select_none" }));
    let size = json!({ "target": "3d", "width": 320, "height": 200 });
    let empty = shot(&ed, size.clone(), "empty");
    ed.call("create_entity", json!({ "classname": "info_player_start", "origin": [0, 0, 0] }));
    ed.call("run_action", json!({ "action": "select_none" }));
    let beauty = shot(&ed, size.clone(), "beauty");
    let mut with_overlays = size.clone();
    with_overlays["overlays"] = json!(true);
    let overlays = shot(&ed, with_overlays, "overlays");
    assert!(image_difference(&beauty, &empty) < 0.5, "the player start box shows in a beauty shot");
    assert!(image_difference(&overlays, &empty) > 1.0, "overlays: true still draws the entity box and edges");

    ed.call("select", json!({ "ids": [floor] }));
    let selected = shot(&ed, size.clone(), "selected_beauty");
    assert!(image_difference(&selected, &beauty) < 0.1, "a selected brush keeps its selection tint, difference {}", image_difference(&selected, &beauty));
    let mut tinted = size.clone();
    tinted["overlays"] = json!(true);
    let tinted = shot(&ed, tinted, "selected_overlays");
    assert!(image_difference(&tinted, &overlays) > 1.0, "the overlay shot right after still shows the selection");
}

/// Lamps that start off with `fixture_off dark` keep their fixture in the lit preview, drawn without its glow like the
/// Godot addon does, and glow again once the lamp starts on.
#[test]
#[ignore]
fn dark_fixtures_lose_their_glow_in_the_lit_preview() {
    let project = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../godot").canonicalize().unwrap();
    let ed = Editor::launch_with("dark_fixture", &["--project", project.to_str().unwrap()]);
    ed.call("set_map_properties", json!({ "properties": { "sun_energy": "0", "ambient_energy": "0.1", "sky_energy": "0.1" } }));
    let panel = ed.call("create_brush", json!({ "min": [-64, 0, -8], "max": [64, 96, 0], "material": "showcase/lamp" }))["ids"][0].as_u64().unwrap();
    ed.call("gameplay", json!({ "op": "brush_entity", "ids": [panel], "classname": "func_illusionary", "properties": { "targetname": "panel" } }));
    let lamp = ed.call(
        "create_entity",
        json!({ "classname": "light", "origin": [0, 48, -64], "properties": { "start_on": "0", "fixture": "panel", "fixture_off": "dark" } }),
    )["id"]
        .as_u64()
        .unwrap();
    ed.call("run_action", json!({ "action": "select_none" }));
    ed.call("set_camera", json!({ "view": "3d", "position": [0, 48, 48], "look_at": [0, 48, 0] }));
    ed.call("set_editor", json!({ "shade": "lit" }));
    let args = json!({ "target": "3d", "width": 160, "height": 120 });
    let luma = |img: &image::RgbaImage| {
        let p = img.get_pixel(80, 60).0;
        (p[0] as u32 + p[1] as u32 + p[2] as u32) / 3
    };
    let dark = shot(&ed, args.clone(), "off");
    ed.call("update_entity", json!({ "id": lamp, "properties": { "start_on": "1" } }));
    let on = shot(&ed, args.clone(), "on");
    ed.call("update_entity", json!({ "id": lamp, "properties": { "start_on": "0", "fixture_off": "hide" } }));
    let hidden = shot(&ed, args, "hidden");
    assert!(luma(&on) > 200 && luma(&dark) + 80 < luma(&on), "the glow goes with the lamp: on {} dark {}", luma(&on), luma(&dark));
    assert!(image_difference(&dark, &hidden) > 5.0, "a dark fixture is still drawn, unlike a hidden one");
}

#[test]
#[ignore]
fn a_camera_inside_a_trigger_sees_out_of_it() {
    let ed = Editor::launch("trigger_inside");
    ed.box_brush([-512.0, -16.0, -512.0], [512.0, 0.0, 512.0]);
    ed.box_brush([-64.0, 0.0, -300.0], [64.0, 128.0, -268.0]);
    ed.call("set_camera", json!({ "view": "3d", "position": [0, 64, 0], "look_at": [0, 48, -300] }));
    let args = json!({ "target": "3d", "width": 320, "height": 200, "overlays": true });
    let outside = shot(&ed, args.clone(), "no_trigger");
    let volume = ed.box_brush([-128.0, 0.0, -128.0], [128.0, 128.0, 128.0]);
    ed.call("select", json!({ "ids": [volume] }));
    ed.call("run_action", json!({ "action": "create_brush_entity", "args": { "classname": "trigger_once" } }));
    ed.call("run_action", json!({ "action": "select_none" }));
    let inside = shot(&ed, args, "inside_trigger");
    assert!(image_difference(&inside, &outside) < 2.0, "the trigger's back faces tint the whole view, difference {}", image_difference(&inside, &outside));
}

#[test]
#[ignore]
fn emissive_model_materials_glow_in_the_lit_preview() {
    let b64 = base64::engine::general_purpose::STANDARD;
    let mut bin: Vec<u8> = Vec::new();
    for f in [-1.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 2.0, 0.0, -1.0, 0.0, 0.0, 1.0, 2.0, 0.0, -1.0, 2.0, 0.0] {
        bin.extend(f.to_le_bytes());
    }

    // A black panel that only its emission can light up.
    let gltf = json!({
        "asset": { "version": "2.0" },
        "extensionsUsed": ["KHR_materials_emissive_strength"],
        "scene": 0, "scenes": [{ "nodes": [0] }], "nodes": [{ "mesh": 0 }],
        "meshes": [{ "primitives": [{ "attributes": { "POSITION": 0 }, "material": 0 }] }],
        "materials": [{ "pbrMetallicRoughness": { "baseColorFactor": [0.0, 0.0, 0.0, 1.0] }, "emissiveFactor": [1.0, 0.2, 0.0],
                        "extensions": { "KHR_materials_emissive_strength": { "emissiveStrength": 4.0 } } }],
        "buffers": [{ "byteLength": bin.len(), "uri": format!("data:application/octet-stream;base64,{}", b64.encode(&bin)) }],
        "bufferViews": [{ "buffer": 0, "byteLength": bin.len() }],
        "accessors": [{ "bufferView": 0, "componentType": 5126, "count": 6, "type": "VEC3", "min": [-1.0, 0.0, 0.0], "max": [1.0, 2.0, 0.0] }]
    });
    let path = artifacts().join("emissive_panel.gltf");
    std::fs::write(&path, gltf.to_string()).unwrap();

    let ed = Editor::launch("emissive_model");
    ed.call("create_entity", json!({ "classname": "prop_model", "origin": [0, 0, 0], "properties": { "model": path.to_string_lossy() } }));
    ed.call("run_action", json!({ "action": "select_none" }));
    ed.call("set_camera", json!({ "view": "3d", "position": [0, 32, 96], "look_at": [0, 32, 0] }));
    ed.call("set_editor", json!({ "shade": "lit" }));
    let img = shot(&ed, json!({ "target": "3d", "width": 200, "height": 200 }), "lit");
    let center = img.get_pixel(100, 100).0;
    assert!(center[0] > 180 && center[2] < 120, "the emissive panel glows orange instead of rendering black, got {center:?}");
}

#[test]
#[ignore]
fn map_file_absolute_open_close_others_and_script_notes() {
    let ed = Editor::launch("mcp_small");
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let name = "e2e_relative_room.gtm";
    let abs = manifest.join(name);
    ed.call("map_file", json!({ "op": "save", "path": abs.clone() }));

    // A relative path resolves against the process directory, opening it should not leave a later save exposed
    // to that changing: the path in the open map is made absolute.
    ed.call("map_file", json!({ "op": "open", "path": name }));
    let opened_path = ed.state()["map"]["path"].as_str().unwrap().to_string();
    assert!(std::path::Path::new(&opened_path).is_absolute(), "opened path should be absolute: {opened_path}");
    let _ = std::fs::remove_file(&abs);

    // A dirty tab left behind by map_file new does not linger: close_others clears it.
    ed.call("map_file", json!({ "op": "new" }));
    ed.box_brush([0.0, 0.0, 0.0], [16.0, 16.0, 16.0]);
    ed.call("map_file", json!({ "op": "new" }));
    assert_eq!(ed.state()["tabs"]["titles"].as_array().unwrap().len(), 2, "the dirty tab stayed open in the background");
    let closed = ed.call("run_action", json!({ "action": "close_tab", "args": { "close_others": true, "discard": true } }));
    assert_eq!(closed["closed"], 1);
    assert_eq!(ed.state()["tabs"]["titles"].as_array().unwrap().len(), 1);

    // run_script reports note steps separately instead of dropping them from the count.
    let steps = json!([
        { "note": "first" },
        { "tool": "create_brush", "args": { "min": [0, 0, 0], "max": [8, 8, 8] } },
        { "note": "second" },
        { "tool": "create_brush", "args": { "min": [8, 0, 0], "max": [16, 8, 8] } }
    ]);
    let summary = ed.call("run_script", json!({ "steps": steps }));
    assert_eq!(summary["steps"], 2);
    assert_eq!(summary["notes"], 2);

    // An unbraced $name/... is an error rather than a path nobody meant to write passing through unexpanded.
    let project = manifest.join("../../godot").canonicalize().unwrap();
    ed.call("open_project", json!({ "path": project }));
    let err = ed.call_err("run_script", json!({ "steps": [{ "tool": "map_file", "args": { "op": "open", "path": "$project/does_not_exist.gtm" } }] }));
    assert!(err.contains("${project}"), "{err}");
}

#[test]
#[ignore]
fn view_panes_maximize_close_and_come_back() {
    let ed = Editor::launch("view_panes");
    let width = |ed: &Editor| ed.screenshot("front", "front").0;
    let docked = width(&ed);
    ed.input("front", json!([{ "type": "move" }, { "type": "key", "key": "Space", "modifiers": ["shift"] }]));
    let maximized = width(&ed);
    assert!(maximized > docked * 3 / 2, "Shift+Space fills the view area with the front view, {docked} to {maximized}");
    ed.input("front", json!([{ "type": "move" }, { "type": "key", "key": "Space", "modifiers": ["shift"] }]));
    assert_eq!(width(&ed), docked, "Shift+Space again brings the four views back");
}

/// Script ergonomics found building the night maps: ids that follow CSG, joined id lists, carve materials, full mesh
/// rotation, exposed-only scatter fills, forward slash paths, lamp fixtures and Node methods as inputs.
#[test]
#[ignore]
fn script_ids_follow_csg_and_night_map_options() {
    let project = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../godot").canonicalize().unwrap();
    let ed = Editor::launch_with("night_options", &["--project", project.to_str().unwrap()]);
    let steps = json!([
        { "tool": "create_brush", "args": { "min": [0, 0, 0], "max": [256, 128, 16], "material": "dev/grey" }, "save": "wall" },
        { "tool": "create_brush", "args": { "min": [512, 0, 0], "max": [576, 64, 64], "material": "dev/grey" }, "save": "block" },
        { "tool": "create_brush", "args": { "min": [96, 32, -8], "max": [160, 96, 24], "material": "dev/orange" }, "save": "window" },
        { "tool": "select", "args": { "ids": "$window.ids" } },
        { "tool": "run_action", "args": { "action": "csg_subtract", "args": { "carve_material": "target" } }, "save": "cut" },
        { "tool": "select", "args": { "ids": "$wall.ids + $block.ids" }, "save": "both" },
        { "tool": "select", "args": { "ids": ["$wall.ids", "$block.ids"] }, "save": "nested" }
    ]);
    let summary = ed.call("run_script", json!({ "steps": steps }));
    assert_eq!(summary["errors"], json!([]));
    let vars = &summary["vars"];
    let pieces = vars["wall"]["ids"].as_array().unwrap();
    assert_eq!(pieces.len(), 4, "the saved wall ids now name the four pieces around the window: {vars}");
    assert_eq!(vars["window"]["ids"], json!([]), "the cutter is gone");
    assert_eq!(vars["cut"]["replaced"].as_object().unwrap().len(), 2, "{}", vars["cut"]);
    assert_eq!(vars["both"]["selected"].as_array().unwrap().len(), 5, "joined lists select the pieces and the block");
    assert_eq!(vars["nested"]["selected"], vars["both"]["selected"]);
    for id in pieces {
        let node = ed.call("get_node", json!({ "id": id }));
        assert!(!node.to_string().contains("dev/orange"), "carved faces kept the wall's material: {node}");
    }

    let flat = ed.call("create_mesh", json!({ "shape": "cylinder", "min": [0, 0, 300], "max": [16, 64, 316], "rotate": [0, 0, 90] }));
    ed.call("select", json!({ "ids": [flat["id"]] }));
    let (min, max) = ed.selection_bounds();
    assert!((max[0] - min[0] - 64.0).abs() < 1e-3 && max[1] - min[1] < 17.0, "the cylinder lies along X: {min:?} {max:?}");

    let ground = ed.box_brush([-512.0, -16.0, 600.0], [512.0, 0.0, 1600.0]);
    ed.box_brush([-512.0, 200.0, 600.0], [0.0, 204.0, 1600.0]);
    ed.call("select", json!({ "ids": [ground] }));
    ed.call("scatter", json!({ "op": "new_set", "name": "weeds", "items": ["res://weed.glb"], "kind": "foliage", "targets": [ground] }));
    let filled = ed.call("scatter", json!({ "op": "fill", "density": 3, "exposed_only": true, "seed": 3 }));
    assert!(filled["placed"].as_u64().unwrap() > 0, "{filled}");
    assert!(filled["set"]["bounds"]["min"][0].as_f64().unwrap() > -64.0, "no weed under the slab over negative X: {filled}");

    let info = ed.call("texture", json!({ "op": "material_info", "material": "night/apartment_lit" }));
    let file = info["material_file"].as_str().expect("the material has a file");
    assert!(!file.contains('\\') && file.ends_with("night/apartment_lit.tres"), "{file}");
    assert!(!ed.state()["game"]["project_root"].as_str().unwrap().contains('\\'));

    ed.call(
        "create_entity",
        json!({ "classname": "light", "origin": [0, 64, 0], "properties": { "targetname": "lamp", "start_on": "0", "fixture": "bulb" },
        "outputs": [{ "output": "switched", "target": "bulb", "input": "set_visible" }, { "output": "switched", "target": "bulb", "input": "set_visibel" }] }),
    );
    let bulb = ed.box_brush([0.0, 70.0, 0.0], [8.0, 78.0, 8.0]);
    ed.call("gameplay", json!({ "op": "brush_entity", "ids": [bulb], "classname": "func_illusionary", "properties": { "targetname": "bulb" } }));
    let issues = ed.call("validate_map", json!({}));
    let inputs: Vec<String> =
        issues["issues"].as_array().unwrap().iter().filter(|i| i["code"] == "io_unknown_input").map(|i| i["message"].to_string()).collect();
    assert_eq!(inputs.len(), 1, "{inputs:?}");
    assert!(inputs[0].contains("set_visibel"));
    ed.call("set_editor", json!({ "shade": "lit" }));
    ed.screenshot("3d", "fixture_hidden_while_off");
}

/// An agent edits a saved map: every call and every script is one labelled undo step, and the person can read what
/// changed and take the whole batch back.
#[test]
#[ignore]
fn agent_edits_are_one_undo_step_and_can_be_reviewed() {
    let ed = Editor::launch("review");
    let floor = ed.box_brush([0.0, 0.0, 0.0], [256.0, 16.0, 256.0]);
    assert_eq!(ed.state()["undo"][0], "MCP: Create Brush");
    let path = artifacts().join("review.gtm");
    ed.call("map_file", json!({ "op": "save", "path": path }));
    let undo_before = ed.state()["undo"].as_array().unwrap().len();

    let run = ed.call(
        "run_script",
        json!({ "label": "Add crates", "steps": [
            { "tool": "create_brush", "args": { "min": [0, 16, 0], "max": [32, 48, 32], "material": "crate" } },
            { "tool": "create_brush", "args": { "min": [64, 16, 0], "max": [96, 48, 32], "material": "crate" } },
            { "tool": "create_entity", "args": { "classname": "light", "origin": [128, 64, 128], "properties": { "energy": "2" } } },
            { "tool": "select", "args": { "ids": [floor] } },
            { "tool": "transform", "args": { "translate": [0, -8, 0] } }
        ] }),
    );
    assert_eq!(run["undo"], "MCP: Add crates, 5 steps");
    let state = ed.state();
    assert_eq!(state["undo"][0], "MCP: Add crates, 5 steps");
    assert_eq!(state["undo"].as_array().unwrap().len(), undo_before + 1);

    let changes = ed.call("changes_since", json!({}));
    assert_eq!(changes["summary"], "3 added, 0 removed, 1 changed", "{changes}");
    assert_eq!(changes["since"], "the last save");
    assert_eq!(changes["changed"][0]["id"].as_u64(), Some(floor));
    assert_eq!(changes["changed"][0]["changes"], json!(["moved"]));
    assert_eq!(changes["changed"][0]["offset"], json!([0.0, -8.0, 0.0]));
    assert_eq!(ed.call("changes_since", json!({ "undo_steps": 1 }))["summary"], changes["summary"]);
    let from_file = ed.call("changes_since", json!({ "file": path }));
    assert_eq!(from_file["summary"], changes["summary"], "{from_file}");
    assert!(ed.call_err("changes_since", json!({ "undo_steps": 99 })).contains("only"));

    let summary = ed.call("summarize_map", json!({}));
    assert_eq!(summary["counts"]["brushes"], 3);
    assert_eq!(summary["entities"]["light"], 1);
    assert_eq!(summary["materials"]["crate"], 12);
    let lights = ed.call("list_nodes", json!({ "property": { "energy": "2" } }));
    assert_eq!(lights["total"], 1);
    assert_eq!(lights["nodes"][0]["properties"]["energy"], "2");
    let near = ed.call("list_nodes", json!({ "type": "brush", "box": { "min": [60, 20, 0], "max": [70, 30, 10] } }));
    assert_eq!(near["total"], 1);
    assert_eq!(ed.call("list_nodes", json!({ "material": "crate", "layer": "Default" }))["total"], 2);
    let compact = ed.call("get_node", json!({ "id": floor, "compact": true }));
    assert!(compact.get("vertices").is_none() && compact["faces"] == 6, "{compact}");

    ed.call("run_action", json!({ "action": "undo" }));
    assert_eq!(ed.brushes(), 1);
    assert_eq!(ed.call("changes_since", json!({}))["summary"], "0 added, 0 removed, 0 changed");
    assert!(!ed.state()["map"]["modified"].as_bool().unwrap());
    ed.call("run_action", json!({ "action": "redo" }));
    assert_eq!(ed.brushes(), 3);
    ed.call("run_action", json!({ "action": "undo", "args": { "steps": 2 } }));
    assert_eq!(ed.brushes(), 0);
}

/// A 4 x 4 BGRA8888 VTF 7.2 without mipmaps or thumbnail.
fn tiny_vtf(bgra: [u8; 4]) -> Vec<u8> {
    let mut b = vec![0u8; 80];
    b[0..4].copy_from_slice(b"VTF\0");
    b[4..8].copy_from_slice(&7u32.to_le_bytes());
    b[8..12].copy_from_slice(&2u32.to_le_bytes());
    b[12..16].copy_from_slice(&80u32.to_le_bytes());
    b[16..18].copy_from_slice(&4u16.to_le_bytes());
    b[18..20].copy_from_slice(&4u16.to_le_bytes());
    b[24..26].copy_from_slice(&1u16.to_le_bytes());
    b[52..56].copy_from_slice(&12i32.to_le_bytes());
    b[56] = 1;
    b[57..61].copy_from_slice(&(-1i32).to_le_bytes());
    b[63..65].copy_from_slice(&1u16.to_le_bytes());
    b.extend(bgra.repeat(16));
    b
}

/// A Quake WAD2 holding one 8 x 8 texture.
fn tiny_wad(name: &str) -> Vec<u8> {
    let mut b = b"WAD2".to_vec();
    b.extend(1u32.to_le_bytes());
    b.extend((12u32 + 40 + 85).to_le_bytes());
    let mut header = [0u8; 40];
    header[..name.len()].copy_from_slice(name.as_bytes());
    header[16..20].copy_from_slice(&8u32.to_le_bytes());
    header[20..24].copy_from_slice(&8u32.to_le_bytes());
    for (k, off) in [40u32, 104, 120, 124].iter().enumerate() {
        header[24 + k * 4..28 + k * 4].copy_from_slice(&off.to_le_bytes());
    }

    b.extend(header);
    b.extend([7u8; 85]);
    let mut entry = [0u8; 32];
    entry[0..4].copy_from_slice(&12u32.to_le_bytes());
    entry[12] = 0x44;
    entry[16..16 + name.len()].copy_from_slice(name.as_bytes());
    b.extend(entry);
    b
}

fn write_material(textures: &std::path::Path, name: &str, size: [u32; 2]) {
    let file = textures.join(name);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    image::RgbaImage::from_pixel(64, 32, image::Rgba([200, 40, 40, 255])).save(file.with_extension("png")).unwrap();
    std::fs::write(
        file.with_extension("tres"),
        format!(
            "[gd_resource type=\"StandardMaterial3D\" format=3]
[ext_resource type=\"Texture2D\" path=\"res://textures/{name}.png\" id=\"1\"]
[resource]
albedo_texture = ExtResource(\"1\")
metadata/texture_size = Vector2({}, {})
",
            size[0], size[1]
        ),
    )
    .unwrap();
}

/// Materials written into the project while the editor runs are found without a restart: validate_map does not
/// report them missing and justify fit uses their texture_size.
#[test]
#[ignore]
fn materials_added_while_running_are_found() {
    let dir = artifacts().join("late_materials");
    let _ = std::fs::remove_dir_all(&dir);
    let textures = dir.join("textures");
    std::fs::create_dir_all(&textures).unwrap();
    std::fs::write(
        dir.join("project.godot"),
        "config_version=5
",
    )
    .unwrap();
    let ed = Editor::launch_with("late_materials", &["--project", dir.to_str().unwrap()]);
    let codes = |ed: &Editor, code: &str| -> Vec<Value> {
        ed.call("validate_map", json!({}))["issues"].as_array().unwrap().iter().filter(|i| i["code"] == code).cloned().collect()
    };

    write_material(&textures, "megaplex/word", [80, 48]);
    let decal = ed.call("create_brush", json!({ "min": [0, 0, 0], "max": [80, 48, 4], "material": "megaplex/word" }))["ids"][0].as_u64().unwrap();
    assert_eq!(codes(&ed, "missing_material"), Vec::<Value>::new(), "the new material is found");
    let faces = ed.call("texture", json!({ "op": "get", "faces": (0..6).map(|f| json!([decal, f])).collect::<Vec<_>>() }));
    let front = faces["faces"].as_array().unwrap().iter().find(|f| f["normal"][2].as_f64().unwrap() > 0.99).unwrap()["face"][1].as_u64().unwrap();
    ed.call("texture", json!({ "op": "justify", "mode": "fit", "faces": [[decal, front]] }));
    let uv = ed.call("texture", json!({ "op": "get", "faces": [[decal, front]] }))["faces"][0]["uv"].clone();
    let corners: Vec<[f64; 2]> = [[0.0, 0.0, 4.0], [80.0, 48.0, 4.0]].iter().map(|p| texel(&uv, *p)).collect();
    let span = [(corners[1][0] - corners[0][0]).abs(), (corners[1][1] - corners[0][1]).abs()];
    assert!(approx(&span, &[80.0, 48.0]), "fit covers one 80 by 48 repeat, not the image's pixel size: {span:?}");

    let before = ed.state()["game"]["materials"].as_u64().unwrap_or_default();
    write_material(&textures, "megaplex/later", [64, 64]);
    let reloaded = ed.call("run_action", json!({ "action": "reload_materials" }));
    assert!(reloaded["materials"].as_u64().unwrap() > before, "{reloaded}");
    ed.call("create_brush", json!({ "min": [0, 64, 0], "max": [64, 128, 16], "material": "megaplex/later" }));
    assert_eq!(codes(&ed, "missing_material"), Vec::<Value>::new());
}

/// validate_map reports faces of two brushes that overlap on one plane with different textures, which z-fight once
/// built, and leaves out the same texture lined up.
#[test]
#[ignore]
fn validate_map_reports_coplanar_faces() {
    let ed = Editor::launch("coplanar");
    let fights = |ed: &Editor| -> Vec<Value> {
        ed.call("validate_map", json!({}))["issues"].as_array().unwrap().iter().filter(|i| i["code"] == "coplanar_faces").cloned().collect()
    };
    ed.call("create_brush", json!({ "min": [0, 0, 0], "max": [64, 64, 16], "material": "dev/grey" }));
    let aligned = ed.call("create_brush", json!({ "min": [32, 16, 0], "max": [96, 48, 16], "material": "dev/grey" }))["ids"][0].as_u64().unwrap();
    assert_eq!(fights(&ed), Vec::<Value>::new(), "one texture lined up draws the same pixels");

    ed.call("texture", json!({ "op": "apply", "material": "dev/orange", "faces": (0..6).map(|f| json!([aligned, f])).collect::<Vec<_>>() }));
    let found = fights(&ed);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0]["node"].as_u64(), Some(aligned));
    assert_eq!(found[0]["severity"], "warning");
    assert!(found[0]["message"].as_str().unwrap().contains("as do 1 more"), "{}", found[0]["message"]);
}

/// A Hammer map and a TrenchBroom map, imported as a user would, find their textures next to them and bring them into
/// the project, after which no face is missing its material.
#[test]
#[ignore]
fn imports_convert_valve_and_quake_textures() {
    let dir = artifacts().join("import_textures");
    let _ = std::fs::remove_dir_all(&dir);
    let project = dir.join("project");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(project.join("project.godot"), "config_version=5\n").unwrap();
    let game = dir.join("mod");
    std::fs::create_dir_all(game.join("materials/brick")).unwrap();
    std::fs::create_dir_all(game.join("maps")).unwrap();
    std::fs::write(game.join("materials/brick/wall.vtf"), tiny_vtf([40, 80, 160, 255])).unwrap();
    std::fs::write(game.join("materials/brick/wall.vmt"), "LightmappedGeneric { $basetexture brick/wall $surfaceprop brick }").unwrap();
    let side = |plane: &str, material: &str| {
        format!("side {{ \"plane\" \"{plane}\" \"material\" \"{material}\" \"uaxis\" \"[1 0 0 0] 0.25\" \"vaxis\" \"[0 -1 0 0] 0.25\" }}")
    };
    let solid = [
        side("(-64 64 64) (64 64 64) (64 -64 64)", "BRICK/WALL"),
        side("(-64 -64 0) (64 -64 0) (64 64 0)", "TOOLS/TOOLSNODRAW"),
        side("(-64 64 64) (-64 -64 64) (-64 -64 0)", "BRICK/WALL"),
        side("(64 64 0) (64 -64 0) (64 -64 64)", "BRICK/WALL"),
        side("(64 64 64) (-64 64 64) (-64 64 0)", "BRICK/WALL"),
        side("(64 -64 0) (-64 -64 0) (-64 -64 64)", "BRICK/WALL"),
    ]
    .join("\n");
    let vmf = game.join("maps/room.vmf");
    std::fs::write(&vmf, format!("world {{ \"classname\" \"worldspawn\" solid {{ {solid} }} }}")).unwrap();

    let ed = Editor::launch_with("import_textures", &["--project", project.to_str().unwrap()]);
    let offered = ed.call("map_file", json!({ "op": "import_vmf", "path": vmf }));
    assert_eq!((offered["missing_materials"].as_u64(), offered["convertible"].as_u64()), (Some(1), Some(1)), "{offered}");
    let missing = |ed: &Editor| {
        let issues = ed.call("validate_map", json!({}));
        issues["issues"].as_array().unwrap().iter().filter(|i| i["code"] == "missing_material").count()
    };
    assert_eq!(missing(&ed), 1, "brick/wall is listed until it is converted");

    let imported = ed.call("map_file", json!({ "op": "import_vmf", "path": vmf, "textures": "auto" }));
    assert_eq!(imported["textures"]["converted"], 1, "{imported}");
    assert!(project.join("textures/brick/wall.png").is_file() && project.join("textures/brick/wall.tres").is_file());
    assert_eq!(missing(&ed), 0);

    let quake = dir.join("quake");
    std::fs::create_dir_all(quake.join("wads")).unwrap();
    std::fs::write(quake.join("wads/base.wad"), tiny_wad("wall1")).unwrap();
    std::fs::write(quake.join("sky.wad"), tiny_wad("sky4")).unwrap();
    let map = quake.join("room.map");
    let brush = "( -64 -64 -16 ) ( -64 -63 -16 ) ( -64 -64 -15 ) WALL1 0 0 0 1 1\n\
( -64 -64 -16 ) ( -64 -64 -15 ) ( -63 -64 -16 ) wall1 0 0 0 1 1\n\
( -64 -64 -16 ) ( -63 -64 -16 ) ( -64 -63 -16 ) skip 0 0 0 1 1\n\
( 64 64 16 ) ( 64 65 16 ) ( 65 64 16 ) sky4 0 0 0 1 1\n\
( 64 64 16 ) ( 65 64 16 ) ( 64 64 17 ) wall1 0 0 0 1 1\n\
( 64 64 16 ) ( 64 64 17 ) ( 64 65 16 ) wall1 0 0 0 1 1\n";
    std::fs::write(&map, format!("{{\n\"classname\" \"worldspawn\"\n\"wad\" \"/somewhere/else/base.wad;sky.wad\"\n{{\n{brush}}}\n}}\n")).unwrap();
    let imported = ed.call("map_file", json!({ "op": "import_map", "path": map, "textures": "auto" }));
    assert_eq!(imported["textures"]["converted"], 1, "the WAD is found by name one folder down: {imported}");
    assert!(project.join("textures/wall1.png").is_file());
    assert_eq!(missing(&ed), 0, "WALL1 finds wall1 and skip is the project's skip texture");
    let brush = ed.call("list_nodes", json!({ "type": "brush" }))["nodes"][0]["id"].clone();
    let faces = ed.call("get_node", json!({ "id": brush })).to_string();
    assert!(!faces.contains("WALL1") && faces.contains("\"wall1\"") && faces.contains("special/skip"), "{faces}");
    assert!(faces.contains("special/sky") && !faces.contains("sky4"), "{faces}");
    let sky = &imported["textures"]["sky"];
    assert!(sky["sky_top_color"].is_string() && sky["sky_horizon_color"].is_string(), "sky colors from sky4: {imported}");

    let again = ed.call("map_file", json!({ "op": "convert_textures", "path": game }));
    assert_eq!((again["found"].as_u64(), again["textures"]["existing"].as_u64()), (Some(1), Some(1)), "{again}");
    ed.call("set_editor", json!({ "shade": "textured" }));
    let (_, _, colors) = ed.screenshot("3d", "quake_room");
    assert!(colors > 2);
}

/// Answers the live link like a Godot editor with `map` open: status lists it and a capture returns a small red PNG.
/// Every capture message is kept.
fn fake_godot_capturing(project: &std::path::Path, map: &std::path::Path) -> (u16, std::sync::Arc<std::sync::Mutex<Vec<Value>>>) {
    use std::io::BufRead;
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let slash = |p: &std::path::Path| p.to_string_lossy().replace('\\', "/");
    let (project, map) = (slash(project), slash(map));
    let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let log = seen.clone();
    let mut png = std::io::Cursor::new(Vec::new());
    image::RgbaImage::from_pixel(8, 4, image::Rgba([220, 30, 30, 255])).write_to(&mut png, image::ImageFormat::Png).unwrap();
    let png = base64::engine::general_purpose::STANDARD.encode(png.into_inner());
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { return };
            let mut reader = std::io::BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            while reader.read_line(&mut line).unwrap_or(0) > 0 {
                let msg: Value = serde_json::from_str(&line).unwrap();
                line.clear();
                let mut reply = match msg["event"].as_str() {
                    Some("status") => json!({ "ok": true, "project": project, "godot": "4.7", "pid": 1, "maps": [{ "path": map, "epoch": 1 }] }),
                    Some("capture") => {
                        log.lock().unwrap().push(msg.clone());
                        json!({ "ok": true, "png": png, "width": 8, "height": 4, "warnings": ["warning: prop model res://gone.glb not found"] })
                    }
                    _ => json!({ "ok": false, "error": "unknown event" }),
                };
                reply["seq"] = msg["seq"].clone();
                if writeln!(&stream, "{reply}").is_err() {
                    break;
                }
            }
        }
    });
    (port, seen)
}

/// `screenshot {source: godot}` asks the connected Godot editor for a picture of the map as shown in GodotTrench and
/// returns it with what Godot logged, and says what is missing when it cannot.
#[test]
#[ignore]
fn godot_captures_come_over_the_live_link() {
    let dir = artifacts().join("godot_capture");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("maps")).unwrap();
    std::fs::write(dir.join("project.godot"), "config_version=5\n").unwrap();
    let map = dir.join("maps/yard.gtm");
    let (godot, captures) = fake_godot_capturing(&dir, &map);

    let ed = Editor::launch("godot_capture");
    ed.call("set_editor", json!({ "live_link_port": godot }));
    ed.call("create_brush", json!({ "min": [0, 0, 0], "max": [256, 16, 256] }));
    assert!(ed.call_err("screenshot", json!({ "source": "godot" })).contains("save the map"));
    ed.call("open_project", json!({ "path": dir }));
    ed.call("map_file", json!({ "op": "save", "path": map }));
    let e = ed.call_err("screenshot", json!({ "source": "godot", "position": [0, 64, 0], "look_at": [0, 64, 0] }));
    assert!(e.contains("look_at"), "{e}");

    ed.call("set_camera", json!({ "view": "3d", "position": [0, 64, 512], "look_at": [0, 64, 0] }));
    let r = ed.try_rpc("tools/call", json!({ "name": "screenshot", "arguments": { "source": "godot", "width": 320, "height": 200 } })).unwrap();
    let content = &r["result"]["content"];
    let png = base64::engine::general_purpose::STANDARD.decode(content[0]["data"].as_str().unwrap_or_else(|| panic!("no image in {r}"))).unwrap();
    assert_eq!(image::load_from_memory(&png).unwrap().to_rgba8().get_pixel(0, 0).0, [220, 30, 30, 255]);
    let note = content[1]["text"].as_str().unwrap();
    assert!(note.contains("yard.gtm") && note.contains("built from the map") && note.contains("res://gone.glb"), "{note}");

    let sent = captures.lock().unwrap().clone();
    assert_eq!(sent.len(), 1);
    assert_eq!((sent[0]["width"].as_u64(), sent[0]["height"].as_u64()), (Some(320), Some(200)));
    let forward: Vec<f64> = sent[0]["camera"]["forward"].as_array().unwrap().iter().map(|v| v.as_f64().unwrap()).collect();
    assert!(approx(&forward, &[0.0, 0.0, -1.0]), "the 3d view's camera looks down -Z: {forward:?}");
    assert!(sent[0]["text"].as_str().is_some_and(|t| t.contains("godottrench-map")), "the map as shown goes along to build");

    ed.call("screenshot", json!({ "source": "godot", "build": false, "position": [512, 64, 0], "look_at": [0, 64, 0] }));
    let sent = captures.lock().unwrap().clone();
    assert!(sent[1]["text"].is_null() && sent[1]["camera"]["position"] == json!([512.0, 64.0, 0.0]), "{}", sent[1]);

    ed.call("set_editor", json!({ "live_link_port": free_port() }));
    let e = ed.call_err("screenshot", json!({ "source": "godot" }));
    assert!(e.contains("not running"), "{e}");
}
