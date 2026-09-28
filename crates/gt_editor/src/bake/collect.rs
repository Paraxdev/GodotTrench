//! Turns the map into what the baker traces: the static geometry the Godot build keeps, the lights and the sky.

use std::collections::HashMap;

use gt_bake::{BakeInput, Light, LightKind, Material, Sky, Surface, SurfaceKey};
use gt_core::{DVec3, NodeId, Vec3};
use gt_doc::{Entity, Map, NodeKind};
use gt_formats::GameConfig;
use gt_geom::{Brush, Mesh, Terrain};

use crate::materials::MaterialLibrary;
use crate::scene::parse_color;

/// Node classes whose geometry moves or is not drawn, so it neither takes a light map nor casts baked shadows.
const DYNAMIC_CLASSES: [&str; 5] = ["AnimatableBody3D", "RigidBody3D", "CharacterBody3D", "VehicleBody3D", "Area3D"];

/// Angular radius in degrees the sun is spread over at full shadow softness.
const SUN_SOFTNESS: f32 = 2.0;
/// Radius in meters lights without a `light_size` are spread over at full shadow softness.
const LIGHT_SOFTNESS: f32 = 0.25;

/// How a light takes part in the bake, the `bake_mode` key of lights and `sun_bake_mode` of worldspawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BakeMode {
    /// Baked, unless the light can be switched: it has a targetname or starts off.
    Auto,
    /// Direct and bounced light go into the light map, Godot's static bake mode.
    Baked,
    /// Godot draws the light in real time and only its bounce is baked, Godot's dynamic bake mode.
    Bounce,
    /// Left out of the bake.
    Realtime,
}

impl BakeMode {
    pub fn parse(s: Option<&str>) -> BakeMode {
        match s.map(str::trim).unwrap_or("") {
            "baked" | "static" => BakeMode::Baked,
            "bounce" | "dynamic" => BakeMode::Bounce,
            "realtime" | "disabled" => BakeMode::Realtime,
            _ => BakeMode::Auto,
        }
    }

    /// The mode `auto` stands for on a light that is or is not switched by I/O.
    pub fn resolve(self, switchable: bool) -> BakeMode {
        match self {
            BakeMode::Auto if switchable => BakeMode::Realtime,
            BakeMode::Auto => BakeMode::Baked,
            m => m,
        }
    }
}

/// Everything the bake of a map needs, gathered on the main thread from a snapshot.
pub struct Collected {
    pub input: BakeInput,
    /// Material names by index, their albedo and glow are read on the worker.
    pub material_names: Vec<String>,
    /// Geometry fingerprint of every baked node.
    pub nodes: std::collections::BTreeMap<u64, u64>,
    pub scene: u64,
    pub counts: Counts,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub surfaces: usize,
    pub baked_lights: usize,
    pub bounce_lights: usize,
    pub realtime_lights: usize,
    pub sun: Option<BakeMode>,
}

pub use gt_doc::lightmap::{Fnv, fingerprint};

/// Whether the geometry node `id` is static level geometry the Godot build keeps.
pub fn is_static(map: &Map, game: &GameConfig, id: NodeId) -> bool {
    let layer = map.layer_of(id);
    if matches!(map.get(layer).map(|n| &n.kind), Some(NodeKind::Layer(l)) if l.omit_from_export) {
        return false;
    }

    match map.owning_entity(id).and_then(|e| map.entity(e)) {
        Some(e) => {
            let class = game.entity(&e.classname).map(|d| d.node_class.as_str()).unwrap_or("");
            !DYNAMIC_CLASSES.contains(&class) && !crate::scene::is_volume(game, e)
        }
        None => true,
    }
}

struct Builder<'a> {
    game: &'a GameConfig,
    textures: &'a std::collections::BTreeMap<String, gt_doc::textures::TextureSettings>,
    materials: HashMap<String, u32>,
    names: Vec<String>,
    surfaces: Vec<Surface>,
}

