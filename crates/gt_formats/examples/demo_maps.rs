//! Writes the demo and test maps used by the Godot project.
//! cargo run -p gt_formats --example demo_maps

use std::path::PathBuf;

use gt_core::{Aabb, DVec3};
use gt_doc::map::{Group, Instance};
use gt_doc::{Entity, IoConnection, Map, NodeKind, format, ops};
use gt_geom::{Brush, csg};

fn boxb(min: [f64; 3], max: [f64; 3], material: &str) -> Brush {
    Brush::from_aabb(&Aabb::new(DVec3::from(min), DVec3::from(max)), material).expect("valid box")
}

fn io(output: &str, target: &str, input: &str, delay: f64) -> IoConnection {
    IoConnection { output: output.into(), target: target.into(), input: input.into(), parameter: String::new(), delay, times: -1 }
}

fn prefab() -> Map {
    let mut m = Map::new();
    m.properties.insert("classname".into(), "worldspawn".into());
    let layer = m.default_layer();
    m.insert(layer, NodeKind::Brush(boxb([-16.0, 0.0, -16.0], [16.0, 128.0, 16.0], "base/wall")));
    let mut light = Entity::new("light");
    light.origin = DVec3::new(32.0, 64.0, 0.0);
    light.properties.insert("targetname".into(), "plight".into());
    m.insert(layer, NodeKind::Entity(light));
    m
}

fn demo() -> Map {
    let mut m = Map::new();
    m.properties.insert("classname".into(), "worldspawn".into());
    let layer = m.default_layer();

    m.insert(layer, NodeKind::Brush(boxb([-256.0, -16.0, -256.0], [256.0, 0.0, 256.0], "base/floor")));

    // Back wall with a doorway cut by CSG subtraction.
    let wall = boxb([-256.0, 0.0, -256.0], [256.0, 128.0, -240.0], "base/wall");
    let doorway = boxb([-32.0, 0.0, -300.0], [32.0, 96.0, -200.0], "base/wall");
    for piece in csg::subtract(&wall, &doorway) {
        m.insert(layer, NodeKind::Brush(piece));
    }

    let props = m.insert(layer, NodeKind::Group(Group::new("props")));
    m.insert(props, NodeKind::Brush(boxb([96.0, 0.0, 96.0], [160.0, 64.0, 160.0], "base/metal")));
    let crate_box = boxb([-128.0, 0.0, 64.0], [-80.0, 48.0, 112.0], "base/metal");
    let center = crate_box.center();
    m.insert(props, NodeKind::Brush(crate_box.transformed(&ops::rotation_about(center, DVec3::Y, 30.0), true)));

    let mut door = Entity::new("func_door");
    door.properties.insert("targetname".into(), "door1".into());
    door.outputs.push(io("opened", "lamp", "turn_on", 0.0));
    let door = m.insert(layer, NodeKind::Entity(door));
    m.insert(door, NodeKind::Brush(boxb([-32.0, 0.0, -252.0], [32.0, 96.0, -244.0], "base/metal")));

    let mut button = Entity::new("func_button");
    button.properties.insert("targetname".into(), "button1".into());
    button.outputs.push(io("pressed", "door1", "open", 0.1));
    let button = m.insert(layer, NodeKind::Entity(button));
    m.insert(button, NodeKind::Brush(boxb([48.0, 32.0, -240.0], [64.0, 48.0, -232.0], "base/metal")));

    let mut trigger = Entity::new("trigger_area");
    trigger.outputs.push(io("body_entered", "door1", "open", 0.0));
    let trigger = m.insert(layer, NodeKind::Entity(trigger));
    // Kept off the floor so the static world does not count as a body entering it.
    m.insert(trigger, NodeKind::Brush(boxb([-64.0, 16.0, -200.0], [64.0, 96.0, -120.0], "special/trigger")));

    let mut lamp = Entity::new("light");
    lamp.origin = DVec3::new(0.0, 112.0, 0.0);
    lamp.angles = DVec3::new(0.0, 90.0, 0.0);
    lamp.properties.insert("targetname".into(), "lamp".into());
    lamp.properties.insert("light_energy".into(), "2.5".into());
    lamp.properties.insert("start_on".into(), "0".into());
    m.insert(layer, NodeKind::Entity(lamp));

    let mut start = Entity::new("info_player_start");
    start.origin = DVec3::new(0.0, 0.0, 128.0);
    start.angles = DVec3::new(0.0, 45.0, 0.0);
    m.insert(layer, NodeKind::Entity(start));

    m.insert(
        layer,
        NodeKind::Instance(Instance {
            path: "prefab.gtm".into(),
            origin: DVec3::new(200.0, 0.0, -64.0),
            angles: DVec3::new(0.0, 90.0, 0.0),
            fixup: "p1".into(),
        }),
    );

    let omitted = m.add_layer("Omitted");
    if let Some(NodeKind::Layer(l)) = m.get_mut(omitted).map(|n| &mut n.kind) {
        l.omit_from_export = true;
    }

    m.insert(omitted, NodeKind::Brush(boxb([1000.0, 0.0, 1000.0], [1064.0, 64.0, 1064.0], "base/wall")));
    m
}

