use std::path::{Path, PathBuf};

use gt_core::{Aabb, DVec3};
use gt_doc::map::Group;
use gt_doc::{Entity, Map, NodeKind};
use gt_geom::{Brush, Terrain};

use super::*;

/// A tiny Godot project: a plain texture with a normal map, a metal material with a roughness map and an OBJ model
/// with its own texture.
fn project(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("gt_export3d_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let tex = dir.join("textures");
    std::fs::create_dir_all(&tex).unwrap();
    std::fs::create_dir_all(dir.join("models")).unwrap();
    std::fs::create_dir_all(dir.join("maps")).unwrap();
    std::fs::write(dir.join("project.godot"), "config_version=5\n").unwrap();
    image::RgbaImage::from_pixel(8, 8, image::Rgba([180, 60, 40, 255])).save(tex.join("brick.png")).unwrap();
    image::RgbaImage::from_pixel(8, 8, image::Rgba([128, 128, 255, 255])).save(tex.join("brick_normal.png")).unwrap();
    image::RgbImage::from_pixel(4, 4, image::Rgb([150, 150, 160])).save(tex.join("metal.jpg")).unwrap();
    image::RgbaImage::from_pixel(4, 4, image::Rgba([90, 90, 90, 255])).save(tex.join("metal_roughness.png")).unwrap();
    std::fs::write(
        tex.join("metal.tres"),
        "[gd_resource type=\"StandardMaterial3D\" format=3]\n[ext_resource type=\"Texture2D\" path=\"res://textures/metal.jpg\" id=\"1\"]\n[ext_resource type=\"Texture2D\" path=\"res://textures/metal_roughness.png\" id=\"2\"]\n[resource]\nalbedo_texture = ExtResource(\"1\")\nroughness_texture = ExtResource(\"2\")\nmetallic = 0.8\n",
    )
    .unwrap();
    image::RgbaImage::from_pixel(2, 2, image::Rgba([40, 160, 60, 255])).save(dir.join("models/crate.png")).unwrap();
    std::fs::write(dir.join("models/crate.mtl"), "newmtl wood\nKd 1 1 1\nmap_Kd crate.png\n").unwrap();
    std::fs::write(
        dir.join("models/crate.obj"),
        "mtllib crate.mtl\no crate\nv 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nvt 0 0\nvt 1 0\nvt 1 1\nvt 0 1\nvn 0 0 1\nusemtl wood\nf 1/1/1 2/2/1 3/3/1 4/4/1\n",
    )
    .unwrap();
    dir
}

fn cube(min: [f64; 3], max: [f64; 3], material: &str) -> Brush {
    Brush::from_aabb(&Aabb::new(DVec3::from(min), DVec3::from(max)), material).unwrap()
}

/// A hollowed room with one clip face, a named pillar in a group, a clip box, a mesh, a terrain, a prop with a model,
/// a trigger, a hidden brush and a light.
fn map() -> Map {
    let mut map = Map::new();
    let layer = map.default_layer();
    let room = gt_geom::csg::hollow(&cube([-256.0, 0.0, -256.0], [256.0, 256.0, 256.0], "brick"), 16.0);
    assert_eq!(room.len(), 6);
    for (i, mut wall) in room.into_iter().enumerate() {
        if i == 0 {
            wall.faces[0].data.material = "special/clip".into();
        }

        map.insert(layer, NodeKind::Brush(wall));
    }

    let group = map.insert(layer, NodeKind::Group(Group::new("Columns")));
    map.insert_labeled(group, NodeKind::Brush(cube([0.0, 16.0, 0.0], [64.0, 128.0, 32.0], "metal")), Some("Pillar".into()));
    map.insert(layer, NodeKind::Brush(cube([400.0, 0.0, 0.0], [464.0, 64.0, 64.0], "special/clip")));
    let mesh = gt_geom::mesh_shapes::cuboid(&Aabb::new(DVec3::new(-64.0, 16.0, -64.0), DVec3::new(-32.0, 48.0, -32.0)), "metal");
    map.insert_labeled(layer, NodeKind::Mesh(mesh), Some("Crate mesh".into()));
    map.insert(layer, NodeKind::Terrain(Terrain::new(DVec3::new(-1024.0, -32.0, -1024.0), [5, 5], 128.0, "brick")));
    let mut prop = Entity::new("prop_model");
    prop.origin = DVec3::new(32.0, 16.0, 64.0);
    prop.angles = DVec3::new(0.0, 90.0, 0.0);
    prop.properties.insert("model".into(), "res://models/crate.obj".into());
    prop.properties.insert("targetname".into(), "box".into());
    map.insert(layer, NodeKind::Entity(prop));
    let trigger = map.insert(layer, NodeKind::Entity(Entity::new("trigger_once")));
    map.insert(trigger, NodeKind::Brush(cube([-32.0, 0.0, -32.0], [32.0, 64.0, 32.0], "special/trigger")));
    let hidden = map.insert_labeled(layer, NodeKind::Brush(cube([600.0, 0.0, 0.0], [664.0, 64.0, 64.0], "brick")), Some("Hidden".into()));
    map.get_mut(hidden).unwrap().hidden = true;
    let mut light = Entity::new("light");
    light.origin = DVec3::new(0.0, 96.0, 0.0);
    map.insert(layer, NodeKind::Entity(light));
    map
}

struct Caches {
    game: GameConfig,
    materials: MaterialLibrary,
    models: ModelCache,
    prefabs: PrefabCache,
    selection: BTreeSet<NodeId>,
}

impl Caches {
    fn new(root: &Path) -> Self {
        let mut game = GameConfig::builtin();
        game.project_root = Some(root.to_path_buf());
        game.textures.base_dir = "res://textures".into();
        let materials = MaterialLibrary::new(&game);
        Self { game, materials, models: ModelCache::default(), prefabs: PrefabCache::default(), selection: BTreeSet::new() }
    }

    fn sources<'a>(&'a mut self, map: &'a Map) -> Sources<'a> {
        Sources {
            map,
            map_path: None,
            game: &self.game,
            materials: &mut self.materials,
            models: &mut self.models,
            prefabs: &mut self.prefabs,
            selection: &self.selection,
        }
    }
}

#[test]
fn glb_holds_named_nodes_in_meters_with_embedded_textures() {
    let root = project("glb");
    let map = map();
    let mut caches = Caches::new(&root);
    let out = root.join("room.glb");
    let report = export(caches.sources(&map), &out, Format::Glb, &Options::default()).unwrap();
    assert!(report.missing.is_empty(), "{:?}", report.missing);

    let (doc, buffers, images) = gltf::import(&out).expect("the export reads back as glTF");
    let names: Vec<&str> = doc.nodes().filter_map(|n| n.name()).collect();
    let layer = map.get(map.default_layer()).unwrap().name();
    for expected in [layer.as_str(), "Columns", "Pillar", "Crate mesh", "terrain 5x5", "prop_model (box)"] {
        assert!(names.contains(&expected), "{expected} in {names:?}");
    }

    assert!(!names.iter().any(|n| n.starts_with("trigger_once") || *n == "Hidden" || n.starts_with("light")), "{names:?}");
    // The layer, the group, six walls, the pillar, the mesh, the terrain and the prop. The clip box has no face left.
    assert_eq!(doc.nodes().count(), 12, "{names:?}");

    let pillar = doc.nodes().find(|n| n.name() == Some("Pillar")).unwrap();
    let t = pillar.transform().decomposed().0;
    assert!((DVec3::from(t.map(f64::from)) - DVec3::new(1.0, 2.25, 0.5)).length() < 1e-5, "centered at {t:?} meters");
    let prim = pillar.mesh().unwrap().primitives().next().unwrap();
    let points: Vec<[f32; 3]> = prim.reader(|b| Some(&buffers[b.index()])).read_positions().unwrap().collect();
    let (min, max) = points.iter().fold((DVec3::INFINITY, DVec3::NEG_INFINITY), |(lo, hi), p| {
        let p = DVec3::from(p.map(f64::from));
        (lo.min(p), hi.max(p))
    });
    assert!((max - min - DVec3::new(2.0, 3.5, 1.0)).length() < 1e-5, "64 x 112 x 32 units are 2 x 3.5 x 1 meters");

    let materials: Vec<&str> = doc.materials().filter_map(|m| m.name()).collect();
    assert!(!materials.iter().any(|m| m.starts_with("special/")), "tool faces are left out: {materials:?}");
    let brick = doc.materials().find(|m| m.name() == Some("brick")).unwrap();
    assert!(brick.pbr_metallic_roughness().base_color_texture().is_some() && brick.normal_texture().is_some());
    let metal = doc.materials().find(|m| m.name() == Some("metal")).unwrap();
    assert!((metal.pbr_metallic_roughness().metallic_factor() - 0.8).abs() < 1e-6);
    assert!(metal.pbr_metallic_roughness().metallic_roughness_texture().is_some(), "the roughness map is baked into glTF's packed map");
    assert!(materials.contains(&"crate mat0"), "the model keeps its own material: {materials:?}");

    assert_eq!(images.len(), 5, "brick, its normal map, metal, the baked roughness and the model texture");
    assert!(images.iter().all(|i| i.width > 0 && i.height > 0), "every embedded image decodes");
    let jpeg = doc.images().any(|i| matches!(i.source(), gltf::image::Source::View { mime_type: "image/jpeg", .. }));
    assert!(jpeg, "a JPEG texture is embedded as it is");

    let prop = doc.nodes().find(|n| n.name() == Some("prop_model (box)")).unwrap();
    let (t, r, _) = prop.transform().decomposed();
    assert_eq!(t, [1.0, 0.5, 2.0]);
    assert!((r[1].abs() - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-5, "turned 90 degrees around Y: {r:?}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn glb_options_add_markers_hidden_objects_and_merge_brushes() {
    let root = project("options");
    let map = map();
    let mut caches = Caches::new(&root);
    let out = root.join("options.glb");
    let options = Options { hidden: true, markers: true, merge_brushes: true, models: false, ..Default::default() };
    export(caches.sources(&map), &out, Format::Glb, &options).unwrap();
    let (doc, ..) = gltf::import(&out).unwrap();
    let names: Vec<&str> = doc.nodes().filter_map(|n| n.name()).collect();
    assert!(!names.iter().any(|n| n.starts_with("brush")), "unnamed brushes are merged: {names:?}");
    let layer = map.get(map.default_layer()).unwrap().name();
    assert!(names.contains(&format!("{layer} brushes").as_str()), "{names:?}");
    assert!(names.contains(&"Hidden") && names.contains(&"Pillar"), "named brushes stay apart: {names:?}");
    let light = doc.nodes().find(|n| n.name() == Some("light")).unwrap();
    assert!(light.mesh().is_none(), "a marker is an empty");
    assert!(doc.nodes().find(|n| n.name() == Some("prop_model (box)")).unwrap().mesh().is_none(), "without models the prop is a marker");

    let wall = map.brushes().next().unwrap().0;
    caches.selection.insert(wall);
    let selected = Options { selection_only: true, ..Default::default() };
    export(caches.sources(&map), &out, Format::Glb, &selected).unwrap();
    let (doc, ..) = gltf::import(&out).unwrap();
    assert_eq!(doc.nodes().count(), 2, "the selected wall inside its layer");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn obj_writes_materials_and_copies_textures() {
    let root = project("obj");
    let map = map();
    let path = root.join("maps/room.gtm");
    gt_doc::format::save(&map, &path).unwrap();
    let out = root.join("out/room.obj");
    std::fs::create_dir_all(out.parent().unwrap()).unwrap();
    // The command line export, which finds the project from the map's folder.
    let report = export_file(&path, &out, Format::Obj).unwrap();
    assert!(report.missing.is_empty(), "{:?}", report.missing);

    let (models, materials) = tobj::load_obj(&out, &tobj::LoadOptions { triangulate: true, single_index: true, ..Default::default() }).unwrap();
    let materials = materials.expect("the .mtl next to the .obj loads");
    let names: Vec<&str> = models.iter().map(|m| m.name.as_str()).collect();
    assert!(names.contains(&"Pillar") && names.contains(&"prop_model_(box)"), "{names:?}");
    assert!(!names.iter().any(|n| n.starts_with("trigger")), "{names:?}");
    let pillar = models.iter().find(|m| m.name == "Pillar").unwrap();
    assert_eq!(pillar.mesh.positions.len() / 3, 20, "the bottom resting on the floor is hidden, as in the Godot build");
    let top = pillar.mesh.positions.chunks(3).map(|p| p[1]).fold(f32::NEG_INFINITY, f32::max);
    assert_eq!(top, 4.0, "world space meters");

    let dir = out.parent().unwrap();
    let brick = materials.iter().find(|m| m.name == "brick").unwrap();
    assert_eq!(brick.diffuse_texture.as_deref(), Some("room_textures/brick.png"));
    assert_eq!(brick.normal_texture.as_deref(), Some("room_textures/brick_normal.png"));
    let metal = materials.iter().find(|m| m.name == "metal").unwrap();
    assert_eq!(metal.diffuse_texture.as_deref(), Some("room_textures/metal.jpg"));
    let model = materials.iter().find(|m| m.name == "crate_mat0").unwrap();
    for texture in [&brick.diffuse_texture, &brick.normal_texture, &metal.diffuse_texture, &model.diffuse_texture] {
        assert!(dir.join(texture.as_deref().unwrap()).is_file(), "{texture:?} is copied next to the .obj");
    }

    assert!(!materials.iter().any(|m| m.name.starts_with("special/")));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn glb_places_scatter_instances_and_prefab_contents() {
    let root = project("scatter");
    let mut prefab = Map::new();
    let layer = prefab.default_layer();
    prefab.insert_labeled(layer, NodeKind::Brush(cube([0.0, 0.0, 0.0], [32.0, 32.0, 32.0], "brick")), Some("Step".into()));
    gt_doc::format::save(&prefab, &root.join("maps/step.gtm")).unwrap();

    let mut map = Map::new();
    let layer = map.default_layer();
    let item = gt_doc::scatter::ScatterItem::new("res://models/crate.obj");
    let mut set = gt_doc::Scatter::new("Crates", gt_doc::ScatterKind::Props, vec![item]);
    for x in [0.0, 64.0, 128.0] {
        set.instances.push(gt_doc::scatter::ScatterInstance { item: 0, position: DVec3::new(x, 0.0, 0.0), angles: DVec3::ZERO, scale: 2.0 });
    }

    map.insert(layer, NodeKind::Scatter(set));
    let instance = gt_doc::map::Instance { path: "res://maps/step.gtm".into(), origin: DVec3::new(320.0, 0.0, 0.0), angles: DVec3::ZERO, fixup: String::new() };
    map.insert(layer, NodeKind::Instance(instance));

    let mut caches = Caches::new(&root);
    caches.prefabs.project_root = Some(root.clone());
    let out = root.join("scatter.glb");
    let counted = counts(&map, &caches.game, &mut caches.models, &caches.selection, &Options::default());
    assert_eq!(counted.scatter, 3, "the dialog shows how many instances the option adds");
    export(caches.sources(&map), &out, Format::Glb, &Options { scatter: true, ..Default::default() }).unwrap();
    let (doc, ..) = gltf::import(&out).unwrap();
    let crates: Vec<gltf::Node> = doc.nodes().filter(|n| n.name() == Some("crate")).collect();
    assert_eq!(crates.len(), 3, "one node per instance");
    assert_eq!(doc.meshes().filter(|m| m.name() == Some("crate")).count(), 1, "the instances share the model's mesh");
    assert_eq!(crates[1].transform().decomposed().0, [2.0, 0.0, 0.0]);
    assert_eq!(crates[1].transform().decomposed().2, [2.0, 2.0, 2.0]);
    let set = doc.nodes().find(|n| n.name() == Some("scatter Crates (3)")).expect("the set groups its instances");
    assert_eq!(set.children().count(), 3);

    let step = doc.nodes().find(|n| n.name() == Some("Step")).expect("the prefab's brush comes along");
    assert_eq!(step.transform().decomposed().0, [10.5, 0.5, 0.5], "moved to where the instance stands");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn dialog_says_what_is_left_out_and_counts_scatter() {
    use egui_kittest::kittest::{NodeT, Queryable};
    let mut state = crate::state::EditorState::new(crate::state::Prefs::default());
    let layer = state.doc.map.default_layer();
    state.doc.edit("Add", |m, _| {
        let mut set = gt_doc::Scatter::new("Rocks", gt_doc::ScatterKind::Props, vec![gt_doc::scatter::ScatterItem::new("res://rock.glb")]);
        for x in [0.0, 64.0] {
            set.instances.push(gt_doc::scatter::ScatterInstance { item: 0, position: DVec3::new(x, 0.0, 0.0), angles: DVec3::ZERO, scale: 1.0 });
        }

        m.insert(layer, NodeKind::Scatter(set));
    });
    let mut harness = egui_kittest::Harness::builder().with_size(egui::vec2(480.0, 420.0)).build_ui_state(
        |ui, (dialog, state): &mut (dialog::ExportDialog, crate::state::EditorState)| {
            dialog.ui(ui, state);
        },
        (dialog::ExportDialog::default(), state),
    );
    harness.run();
    harness.get_by_label_contains("Entity logic, I/O wiring, triggers, scripts and gameplay behaviour are left out");
    assert!(harness.get_by_label("Selection only").accesskit_node().is_disabled(), "nothing is selected");
    harness.get_by_label("Scatter instances (2)").click();
    harness.run();
    assert!(harness.state().0.options.scatter);
}

/// A pillar standing on a floor in a layer of its own.
fn pillar_on_floor() -> (Map, NodeId, NodeId) {
    let mut map = Map::new();
    let ground = map.add_layer("Ground");
    let floor = map.insert_labeled(ground, NodeKind::Brush(cube([-128.0, -16.0, -128.0], [128.0, 0.0, 128.0], "brick")), Some("Floor".into()));
    let layer = map.default_layer();
    let pillar = map.insert_labeled(layer, NodeKind::Brush(cube([0.0, 0.0, 0.0], [32.0, 128.0, 32.0], "brick")), Some("Pillar".into()));
    (map, floor, pillar)
}

/// Vertices of the object `name` in a written .glb.
fn vertex_count(path: &Path, name: &str) -> usize {
    let (doc, ..) = gltf::import(path).unwrap();
    let node = doc.nodes().find(|n| n.name() == Some(name)).unwrap_or_else(|| panic!("no {name}"));
    node.mesh().unwrap().primitives().map(|p| p.get(&gltf::Semantic::Positions).unwrap().count()).sum()
}

#[test]
fn only_what_is_exported_hides_faces() {
    let root = project("cull");
    let (mut map, floor, pillar) = pillar_on_floor();
    let mut caches = Caches::new(&root);
    let out = root.join("cull.glb");
    let pillar_vertices = |map: &Map, caches: &mut Caches, options: Options| {
        export(caches.sources(map), &out, Format::Glb, &options).unwrap();
        vertex_count(&out, "Pillar")
    };
    assert_eq!(pillar_vertices(&map, &mut caches, Options::default()), 20, "on the floor its bottom is hidden, as in the Godot build");

    caches.selection.insert(pillar);
    assert_eq!(pillar_vertices(&map, &mut caches, Options { selection_only: true, ..Default::default() }), 24, "exported alone it keeps its base");
    caches.selection.clear();

    // A cordon only narrows the views, the export still holds both and culls between them.
    map.editor.cordon = Some(Aabb::new(DVec3::splat(4096.0), DVec3::splat(4608.0)));
    map.editor.cordon_enabled = true;
    assert_eq!(pillar_vertices(&map, &mut caches, Options::default()), 20);

    map.get_mut(floor).unwrap().hidden = true;
    assert_eq!(pillar_vertices(&map, &mut caches, Options::default()), 24, "a hidden floor stays out and hides nothing");
    assert_eq!(pillar_vertices(&map, &mut caches, Options { hidden: true, ..Default::default() }), 20, "exported with it, the floor hides the base");
    map.get_mut(floor).unwrap().hidden = false;

    let ground = map.layer_of(floor);
    if let NodeKind::Layer(l) = &mut map.get_mut(ground).unwrap().kind {
        l.omit_from_export = true;
    }

    assert_eq!(pillar_vertices(&map, &mut caches, Options::default()), 24, "a layer left out of the export leaves no hole");
    let _ = std::fs::remove_dir_all(&root);
}

/// An OBJ model of a flat grid of `cells` by `cells` quads, two triangles each.
fn grid_model(path: &Path, cells: usize) {
    use std::fmt::Write as _;
    let mut text = String::from("o grid\nvt 0 0\nvn 0 1 0\n");
    for z in 0..=cells {
        for x in 0..=cells {
            let _ = writeln!(text, "v {x} 0 {z}");
        }
    }

    let at = |x: usize, z: usize| z * (cells + 1) + x + 1;
    for z in 0..cells {
        for x in 0..cells {
            let _ = writeln!(text, "f {}/1/1 {}/1/1 {}/1/1 {}/1/1", at(x, z), at(x, z + 1), at(x + 1, z + 1), at(x + 1, z));
        }
    }

    std::fs::write(path, text).unwrap();
}

#[test]
fn obj_with_a_forest_of_scatter_instances_is_refused_before_writing() {
    use egui_kittest::kittest::{NodeT, Queryable};
    let root = project("forest");
    grid_model(&root.join("models/grid.obj"), 224);
    let mut map = Map::new();
    let layer = map.default_layer();
    let mut set = gt_doc::Scatter::new("Forest", gt_doc::ScatterKind::Props, vec![gt_doc::scatter::ScatterItem::new("res://models/grid.obj")]);
    for i in 0..101 {
        set.instances.push(gt_doc::scatter::ScatterInstance { item: 0, position: DVec3::new(i as f64 * 256.0, 0.0, 0.0), angles: DVec3::ZERO, scale: 1.0 });
    }

    map.insert(layer, NodeKind::Scatter(set));
    let mut caches = Caches::new(&root);
    let options = Options { scatter: true, ..Default::default() };
    let counted = counts(&map, &caches.game, &mut caches.models, &caches.selection, &options);
    assert_eq!((counted.scatter_triangles, counted.scatter_vertices), (101 * 224 * 224 * 2, 101 * 225 * 225));

    let obj = root.join("forest.obj");
    let refused = export(caches.sources(&map), &obj, Format::Obj, &options).unwrap_err();
    assert!(refused.contains("10.1 million triangles") && refused.contains("glTF"), "{refused}");
    assert!(std::fs::read_dir(&root).unwrap().all(|f| !f.unwrap().file_name().to_string_lossy().starts_with("forest")), "nothing is written");
    let glb = root.join("forest.glb");
    let report = export(caches.sources(&map), &glb, Format::Glb, &options).unwrap();
    assert!(report.bytes < 4 << 20, "glTF shares the model's mesh between the instances: {} bytes", report.bytes);

    // The dialog says so before anything is exported.
    let mut state = crate::state::EditorState::new(crate::state::Prefs::default());
    state.game = caches.game.clone();
    state.doc.map = map;
    let mut dialog = dialog::ExportDialog::default();
    (dialog.format, dialog.options) = (Format::Obj, options);
    let mut harness = egui_kittest::Harness::builder().with_size(egui::vec2(480.0, 520.0)).build_ui_state(
        |ui, (dialog, state): &mut (dialog::ExportDialog, crate::state::EditorState)| {
            dialog.ui(ui, state);
        },
        (dialog, state),
    );
    harness.run();
    harness.get_by_label_contains("OBJ writes every scatter instance as a full copy of its model: 10.1 million triangles");
    assert!(harness.get_by_label("Export…").accesskit_node().is_disabled());
    harness.get_by_label(Format::Glb.label()).click();
    harness.run();
    assert!(harness.query_by_label_contains("OBJ writes every scatter instance").is_none());
    assert!(!harness.get_by_label("Export…").accesskit_node().is_disabled());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn glb_size_is_known_before_the_buffer_is_built() {
    let root = project("size");
    let map = map();
    let mut caches = Caches::new(&root);
    let scene = collect::collect(caches.sources(&map), &Options::default(), "room".into());
    let out = root.join("room.glb");
    let bytes = glb::write(&scene, &out).unwrap();
    let file = std::fs::read(&out).unwrap();
    assert_eq!(bytes, file.len() as u64);
    assert_eq!(u32::from_le_bytes(file[8..12].try_into().unwrap()) as usize, file.len());
    let json = u32::from_le_bytes(file[12..16].try_into().unwrap()) as usize;
    let bin = u32::from_le_bytes(file[20 + json..24 + json].try_into().unwrap()) as u64;
    assert_eq!(bin, glb::buffer_size(&scene), "the 4 GB check sees the real size");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn dialog_counts_follow_the_selection() {
    use egui_kittest::kittest::Queryable;
    let mut state = crate::state::EditorState::new(crate::state::Prefs::default());
    let layer = state.doc.map.default_layer();
    let light = state.doc.edit("Add", |m, s| {
        let brush = m.insert(layer, NodeKind::Brush(cube([0.0, 0.0, 0.0], [32.0, 32.0, 32.0], "brick")));
        s.select_node(brush);
        m.insert(layer, NodeKind::Entity(Entity::new("light")))
    });
    let mut dialog = dialog::ExportDialog::default();
    dialog.options.selection_only = true;
    let mut harness = egui_kittest::Harness::builder().with_size(egui::vec2(480.0, 420.0)).build_ui_state(
        |ui, (dialog, state): &mut (dialog::ExportDialog, crate::state::EditorState)| {
            dialog.ui(ui, state);
        },
        (dialog, state),
    );
    harness.run();
    harness.get_by_label_contains("Exports 1 brushes, 0 meshes");
    harness.state_mut().1.doc.select(|_, s| {
        s.clear();
        s.select_node(light);
    });
    harness.run();
    harness.get_by_label_contains("Exports 0 brushes, 0 meshes");
}
