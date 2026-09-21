use base64::Engine;
use serde_json::{Value, json};

use super::{ToolExecutor, ToolResult};

pub const PROTOCOL_VERSION: &str = "2025-06-18";
const SUPPORTED_VERSIONS: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];

fn error(id: &Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

fn result(id: &Value, value: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": value })
}

/// Handles one JSON-RPC message or batch. Returns None for notifications and responses.
pub fn handle(msg: Value, exec: &dyn ToolExecutor) -> Option<Value> {
    if let Value::Array(items) = msg {
        let out: Vec<Value> = items.into_iter().filter_map(|m| handle(m, exec)).collect();
        return (!out.is_empty()).then_some(Value::Array(out));
    }

    let id = msg.get("id").cloned()?;
    let Some(method) = msg.get("method").and_then(|m| m.as_str()) else {
        // A response from the client, nothing to answer.
        return None;
    };
    let params = msg.get("params").cloned().unwrap_or(Value::Null);
    Some(match method {
        "initialize" => {
            let requested = params.get("protocolVersion").and_then(|v| v.as_str()).unwrap_or(PROTOCOL_VERSION);
            let version = if SUPPORTED_VERSIONS.contains(&requested) { requested } else { PROTOCOL_VERSION };
            result(
                &id,
                json!({
                    "protocolVersion": version,
                    "capabilities": { "tools": { "listChanged": false } },
                    "serverInfo": { "name": "godottrench", "title": "GodotTrench Level Editor", "version": env!("CARGO_PKG_VERSION") },
                    "instructions": INSTRUCTIONS,
                }),
            )
        }
        "ping" => result(&id, json!({})),
        "tools/list" => result(&id, json!({ "tools": tool_definitions() })),
        "tools/call" => {
            let Some(name) = params.get("name").and_then(|n| n.as_str()) else {
                return Some(error(&id, -32602, "missing tool name"));
            };
            if !tool_definitions().iter().any(|t| t["name"] == name) {
                return Some(error(&id, -32602, &format!("unknown tool {name}")));
            }

            let args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
            result(&id, tool_result_json(exec.call(name, args)))
        }
        "resources/list" => result(&id, json!({ "resources": [] })),
        "prompts/list" => result(&id, json!({ "prompts": [] })),
        _ => error(&id, -32601, &format!("method not found: {method}")),
    })
}

pub fn tool_result_json(r: ToolResult) -> Value {
    match r {
        ToolResult::Json(v) => {
            let text = serde_json::to_string_pretty(&v).unwrap_or_default();
            json!({ "content": [{ "type": "text", "text": text }], "structuredContent": if v.is_object() { v } else { json!({ "value": v }) }, "isError": false })
        }
        ToolResult::Image { png, note } => json!({
            "content": [
                { "type": "image", "data": base64::engine::general_purpose::STANDARD.encode(png), "mimeType": "image/png" },
                { "type": "text", "text": note }
            ],
            "isError": false
        }),
        ToolResult::Error(e) => json!({ "content": [{ "type": "text", "text": e }], "isError": true }),
    }
}

const INSTRUCTIONS: &str = "GodotTrench is a brush based level editor for Godot. Coordinates are map units, Y is up, -Z is forward (Godot convention). \
Default scale is 32 units per meter. Use get_state first, create_brush / create_entity to build, screenshot to look at the result, \
simulate_input to test the UI exactly like a user would, and run_action for any menu command. hierarchy organizes layers and groups, \
terrain_edit and scatter build landscapes, gameplay sets up doors, lifts, triggers, spawners and I/O links, code_reference shows GDScript and C# \
for any entity, and run_script replays a JSON list of these calls (see examples/mcp in the repository).";

fn vec3_schema(desc: &str) -> Value {
    json!({ "type": "array", "items": { "type": "number" }, "minItems": 3, "maxItems": 3, "description": desc })
}

pub fn tool_definitions() -> Vec<Value> {
    let view_enum = json!(["window", "3d", "top", "front", "side"]);
    vec![
        json!({
            "name": "get_state",
            "description": "Editor state: open map, modified flag, counts, selection, grid, tool, cameras, game config.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "list_nodes",
            "description": "Lists map nodes (layers, groups, entities, brushes) with ids, names, parents and bounds.",
            "inputSchema": { "type": "object", "properties": {
                "type": { "type": "string", "enum": ["layer", "group", "entity", "brush", "instance", "mesh", "terrain", "scatter"] },
                "classname": { "type": "string" },
                "selected_only": { "type": "boolean" },
                "limit": { "type": "integer", "default": 200 }
            } }
        }),
        json!({
            "name": "get_node",
            "description": "Full data of one node exactly as stored in the .gtm file.",
            "inputSchema": { "type": "object", "properties": { "id": { "type": "integer" } }, "required": ["id"] }
        }),
        json!({
            "name": "run_action",
            "description": "Runs an editor command. Actions: new_map, save, undo, redo, delete, duplicate, select_all, select_none, select_inverse, select_touching, select_inside, select_siblings, select_same_material, group, ungroup, hide_selected, isolate_selected, unhide_all, lock_selected, unlock_all, grid_up, grid_down, toggle_snap, toggle_uv_lock, toggle_textured, csg_subtract, csg_merge, csg_intersect, csg_hollow, rotate {axis, degrees}, flip {axis}, focus_selection, create_brush_entity {classname}, place_entities {classnames, at, normal, row}, move_to_world, add_layer, snap_vertices, apply_material {material}, copy, cut, paste {text}, nudge {offset}, set_tool {tool}, clip_apply, insert_prefab {path, origin, angles, fixup}, explode_instances, create_displacement {power}, remove_displacement, sew_displacements, sculpt {center, mode: raise|lower|smooth|flatten|noise|terrace|paint_alpha|erase_alpha|paint_layer|hole|unhole, radius, strength, height, layer, step} (displacements of selected brushes and terrains), sprinkle {center, items, radius, density, min_spacing, seed} (point entities, see the scatter tool for trees and foliage), set_shade {shade: textured|flat|lit}, edit_mesh, convert_to_mesh, convert_to_brushes, join_meshes, mesh_op {op: merge_at_center|fill|delete|subdivide|triangulate|flip_normals|smooth_vertices|solidify|duplicate|separate|shade_smooth|shade_flat|mirror_x|snap_vertices_to_grid}, duplicate_linked, unlink_groups, set_cordon, toggle_cordon, clear_cordon, store_camera {slot}, recall_camera {slot}, new_tab, next_tab, close_tab, hotspot_texture, terrain_auto_paint, reload_models, open_godot_editor, run_godot_project, focus_godot, build_in_godot (full build in the connected Godot editor of the map as shown, saved or not), toggle_live_mode, start_tour {chapter, step} (guided tour, indices from 0), stop_tour, show_guide {chapter}.",
            "inputSchema": { "type": "object", "properties": {
                "action": { "type": "string" },
                "args": { "type": "object" }
            }, "required": ["action"] }
        }),
        json!({
            "name": "create_brush",
            "description": "Creates brushes inside the given bounds and selects them. shape defaults to box. openings are boxes [[min], [max]] carved out with CSG (doors, windows). hollow turns the box into walls of that thickness. uv_scale sets world units per texel. entity {classname, properties, outputs} wraps the brushes in a brush entity.",
            "inputSchema": { "type": "object", "properties": {
                "min": vec3_schema("minimum corner"),
                "max": vec3_schema("maximum corner"),
                "openings": { "type": "array", "items": { "type": "array" } },
                "hollow": { "type": "number" },
                "uv_scale": { "type": "number" },
                "entity": { "type": "object" },
                "shape": { "type": "string", "enum": ["box", "cylinder", "cone", "sphere", "wedge", "spike", "arch", "pipe", "stairs", "spiral_stairs", "gable"] }, "turns": { "type": "number" }, "ridge_x": { "type": "boolean" },
                "material": { "type": "string" },
                "sides": { "type": "integer" },
                "thickness": { "type": "number" },
                "steps": { "type": "integer" },
                "parent": { "type": "integer", "description": "layer, group or brush entity id" }
            }, "required": ["min", "max"] }
        }),
        json!({
            "name": "create_mesh",
            "description": "Creates an editable polygon mesh and selects it. Shapes: cuboid, cylinder, cone, sphere, torus, grid, arch_wall, gable_roof, spire, lathe (profile of [radius, height] around center), prism (footprint of [x, z] between bottom and top). smooth_angle enables smooth shading. uv_scale projects face aligned textures, rotate_y turns it around its center, translate moves it, roughen {amount, seed} displaces vertices into rock shapes.",
            "inputSchema": { "type": "object", "properties": {
                "shape": { "type": "string" },
                "min": vec3_schema("minimum corner"), "max": vec3_schema("maximum corner"),
                "material": { "type": "string" }, "sides": { "type": "integer" }, "thickness": { "type": "number" },
                "divisions": { "type": "integer" }, "opening_width": { "type": "number" }, "opening_height": { "type": "number" },
                "ridge_z": { "type": "boolean" }, "overhang": { "type": "number" },
                "profile": { "type": "array", "items": { "type": "array", "items": { "type": "number" } } },
                "center": vec3_schema("lathe axis point"), "caps": { "type": "boolean" },
                "footprint": { "type": "array", "items": { "type": "array", "items": { "type": "number" } } },
                "bottom": { "type": "number" }, "top": { "type": "number" },
                "smooth_angle": { "type": "number" }, "parent": { "type": "integer" },
                "uv_scale": { "type": "number" }, "translate": vec3_schema("moves the mesh after building"), "rotate_y": { "type": "number" }, "roughen": { "type": "object" }
            }, "required": ["shape"] }
        }),
        json!({
            "name": "mesh_edit",
            "description": "Topology operation on a mesh by vertex, edge and face indices (see get_node). ops: extrude_faces {faces, offset|distance}, extrude_edges {edges, offset}, inset {faces, thickness}, loop_cut {edges:[[a,b]], cuts}, bisect {point, normal, faces?, delete: front|back, fill}, subdivide {faces}, merge {verts, at?}, weld {distance}, fill {verts}, delete_faces {faces}, delete_faces_in_box {min, max} (faces whose center lies inside, e.g. door and window holes), set_material_in_box {min, max, material}, delete_vertices {verts}, bevel_edges {edges, width}, bevel_vertices {verts, width}, translate {verts, offset}, smooth {verts, factor, iterations}, solidify {thickness}, flip {faces}, triangulate {faces}, smooth_angle {angle}, set_material {faces, material}, separate {faces}.",
            "inputSchema": { "type": "object", "properties": {
                "id": { "type": "integer" }, "op": { "type": "string" },
                "faces": { "type": "array", "items": { "type": "integer" } },
                "verts": { "type": "array", "items": { "type": "integer" } },
                "edges": { "type": "array", "items": { "type": "array", "items": { "type": "integer" } } },
                "offset": vec3_schema("offset"), "point": vec3_schema("plane point"), "normal": vec3_schema("plane normal"),
                "distance": { "type": "number" }, "thickness": { "type": "number" }, "width": { "type": "number" }, "cuts": { "type": "integer" }
            }, "required": ["id", "op"] }
        }),
        json!({
            "name": "texture",
            "description": "Texture alignment on brush and mesh faces. faces are [[node, face], ...], the selection when omitted. ops: apply {material, reset}, justify {mode: left|right|top|bottom|center|fit|fit_width|fit_height, treat_as_one}, align_view, reset {scale}, wrap {source: [node, face], material} (continues the source alignment across edges), shift {texels: [u, v]}, scale {factor}, rotate {degrees}, density {units_per_texel}, pick (eyedropper into the clipboard and current material), paste {with_material}, mesh_uv {kind: planar|box|cylinder|sphere|view|unfold|normalize}, set_hotspots {material, rects: [[x, y, w, h]]}, material_info {material}, get. Returns the faces' materials and UVs.",
            "inputSchema": { "type": "object", "properties": {
                "op": { "type": "string" },
                "faces": { "type": "array", "items": { "type": "array", "items": { "type": "integer" } } },
                "source": { "type": "array", "items": { "type": "integer" } },
                "material": { "type": "string" }, "mode": { "type": "string" }, "kind": { "type": "string" },
                "treat_as_one": { "type": "boolean" }, "reset": { "type": "boolean" }, "with_material": { "type": "boolean" },
                "texels": { "type": "array", "items": { "type": "number" } }, "factor": {}, "scale": {},
                "degrees": { "type": "number" }, "units_per_texel": { "type": "number" },
                "rects": { "type": "array", "items": { "type": "array", "items": { "type": "number" } } }
            }, "required": ["op"] }
        }),
        json!({
            "name": "create_terrain",
            "description": "Creates a heightmap terrain. shape: flat, hills, mountain, island, valley, ridges. layers are [material, world units per repeat] for up to four blend layers (base, slope, peak, low), auto painted from slope and height.",
            "inputSchema": { "type": "object", "properties": {
                "origin": vec3_schema("minimum corner"), "resolution": { "type": "integer", "description": "vertices per side, e.g. 65, 129, 257" },
                "cell_size": { "type": "number" }, "shape": { "type": "string" }, "height": { "type": "number" }, "seed": { "type": "integer" },
                "feature_size": { "type": "number" }, "erosion": { "type": "integer" },
                "layers": { "type": "array", "items": { "type": "array" } }, "auto_paint": { "type": "boolean" }, "parent": { "type": "integer" }
            } }
        }),
        json!({
            "name": "import_model",
            "description": "Imports a model. mode mesh (Blockbench as an editable mesh), brushes (Blockbench cubes as brushes) or prop (entity referencing a .bbmodel, .glb or .gltf).",
            "inputSchema": { "type": "object", "properties": {
                "path": { "type": "string" }, "mode": { "type": "string", "enum": ["mesh", "brushes", "prop"] }, "origin": vec3_schema("placement")
            }, "required": ["path"] }
        }),
        json!({
            "name": "create_entity",
            "description": "Creates a point entity and selects it. outputs are I/O connections {output, target, input, parameter, delay, times}.",
            "inputSchema": { "type": "object", "properties": {
                "classname": { "type": "string" },
                "origin": vec3_schema("position"),
                "angles": vec3_schema("pitch yaw roll in degrees"),
                "properties": { "type": "object" },
                "outputs": { "type": "array" },
                "parent": { "type": "integer" }
            }, "required": ["classname", "origin"] }
        }),
        json!({
            "name": "update_entity",
            "description": "Changes an entity. Property values of null remove the key. outputs replaces all I/O connections.",
            "inputSchema": { "type": "object", "properties": {
                "id": { "type": "integer" },
                "classname": { "type": "string" },
                "origin": vec3_schema("position"),
                "angles": vec3_schema("pitch yaw roll"),
                "properties": { "type": "object" },
                "outputs": { "type": "array", "items": { "type": "object", "properties": {
                    "output": { "type": "string" }, "target": { "type": "string" }, "input": { "type": "string" },
                    "parameter": { "type": "string" }, "delay": { "type": "number" }, "times": { "type": "integer" }
                }, "required": ["output", "target", "input"] } }
            }, "required": ["id"] }
        }),
        json!({
            "name": "select",
            "description": "Changes the selection. faces are [brush_id, face_index] pairs.",
            "inputSchema": { "type": "object", "properties": {
                "ids": { "type": "array", "items": { "type": "integer" } },
                "faces": { "type": "array", "items": { "type": "array", "items": { "type": "integer" } } },
                "mode": { "type": "string", "enum": ["replace", "add", "remove", "clear"], "default": "replace" }
            } }
        }),
        json!({
            "name": "transform",
            "description": "Transforms the selection: translate, rotate about an axis, flip, or scale into new bounds.",
            "inputSchema": { "type": "object", "properties": {
                "translate": vec3_schema("offset"),
                "rotate": { "type": "object", "properties": { "axis": { "type": "string", "enum": ["x", "y", "z"] }, "degrees": { "type": "number" }, "center": vec3_schema("pivot") } },
                "flip": { "type": "string", "enum": ["x", "y", "z"] },
                "scale_to": { "type": "object", "properties": { "min": vec3_schema("min"), "max": vec3_schema("max") } }
            } }
        }),
        json!({
            "name": "set_face",
            "description": "Edits one brush face: material and UV projection.",
            "inputSchema": { "type": "object", "properties": {
                "id": { "type": "integer" }, "face": { "type": "integer" },
                "material": { "type": "string" },
                "offset": { "type": "array", "items": { "type": "number" } },
                "scale": { "type": "array", "items": { "type": "number" } },
                "rotate_by": { "type": "number" },
                "fit": { "type": "boolean" }
            }, "required": ["id", "face"] }
        }),
        json!({
            "name": "map_file",
            "description": "new, open or save .gtm maps, import or export Quake/TrenchBroom .map files. save without path saves to the current file.",
            "inputSchema": { "type": "object", "properties": {
                "op": { "type": "string", "enum": ["new", "open", "open_tab", "save", "import_map", "import_vmf", "export_map"] },
                "path": { "type": "string" }
            }, "required": ["op"] }
        }),
        json!({
            "name": "open_project",
            "description": "Opens a Godot project folder (containing project.godot) and loads its godottrench_game.json.",
            "inputSchema": { "type": "object", "properties": { "path": { "type": "string" } }, "required": ["path"] }
        }),
        json!({
            "name": "set_editor",
            "description": "Editor settings: grid size, snapping, uv lock, active tool (select, clip, vertex, rotate, scale, mesh, sculpt, blend, paint, scatter, volume, path, measure, texture), current material, shading, mesh component mode, scatter palette items, brush radius, sculpt mode and interface scale.",
            "inputSchema": { "type": "object", "properties": {
                "grid": { "type": "number" }, "snap": { "type": "boolean" }, "uv_lock": { "type": "boolean" },
                "tool": { "type": "string" }, "material": { "type": "string" }, "textured": { "type": "boolean" },
                "shade": { "type": "string", "enum": ["textured", "flat", "lit", "wireframe"] },
                "mesh_component": { "type": "string", "enum": ["vertex", "edge", "face"] },
                "scatter_items": { "type": "array", "items": { "type": "string" } },
                "brush_radius": { "type": "number" },
                "sculpt_mode": { "type": "string" },
                "ui_scale": { "type": "number", "description": "Interface scale, 1 is 100%" },
                "follow_display_scaling": { "type": "boolean" },
                "live_mode": { "type": "boolean", "description": "Send edits to a Godot editor that has the map open before saving" }
            } }
        }),
        json!({
            "name": "set_camera",
            "description": "Moves a viewport camera. 3d uses position and look_at, 2d views use center and zoom (pixels per unit). focus frames given bounds.",
            "inputSchema": { "type": "object", "properties": {
                "view": { "type": "string", "enum": ["3d", "top", "front", "side"] },
                "position": vec3_schema("3d camera position"),
                "look_at": vec3_schema("3d look target"),
                "center": vec3_schema("2d view center"),
                "zoom": { "type": "number" },
                "focus": { "type": "object", "properties": { "min": vec3_schema("min"), "max": vec3_schema("max") } }
            }, "required": ["view"] }
        }),
        json!({
            "name": "screenshot",
            "description": "PNG of the whole window or of one viewport. Use after changes to verify them visually.",
            "inputSchema": { "type": "object", "properties": { "target": { "type": "string", "enum": view_enum, "default": "window" } } }
        }),
        json!({
            "name": "simulate_input",
            "description": "Plays real mouse and keyboard input into the UI frame by frame, like a user. Positions are points relative to target (window or a viewport), or world coordinates projected through that viewport. Event types: move, click, double_click, drag (from x,y or world to to/to_world), key (key name like Delete, A, Enter, F2), text, scroll (delta). modifiers: ctrl, shift, alt.",
            "inputSchema": { "type": "object", "properties": {
                "target": { "type": "string", "enum": view_enum, "default": "window" },
                "events": { "type": "array", "items": { "type": "object", "properties": {
                    "type": { "type": "string", "enum": ["move", "click", "double_click", "drag", "key", "text", "scroll"] },
                    "x": { "type": "number" }, "y": { "type": "number" },
                    "world": vec3_schema("world position (viewport targets)"),
                    "to": { "type": "array", "items": { "type": "number" } },
                    "to_world": vec3_schema("drag end in world"),
                    "button": { "type": "string", "enum": ["left", "right", "middle"] },
                    "modifiers": { "type": "array", "items": { "type": "string", "enum": ["ctrl", "shift", "alt"] } },
                    "key": { "type": "string" }, "text": { "type": "string" }, "delta": { "type": "number" },
                    "steps": { "type": "integer", "description": "frames used for a drag, default 8" }
                }, "required": ["type"] } }
            }, "required": ["events"] }
        }),
        json!({
            "name": "validate_map",
            "description": "Checks the map for problems: invalid brushes, entities without definitions, broken I/O targets, empty brush entities.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "scatter",
            "description": "Scatter trees, rocks and foliage into scatter sets that live on their own layers and only cover their target surfaces. ops: palette {items: [model path or {source, weight, scale: [min, max], spacing, align, tilt, sink}], preset: forest|pines|undergrowth|rocks|grass, kind: props|foliage}, install_models (writes the built-in nature models to res://godottrench/nature), new_set {name, targets, collision, cast_shadows, visibility_range}, activate {id}, paint {center [x, y, z] or [x, z] dropped to the ground, radius, density, slope, height, falloff, seed}, stroke {points, ...}, erase {center, radius, amount}, fill {id, targets, density, slope, height, seed} (whole target area), toggle_target {target}, clear, get {id}, to_entities {id}. Settings passed to paint, stroke, erase and fill apply to that call; palette and preset persist.",
            "inputSchema": { "type": "object", "properties": {
                "op": { "type": "string" }, "id": { "type": "integer" }, "name": { "type": "string" },
                "items": { "type": "array" }, "preset": { "type": "string" }, "kind": { "type": "string", "enum": ["props", "foliage"] },
                "center": { "type": "array", "items": { "type": "number" } }, "points": { "type": "array" },
                "targets": { "type": "array", "items": { "type": "integer" } }, "target": { "type": "integer" },
                "radius": { "type": "number" }, "density": { "type": "number" }, "slope": { "type": "array" }, "height": { "type": "array" },
                "falloff": { "type": "number" }, "amount": { "type": "number" }, "seed": { "type": "integer" }, "only_targets": { "type": "boolean" },
                "collision": { "type": "string", "enum": ["none", "convex", "trimesh"] }, "cast_shadows": { "type": "boolean" }, "visibility_range": { "type": "number" },
                "output": { "type": "string", "enum": ["set", "entities"] }, "overwrite": { "type": "boolean" }
            }, "required": ["op"] }
        }),
        json!({
            "name": "blend",
            "description": "Material blending. Terrains blend their four layers, displacements and brush or mesh faces blend towards a blend material stored per face. ops: set_material {faces or ids, material}, clear_material, dab {center, mode: paint|erase|smooth|sharpen|noise|slope|height, falloff: smooth|linear|constant|spray, radius, strength 0..1, layer, slope [min, max], height [min, max], noise_scale, seed}, stroke {points, ...}, weights {center, id} (terrain weights at a point). Targets are the selection (ids selects first) or everything blendable.",
            "inputSchema": { "type": "object", "properties": {
                "op": { "type": "string" }, "ids": { "type": "array", "items": { "type": "integer" } },
                "faces": { "type": "array", "items": { "type": "array", "items": { "type": "integer" } } }, "material": { "type": "string" },
                "center": { "type": "array", "items": { "type": "number" } }, "points": { "type": "array" }, "mode": { "type": "string" }, "falloff": { "type": "string" },
                "radius": { "type": "number" }, "strength": { "type": "number" }, "layer": { "type": "integer" }, "slope": { "type": "array" }, "height": { "type": "array" },
                "noise_scale": { "type": "number" }, "seed": { "type": "integer" }, "id": { "type": "integer" }
            }, "required": ["op"] }
        }),
        json!({
            "name": "gameplay",
            "description": "Gameplay setups with the FuncGodot fork's entity library. ops: make_door {ids, kind: hinged|sliding, side: left|right, angle, direction: up|down|left|right, lip, trigger, properties} (hinge and travel computed from the brushes), make_platform {ids, travel [x, y, z], mode 0 toggle|1 ping pong|2 once}, make_button {ids, target, input}, brush_entity {ids, classname, properties, outputs}, volume {classname: trigger_once|trigger_multiple|trigger_call|trigger_spawn_area|trigger_hurt|trigger_teleport|trigger_push, min, max, properties, outputs}, link {from, to, output, input, parameter, delay} (names the target when needed), place {classname, origin ([x, z] snaps to the ground), angles, properties, outputs}, gizmos {ids} (viewport handle positions). Outputs can target /root/Node paths, @groups, targetnames with * wildcards or !activator, calling GDScript or C# methods.",
            "inputSchema": { "type": "object", "properties": {
                "op": { "type": "string" }, "ids": { "type": "array", "items": { "type": "integer" } }, "kind": { "type": "string" }, "side": { "type": "string" },
                "angle": { "type": "number" }, "direction": { "type": "string" }, "lip": { "type": "number" }, "trigger": { "type": "boolean" },
                "travel": vec3_schema("offset in map units"), "mode": { "type": "integer" }, "target": { "type": "string" }, "input": { "type": "string" }, "output": { "type": "string" },
                "classname": { "type": "string" }, "min": vec3_schema("volume min"), "max": vec3_schema("volume max"), "properties": { "type": "object" }, "outputs": { "type": "array" },
                "from": { "type": "integer" }, "to": { "type": "integer" }, "parameter": { "type": "string" }, "delay": { "type": "number" },
                "origin": { "type": "array", "items": { "type": "number" } }, "angles": vec3_schema("pitch yaw roll")
            }, "required": ["op"] }
        }),
        json!({
            "name": "code_reference",
            "description": "How to use an entity from game code: its definition, gizmos, and generated GDScript and C# classes, usage snippets (calling inputs, connecting outputs, GodotTrenchIO events) and the FuncGodot .tres resource. Without classname lists every entity and tool.",
            "inputSchema": { "type": "object", "properties": {
                "classname": { "type": "string" }, "kind": { "type": "string", "enum": ["all", "gdscript", "csharp", "gdscript_usage", "csharp_usage", "fgd"] }
            } }
        }),
        json!({
            "name": "hierarchy",
            "description": "Layers and groups. New objects go into the innermost open group, else the current layer. ops: add_layer {name} (becomes current), add_group {name, parent, open (default true)}, open_group {id}, close_group {all}, set_current_layer {id}, reparent {ids, parent}, rename {id, name} (entities get a targetname), set_flags {ids, hidden, locked, omit_from_export}.",
            "inputSchema": { "type": "object", "properties": {
                "op": { "type": "string" }, "name": { "type": "string" }, "id": { "type": "integer" }, "parent": { "type": "integer" },
                "ids": { "type": "array", "items": { "type": "integer" } }, "open": { "type": "boolean" }, "all": { "type": "boolean" },
                "hidden": { "type": "boolean" }, "locked": { "type": "boolean" }, "omit_from_export": { "type": "boolean" }
            }, "required": ["op"] }
        }),
        json!({
            "name": "set_map_properties",
            "description": "Sets worldspawn keys, e.g. message, sun_angles, sun_color, sun_energy, ambient_color, sky_top_color, sky_horizon_color, sky_ground_color, fog_color, fog_density. null removes a key.",
            "inputSchema": { "type": "object", "properties": { "properties": { "type": "object" } }, "required": ["properties"] }
        }),
        json!({
            "name": "terrain_edit",
            "description": "Shapes a terrain (id, else the selected or first one). ops: sculpt {center, mode: raise|lower|smooth|flatten|noise|terrace|paint_layer|hole|unhole, radius, strength, height, step, layer}, sculpt_path {points, spacing, ...}, paint_path {points, layer, radius, strength}, flatten_rect {min [x, z], max [x, z], height, margin} (building pads), ramp {from [x, y, z], to [x, y, z], width, margin} (roads and slopes), erode {iterations, talus}, auto_paint {rock_slope, top_height, low_height} (world heights), set_layers {layers: [[material, tile]]}, holes {center, radius, hole}. probe [x, z] returns the height there.",
            "inputSchema": { "type": "object", "properties": {
                "op": { "type": "string" }, "id": { "type": "integer" }, "center": { "type": "array" }, "points": { "type": "array" }, "mode": { "type": "string" },
                "radius": { "type": "number" }, "strength": { "type": "number" }, "height": { "type": "number" }, "step": { "type": "number" }, "layer": { "type": "integer" },
                "spacing": { "type": "number" }, "min": { "type": "array" }, "max": { "type": "array" }, "margin": { "type": "number" }, "from": vec3_schema("ramp start"),
                "to": vec3_schema("ramp end"), "width": { "type": "number" }, "iterations": { "type": "integer" }, "talus": { "type": "number" },
                "rock_slope": { "type": "number" }, "top_height": { "type": "number" }, "low_height": { "type": "number" }, "layers": { "type": "array" },
                "hole": { "type": "boolean" }, "probe": { "type": "array" }
            }, "required": ["op"] }
        }),
        json!({
            "name": "duplicate",
            "description": "Array duplicate like Hammer's paste special: count copies of ids (or the selection), each offset further and optionally rotated around Y by rotate_y degrees per copy (about pivot or each copy's center). linked makes linked groups that stay in sync.",
            "inputSchema": { "type": "object", "properties": {
                "ids": { "type": "array", "items": { "type": "integer" } }, "offset": vec3_schema("offset per copy"), "count": { "type": "integer" },
                "linked": { "type": "boolean" }, "rotate_y": { "type": "number" }, "pivot": vec3_schema("rotation pivot")
            } }
        }),
        json!({
            "name": "run_script",
            "description": "Runs a GodotTrench MCP script: {\"format\": \"godottrench-mcp-script\", \"steps\": [{\"tool\", \"args\", \"save\"}]}. A step's result is saved under its save name; \"$name.path\" in later args is replaced by that value, \"${name.path}\" inside strings by its text. $script_dir and $project are predefined. Pass path to a .json script, or steps inline. screenshot and simulate_input cannot run inside scripts.",
            "inputSchema": { "type": "object", "properties": {
                "path": { "type": "string" }, "steps": { "type": "array" }, "vars": { "type": "object" }, "continue_on_error": { "type": "boolean" }
            } }
        }),
        json!({
            "name": "get_game_config",
            "description": "Entity definitions and texture settings of the loaded game config. Pass classname for one definition.",
            "inputSchema": { "type": "object", "properties": { "classname": { "type": "string" } } }
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Echo;
    impl ToolExecutor for Echo {
        fn call(&self, name: &str, args: Value) -> ToolResult {
            ToolResult::Json(json!({ "tool": name, "args": args }))
        }
    }

    #[test]
    fn initialize_and_list() {
        let init = handle(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26"}}), &Echo).unwrap();
        assert_eq!(init["result"]["protocolVersion"], "2025-03-26");
        assert!(handle(json!({"jsonrpc":"2.0","method":"notifications/initialized"}), &Echo).is_none());
        let list = handle(json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}), &Echo).unwrap();
        let tools = list["result"]["tools"].as_array().unwrap();
        assert!(tools.len() >= 15);
        for t in tools {
            assert!(t["name"].is_string() && t["inputSchema"]["type"] == "object");
        }
    }

    #[test]
    fn call_routes_to_executor() {
        let r = handle(json!({"jsonrpc":"2.0","id":"a","method":"tools/call","params":{"name":"get_state","arguments":{"x":1}}}), &Echo).unwrap();
        assert_eq!(r["id"], "a");
        assert_eq!(r["result"]["structuredContent"]["tool"], "get_state");
        let bad = handle(json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"nope"}}), &Echo).unwrap();
        assert_eq!(bad["error"]["code"], -32602);
        let missing = handle(json!({"jsonrpc":"2.0","id":4,"method":"bogus"}), &Echo).unwrap();
        assert_eq!(missing["error"]["code"], -32601);
    }

    #[test]
    fn image_results_are_base64() {
        let v = tool_result_json(ToolResult::Image { png: vec![1, 2, 3], note: "n".into() });
        assert_eq!(v["content"][0]["data"], "AQID");
        assert_eq!(v["content"][0]["mimeType"], "image/png");
    }
}