impl Builder<'_> {
    fn material(&mut self, name: &str) -> u32 {
        if let Some(i) = self.materials.get(name) {
            return *i;
        }

        let i = self.names.len() as u32;
        self.names.push(name.to_string());
        self.materials.insert(name.to_string(), i);
        i
    }

    /// Adds a surface, turning triangles that face away from their normals around.
    fn push(&mut self, key: SurfaceKey, positions: Vec<Vec3>, normals: Vec<Vec3>, mut triangles: Vec<[u32; 3]>, plane_normal: Vec3, material: u32) {
        if triangles.is_empty() {
            return;
        }

        for t in &mut triangles {
            let [a, b, c] = t.map(|i| positions[i as usize]);
            let n = normals[t[0] as usize] + normals[t[1] as usize] + normals[t[2] as usize];
            if (b - a).cross(c - a).dot(n) < 0.0 {
                t.swap(1, 2);
            }
        }

        let axes = chart_axes(&positions, plane_normal);
        let t = self.textures.get(&self.names[material as usize]).cloned().unwrap_or_default();
        if !t.bake && !t.casts {
            return;
        }

        self.surfaces.push(Surface { key, positions, normals, triangles, axes, material, receives: t.bake, casts: t.casts, texel_scale: t.texel_scale as f32 });
    }

    fn brush(&mut self, id: NodeId, brush: &Brush) {
        let has_disp = brush.faces.iter().any(|f| f.data.disp.is_some());
        for (fi, face) in brush.faces.iter().enumerate() {
            let mat = face.data.material.as_str();
            if self.game.is_tool_texture(mat) || (has_disp && face.data.disp.is_none()) {
                continue;
            }

            let material = self.material(mat);
            let key = SurfaceKey { node: id.0, face: fi as u32 };
            let n = face.plane.normal.as_vec3();
            if let Some(grid) = gt_geom::displacement::grid(brush, fi) {
                let positions = grid.positions.iter().map(|p| p.as_vec3()).collect();
                let normals = grid.normals.iter().map(|p| p.as_vec3()).collect();
                let tris = gt_geom::displacement::triangles(grid.size).map(|(a, b, c)| [a as u32, b as u32, c as u32]).collect();
                self.push(key, positions, normals, tris, n, material);
                continue;
            }

            let positions: Vec<Vec3> = face.indices.iter().map(|i| brush.vertices[*i as usize].as_vec3()).collect();
            let tris = (1..positions.len().saturating_sub(1) as u32).map(|k| [0, k, k + 1]).collect();
            let normals = vec![n; positions.len()];
            self.push(key, positions, normals, tris, n, material);
        }
    }

    fn mesh(&mut self, id: NodeId, mesh: &Mesh) {
        if mesh.decal {
            return;
        }

        let normals = mesh.corner_normals();
        for (fi, face) in mesh.faces.iter().enumerate() {
            let mat = face.data.material.as_str();
            if face.indices.len() < 3 || face.indices.iter().any(|i| *i as usize >= mesh.vertices.len()) || self.game.is_tool_texture(mat) {
                continue;
            }

            let material = self.material(mat);
            let positions: Vec<Vec3> = mesh.face_points(fi).iter().map(|p| p.as_vec3()).collect();
            let tris = mesh.triangulate_corners(fi).iter().map(|t| t.map(|k| k as u32)).collect();
            let corner_normals = normals[fi].iter().map(|n| n.as_vec3()).collect();
            self.push(SurfaceKey { node: id.0, face: fi as u32 }, positions, corner_normals, tris, mesh.face_normal(fi).as_vec3(), material);
        }
    }

    fn terrain(&mut self, id: NodeId, t: &Terrain) {
        if !t.is_valid() {
            return;
        }

        let material = self.material(t.layers.first().map(|l| l.material.as_str()).unwrap_or(""));
        let [w, d] = t.resolution;
        let mut positions = Vec::with_capacity((w * d) as usize);
        let mut normals = Vec::with_capacity((w * d) as usize);
        for j in 0..d {
            for i in 0..w {
                positions.push(t.vertex(i, j).as_vec3());
                normals.push(t.normal(i, j).as_vec3());
            }
        }

        let [cw, cd] = t.cells();
        let mut tris = Vec::with_capacity((cw * cd * 2) as usize);
        for cj in 0..cd {
            for ci in 0..cw {
                if t.is_hole(ci, cj) {
                    continue;
                }

                for tri in Terrain::cell_triangles(ci, cj) {
                    tris.push(tri.map(|(i, j)| j * w + i));
                }
            }
        }

        self.push(SurfaceKey { node: id.0, face: 0 }, positions, normals, tris, Vec3::Y, material);
    }
}

