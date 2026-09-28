use glam::{Vec2, Vec3};
use gt_bake::*;

/// A rectangle facing `normal`, `u` and `v` its half extents.
fn quad(key: u32, center: Vec3, u: Vec3, v: Vec3, material: u32) -> Surface {
    let n = u.cross(v).normalize();
    Surface {
        key: SurfaceKey { node: 1, face: key },
        positions: vec![center - u - v, center + u - v, center + u + v, center - u + v],
        normals: vec![n; 4],
        triangles: vec![[0, 1, 2], [0, 2, 3]],
        axes: [u.normalize(), v.normalize()],
        material,
        ..Surface::default()
    }
}

/// The six faces of a box, facing out, or in for a room.
fn cube(first: u32, center: Vec3, half: Vec3, inward: bool, material: u32) -> Vec<Surface> {
    let (x, y, z) = (Vec3::X * half.x, Vec3::Y * half.y, Vec3::Z * half.z);
    let faces = [(x, z, y), (-x, y, z), (y, x, z), (-y, z, x), (z, y, x), (-z, x, y)];
    faces
        .iter()
        .enumerate()
        .map(|(k, (n, a, b))| {
            let (a, b) = if inward { (*b, *a) } else { (*a, *b) };
            // Keep the winding counter-clockwise seen from the side the face looks at.
            let (a, b) = if a.cross(b).dot(if inward { -*n } else { *n }) > 0.0 { (a, b) } else { (b, a) };
            quad(first + k as u32, center + *n, a, b, material)
        })
        .collect()
}

fn value(map: &Lightmap, key: u32, p: Vec3) -> (Vec3, f32, f32) {
    let chart = map.chart(SurfaceKey { node: 1, face: key }).unwrap();
    let uv = Vec2::new(Vec3::from_slice(&chart.rows[0]).dot(p) + chart.rows[0][3], Vec3::from_slice(&chart.rows[1]).dot(p) + chart.rows[1][3]);
    let x = ((uv.x * map.width as f32) as u32).min(map.width - 1);
    let y = ((uv.y * map.height as f32) as u32).min(map.height - 1);
    let i = (y * map.width + x) as usize;
    (Vec3::from(map.light[i]), map.shadow[i], map.ao[i])
}

fn settings() -> Settings {
    Settings { texel_size: 8.0, rays: 64, bounces: 0, shadow_samples: 4, threads: 2, ..Settings::default() }
}

fn floor(size: f32) -> Surface {
    quad(0, Vec3::ZERO, Vec3::Z * size, Vec3::X * size, 0)
}

#[test]
fn sun_straight_down_lights_an_open_floor_fully_and_a_block_shadows_it() {
    let mut surfaces = vec![floor(256.0)];
    surfaces.extend(cube(10, Vec3::new(128.0, 200.0, 0.0), Vec3::splat(40.0), false, 0));
    let input = BakeInput {
        surfaces,
        materials: vec![Material::default()],
        lights: vec![Light::sun(Vec3::NEG_Y, Vec3::ONE)],
        sky: Sky::NONE,
        units_per_meter: 32.0,
        ..Default::default()
    };
    let map = bake(&input, &settings(), &Progress::default()).unwrap();
    let (lit, sun, ao) = value(&map, 0, Vec3::new(-128.0, 0.0, 0.0));
    assert!((lit - Vec3::ONE).abs().max_element() < 0.02, "open floor reads the sun energy, got {lit}");
    assert!(sun > 0.99 && ao > 0.95);
    let (shade, sun, _) = value(&map, 0, Vec3::new(128.0, 0.0, 0.0));
    assert!(shade.max_element() < 0.02, "under the block is dark, got {shade}");
    assert!(sun < 0.01, "and out of the sun in the shadow mask");
}

#[test]
fn omni_light_follows_godots_falloff() {
    let mut light = Light::omni(Vec3::new(0.0, 32.0, 0.0), Vec3::ONE, 320.0);
    light.bake_direct = true;
    let input = BakeInput {
        surfaces: vec![floor(128.0)],
        materials: vec![Material::default()],
        lights: vec![light],
        sky: Sky::NONE,
        units_per_meter: 32.0,
        ..Default::default()
    };
    let map = bake(&input, &settings(), &Progress::default()).unwrap();
    // One meter straight below, well inside the range: Godot's window is almost 1 and the decay 1 / 1 m.
    let (lit, _, _) = value(&map, 0, Vec3::ZERO);
    assert!((lit.x - 1.0).abs() < 0.03, "{lit}");
    // Two meters to the side the light comes in at 1 / sqrt(5) m and the cosine is 1 / sqrt(5).
    let (side, _, _) = value(&map, 0, Vec3::new(64.0, 0.0, 0.0));
    let d = 5f32.sqrt();
    let window = (1.0 - (d / 10.0).powi(4)).powi(2);
    assert!((side.x - window / d / d).abs() < 0.03, "{side}");
}

#[test]
fn corners_are_occluded_and_the_sky_lights_an_open_floor() {
    let mut surfaces = vec![floor(256.0)];
    // A wall standing on the floor along x = 0.
    surfaces.push(quad(1, Vec3::new(0.0, 64.0, 0.0), Vec3::Z * 256.0, Vec3::Y * 64.0, 0));
    let sky = Sky { top: Vec3::ONE, horizon: Vec3::ONE, ground: Vec3::ONE, ambient: Vec3::ZERO, sky_share: 1.0, energy: 1.0 };
    let input = BakeInput {
        surfaces,
        materials: vec![Material { albedo: Vec3::ZERO, ..Material::default() }],
        lights: vec![],
        sky,
        units_per_meter: 32.0,
        ..Default::default()
    };
    let map = bake(&input, &Settings { rays: 256, ..settings() }, &Progress::default()).unwrap();
    let (open, _, open_ao) = value(&map, 0, Vec3::new(-200.0, 0.0, 0.0));
    let (corner, _, corner_ao) = value(&map, 0, Vec3::new(-6.0, 0.0, 0.0));
    assert!(open.x > 0.9, "a white sky over an open floor reads 1, got {open}");
    assert!(corner.x < open.x * 0.8, "the wall hides half the sky at its foot, {corner} against {open}");
    assert!(corner_ao < open_ao - 0.2, "{corner_ao} against {open_ao}");
}

