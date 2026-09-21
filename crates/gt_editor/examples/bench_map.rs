//! Headless timing of the per-frame scene rebuild work for a heavy map.
//! Run: cargo run --release --example bench_map -- "C:\\Users\\parax\\Downloads\\map.gtm"

use std::collections::BTreeSet;
use std::time::Instant;

use gt_core::DVec3;
use gt_doc::NodeKind;
use gt_editor::face_cull::FaceCull;
use gt_formats::GameConfig;

fn main() {
    let path = std::env::args().nth(1).expect("pass the .gtm path");
    let t = Instant::now();
    let text = std::fs::read_to_string(&path).expect("read map");
    println!("read {} bytes in {:?}", text.len(), t.elapsed());

    let t = Instant::now();
    let mut map = gt_doc::format::from_str(&text).expect("parse map");
    println!("parse map in {:?}", t.elapsed());

    let mesh_ids: Vec<_> = map.meshes().map(|(id, m)| (id, m.vertices.len(), m.faces.len())).collect();
    println!("meshes: {}", mesh_ids.len());
    for (id, v, f) in &mesh_ids {
        println!("  mesh {id:?}: {v} verts, {f} faces");
    }
    let (big_id, _, _) = *mesh_ids.iter().max_by_key(|(_, _, f)| *f).unwrap();

    let game = GameConfig::default();
    let opaque = |_: &str| true;

    // Full build (map load / import).
    let mut cull = FaceCull::default();
    let t = Instant::now();
    cull.update(&map, &game, &opaque, &BTreeSet::new(), true);
    println!("\nface_cull full build: {:?}", t.elapsed());

    // Pure mesh geometry rebuild that Builder::mesh does, for the biggest mesh.
    {
        let m = map.mesh(big_id).unwrap();
        let t = Instant::now();
        let _ = m.corner_normals();
        let cn = t.elapsed();
        let t = Instant::now();
        let _ = m.edge_faces();
        let ef = t.elapsed();
        let t = Instant::now();
        let mut tris = 0usize;
        for fi in 0..m.faces.len() {
            tris += m.triangulate_corners(fi).len();
        }
        let tri = t.elapsed();
        let t = Instant::now();
        let _fnormals: Vec<DVec3> = (0..m.faces.len()).map(|f| m.face_normal(f)).collect();
        let fnrm = t.elapsed();
        println!(
            "biggest mesh geometry: corner_normals {cn:?}, edge_faces {ef:?}, triangulate_all {tri:?} ({tris} tris), face_normals {fnrm:?}",
        );
    }

    // Simulate one drag-move frame: translate the biggest mesh, then do the per-frame rebuild work.
    for frame in 0..5 {
        let t = Instant::now();
        if let Some(m) = map.mesh_mut(big_id) {
            *m = m.translated(DVec3::new(1.0, 0.0, 0.0), false);
        }
        let translate = t.elapsed();

        let dirty: BTreeSet<_> = std::iter::once(big_id).collect();
        let t = Instant::now();
        cull.update(&map, &game, &opaque, &dirty, false);
        let cull_t = t.elapsed();

        // The Builder::mesh geometry cost for the moved mesh.
        let m = map.mesh(big_id).unwrap();
        let t = Instant::now();
        let _ = m.corner_normals();
        let _ = m.edge_faces();
        for fi in 0..m.faces.len() {
            let _ = m.triangulate_corners(fi);
        }
        let build_t = t.elapsed();

        println!(
            "drag frame {frame}: translate {translate:?} + face_cull {cull_t:?} + mesh build {build_t:?} = {:?}  (deferred-cull frame ~= {:?})",
            translate + cull_t + build_t,
            translate + build_t,
        );
        let _ = matches!(map.get(big_id).map(|n| &n.kind), Some(NodeKind::Mesh(_)));
    }
}