/// Axes in the plane of `normal` for a chart, the first along the longest edge between consecutive points so
/// rotated walls lie flat in the atlas instead of wasting a diagonal's worth of texels.
fn chart_axes(points: &[Vec3], normal: Vec3) -> [Vec3; 2] {
    let n = normal.normalize_or(Vec3::Y);
    let mut best = Vec3::ZERO;
    if points.len() <= 16 {
        for k in 0..points.len() {
            let e = points[(k + 1) % points.len()] - points[k];
            let e = e - n * e.dot(n);
            if e.length_squared() > best.length_squared() {
                best = e;
            }
        }
    }

    let u = if best.length_squared() > 1e-8 { best.normalize() } else { n.any_orthonormal_vector() };
    [u, n.cross(u).normalize()]
}

fn linear(c: Vec3) -> Vec3 {
    Vec3::new(gt_render::srgb_to_linear(c.x), gt_render::srgb_to_linear(c.y), gt_render::srgb_to_linear(c.z))
}

fn float(e: &Entity, key: &str) -> Option<f32> {
    e.property(key).and_then(|s| s.trim().parse::<f32>().ok()).filter(|v| v.is_finite())
}

/// Whether a point entity is a light the bake counts: a Godot light class, or any entity with light keys, so
/// entities from a project's own definitions take part too.
pub fn is_light(game: &GameConfig, e: &Entity) -> bool {
    let class = game.entity(&e.classname).map(|d| d.node_class.as_str()).unwrap_or("");
    matches!(class, "OmniLight3D" | "SpotLight3D" | "DirectionalLight3D") || e.properties.contains_key("light_energy")
}

/// The light an entity stands for, with the mode it is baked in.
pub fn light_of(game: &GameConfig, e: &Entity, upm: f32, softness: f32) -> Option<(Light, BakeMode)> {
    if !is_light(game, e) {
        return None;
    }

    let class = game.entity(&e.classname).map(|d| d.node_class.as_str()).unwrap_or("");
    let switchable = e.targetname().is_some() || e.property("start_on") == Some("0");
    let mode = BakeMode::parse(e.property("bake_mode")).resolve(switchable);
    let color = linear(e.property("light_color").and_then(parse_color).unwrap_or(Vec3::ONE)) * float(e, "light_energy").unwrap_or(1.0).max(0.0);
    let forward = (e.rotation() * DVec3::NEG_Z).as_vec3();
    let directional = class == "DirectionalLight3D" || e.classname.contains("directional") || e.classname == "light_environment";
    let spot = !directional && (class == "SpotLight3D" || e.classname.contains("spot"));
    let size = float(e, "light_size").unwrap_or(0.0).max(0.0) * upm;
    let mut light = Light::omni(e.origin.as_vec3(), color, 0.0);
    light.direction = forward;
    light.indirect_energy = float(e, "light_indirect_energy").unwrap_or(1.0).max(0.0);
    if directional {
        light.kind = LightKind::Sun;
        light.size = float(e, "light_angular_distance").unwrap_or(0.0).max(softness * SUN_SOFTNESS).to_radians();
        light.shadow_mask = true;
    } else {
        light.kind = if spot { LightKind::Spot } else { LightKind::Omni };
        let range_key = if spot { "spot_range" } else { "omni_range" };
        light.range = float(e, range_key).unwrap_or(if spot { 15.0 } else { 10.0 }).max(0.0) * upm;
        light.attenuation = float(e, if spot { "spot_attenuation" } else { "omni_attenuation" }).unwrap_or(1.0);
        light.spot_cos = float(e, "spot_angle").unwrap_or(45.0).clamp(0.1, 179.0).to_radians().cos();
        light.spot_attenuation = float(e, "spot_angle_attenuation").unwrap_or(1.0);
        light.size = size.max(softness * LIGHT_SOFTNESS * upm);
    }

    match mode {
        BakeMode::Realtime => {
            light.bake_direct = false;
            light.indirect_energy = 0.0;
            light.shadow_mask = false;
        }
        BakeMode::Bounce => light.bake_direct = false,
        _ => light.bake_direct = true,
    }

    Some((light, mode))
}

