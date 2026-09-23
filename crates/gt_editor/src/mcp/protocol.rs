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
for any entity, and run_script replays a JSON list of these calls (see examples/mcp in the repository). \
Each call, and each run_script as a whole, is one undo step named \"MCP: ...\", so the person can take it back in one go.";

fn vec3_schema(desc: &str) -> Value {
    json!({ "type": "array", "items": { "type": "number" }, "minItems": 3, "maxItems": 3, "description": desc })
}

fn numbers() -> Value {
    json!({ "type": "array", "items": { "type": "number" } })
}

/// [[x, z] or [x, y, z], ...]
fn points_schema() -> Value {
    json!({ "type": "array", "items": { "type": "array", "items": { "type": "number" }, "minItems": 2, "maxItems": 3 } })
}

/// A number, or [u, v] for separate axes.
fn scalar_or_pair() -> Value {
    json!({ "type": ["number", "array"], "items": { "type": "number" }, "minItems": 2, "maxItems": 2 })
}

fn outputs_schema() -> Value {
    json!({ "type": "array", "items": { "type": "object", "properties": {
        "output": { "type": "string" }, "target": { "type": "string" }, "input": { "type": "string" },
        "parameter": { "type": "string" }, "delay": { "type": "number" }, "times": { "type": "integer" }
    }, "required": ["output", "target", "input"] } })
}