fn terrain() -> Map {
    use gt_doc::terrain::{self, SculptBrush, SculptMode};
    let mut m = Map::new();
    m.properties.insert("classname".into(), "worldspawn".into());
    let layer = m.default_layer();
    let ground = m.insert(layer, NodeKind::Brush(boxb([0.0, -32.0, 0.0], [256.0, 0.0, 256.0], "base/floor")));
    let top = m.brush(ground).unwrap().faces.iter().position(|f| f.plane.normal.y > 0.5).unwrap();
    terrain::create_displacements(&mut m, &[(ground, top)], 3);
    let faces = terrain::displacement_faces(&m, &[ground]);
    // A single dab of exactly 64 units at the center vertex, falloff shapes the hill around it.
    terrain::sculpt(
        &mut m,
        &faces,
        DVec3::new(128.0, 0.0, 128.0),
        &SculptBrush { mode: SculptMode::Raise, radius: 96.0, strength: 64.0, flatten_height: 0.0, ..Default::default() },
    );
    terrain::sculpt(
        &mut m,
        &faces,
        DVec3::new(64.0, 0.0, 64.0),
        &SculptBrush { mode: SculptMode::PaintAlpha, radius: 48.0, strength: 1.0, flatten_height: 0.0, ..Default::default() },
    );

    let mut decal = Entity::new("infodecal");
    decal.origin = DVec3::new(128.0, 64.0, 128.0);
    decal.properties.insert("texture".into(), "res://demo/textures/base/wall.png".into());
    decal.properties.insert("size".into(), "96 64 96".into());
    m.insert(layer, NodeKind::Entity(decal));

    let cube = m.insert(layer, NodeKind::Brush(boxb([300.0, 0.0, 0.0], [332.0, 32.0, 32.0], "base/metal")));
    terrain::paint_vertices(&mut m, &[cube], DVec3::new(300.0, 0.0, 0.0), 16.0, [1.0, 0.0, 0.0, 1.0], 1.0);
    m
}

/// Meshes, a terrain and a model prop for the Godot geometry tests.
fn geometry() -> Map {
    use gt_geom::heightfield::{TerrainGen, TerrainShape};
    use gt_geom::{Mesh, Terrain, TerrainLayer, mesh_shapes};
    let mut m = Map::new();
    m.properties.insert("classname".into(), "worldspawn".into());
    for (k, v) in [("sun_angles", "-30 60"), ("sky_top_color", "51 90 150"), ("fog_density", "0.004"), ("fog_color", "200 210 220")] {
        m.properties.insert(k.into(), v.into());
    }

    let layer = m.default_layer();

    // A 64 cube with its top extruded by 32: closed, flat shaded, one surface.
    let mut tower = mesh_shapes::cuboid(&Aabb::new(DVec3::new(0.0, 0.0, 0.0), DVec3::new(64.0, 64.0, 64.0)), "base/wall");
    let top = (0..tower.faces.len()).max_by(|a, b| tower.face_normal(*a).y.total_cmp(&tower.face_normal(*b).y)).unwrap();
    let verts = tower.extrude_faces(&[top]);
    tower.transform_vertices(&verts, &gt_core::DMat4::from_translation(DVec3::new(0.0, 32.0, 0.0)));
    m.insert(layer, NodeKind::Mesh(tower));

    // Smooth cylinder with explicit UVs on one face.
    let mut column: Mesh = mesh_shapes::cylinder(&Aabb::new(DVec3::new(128.0, 0.0, 0.0), DVec3::new(192.0, 96.0, 64.0)), 16, "base/metal");
    column.faces[0].uvs = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    m.insert(layer, NodeKind::Mesh(column));

    let mut terrain = Terrain::new(DVec3::new(-320.0, -8.0, -320.0), [17, 17], 40.0, "base/floor");
    terrain.layers.push(TerrainLayer::new("base/wall", 128.0));
    terrain.generate(&TerrainGen { shape: TerrainShape::Hills, height: 96.0, feature_size: 400.0, erosion_iterations: 0, ..Default::default() });
    // Known height at the center vertex for the test.
    let center = terrain.index(8, 8);
    terrain.heights[center] = 160.0;
    terrain.paint_layer(DVec3::new(0.0, 0.0, 0.0), 60.0, 1, 1.0);
    m.insert(layer, NodeKind::Terrain(terrain));

    let mut prop = Entity::new("prop_model");
    prop.origin = DVec3::new(256.0, 0.0, 0.0);
    prop.properties.insert("model".into(), "res://demo/models/bush.bbmodel".into());
    prop.properties.insert("collision".into(), "convex".into());
    m.insert(layer, NodeKind::Entity(prop));
    m
}