/// The worldspawn sun, or None when the map builds no sun.
fn sun(map: &Map, softness: f32) -> Option<(Light, BakeMode)> {
    let props = &map.properties;
    if props.get("environment").is_some_and(|v| v.trim() == "0") {
        return None;
    }

    let angles: Vec<f64> = props.get("sun_angles").map_or("-40 -45", |s| s.as_str()).split_whitespace().filter_map(|p| p.parse().ok()).collect();
    let direction = if angles.len() >= 2 {
        let q = gt_core::DQuat::from_euler(gt_core::EulerRot::YXZ, angles[1].to_radians(), angles[0].to_radians(), 0.0);
        (q * DVec3::NEG_Z).as_vec3()
    } else {
        Vec3::new(-0.45, -0.8, -0.35)
    };
    let color = linear(props.get("sun_color").and_then(|s| parse_color(s)).unwrap_or(Vec3::new(1.0, 0.96, 0.88)));
    let energy = props.get("sun_energy").and_then(|s| s.trim().parse::<f32>().ok()).unwrap_or(1.1).max(0.0);
    let mode = BakeMode::parse(props.get("sun_bake_mode").map(String::as_str)).resolve(false);
    let mut light = Light::sun(direction, color * energy);
    light.size = (softness * SUN_SOFTNESS).to_radians();
    match mode {
        BakeMode::Realtime => {
            light.bake_direct = false;
            light.indirect_energy = 0.0;
        }
        BakeMode::Bounce => light.bake_direct = false,
        _ => {}
    }

    Some((light, mode))
}

/// The sky and ambient light rays leaving the map see, as the addon's WorldEnvironment builds them.
pub fn sky(map: &Map) -> Sky {
    let props = &map.properties;
    if props.get("environment").is_some_and(|v| v.trim() == "0") {
        return Sky::NONE;
    }

    let color = |key: &str, default: Vec3| linear(props.get(key).and_then(|s| parse_color(s)).unwrap_or(default));
    let energy = |key: &str| props.get(key).and_then(|s| s.trim().parse::<f32>().ok()).unwrap_or(1.0).max(0.0);
    let sky_energy = energy("sky_energy");
    let ambient = props.get("ambient_color").and_then(|s| parse_color(s));
    Sky {
        top: color("sky_top_color", Vec3::new(0.32, 0.5, 0.78)) * sky_energy,
        horizon: color("sky_horizon_color", Vec3::new(0.72, 0.8, 0.88)) * sky_energy,
        ground: color("sky_ground_color", Vec3::new(0.42, 0.44, 0.46)) * sky_energy,
        ambient: ambient.map(linear).unwrap_or(Vec3::ZERO),
        sky_share: if ambient.is_some() { 0.5 } else { 1.0 },
        energy: energy("ambient_energy"),
    }
}