pub fn tool_definitions() -> Vec<Value> {
    let view_enum = json!(["window", "3d", "top", "front", "side"]);
    let mesh_ops = super::tools::mesh_op_names().join("|");
    let presets = gt_doc::scatter::PRESETS.join("|");
    vec![
        json!({
            "name": "get_state",
            "description": "Editor state: open map, modified flag, counts, selection, grid, tool, cameras, game config.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "list_nodes",
            "description": "Lists map nodes (layers, groups, entities, brushes) with ids, names, parents and bounds. Filters combine: type, classname, layer (id or name), material (a face uses it), property ({key: value} on entities, * matches anything, so {\"model\": \"*crate*\"}; the matched keys are returned), box ({min, max}, nodes whose bounds touch it) and selected_only.",
            "inputSchema": { "type": "object", "properties": {
                "type": { "type": "string", "enum": ["layer", "group", "entity", "brush", "instance", "mesh", "terrain", "scatter"] },
                "classname": { "type": "string" },
                "layer": { "type": ["integer", "string"] },
                "material": { "type": "string" },
                "property": { "type": "object" },
                "box": { "type": "object", "properties": { "min": vec3_schema("box min"), "max": vec3_schema("box max") } },
                "selected_only": { "type": "boolean" },
                "limit": { "type": "integer", "default": 200 }
            } }
        }),
        json!({
            "name": "get_node",
            "description": "Full data of one node exactly as stored in the .gtm file. compact leaves out vertices, faces and heights and gives bounds, layer, materials and entity keys instead.",
            "inputSchema": { "type": "object", "properties": { "id": { "type": "integer" }, "compact": { "type": "boolean" } }, "required": ["id"] }
        }),
        json!({
            "name": "summarize_map",
            "description": "One call overview of the open map: layers with bounds and contents, entity counts by class, the I/O links as \"source.output -> target.input\" (up to limit), materials in use with face counts, model paths in entity keys, missing materials and entity classes, and the worldspawn keys. Start here on a map you did not build, then use list_nodes filters and get_node compact.",
            "inputSchema": { "type": "object", "properties": { "limit": { "type": "integer", "default": 100 } } }
        }),
        json!({
            "name": "changes_since",
            "description": "What changed in the open map: nodes added and removed (a new group is listed once with nodes_inside), and nodes changed with what changed (moved with offset, rotated, geometry, uv, materials, paint, properties with [old, new], outputs, parent, name, hidden, locked), plus counts by type and by layer. Compares with the last save by default, undo_steps back in the history (1 is before the last call or script, see get_state undo), or a map file (.gtm or .json, such as an older version from git). Use it to show the person what you changed.",
            "inputSchema": { "type": "object", "properties": {
                "undo_steps": { "type": "integer" }, "file": { "type": "string" }, "limit": { "type": "integer", "default": 100, "description": "most entries per list" }
            } }
        }),
        json!({
            "name": "run_action",
            "description": "Runs an editor command. Actions: new_map, save, undo {steps}, redo {steps}, delete, duplicate, select_all, select_none (clears the selection, whatever tool is active), select_inverse, select_touching, select_inside, select_siblings, select_same_material, group, ungroup, hide_selected, isolate_selected, unhide_all, lock_selected, unlock_all, grid_up, grid_down, toggle_snap, toggle_uv_lock, toggle_textured, csg_subtract {carve_material: cutter|target} (carved faces take the cutter's material, or with target the material of the target face they are cut from), csg_merge, csg_intersect, csg_hollow, rotate {axis, degrees}, flip {axis}, focus_selection, create_brush_entity {classname}, place_entities {classnames, at, normal, row}, move_to_world, add_layer, snap_vertices, apply_material {material}, create_decal {material, at, normal, size: [width, height]} (a decal sheet on the surface at `at` facing along normal, one metre square by default, blended over the surface and never z-fighting in Godot: use it for grime, stains, cracks, posters and signs instead of thin brushes or illusionary boxes, and move, rotate or scale it like a mesh), copy and cut (return the clipboard text), paste {text, offset, origin, parent} (text from copy or cut, moved by offset or centered on origin), nudge {offset}, set_tool {tool}, move_vertices {vertices, offset} (vertex tool on the selected brushes: moves the listed corner, edge midpoint or face center positions, refused when a brush would turn concave), clip_apply {point, normal or points: three points, keep: front|back|both} (splits the selected brushes by that plane, front is where the normal points; without a plane it applies the clip tool's points), insert_prefab {path, origin, angles, fixup}, explode_instances, create_displacement {power}, remove_displacement, sew_displacements, sculpt {center, mode: raise|lower|smooth|flatten|noise|terrace|paint_alpha|erase_alpha|paint_layer|hole|unhole, radius, strength, height, layer, step, seed} (displacements of selected brushes and terrains), sprinkle {center, items, radius, density, min_spacing, seed} (point entities, see the scatter tool for trees and foliage), set_shade {shade: textured|flat|lit}, edit_mesh, convert_to_mesh, convert_to_brushes, join_meshes, mesh_op {op: {mesh_ops}}, duplicate_linked, unlink_groups, set_cordon, toggle_cordon, clear_cordon, store_camera {slot}, recall_camera {slot}, new_tab, next_tab, close_tab {discard} (refused with unsaved changes unless discard is true), hotspot_texture, terrain_auto_paint {sea_level} (selected terrains, bands measured from sea_level, else from the lowest point; the value stays set for the menu), reload_models, reload_materials (rescans the project's texture folder now, the editor also picks up new or changed material files on its own within about two seconds), open_godot_editor, run_godot_project, focus_godot, build_in_godot (full build in the connected Godot editor of the map as shown, saved or not), toggle_live_mode. save needs a map that already has a file (use map_file save with a path otherwise), and actions that open file dialogs are refused. CSG actions and clip_apply replace the brushes they touch and return replaced {\"old id\": [new ids]} (removed brushes such as cutters map to []); run_script rewrites saved ids from it. Failed actions return an error, status is only the message this action set.".replace("{mesh_ops}", &mesh_ops),
            "inputSchema": { "type": "object", "properties": {
                "action": { "type": "string" },
                "args": { "type": "object" }
            }, "required": ["action"] }
        }),
        json!({
            "name": "create_brush",
            "description": "Creates brushes inside the given bounds and selects them. shape defaults to box. openings are boxes [[min], [max]] carved out with CSG (doors, windows), or {min, max, count, step, rows, row_step} to repeat one box along step and row_step, a row of windows over several floors in one entry. hollow turns the box into walls of that thickness. uv_scale sets world units per texel. entity {classname, properties, outputs} wraps the brushes in a brush entity. shape text builds block letters of text (A-Z, 0-9, . , ! ? ' - + : /, \n for a new line) as large as fit the bounds: lying on the floor with their tops towards -Z by default, standing with their tops up when standing is true; spacing is the gap between letters in font cells (default 1) and letters lists each character's brush ids.",
            "inputSchema": { "type": "object", "properties": {
                "min": vec3_schema("minimum corner"),
                "max": vec3_schema("maximum corner"),
                "openings": { "type": "array", "items": { "anyOf": [
                    { "type": "array", "items": vec3_schema("corner"), "minItems": 2, "maxItems": 2 },
                    { "type": "object", "properties": { "min": vec3_schema("corner"), "max": vec3_schema("corner"), "count": { "type": "integer" }, "step": vec3_schema("offset per copy"),
                        "rows": { "type": "integer" }, "row_step": vec3_schema("offset per row") }, "required": ["min", "max"] }
                ] } },
                "hollow": { "type": "number" },
                "uv_scale": { "type": "number" },
                "entity": { "type": "object", "properties": { "classname": { "type": "string" }, "properties": { "type": "object" }, "outputs": outputs_schema() } },
                "shape": { "type": "string", "enum": ["box", "cylinder", "cone", "sphere", "wedge", "spike", "arch", "pipe", "stairs", "spiral_stairs", "gable", "text"] }, "turns": { "type": "number" }, "ridge_x": { "type": "boolean" },
                "text": { "type": "string" }, "standing": { "type": "boolean" }, "spacing": { "type": "number" },
                "material": { "type": "string" },
                "sides": { "type": "integer" },
                "thickness": { "type": "number" },
                "steps": { "type": "integer" },
                "parent": { "type": "integer", "description": "layer, group or brush entity id, unlocked" }
            }, "required": ["min", "max"] }
        }),
        json!({
            "name": "create_mesh",
            "description": "Creates an editable polygon mesh and selects it. Shapes: cuboid, cylinder, cone, sphere, torus, grid, arch_wall, gable_roof, spire, lathe (profile of [radius, height] around center), prism (footprint of [x, z] between bottom and top). smooth_angle enables smooth shading. uv_scale projects face aligned textures, rotate [x, y, z] degrees turns it around its center like Godot's rotation_degrees (YXZ order, so [0, 0, 90] lays a cylinder along X and [90, 0, 0] along Z), rotate_y turns it around Y only, translate moves it, roughen {amount, seed} displaces vertices into rock shapes.",
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
                "smooth_angle": { "type": "number" }, "parent": { "type": "integer", "description": "layer, group or brush entity id, unlocked" },
                "uv_scale": { "type": "number" }, "translate": vec3_schema("moves the mesh after building"), "rotate": vec3_schema("degrees about X, Y and Z, Godot rotation_degrees"), "rotate_y": { "type": "number" },
                "roughen": { "type": "object", "properties": { "amount": { "type": "number" }, "seed": { "type": "integer" } } }
            }, "required": ["shape"] }
        }),
        json!({
            "name": "mesh_edit",
            "description": "Topology operation on a mesh by vertex, edge and face indices (see get_node). ops: extrude_faces {faces, offset|distance}, extrude_edges {edges, offset}, inset {faces, thickness}, loop_cut {edges:[[a,b]], cuts}, bisect {point, normal, faces?, delete: front|back, fill}, subdivide {faces}, merge {verts, at?}, weld {distance}, fill {verts}, delete_faces {faces}, delete_faces_in_box {min, max} (faces whose center lies inside, e.g. door and window holes), set_material_in_box {min, max, material}, delete_vertices {verts}, bevel_edges {edges, width}, bevel_vertices {verts, width}, translate {verts, offset}, smooth {verts, factor, iterations}, solidify {thickness}, flip {faces}, triangulate {faces}, smooth_angle {angle}, set_material {faces, material}, separate {faces}. Indices outside the mesh are an error.",
            "inputSchema": { "type": "object", "properties": {
                "id": { "type": "integer" }, "op": { "type": "string" },
                "faces": { "type": "array", "items": { "type": "integer" } },
                "verts": { "type": "array", "items": { "type": "integer" } },
                "edges": { "type": "array", "items": { "type": "array", "items": { "type": "integer" } } },
                "offset": vec3_schema("offset"), "point": vec3_schema("plane point"), "normal": vec3_schema("plane normal"),
                "distance": { "type": "number" }, "thickness": { "type": "number" }, "width": { "type": "number" }, "cuts": { "type": "integer" },
                "at": vec3_schema("merge target, default the vertices' center"), "min": vec3_schema("box min"), "max": vec3_schema("box max"),
                "material": { "type": "string" }, "factor": { "type": "number" }, "iterations": { "type": "integer" }, "angle": { "type": "number" },
                "delete": { "type": "string", "enum": ["front", "back"] }, "fill": { "type": "boolean" }
            }, "required": ["id", "op"] }
        }),
        json!({
            "name": "texture",
            "description": "Texture alignment on brush and mesh faces. faces are [[node, face], ...], the selection when omitted. ops: apply {material, reset}, justify {mode: left|right|top|bottom|center|fit|fit_width|fit_height, treat_as_one}, align_view, reset {scale}, wrap {source: [node, face], material} (continues the source alignment across edges), shift {texels: [u, v]}, scale {factor}, rotate {degrees}, density {units_per_texel}, pick (eyedropper into the clipboard and current material), paste {with_material}, mesh_uv {kind: planar|planar_x|planar_y|planar_z|box|cylinder_x|cylinder_y|cylinder_z|sphere|view|unfold|world|bake|clear|normalize|pack} (explicit UVs on mesh faces at the texel density of each face's texture and scale: planar projects all faces along the main axis of their average normal as one piece, cylinder alone means cylinder_y, world gives each face the projection of a reset brush face, bake writes the current planar look as explicit UVs, clear drops them), uv_adjust {adjust: flip_u|flip_v|rotate {degrees}|scale {factor}|move {texels: [u, v]}|center {texels: [u, v]}|align_horizontal|align_vertical|straighten|fit {rect: [x, y, w, h] in texels like shift, the whole texture by default}|fit_hotspot|snap (to image pixels), corners: [[mesh, face, corner], ...] (every explicit corner of faces when omitted), stitch (default true, corners sharing a vertex and UV move together)}, set_hotspots {material, rects: [[x, y, w, h]]}, material_info {material}, get. Returns the faces' materials and UVs.",
            "inputSchema": { "type": "object", "properties": {
                "op": { "type": "string" },
                "faces": { "type": "array", "items": { "type": "array", "items": { "type": "integer" } } },
                "source": { "type": "array", "items": { "type": "integer" } },
                "material": { "type": "string" }, "mode": { "type": "string" }, "kind": { "type": "string" },
                "treat_as_one": { "type": "boolean" }, "reset": { "type": "boolean" }, "with_material": { "type": "boolean" },
                "texels": { "type": "array", "items": { "type": "number" } }, "factor": scalar_or_pair(), "scale": scalar_or_pair(),
                "degrees": { "type": "number" }, "units_per_texel": { "type": "number" },
                "rects": { "type": "array", "items": { "type": "array", "items": { "type": "number" } } },
                "adjust": { "type": "string" }, "corners": { "type": "array", "items": { "type": "array", "items": { "type": "integer" } } },
                "stitch": { "type": "boolean" }, "rect": { "type": "array", "items": { "type": "number" } }
            }, "required": ["op"] }
        }),
        json!({
            "name": "create_terrain",
            "description": "Creates a heightmap terrain. shape: flat, hills, mountain, island, valley, ridges. layers are [material, world units per repeat] for up to four blend layers (base, slope, peak, low), auto painted from slope and height, with the height bands measured from sea_level when given, else from the lowest point. A band whose layer is missing paints the last layer.",
            "inputSchema": { "type": "object", "properties": {
                "origin": vec3_schema("minimum corner"), "resolution": { "type": "integer", "description": "vertices per side, e.g. 65, 129, 257" },
                "cell_size": { "type": "number" }, "shape": { "type": "string" }, "height": { "type": "number" }, "seed": { "type": "integer" },
                "feature_size": { "type": "number" }, "erosion": { "type": "integer" },
                "layers": { "type": "array", "items": { "type": "array", "items": { "type": ["string", "number"] }, "description": "[material, world units per repeat]" } }, "auto_paint": { "type": "boolean" }, "sea_level": { "type": "number" },
                "parent": { "type": "integer", "description": "layer, group or brush entity id, unlocked" }
            } }
        }),
        json!({
            "name": "import_model",
            "description": "Imports a model file that must exist (.bbmodel, .glb, .gltf, .obj, .stl, .md2 or .md3). mode mesh (default) makes an editable mesh, brushes turns Blockbench .bbmodel cubes into brushes (other formats are refused), prop places an entity that references the model by its res:// path, so the file must be inside the open Godot project.",
            "inputSchema": { "type": "object", "properties": {
                "path": { "type": "string" }, "mode": { "type": "string", "enum": ["mesh", "brushes", "prop"] }, "origin": vec3_schema("placement")
            }, "required": ["path"] }
        }),
        json!({
            "name": "create_entity",
            "description": "Creates a point entity and selects it. origin defaults to [0, 0, 0]. outputs are I/O connections {output, target, input, parameter, delay, times}.",
            "inputSchema": { "type": "object", "properties": {
                "classname": { "type": "string" },
                "origin": vec3_schema("position"),
                "angles": vec3_schema("pitch yaw roll in degrees"),
                "properties": { "type": "object" },
                "outputs": outputs_schema(),
                "parent": { "type": "integer", "description": "layer or group id, unlocked" }
            }, "required": ["classname"] }
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
                "outputs": outputs_schema()
            }, "required": ["id"] }
        }),
        json!({
            "name": "select",
            "description": "Changes the selection. faces are [brush or mesh id, face index] pairs. Unknown ids are an error, hidden or locked nodes are left out and listed in skipped, like clicking them in a view.",
            "inputSchema": { "type": "object", "properties": {
                "ids": { "type": "array", "items": { "type": "integer" } },
                "faces": { "type": "array", "items": { "type": "array", "items": { "type": "integer" } } },
                "mode": { "type": "string", "enum": ["replace", "add", "remove", "clear"], "default": "replace" }
            } }
        }),
        json!({
            "name": "transform",
            "description": "Transforms the selection as one undo step, applied in this order: translate, rotate about an axis, flip, scale_to new bounds. rotate (without center) and flip pivot on the selection center at that step, so after a translate they turn the selection in place.",
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
            "description": "new, open or save .gtm maps, import or export Quake/TrenchBroom .map files, import Hammer .vmf files (func_instance maps are inlined as groups), convert textures. new and open keep a map with unsaved changes in its own tab, like the File menu. save without path saves to the current file. export_map returns the written .map path. import_map and import_vmf report the materials the project lacks and how many of them the .vmt/.vtf, .wad or .wal files near the map could provide; textures: \"auto\" converts those, or a folder or file path converts them from there. convert_textures {path, only_missing, overwrite} turns a folder (or one .wad) of Valve, Quake or Half-Life textures into PNGs and .tres materials in the project texture folder, named so imported faces find them.",
            "inputSchema": { "type": "object", "properties": {
                "op": { "type": "string", "enum": ["new", "open", "open_tab", "save", "import_map", "import_vmf", "convert_textures", "export_map"] },
                "path": { "type": "string" },
                "textures": { "type": "string", "description": "import_map, import_vmf: \"auto\" or a folder or file to convert the map's missing textures from" },
                "only_missing": { "type": "boolean", "description": "convert_textures: only what the open map lacks" },
                "overwrite": { "type": "boolean", "description": "convert_textures: replace files already in the project" }
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
                "grid": { "type": "number", "exclusiveMinimum": 0, "description": "positive, clamped to 0.125..1024" }, "snap": { "type": "boolean" }, "uv_lock": { "type": "boolean" },
                "tool": { "type": "string" }, "material": { "type": "string" }, "textured": { "type": "boolean" },
                "shade": { "type": "string", "enum": ["textured", "flat", "lit", "wireframe"] },
                "mesh_component": { "type": "string", "enum": ["vertex", "edge", "face"] },
                "scatter_items": { "type": "array", "items": { "type": "string" } },
                "brush_radius": { "type": "number" },
                "sculpt_mode": { "type": "string" },
                "ui_scale": { "type": "number", "description": "Interface scale, 1 is 100%" },
                "follow_display_scaling": { "type": "boolean" },
                "live_mode": { "type": "boolean", "description": "Send edits to a Godot editor that has the map open before saving" },
                "live_link_port": { "type": "integer", "description": "Port of the Godot live link, 7842 unless the Godot project changed godottrench/live_link_port" }
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
            "description": "PNG of the whole window or of one viewport. Use after changes to verify them visually. width and height render a view's camera offscreen at that size (up to 4096), larger than its docked pane. overlays: false renders a beauty shot without the grid, entity boxes, trigger volumes, edges, selection outlines and tint, links and gizmos, the default for offscreen captures. A viewport capture without width and height shows the pane as it is unless overlays is false. source: godot instead asks the Godot editor connected over the live link to render the scene using this map, with its real lights, environment, fog and post processing, from the 3d view's camera or from position and look_at (map units), 1280x720 unless width and height are given. It needs the map saved in the open project and a scene open in Godot whose FuncGodotMap builds it. Without live mode Godot first builds the map as shown here, like build_in_godot; build: false captures what Godot has. Errors and warnings Godot logged meanwhile come back as text.",
            "inputSchema": { "type": "object", "properties": {
                "target": { "type": "string", "enum": view_enum, "default": "window" },
                "source": { "type": "string", "enum": ["editor", "godot"], "default": "editor" },
                "width": { "type": "integer" }, "height": { "type": "integer" },
                "overlays": { "type": "boolean", "description": "Draw the editor overlays. Defaults to false with width and height, true without." },
                "position": vec3_schema("godot source: camera position, default the 3d view's"),
                "look_at": vec3_schema("godot source: look target, default along the 3d view"),
                "fov": { "type": "number", "description": "godot source: horizontal field of view in degrees, default the 3d view's" },
                "build": { "type": "boolean", "description": "godot source: build the map as shown here before capturing, default true unless live mode is on" }
            } }
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
            "description": "Checks the map for problems: invalid brushes, entities without definitions, broken I/O targets, output names the entity definitions do not know, inputs the target answers to neither as a declared input nor as a method or property of its Godot class or script (Node methods such as set_visible are valid inputs), empty brush entities, face materials the project does not have (missing_material, one per material, after looking for material files added since the last scan), faces of two brushes that lie on one plane and overlap where both still draw in the Godot build, so they z-fight (coplanar_faces, one per brush pair, naming the other brush and where they overlap; tool textures, faces the build hides and overlaps under 3 units wide are left out).",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "scatter",
            "description": "Scatter trees, rocks and foliage into scatter sets that live on their own layers and only cover their target surfaces. A set is its own palette: its items are the models it scatters, each switched on or off for painting. ops: palette {items: [model path or {source, weight, scale: [min, max], spacing, align, tilt, sink, enabled}], preset: {presets}, kind: props|foliage} (the listed models become the active set's painted models, the others are switched off and keep their instances; without an active set this starts one), install_models (writes the built-in nature models to res://godottrench/nature), new_set {name, preset or items, targets, collision, cast_shadows, visibility_range, chunk_size, material}, activate {id}, paint {center [x, y, z] or [x, z] dropped onto what is below, scattered props included, normal, radius, density, slope, height, falloff, avoid_other_sets, only_targets, exposed_only, clearance, seed, erase} (erase true removes instead; a set without targets takes the surface under the first dab as its target, which can be another set's instances), stroke {points, ...}, erase {center, radius, amount}, fill {id, targets, density, slope, height, exposed_only, clearance, seed} (whole target area; exposed_only skips points with geometry above them within clearance units, default 512, so nothing grows under slabs), toggle_target {target} (a scatter set works too, so foliage grows on scattered rocks), material {material, item} (retextures the whole set, or one item when item is given, empty clears it), clear, get {id}, to_entities {id}. Items or a preset passed to paint, stroke or fill become the active set's painted models; other settings apply to that call only. Painting a set whose models are all switched off is an error.".replace("{presets}", &presets),
            "inputSchema": { "type": "object", "properties": {
                "op": { "type": "string" }, "id": { "type": "integer" }, "name": { "type": "string" },
                "items": { "type": "array", "items": { "type": ["string", "object"] } }, "preset": { "type": "string", "enum": gt_doc::scatter::PRESETS },
                "kind": { "type": "string", "enum": ["props", "foliage"] },
                "center": numbers(), "points": points_schema(), "normal": vec3_schema("surface normal for paint, default up"),
                "erase": { "type": "boolean" }, "avoid_other_sets": { "type": "boolean" },
                "targets": { "type": "array", "items": { "type": "integer" } }, "target": { "type": "integer" },
                "radius": { "type": "number" }, "density": { "type": "number" }, "slope": numbers(), "height": numbers(),
                "falloff": { "type": "number" }, "amount": { "type": "number" }, "seed": { "type": "integer" }, "only_targets": { "type": "boolean" },
                "exposed_only": { "type": "boolean" }, "clearance": { "type": "number" },
                "collision": { "type": "string", "enum": ["none", "convex", "trimesh"] }, "cast_shadows": { "type": "boolean" }, "visibility_range": { "type": "number" },
                "output": { "type": "string", "enum": ["set", "entities"] }, "overwrite": { "type": "boolean" },
                "chunk_size": { "type": "number" }, "material": { "type": "string" }, "item": { "type": "integer" }
            }, "required": ["op"] }
        }),
        json!({
            "name": "blend",
            "description": "Material blending. Terrains blend their four layers, displacements and brush or mesh faces blend towards a blend material stored per face. ops: set_material {faces or ids, material}, clear_material, dab {center, mode: paint|erase|smooth|sharpen|noise|slope|height, falloff: smooth|linear|constant|spray, radius, strength 0..1, layer, slope [min, max], height [min, max], noise_scale, seed}, stroke {points, ...}, weights {center, id} (terrain weights at a point). Targets are the selection (ids selects first) or everything blendable.",
            "inputSchema": { "type": "object", "properties": {
                "op": { "type": "string" }, "ids": { "type": "array", "items": { "type": "integer" } },
                "faces": { "type": "array", "items": { "type": "array", "items": { "type": "integer" } } }, "material": { "type": "string" },
                "center": numbers(), "points": points_schema(), "mode": { "type": "string" }, "falloff": { "type": "string" },
                "radius": { "type": "number" }, "strength": { "type": "number" }, "layer": { "type": "integer" }, "slope": numbers(), "height": numbers(),
                "noise_scale": { "type": "number" }, "seed": { "type": "integer" }, "id": { "type": "integer" }
            }, "required": ["op"] }
        }),
        json!({
            "name": "gameplay",
            "description": "Gameplay setups with the Godot addon's entity library. ops: make_door {ids, kind: hinged|sliding, side: left|right, angle, direction: up|down|left|right, lip, trigger, properties} (hinge and travel computed from the brushes), make_platform {ids, travel [x, y, z], mode 0 toggle|1 ping pong|2 once, properties}, make_button {ids, target, input, properties}, brush_entity {ids, classname, properties, outputs}, volume {classname: trigger_once|trigger_multiple|trigger_call|trigger_spawn_area|trigger_hurt|trigger_teleport|trigger_push, min, max, properties, outputs}, link {from, to, output, input, parameter, delay} (names the target when needed, output and input default to the only one the definitions offer), place {classname, origin ([x, z] or snap_to_ground drops it onto the ground), angles, properties, outputs}, gizmos {ids} (viewport handle positions). outputs on make_door, make_platform and make_button are added to the new entity next to the ones the wizard wires. Outputs can target /root/Node paths, @groups, targetnames with * wildcards or !activator, calling GDScript or C# methods.",
            "inputSchema": { "type": "object", "properties": {
                "op": { "type": "string" }, "ids": { "type": "array", "items": { "type": "integer" } }, "kind": { "type": "string" }, "side": { "type": "string" },
                "angle": { "type": "number" }, "direction": { "type": "string" }, "lip": { "type": "number" }, "trigger": { "type": "boolean" },
                "travel": vec3_schema("offset in map units"), "mode": { "type": "integer" }, "target": { "type": "string" }, "input": { "type": "string" }, "output": { "type": "string" },
                "classname": { "type": "string" }, "min": vec3_schema("volume min"), "max": vec3_schema("volume max"), "properties": { "type": "object" }, "outputs": outputs_schema(),
                "from": { "type": "integer" }, "to": { "type": "integer" }, "parameter": { "type": "string" }, "delay": { "type": "number" },
                "origin": numbers(), "angles": vec3_schema("pitch yaw roll"), "snap_to_ground": { "type": "boolean" }
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
            "description": "Layers and groups. New objects go into the innermost open group, else the current layer. ops: add_layer {name, default Layer N} (becomes current), add_group {name, parent, open (default true)}, open_group {id} (a group), close_group {all}, set_current_layer {id}, reparent {ids, parent} (into a layer or group, brushes and meshes also into a brush entity), rename {id, name} (entities get a targetname), set_flags {ids, hidden, locked, omit_from_export}. Locked layers and groups accept no new objects.",
            "inputSchema": { "type": "object", "properties": {
                "op": { "type": "string" }, "name": { "type": "string" }, "id": { "type": "integer" }, "parent": { "type": "integer" },
                "ids": { "type": "array", "items": { "type": "integer" } }, "open": { "type": "boolean" }, "all": { "type": "boolean" },
                "hidden": { "type": "boolean" }, "locked": { "type": "boolean" }, "omit_from_export": { "type": "boolean" }
            }, "required": ["op"] }
        }),
        json!({
            "name": "set_map_properties",
            "description": "Sets worldspawn keys, e.g. message, sun_angles, sun_color, sun_energy, ambient_color, sky_top_color, sky_horizon_color, sky_ground_color, fog_color, fog_density, ambient_energy, sky_energy, glow_intensity (Godot glow for emissive materials and lamps), ssr (1 turns on Godot's screen space reflections for wet streets and glossy floors). null removes a key.",
            "inputSchema": { "type": "object", "properties": { "properties": { "type": "object" } }, "required": ["properties"] }
        }),
        json!({
            "name": "terrain_edit",
            "description": "Shapes a terrain (id, else the selected or first one). ops: sculpt {center, mode: raise|lower|smooth|flatten|noise|terrace|paint_layer|hole|unhole, radius, strength, height, step, layer, seed (noise pattern, default 7, a path adds one per dab)}, sculpt_path {points, spacing, ...}, paint_path {points, layer, radius, strength}, flatten_rect {min [x, z], max [x, z], height, margin} (building pads), ramp {from [x, y, z], to [x, y, z], width, margin} (roads and slopes), erode {iterations, talus}, auto_paint {rock_slope, top_height, low_height} (world heights) or auto_paint {sea_level} (the default bands measured from sea_level, from the lowest point without it), set_layers {layers: [[material, tile, detile, sharpen]]} (detile 0..1 breaks up the repeat for paths, sharpen 0..1 keeps it crisp), holes {center, radius, hole}. probe [x, z] returns the height there, after the op, or alone without op to only read it.",
            "inputSchema": { "type": "object", "properties": {
                "op": { "type": "string" }, "id": { "type": "integer" }, "center": numbers(), "points": points_schema(), "mode": { "type": "string" },
                "radius": { "type": "number" }, "strength": { "type": "number" }, "height": { "type": "number" }, "step": { "type": "number" }, "layer": { "type": "integer" },
                "spacing": { "type": "number" }, "min": numbers(), "max": numbers(), "margin": { "type": "number" }, "from": vec3_schema("ramp start"),
                "to": vec3_schema("ramp end"), "width": { "type": "number" }, "iterations": { "type": "integer" }, "talus": { "type": "number" },
                "rock_slope": { "type": "number" }, "top_height": { "type": "number" }, "low_height": { "type": "number" },
                "layers": { "type": "array", "items": { "type": "array", "items": { "type": ["string", "number"] }, "description": "[material, tile, detile, sharpen]" } },
                "hole": { "type": "boolean" }, "probe": numbers(), "sea_level": { "type": "number" }, "seed": { "type": "integer" }
            } }
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
            "description": "Runs a GodotTrench MCP script: {\"format\": \"godottrench-mcp-script\", \"steps\": [{\"tool\", \"args\", \"save\"}]}. A step's result is saved under its save name; \"$name.path\" in later args is replaced by that value, \"${name.path}\" inside strings by its text, and $$ is a literal $ (a $ that starts no reference, like $5, stays as it is, an unknown $name is an error). \"$a.ids + $b.ids\" joins saved values into one array, and id lists also accept nested arrays such as [\"$a.ids\", \"$b.ids\"]. When a step replaces brushes (CSG, clip_apply) the ids in earlier saved results follow: in a list a replaced id becomes all of its replacements, a single id its one replacement (a list for several, null when removed). $script_dir (the script's folder, for inline steps the project or working directory) and $project (only while a project is open) are predefined. Pass path to a .json script, or steps inline. screenshot and simulate_input cannot run inside scripts. The whole run is one undo step named \"MCP: <label>, N steps\" (label defaults to the file name) and returned as undo.",
            "inputSchema": { "type": "object", "properties": {
                "path": { "type": "string" }, "vars": { "type": "object" }, "continue_on_error": { "type": "boolean" },
                "label": { "type": "string", "description": "what the script does, for the undo step, such as \"Dress the kitchen\"" },
                "steps": { "type": "array", "items": { "type": "object", "properties": {
                    "tool": { "type": "string" }, "args": { "type": "object" }, "save": { "type": "string" }, "note": { "type": "string" }
                } } }
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

    fn check_schema(path: &str, v: &Value) {
        let Some(o) = v.as_object() else { return };
        assert!(!o.is_empty(), "{path} has an empty schema");
        let is_array = o.get("type").is_some_and(|t| t == "array" || t.as_array().is_some_and(|a| a.iter().any(|x| x == "array")));
        assert!(!is_array || o.contains_key("items"), "{path} is an array without items");
        if let Some(props) = o.get("properties").and_then(|p| p.as_object()) {
            for (k, p) in props {
                check_schema(&format!("{path}.{k}"), p);
            }
        }

        if let Some(items) = o.get("items") {
            check_schema(&format!("{path}[]"), items);
        }
    }

    #[test]
    fn schemas_are_complete_and_lists_match_the_editor() {
        let tools = tool_definitions();
        for t in &tools {
            check_schema(t["name"].as_str().unwrap(), &t["inputSchema"]);
        }

        let text = |name: &str| tools.iter().find(|t| t["name"] == name).unwrap()["description"].as_str().unwrap().to_string();
        for op in crate::mcp::tools::mesh_op_names() {
            assert!(text("run_action").contains(&op), "{op}");
        }

        for preset in gt_doc::scatter::PRESETS {
            assert!(text("scatter").contains(preset), "{preset}");
        }
    }

    #[test]
    fn image_results_are_base64() {
        let v = tool_result_json(ToolResult::Image { png: vec![1, 2, 3], note: "n".into() });
        assert_eq!(v["content"][0]["data"], "AQID");
        assert_eq!(v["content"][0]["mimeType"], "image/png");
    }
}