/// A small playable cutscene wired entirely through I/O: walk into the trigger and a guide spawns and walks a path,
/// text appears, an explosive barrel goes off and hurts the player, a light switches on and the exit opens.
fn scripted_scene() -> Map {
    let mut m = Map::new();
    m.properties.insert("classname".into(), "worldspawn".into());
    for (k, v) in
        [("message", "Scripted Scene"), ("sun_angles", "-40 -55"), ("sun_energy", "0.6"), ("ambient_color", "60 66 82"), ("sky_top_color", "40 54 92")]
    {
        m.properties.insert(k.into(), v.into());
    }

    let layer = m.default_layer();
    m.insert(layer, NodeKind::Brush(boxb([-320.0, -16.0, -320.0], [320.0, 0.0, 320.0], "base/floor")));
    m.insert(layer, NodeKind::Brush(boxb([-320.0, 0.0, 312.0], [320.0, 248.0, 320.0], "base/wall")));
    m.insert(layer, NodeKind::Brush(boxb([-320.0, 0.0, -320.0], [320.0, 248.0, -312.0], "base/wall")));
    m.insert(layer, NodeKind::Brush(boxb([-320.0, 0.0, -320.0], [-312.0, 248.0, 320.0], "base/wall")));
    m.insert(layer, NodeKind::Brush(boxb([312.0, 0.0, -320.0], [320.0, 248.0, 320.0], "base/wall")));

    let mut start = Entity::new("info_player_start");
    start.origin = DVec3::new(0.0, 0.0, 260.0);
    start.angles = DVec3::new(0.0, 180.0, 0.0);
    m.insert(layer, NodeKind::Entity(start));

    let mut corner_a = Entity::new("path_corner");
    corner_a.origin = DVec3::new(0.0, 8.0, 64.0);
    corner_a.properties.insert("targetname".into(), "corner_a".into());
    corner_a.properties.insert("target".into(), "corner_b".into());
    m.insert(layer, NodeKind::Entity(corner_a));

    let mut corner_b = Entity::new("path_corner");
    corner_b.origin = DVec3::new(-176.0, 8.0, -176.0);
    corner_b.properties.insert("targetname".into(), "corner_b".into());
    m.insert(layer, NodeKind::Entity(corner_b));

    let mut guide = Entity::new("npc_walker");
    guide.origin = DVec3::new(0.0, 8.0, 176.0);
    guide.properties.insert("targetname".into(), "guide".into());
    guide.properties.insert("target".into(), "corner_a".into());
    guide.properties.insert("model".into(), "res://demo/scenes/npc.tscn".into());
    guide.properties.insert("speed".into(), "2.5".into());
    m.insert(layer, NodeKind::Entity(guide));

    let mut hint = Entity::new("game_text");
    hint.origin = DVec3::new(0.0, 96.0, 0.0);
    hint.properties.insert("targetname".into(), "hint".into());
    hint.properties.insert("text".into(), "Follow the guide".into());
    hint.properties.insert("world_size".into(), "0.03".into());
    m.insert(layer, NodeKind::Entity(hint));

    let mut hud = Entity::new("game_text");
    hud.origin = DVec3::new(0.0, 8.0, 240.0);
    hud.properties.insert("targetname".into(), "hpmsg".into());
    hud.properties.insert("place".into(), "hud".into());
    m.insert(layer, NodeKind::Entity(hud));

    let mut lamp = Entity::new("light");
    lamp.origin = DVec3::new(0.0, 200.0, -40.0);
    lamp.properties.insert("targetname".into(), "lamp".into());
    lamp.properties.insert("start_on".into(), "0".into());
    lamp.properties.insert("light_energy".into(), "3.0".into());
    lamp.properties.insert("omni_range".into(), "16".into());
    m.insert(layer, NodeKind::Entity(lamp));

    let mut barrel = Entity::new("prop_physics");
    barrel.origin = DVec3::new(0.0, 24.0, 176.0);
    barrel.properties.insert("targetname".into(), "barrel".into());
    barrel.properties.insert("model".into(), "res://demo/scenes/barrel.tscn".into());
    barrel.properties.insert("explosive".into(), "1".into());
    barrel.properties.insert("explosion_radius".into(), "256".into());
    barrel.properties.insert("explosion_damage".into(), "25".into());
    barrel.properties.insert("health".into(), "5".into());
    barrel.properties.insert("size".into(), "16 24 16".into());
    barrel.outputs.push(io("broken", "hp_readout", "run", 0.0));
    m.insert(layer, NodeKind::Entity(barrel));

    let mut readout = Entity::new("logic_script");
    readout.origin = DVec3::new(0.0, 8.0, 288.0);
    readout.properties.insert("targetname".into(), "hp_readout".into());
    readout.properties.insert(
        "source".into(),
        "var players = io.find_targets(this, \"!player\", null)\nif players.is_empty():\n\treturn\nvar hp = int(players[0].health)\nfor label in io.find_targets(this, \"hpmsg\", null):\n\tlabel.set_text(\"HP \" + str(hp))\n\tlabel.show()".into(),
    );
    m.insert(layer, NodeKind::Entity(readout));

    let mut cutscene = Entity::new("logic_sequence");
    cutscene.origin = DVec3::new(0.0, 8.0, 296.0);
    cutscene.properties.insert("targetname".into(), "cutscene".into());
    cutscene.properties.insert("steps".into(), "5".into());
    cutscene.properties.insert("interval".into(), "1.5".into());
    for (output, target, input) in [
        ("step_1", "hint", "show"),
        ("step_2", "guide", "start"),
        ("step_3", "barrel", "ignite"),
        ("step_4", "lamp", "turn_on"),
        ("step_5", "exit_door", "open"),
    ] {
        cutscene.outputs.push(io(output, target, input, 0.0));
    }

    m.insert(layer, NodeKind::Entity(cutscene));

    let mut door = Entity::new("func_door");
    door.properties.insert("targetname".into(), "exit_door".into());
    door.properties.insert("travel".into(), "0 168 0".into());
    door.properties.insert("speed".into(), "2".into());
    door.properties.insert("wait".into(), "-1".into());
    let door = m.insert(layer, NodeKind::Entity(door));
    m.insert(door, NodeKind::Brush(boxb([-32.0, 8.0, -318.0], [32.0, 176.0, -312.0], "base/metal")));

    let mut trigger = Entity::new("trigger_once");
    trigger.properties.insert("targetname".into(), "start_zone".into());
    trigger.outputs.push(io("triggered", "cutscene", "start", 0.0));
    let trigger = m.insert(layer, NodeKind::Entity(trigger));
    m.insert(trigger, NodeKind::Brush(boxb([-160.0, 8.0, 120.0], [160.0, 200.0, 232.0], "special/trigger")));
    m
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../godot");
    for dir in ["tests/maps", "demo/maps"] {
        let dir = root.join(dir);
        std::fs::create_dir_all(&dir).unwrap();
        format::save(&demo(), &dir.join(if dir.ends_with("tests/maps") { "basic.gtm" } else { "demo.gtm" })).unwrap();
        if dir.ends_with("demo/maps") {
            format::save(&scripted_scene(), &dir.join("scripted_scene.gtm")).unwrap();
        }

        format::save(&prefab(), &dir.join("prefab.gtm")).unwrap();
        format::save(&terrain(), &dir.join("terrain.gtm")).unwrap();
        format::save(&geometry(), &dir.join("geometry.gtm")).unwrap();
        if dir.ends_with("tests/maps") {
            // The JSON .gtm older editors wrote, which the Godot tests check builds the same as the binary files.
            std::fs::write(dir.join("basic_json.gtm"), format::to_string(&demo())).unwrap();
            std::fs::write(dir.join("geometry_json.gtm"), format::to_string(&geometry())).unwrap();
        }

        // The same demo as a Valve 220 .map, built by FuncGodot's own parser in the Godot tests.
        let options = gt_formats::quake_map::ExportOptions { skip_omitted_layers: true, ..Default::default() };
        std::fs::write(dir.join("basic_export.map"), gt_formats::quake_map::export_with(&demo(), options)).unwrap();
        println!("wrote maps to {}", dir.display());
    }
}