/// Gathers the bake of `map`. `softness`, 0 to 1, spreads lights without a size of their own, 0 keeps shadows hard.
pub fn collect(map: &Map, game: &GameConfig, softness: f32) -> Collected {
    let upm = game.units_per_meter as f32;
    let mut b = Builder { game, textures: &map.textures, materials: HashMap::new(), names: Vec::new(), surfaces: Vec::new() };
    let mut nodes = std::collections::BTreeMap::new();
    let mut scene = Fnv::default();
    for id in map.walk() {
        let Some(node) = map.get(id) else { continue };
        if !node.kind.is_geometry() || !is_static(map, game, id) {
            continue;
        }

        let Some(print) = fingerprint(&node.kind) else { continue };
        nodes.insert(id.0, print);
        scene.bytes(&id.0.to_le_bytes()).bytes(&print.to_le_bytes());
        match &node.kind {
            NodeKind::Brush(brush) => b.brush(id, brush),
            NodeKind::Mesh(mesh) => b.mesh(id, mesh),
            NodeKind::Terrain(t) => b.terrain(id, t),
            _ => {}
        }
    }

    let mut counts = Counts::default();
    let mut lights = Vec::new();
    let mut probe_points = Vec::new();
    if let Some((light, mode)) = sun(map, softness) {
        counts.sun = Some(mode);
        lights.push(light);
    }

    for (id, e) in map.entities() {
        if !map.is_point_entity(id) || !is_static(map, game, id) {
            continue;
        }

        if game.entity(&e.classname).is_some_and(|d| d.node_class == "LightmapProbe") {
            let p = e.origin.as_vec3();
            scene.f64(p.x as f64).f64(p.y as f64).f64(p.z as f64);
            probe_points.push(p);
            continue;
        }

        let Some((light, mode)) = light_of(game, e, upm, softness) else { continue };
        if light.kind == LightKind::Sun {
            // A directional light entity replaces the worldspawn sun, like the lit preview.
            lights.retain(|l: &Light| l.kind != LightKind::Sun);
            counts.sun = Some(mode);
            lights.insert(0, light);
            continue;
        }

        match mode {
            BakeMode::Realtime => counts.realtime_lights += 1,
            BakeMode::Bounce => counts.bounce_lights += 1,
            _ => counts.baked_lights += 1,
        }

        lights.push(light);
    }

    for l in &lights {
        scene.bytes(format!("{l:?}").as_bytes());
    }

    let sky = sky(map);
    scene.bytes(format!("{sky:?}").as_bytes());
    // Materials color the bounce, and their settings decide what is baked at all.
    for name in &b.names {
        scene.bytes(name.as_bytes()).bytes(format!("{:?}", map.textures.get(name)).as_bytes());
    }

    counts.surfaces = b.surfaces.len();
    let materials = vec![Material::default(); b.names.len()];
    Collected {
        input: BakeInput { surfaces: b.surfaces, materials, lights, sky, units_per_meter: upm, probe_points },
        material_names: b.names,
        nodes,
        scene: scene.finish(),
        counts,
    }
}

