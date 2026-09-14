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
    terrain.layers.push(TerrainLayer { material: "base/wall".into(), tile: 128.0 });
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

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../godot");
    for dir in ["tests/maps", "demo/maps"] {
        let dir = root.join(dir);
        std::fs::create_dir_all(&dir).unwrap();
        format::save(&demo(), &dir.join(if dir.ends_with("tests/maps") { "basic.gtm" } else { "demo.gtm" })).unwrap();
        format::save(&prefab(), &dir.join("prefab.gtm")).unwrap();
        format::save(&terrain(), &dir.join("terrain.gtm")).unwrap();
        format::save(&geometry(), &dir.join("geometry.gtm")).unwrap();
        // The same demo as a Valve 220 .map, built by FuncGodot's own parser in the Godot tests.
        let options = gt_formats::quake_map::ExportOptions { skip_omitted_layers: true, ..Default::default() };
        std::fs::write(dir.join("basic_export.map"), gt_formats::quake_map::export_with(&demo(), options)).unwrap();
        println!("wrote maps to {}", dir.display());
    }
}
