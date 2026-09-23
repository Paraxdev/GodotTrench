//! Methods and properties of the Godot node classes that entity definitions use, taken from Godot's extension API by
//! `tools/gen_godot_methods.py`. The I/O runtime calls any method of a target and sets any property, so these are
//! valid inputs next to the ones a definition declares.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use serde::Deserialize;

#[derive(Deserialize)]
struct Class {
    inherits: String,
    members: Vec<String>,
}

static CLASSES: LazyLock<BTreeMap<String, Class>> = LazyLock::new(|| serde_json::from_str(include_str!("godot_methods.json")).unwrap_or_default());

/// Methods and properties of `class` and its ancestors, None for a class the table does not know.
pub fn class_members(class: &str) -> Option<Vec<&'static str>> {
    let mut class = CLASSES.get(class)?;
    let mut out = Vec::new();
    loop {
        out.extend(class.members.iter().map(String::as_str));
        match CLASSES.get(&class.inherits) {
            Some(parent) => class = parent,
            None => return Some(out),
        }
    }
}

/// Names of the functions a GDScript source defines, static ones included.
pub fn script_functions(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|l| l.strip_prefix("static func ").or_else(|| l.strip_prefix("func ")))
        .filter_map(|rest| rest.split('(').next())
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes_inherit_their_members() {
        let light = class_members("OmniLight3D").unwrap();
        for m in ["omni_range", "light_energy", "set_visible", "visible", "queue_free", "set_process"] {
            assert!(light.contains(&m), "{m}");
        }

        assert!(!light.contains(&"set_visibel"));
        assert!(class_members("StaticBody3D").unwrap().contains(&"set_collision_layer"));
        assert!(class_members("NpcGuard").is_none(), "custom classes are unknown");
    }

    #[test]
    fn script_functions_are_found() {
        let src = "extends Node3D\nfunc turn_on() -> void:\n\tpass\nstatic func helper(a):\n\tpass\n# func not_this()\nvar func_x := 1\n";
        assert_eq!(script_functions(src), vec!["turn_on", "helper"]);
    }
}