/// Average linear albedo and glow of a material as the bounce sees it. Unknown materials are mid grey.
pub fn material(lib: &mut MaterialLibrary, name: &str) -> Material {
    let Some(loaded) = lib.load_material(name) else { return Material::default() };
    let info = &loaded.info;
    let average = |img: &image::RgbaImage| {
        let step = ((img.width() * img.height()) as usize / 4096).max(1);
        let (mut sum, mut weight) = (Vec3::ZERO, 0.0f32);
        for p in img.pixels().step_by(step) {
            let a = p.0[3] as f32 / 255.0;
            sum += linear(Vec3::new(p.0[0] as f32, p.0[1] as f32, p.0[2] as f32) / 255.0) * a;
            weight += a;
        }

        if weight > 0.0 { sum / weight } else { Vec3::ZERO }
    };
    let tint = linear(Vec3::new(info.albedo_color[0], info.albedo_color[1], info.albedo_color[2]));
    let albedo = (average(&loaded.albedo) * tint).clamp(Vec3::ZERO, Vec3::splat(0.95));
    let emission = match info.emission {
        Some(c) => {
            let color = linear(Vec3::from(c));
            let texture = loaded.emission.as_ref().map(average);
            let glow = match (texture, info.emission_multiply) {
                (Some(t), true) => color * t,
                (Some(t), false) => color + t,
                (None, _) => color,
            };
            glow * info.emission_energy.max(0.0)
        }
        None => Vec3::ZERO,
    };
    Material { albedo, emission, double_sided: info.double_sided }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gt_core::Aabb;

    #[test]
    fn a_room_with_a_lamp_collects_its_faces_and_the_lamp() {
        let mut map = Map::new();
        map.properties.insert("sun_energy".into(), "0".into());
        let layer = map.default_layer();
        let floor = map
            .insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::new(-128.0, -16.0, -128.0), DVec3::new(128.0, 0.0, 128.0)), "floor").unwrap()));
        let door = map.insert(layer, NodeKind::Entity(Entity::new("func_door")));
        map.insert(door, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::splat(32.0)), "door").unwrap()));
        let mut lamp = Entity::new("light");
        lamp.origin = DVec3::new(0.0, 64.0, 0.0);
        map.insert(layer, NodeKind::Entity(lamp.clone()));
        lamp.properties.insert("targetname".into(), "switch_me".into());
        map.insert(layer, NodeKind::Entity(lamp));
        // func_door comes from the Gameplay entities pack, as in a project that installed it.
        let game = GameConfig::with_gameplay_pack();
        let c = collect(&map, &game, 0.0);
        assert_eq!(c.counts.surfaces, 6, "only the floor brush, the door moves");
        assert!(c.input.surfaces.iter().all(|s| s.key.node == floor.0));
        assert_eq!((c.counts.baked_lights, c.counts.realtime_lights), (1, 1), "a lamp with a targetname stays switchable");
        assert_eq!(c.nodes.len(), 1);
        for s in &c.input.surfaces {
            for t in &s.triangles {
                let [a, b, cc] = t.map(|i| s.positions[i as usize]);
                assert!((b - a).cross(cc - a).dot(s.normals[0]) > 0.0, "triangles face the way the face does");
            }

            assert!(s.axes[0].dot(s.normals[0]).abs() < 1e-5 && s.axes[1].dot(s.normals[0]).abs() < 1e-5);
        }
    }

    #[test]
    fn fingerprints_follow_geometry_not_texture() {
        let b = Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::splat(32.0)), "a").unwrap();
        let mut retextured = b.clone();
        retextured.set_material("b");
        let moved = b.translated(DVec3::X, true);
        let f = |b: &Brush| fingerprint(&NodeKind::Brush(b.clone()));
        assert_eq!(f(&b), f(&retextured));
        assert_ne!(f(&b), f(&moved));
    }

    #[test]
    fn texture_settings_decide_what_is_baked() {
        use gt_doc::textures::TextureSettings;
        let mut map = Map::new();
        let layer = map.default_layer();
        map.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::splat(32.0)), "glass").unwrap()));
        map.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::X * 64.0, DVec3::splat(96.0)), "wall").unwrap()));
        let game = GameConfig::default();
        let before = collect(&map, &game, 0.0);
        assert_eq!(before.counts.surfaces, 12);

        map.set_texture("glass", TextureSettings { casts: false, texel_scale: 0.5, ..Default::default() });
        map.set_texture("wall", TextureSettings { bake: false, casts: false, ..Default::default() });
        let c = collect(&map, &game, 0.0);
        assert_eq!(c.counts.surfaces, 6, "a texture neither baked nor casting is left out");
        assert!(c.input.surfaces.iter().all(|s| s.receives && !s.casts && s.texel_scale == 0.5));
        assert_ne!(c.scene, before.scene, "changed settings make the bake out of date");
    }

    #[test]
    fn bake_modes_read_godot_names_and_auto_follows_switching() {
        assert_eq!(BakeMode::parse(Some("static")), BakeMode::Baked);
        assert_eq!(BakeMode::parse(Some("dynamic")), BakeMode::Bounce);
        assert_eq!(BakeMode::parse(None).resolve(true), BakeMode::Realtime);
        assert_eq!(BakeMode::parse(Some("auto")).resolve(false), BakeMode::Baked);
        assert_eq!(BakeMode::parse(Some("baked")).resolve(true), BakeMode::Baked, "an explicit mode wins");
    }
}