#[test]
fn bounces_brighten_a_closed_white_room() {
    let surfaces = cube(0, Vec3::ZERO, Vec3::splat(128.0), true, 0);
    let light = Light { bake_direct: true, ..Light::omni(Vec3::ZERO, Vec3::ONE, 640.0) };
    let input = BakeInput {
        surfaces,
        materials: vec![Material { albedo: Vec3::splat(0.8), ..Material::default() }],
        lights: vec![light],
        sky: Sky::NONE,
        units_per_meter: 32.0,
        ..Default::default()
    };
    let floor_center = Vec3::new(0.0, -128.0, 0.0);
    let direct = bake(&input, &settings(), &Progress::default()).unwrap();
    let bounced = bake(&input, &Settings { bounces: 2, ..settings() }, &Progress::default()).unwrap();
    let floor_key = 3;
    let (d, _, _) = value(&direct, floor_key, floor_center);
    let (b, _, _) = value(&bounced, floor_key, floor_center);
    assert!(d.x > 0.1, "{d}");
    assert!(b.x > d.x * 1.2, "light bouncing off the walls adds up, {b} against {d}");
    assert!(direct.ao.iter().all(|a| a.is_finite()));
}

#[test]
fn a_glowing_ceiling_lights_the_room_without_any_light() {
    let mut surfaces = cube(0, Vec3::ZERO, Vec3::splat(128.0), true, 0);
    // The ceiling faces down into the room.
    let ceiling = surfaces.iter_mut().find(|s| s.normals[0].y < -0.5).unwrap();
    ceiling.material = 1;
    let materials = vec![Material::default(), Material { emission: Vec3::splat(2.0), ..Material::default() }];
    let input = BakeInput { surfaces, materials, lights: vec![], sky: Sky::NONE, units_per_meter: 32.0, ..Default::default() };
    let map = bake(&input, &settings(), &Progress::default()).unwrap();
    let (lit, _, _) = value(&map, 3, Vec3::new(0.0, -128.0, 0.0));
    assert!(lit.x > 0.3, "{lit}");
}

#[test]
fn a_cancelled_bake_stops() {
    let progress = Progress::default();
    progress.cancel();
    let input =
        BakeInput { surfaces: vec![floor(64.0)], materials: vec![Material::default()], lights: vec![Light::sun(Vec3::NEG_Y, Vec3::ONE)], ..Default::default() };
    assert_eq!(bake(&input, &settings(), &progress), Err(BakeError::Cancelled));
    assert_eq!(bake(&BakeInput::default(), &settings(), &Progress::default()), Err(BakeError::NothingToBake));
}

#[test]
fn probes_fill_the_open_space_with_light_from_the_glowing_ceiling() {
    let mut surfaces = cube(0, Vec3::ZERO, Vec3::splat(128.0), true, 0);
    let ceiling = surfaces.iter_mut().find(|s| s.normals[0].y < -0.5).unwrap();
    ceiling.material = 1;
    surfaces.extend(cube(10, Vec3::new(64.0, -96.0, 64.0), Vec3::splat(32.0), false, 0));
    let materials = vec![Material::default(), Material { emission: Vec3::splat(2.0), ..Material::default() }];
    let hand = Vec3::new(-100.0, 10.0, 7.0);
    let input = BakeInput { surfaces, materials, lights: vec![], sky: Sky::NONE, units_per_meter: 32.0, probe_points: vec![hand] };
    let map = bake(&input, &settings(), &Progress::default()).unwrap();
    let probes = &map.probes;
    assert_eq!(probes.points[0], hand, "the probe placed by hand comes first");
    assert!(probes.points.len() > 20 && probes.sh.len() == probes.points.len(), "{} probes", probes.points.len());
    let block = |p: Vec3| (p - Vec3::new(64.0, -96.0, 64.0)).abs().max_element() < 31.0;
    assert!(!probes.points.iter().any(|p| block(*p)), "no probe inside the solid block");
    for (p, sh) in probes.points.iter().zip(&probes.sh) {
        // Godot's ambient is 0.886227 * sh[0], the up facing light adds 1.023327 * sh[1].
        let ambient = sh[0].x * 0.886227;
        assert!(ambient > 0.05, "probe at {p} sees the ceiling, {ambient}");
        assert!(sh[1].x > 0.0, "probe at {p} gets more light from above, {}", sh[1].x);
    }

    assert!(!probes.tetrahedra.is_empty() && !probes.bsp.is_empty());
    let center = glam::DVec3::new(-10.0, 3.0, -20.0);
    let t = probes::find(&probes.bsp, center).expect("the room's center lies in a tetrahedron");
    let [a, b, c, d] = probes.tetrahedra[t].map(|i| probes.points[i as usize].as_dvec3());
    let vol = |a: glam::DVec3, b: glam::DVec3, c: glam::DVec3, d: glam::DVec3| (b - a).cross(c - a).dot(d - a).abs();
    let parts = vol(center, b, c, d) + vol(a, center, c, d) + vol(a, b, center, d) + vol(a, b, c, center);
    assert!((parts - vol(a, b, c, d)).abs() < 1e-3 * vol(a, b, c, d), "the tree uses map units, like the points");
}
