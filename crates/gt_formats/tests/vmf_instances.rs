//! `func_instance` maps are inlined as groups, placed, renamed and wired the way vbsp collapses them.

use gt_core::DVec3;
use gt_doc::NodeKind;
use gt_formats::vmf::{ImportOptions, import_file};

fn fixture() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vmf/maps/sub/main.vmf")
}

#[test]
fn inlines_instances_found_up_the_folders() {
    let (map, report) = import_file(&fixture(), &ImportOptions::default()).unwrap();
    assert_eq!(report.instances, 1);
    assert_eq!(report.missing_instances, vec!["instances/missing.vmf".to_string()]);

    let group = map.nodes.values().find(|n| matches!(&n.kind, NodeKind::Group(g) if g.name == "inst (lamp_box)")).expect("instance group");
    let (brush_id, brush) = map.brushes().next().expect("instance brush");
    assert!(map.is_ancestor(group.id, brush_id));
    assert!(brush.faces.iter().all(|f| f.data.material == "brick/wall"), "#material replacement");
    // 16 x 8 x 4 in the instance, turned 90 degrees and moved 100 along id x. Godot x is id y, z is id x.
    let bounds = brush.bounds();
    assert!((bounds.min - DVec3::new(0.0, 0.0, 92.0)).length() < 1e-6 && (bounds.max - DVec3::new(16.0, 4.0, 100.0)).length() < 1e-6, "{bounds:?}");

    let by_name = |name: &str| map.entities().find(|(_, e)| e.targetname() == Some(name)).map(|(_, e)| e.clone());
    let lamp = by_name("inst-lamp").expect("prefixed lamp");
    assert_eq!(lamp.property("_light"), Some("255 0 0 200"), "$color replaced");
    assert_eq!(lamp.property("speed"), Some("50"), "func_instance_parms default");
    assert!((lamp.origin - DVec3::new(8.0, 0.0, 100.0)).length() < 1e-6, "{}", lamp.origin);
    let facing = lamp.rotation() * DVec3::NEG_Z;
    assert!((facing - DVec3::X).length() < 1e-6, "a light facing id +x turns to id +y, Godot +x: {facing}");

    let relay = by_name("inst-relay").expect("prefixed relay");
    let targets: Vec<&str> = relay.outputs.iter().map(|o| o.target.as_str()).collect();
    assert!(targets.contains(&"inst-lamp") && targets.contains(&"@global"), "{targets:?}");
    assert!(relay.outputs.iter().any(|o| o.output == "OnTrigger" && o.target == "door" && o.input == "Open"), "instance output wired out");

    let (_, auto) = map.entities().find(|(_, e)| e.classname == "logic_auto").unwrap();
    assert_eq!((auto.outputs[0].target.as_str(), auto.outputs[0].input.as_str()), ("inst-relay", "Trigger"), "instance input wired in");
    assert!(map.entities().all(|(_, e)| e.classname != "func_instance_parms"));
    assert_eq!(map.entities().filter(|(_, e)| e.classname == "func_instance").count(), 1, "the missing instance stays an entity");
}
