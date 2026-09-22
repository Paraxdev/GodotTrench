//! Code references for entity definitions: GDScript and C# classes that implement an entity, snippets for driving its
//! inputs and outputs from game code, and the FuncGodot `.tres` resource that defines it.

use gt_formats::{EntityDef, EntityKind, PropertyDef, PropertyType};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodeKind {
    GdscriptClass,
    CsharpClass,
    GdscriptUsage,
    CsharpUsage,
    FgdResource,
}

impl CodeKind {
    pub const ALL: [CodeKind; 5] = [CodeKind::GdscriptUsage, CodeKind::CsharpUsage, CodeKind::GdscriptClass, CodeKind::CsharpClass, CodeKind::FgdResource];

    pub fn label(&self) -> &'static str {
        match self {
            CodeKind::GdscriptClass => "GDScript class",
            CodeKind::CsharpClass => "C# class",
            CodeKind::GdscriptUsage => "Use from GDScript",
            CodeKind::CsharpUsage => "Use from C#",
            CodeKind::FgdResource => "FGD .tres",
        }
    }

    pub fn from_name(name: &str) -> Option<CodeKind> {
        match name.to_ascii_lowercase().replace([' ', '-'], "_").as_str() {
            "gdscript" | "gdscript_class" => Some(CodeKind::GdscriptClass),
            "csharp" | "c#" | "csharp_class" | "c#_class" => Some(CodeKind::CsharpClass),
            "gdscript_usage" | "use_from_gdscript" => Some(CodeKind::GdscriptUsage),
            "csharp_usage" | "use_from_c#" => Some(CodeKind::CsharpUsage),
            "fgd" | "tres" | "fgd_resource" | "fgd_.tres" => Some(CodeKind::FgdResource),
            _ => None,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            CodeKind::GdscriptClass | CodeKind::GdscriptUsage => "gd",
            CodeKind::CsharpClass | CodeKind::CsharpUsage => "cs",
            CodeKind::FgdResource => "tres",
        }
    }
}

pub fn generate(def: &EntityDef, kind: CodeKind) -> String {
    match kind {
        CodeKind::GdscriptClass => gdscript_class(def),
        CodeKind::CsharpClass => csharp_class(def),
        CodeKind::GdscriptUsage => gdscript_usage(def),
        CodeKind::CsharpUsage => csharp_usage(def),
        CodeKind::FgdResource => fgd_resource(def),
    }
}

pub fn pascal(name: &str) -> String {
    name.split(['_', ' ', '-']).filter(|p| !p.is_empty()).map(|p| p[..1].to_ascii_uppercase() + &p[1..]).collect()
}

fn node_class(def: &EntityDef) -> &str {
    if !def.node_class.is_empty() {
        &def.node_class
    } else if def.kind == EntityKind::Solid {
        "StaticBody3D"
    } else {
        "Node3D"
    }
}

fn is_name_property(p: &PropertyDef) -> bool {
    matches!(p.ty, PropertyType::TargetSource) || p.name == "targetname"
}

fn gd_type(p: &PropertyDef) -> (&'static str, String) {
    let d = p.default.trim();
    match p.ty {
        PropertyType::Int | PropertyType::Flags => ("int", d.parse::<i64>().unwrap_or(0).to_string()),
        PropertyType::Float => ("float", format!("{:?}", d.parse::<f64>().unwrap_or(0.0))),
        PropertyType::Bool => ("bool", (matches!(d, "1" | "true" | "True")).to_string()),
        PropertyType::Vector3 => {
            let v: Vec<f64> = d.split_whitespace().filter_map(|x| x.parse().ok()).collect();
            ("Vector3", if v.len() >= 3 { format!("Vector3({}, {}, {})", v[0], v[1], v[2]) } else { "Vector3.ZERO".into() })
        }
        PropertyType::Color => ("Color", "Color.WHITE".into()),
        _ => ("String", format!("{d:?}")),
    }
}

fn cs_type(p: &PropertyDef) -> (&'static str, String) {
    let d = p.default.trim();
    match p.ty {
        PropertyType::Int | PropertyType::Flags => ("int", d.parse::<i64>().unwrap_or(0).to_string()),
        PropertyType::Float => ("float", format!("{}f", d.parse::<f64>().unwrap_or(0.0))),
        PropertyType::Bool => ("bool", (matches!(d, "1" | "true" | "True")).to_string()),
        PropertyType::Vector3 => {
            let v: Vec<f64> = d.split_whitespace().filter_map(|x| x.parse().ok()).collect();
            ("Vector3", if v.len() >= 3 { format!("new Vector3({}f, {}f, {}f)", v[0], v[1], v[2]) } else { "Vector3.Zero".into() })
        }
        PropertyType::Color => ("Color", "Colors.White".into()),
        _ => ("string", format!("{d:?}")),
    }
}

fn params(p: &str) -> Vec<String> {
    p.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

pub fn gdscript_class(def: &EntityDef) -> String {
    let class = pascal(&def.classname);
    let mut out = format!("@tool\nclass_name {class} extends {}\n", node_class(def));
    if !def.description.is_empty() {
        out += &format!("## {}\n", def.description);
    }

    out += "\n";
    for o in &def.outputs {
        let args: Vec<String> = params(&o.parameter).iter().map(|a| format!("{a}: Variant")).collect();
        out += &format!("signal {}{}\n", o.name, if args.is_empty() { String::new() } else { format!("({})", args.join(", ")) });
    }

    if !def.outputs.is_empty() {
        out += "\n";
    }

    let props: Vec<&PropertyDef> = def.properties.iter().filter(|p| !is_name_property(p)).collect();
    for p in &props {
        let (ty, default) = gd_type(p);
        if !p.description.is_empty() {
            out += &format!("## {}\n", p.description);
        }

        out += &format!("@export var {}: {ty} = {default}\n", p.name);
    }

    out += "\n## FuncGodot passes the entity's map properties here when the map is built.\nfunc _func_godot_apply_properties(props: Dictionary) -> void:\n";
    if props.is_empty() {
        out += "\tpass\n";
    }

    for p in &props {
        let line = match p.ty {
            PropertyType::Int | PropertyType::Flags => format!("\t{0} = int(props.get(\"{0}\", {0}))\n", p.name),
            PropertyType::Float => format!("\t{0} = float(props.get(\"{0}\", {0}))\n", p.name),
            PropertyType::Bool => format!("\t{0} = GodotTrenchIO.to_bool(props.get(\"{0}\", {0}))\n", p.name),
            PropertyType::Vector3 => format!("\t{0} = GodotTrenchIO.to_vector3(props.get(\"{0}\", {0}))\n", p.name),
            PropertyType::Color => format!("\t{0} = GodotTrenchIO.to_color(props.get(\"{0}\", {0}))\n", p.name),
            _ => format!("\t{0} = str(props.get(\"{0}\", {0}))\n", p.name),
        };
        out += &line;
    }

    for i in &def.inputs {
        let args: Vec<String> = params(&i.parameter).iter().map(|a| format!("{a}: Variant = null")).collect();
        out += &format!(
            "\n## Input: {}\nfunc {}({}) -> void:\n\tpass\n",
            if i.description.is_empty() { &i.name } else { &i.description },
            i.name,
            args.join(", ")
        );
    }

    out
}

pub fn csharp_class(def: &EntityDef) -> String {
    let class = pascal(&def.classname);
    let mut out = String::from("using Godot;\nusing Godot.Collections;\n\n");
    if !def.description.is_empty() {
        out += &format!("/// <summary>{}</summary>\n", def.description);
    }

    out += &format!("[GlobalClass, Tool]\npublic partial class {class} : {}\n{{\n", node_class(def));
    for o in &def.outputs {
        let args: Vec<String> = params(&o.parameter).iter().map(|a| format!("Variant {a}")).collect();
        out += &format!("    [Signal] public delegate void {}EventHandler({});\n", pascal(&o.name), args.join(", "));
    }

    if !def.outputs.is_empty() {
        out += "\n";
    }

    let props: Vec<&PropertyDef> = def.properties.iter().filter(|p| !is_name_property(p)).collect();
    for p in &props {
        let (ty, default) = cs_type(p);
        if !p.description.is_empty() {
            out += &format!("    /// <summary>{}</summary>\n", p.description);
        }

        out += &format!("    [Export] public {ty} {} {{ get; set; }} = {default};\n", pascal(&p.name));
    }

    out += "\n    // FuncGodot calls this by name when the map is built, so it keeps the snake_case spelling.\n";
    out += "    public void _func_godot_apply_properties(Dictionary props)\n    {\n";
    for p in &props {
        let getter = match p.ty {
            PropertyType::Int | PropertyType::Flags => "GodotTrench.Int",
            PropertyType::Float => "GodotTrench.Float",
            PropertyType::Bool => "GodotTrench.Bool",
            PropertyType::Vector3 => "GodotTrench.Vector3",
            PropertyType::Color => "GodotTrench.Color",
            _ => "GodotTrench.String",
        };
        out += &format!("        {} = {getter}(props, \"{}\", {});\n", pascal(&p.name), p.name, pascal(&p.name));
    }

    out += "    }\n";
    for i in &def.inputs {
        let args: Vec<String> = params(&i.parameter).iter().map(|a| format!("Variant {a} = default")).collect();
        out += &format!(
            "\n    // Input \"{}\": GodotTrench I/O also finds PascalCase methods.\n    public void {}({})\n    {{\n    }}\n",
            i.name,
            pascal(&i.name),
            args.join(", ")
        );
    }

    out += "}\n";
    out
}

pub fn gdscript_usage(def: &EntityDef) -> String {
    let input = def.inputs.first().map(|i| i.name.as_str()).unwrap_or("toggle");
    let output = def.outputs.first().map(|o| o.name.as_str()).unwrap_or("triggered");
    let name = format!("my_{}", def.classname.split_once('_').map(|(_, r)| r).unwrap_or(&def.classname));
    let mut out = format!("# {}: {}\n", def.classname, def.description);
    out += &format!(
        "# Script: {}\n# Node: {}\n\n",
        if def.script.is_empty() { "(none, set script_class in the FGD entry)" } else { &def.script },
        node_class(def)
    );
    out += &format!(
        "# Call an input the way a map output would (entities are found by targetname):\nfor node in GodotTrenchIO.find_targets(self, \"{name}\", null):\n\tGodotTrenchIO.invoke(node, &\"{input}\", \"\", player)\n\n"
    );
    out += &format!(
        "# Wildcards, groups and node paths work as targets too:\nGodotTrenchIO.find_targets(self, \"{name}*\", null)\nGodotTrenchIO.find_targets(self, \"@enemies\", null)\nGodotTrenchIO.find_targets(self, \"/root/Game\", null)\n\n"
    );
    out += &format!(
        "# React to an output in code:\nvar entity := GodotTrenchIO.find_targets(self, \"{name}\", null)[0]\nentity.{output}.connect(func(...args): print(\"{output}\", args))\n\n"
    );
    out +=
        &format!("# Fire an entity's outputs manually (runs its I/O connections with delays):\nGodotTrenchIO.fire_output(entity, &\"{output}\", player)\n\n");
    out += "# Every I/O event in the running game, for debugging or achievements:\nGodotTrenchIO.events().fired.connect(func(source, output, target, input, parameter): print(source.name, \".\", output, \" > \", target, \".\", input))\n";
    out += "\n# Map outputs can call your own code without any entity in between:\n#   output: triggered   target: /root/Game   input: add_score   parameter: [10, \"coin\"]\n";
    if !def.properties.is_empty() {
        out += "\n# Properties set in the editor:\n";
        for p in &def.properties {
            out += &format!("#   {} ({:?}) = {:?}  {}\n", p.name, p.ty, p.default, p.description);
        }
    }

    out
}

pub fn csharp_usage(def: &EntityDef) -> String {
    let input = def.inputs.first().map(|i| i.name.as_str()).unwrap_or("toggle");
    let output = def.outputs.first().map(|o| o.name.as_str()).unwrap_or("triggered");
    let name = format!("my_{}", def.classname.split_once('_').map(|(_, r)| r).unwrap_or(&def.classname));
    let mut out = format!(
        "// {}: {}\n// GodotTrench I/O is GDScript, C# reaches it through the GodotTrench helper (GodotTrench.cs).\n\n",
        def.classname, def.description
    );
    out += &format!(
        "// Call an input the way a map output would:\nforeach (Node node in GodotTrench.FindTargets(this, \"{name}\"))\n    GodotTrench.Invoke(node, \"{input}\", \"\", player);\n\n"
    );
    out += &format!(
        "// React to an output:\nvar entity = GodotTrench.FindTargets(this, \"{name}\")[0];\nentity.Connect(\"{output}\", Callable.From(() => GD.Print(\"{output}\")));\n\n"
    );
    out += &format!("// Fire an entity's outputs manually:\nGodotTrench.FireOutput(entity, \"{output}\", player);\n\n");
    out += "// Outputs in the map can target C# nodes directly. Methods are found by their snake_case name first,\n// then PascalCase, so input \"add_score\" calls public void AddScore(int amount).\n";
    out
}

fn gd_string(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn gd_float(v: f64) -> String {
    let s = format!("{v}");
    if s.contains('.') || s.contains('e') { s } else { format!("{s}.0") }
}

/// A choice value as the Godot int or string FuncGodot compares against.
fn choice_value(v: &str) -> String {
    v.parse::<i64>().map(|i| i.to_string()).unwrap_or_else(|_| gd_string(v))
}

fn godot_value(p: &PropertyDef) -> String {
    let d = p.default.trim();
    match p.ty {
        PropertyType::Int | PropertyType::Flags => d.parse::<i64>().unwrap_or(0).to_string(),
        PropertyType::Float => gd_float(d.parse::<f64>().unwrap_or(0.0)),
        PropertyType::Bool => (matches!(d, "1" | "true" | "True")).to_string(),
        PropertyType::Color => {
            let v: Vec<f64> = d.split_whitespace().filter_map(|x| x.parse().ok()).collect();
            let s = if v.iter().any(|c| *c > 1.0) { 255.0 } else { 1.0 };
            if v.len() >= 3 { format!("Color({}, {}, {}, 1)", v[0] / s, v[1] / s, v[2] / s) } else { "Color(1, 1, 1, 1)".into() }
        }
        PropertyType::Choices => {
            let entries: Vec<String> = p.options.iter().map(|(l, v)| format!("{}: {}", gd_string(l), choice_value(v))).collect();
            format!("{{\n{}\n}}", entries.join(",\n"))
        }
        _ => gd_string(d),
    }
}

/// Property types that cannot be told from the Godot default value, exported through meta properties.
fn declared_type(p: &PropertyDef) -> Option<&'static str> {
    match p.ty {
        PropertyType::Vector3 => Some("vector3"),
        PropertyType::Resource => Some("resource"),
        PropertyType::TargetSource if p.name != "targetname" => Some("target_source"),
        PropertyType::TargetDestination if !["target", "killtarget", "parent"].contains(&p.name.as_str()) => Some("target_destination"),
        // The Godot exporter infers target_destination from these names, so a plain string has to be declared.
        PropertyType::String if ["target", "killtarget", "parent"].contains(&p.name.as_str()) => Some("string"),
        PropertyType::NodePath => Some("node_path"),
        _ => None,
    }
}

fn gizmo_dict(g: &gt_formats::GizmoDef) -> String {
    let value = serde_json::to_value(g).unwrap_or_default();
    let entries: Vec<String> = value
        .as_object()
        .map(|o| o.iter().map(|(k, v)| format!("{}: {}", gd_string(k), if let Some(s) = v.as_str() { gd_string(s) } else { v.to_string() })).collect())
        .unwrap_or_default();
    format!("{{{}}}", entries.join(", "))
}

fn name_list(names: &[gt_formats::IoDef]) -> String {
    format!("[{}]", names.iter().map(|n| gd_string(&n.name)).collect::<Vec<_>>().join(", "))
}

pub fn fgd_resource(def: &EntityDef) -> String {
    let solid = def.kind == EntityKind::Solid;
    let base = if solid { "func_godot_fgd_solid_class" } else { "func_godot_fgd_point_class" };
    let class = if solid { "FuncGodotFGDSolidClass" } else { "FuncGodotFGDPointClass" };
    let mut out = format!("[gd_resource type=\"Resource\" script_class=\"{class}\" load_steps={} format=3]\n\n", if def.script.is_empty() { 2 } else { 3 });
    out += &format!("[ext_resource type=\"Script\" path=\"res://addons/func_godot/src/fgd/{base}.gd\" id=\"1_class\"]\n");
    if !def.script.is_empty() {
        out += &format!("[ext_resource type=\"Script\" path=\"{}\" id=\"2_script\"]\n", def.script);
    }

    out +=
        &format!("\n[resource]\nscript = ExtResource(\"1_class\")\nclassname = {}\ndescription = {}\n", gd_string(&def.classname), gd_string(&def.description));
    if solid && def.classname.starts_with("trigger") {
        out += "build_visuals = false\n";
    }

    if solid {
        out += "collision_shape_type = 1\n";
    }

    if !def.script.is_empty() {
        out += "script_class = ExtResource(\"2_script\")\n";
    }

    let props: Vec<String> = def.properties.iter().map(|p| format!("{}: {}", gd_string(&p.name), godot_value(p))).collect();
    out += &format!("class_properties = Dictionary[String, Variant]({{\n{}\n}})\n", props.join(",\n"));
    let descs: Vec<String> = def
        .properties
        .iter()
        .map(|p| {
            let text = if p.description.is_empty() { p.name.clone() } else { p.description.clone() };
            if p.ty == PropertyType::Choices {
                format!("{}: [{}, {}]", gd_string(&p.name), gd_string(&text), choice_value(p.default.trim()))
            } else {
                format!("{}: {}", gd_string(&p.name), gd_string(&text))
            }
        })
        .collect();
    out += &format!("class_property_descriptions = Dictionary[String, Variant]({{\n{}\n}})\n", descs.join(",\n"));
    let c = def.color;
    let mut meta = vec![format!("\"color\": Color({}, {}, {}, 1)", c.r, c.g, c.b)];
    if !solid {
        let [a, b] = def.size;
        meta.push(format!("\"size\": AABB({}, {}, {}, {}, {}, {})", a.z, a.x, a.y, b.z, b.x, b.y));
    }

    let types: Vec<String> = def.properties.iter().filter_map(|p| declared_type(p).map(|t| format!("{}: {}", gd_string(&p.name), gd_string(t)))).collect();
    if !types.is_empty() {
        meta.push(format!("\"property_types\": {{{}}}", types.join(", ")));
    }

    if !def.gizmos.is_empty() {
        meta.push(format!("\"gizmos\": [{}]", def.gizmos.iter().map(gizmo_dict).collect::<Vec<_>>().join(", ")));
    }

    if !def.inputs.is_empty() {
        meta.push(format!("\"inputs\": {}", name_list(&def.inputs)));
    }

    if !def.outputs.is_empty() {
        meta.push(format!("\"outputs\": {}", name_list(&def.outputs)));
    }

    out += &format!("meta_properties = Dictionary[String, Variant]({{\n{}\n}})\n", meta.join(",\n"));
    out += &format!("node_class = {}\n", gd_string(node_class(def)));
    out
}

/// Classnames whose definitions ship with the FuncGodot fork in `addons/func_godot/fgd/godottrench/`.
pub const ADDON_ENTITIES: [&str; 35] = [
    "info_teleport_destination",
    "path_corner",
    "light",
    "prop_model",
    "func_door",
    "func_door_rotating",
    "func_gate",
    "func_platform",
    "func_train",
    "func_button",
    "trigger_once",
    "trigger_multiple",
    "trigger_call",
    "trigger_spawn_area",
    "trigger_hurt",
    "trigger_teleport",
    "trigger_push",
    "info_spawner",
    "logic_call",
    "logic_relay",
    "logic_timer",
    "logic_counter",
    "logic_auto",
    "logic_debug",
    "logic_script",
    "logic_sequence",
    "logic_animate",
    "game_text",
    "env_explosion",
    "prop_physics",
    "npc_walker",
    "light_spot",
    "env_sound",
    "env_particles",
    "logic_branch",
];

/// Every addon FGD file as (file name, text): one resource per entity plus the FuncGodotFGDFile listing them.
pub fn addon_fgd_files() -> Vec<(String, String)> {
    let cfg = gt_formats::GameConfig::builtin();
    let mut out = Vec::new();
    for class in ADDON_ENTITIES {
        if let Some(def) = cfg.entity(class) {
            out.push((format!("{class}.tres"), fgd_resource(def)));
        }
    }

    let mut file = format!("[gd_resource type=\"Resource\" script_class=\"FuncGodotFGDFile\" load_steps={} format=3]\n\n", ADDON_ENTITIES.len() + 2);
    file += "[ext_resource type=\"Script\" path=\"res://addons/func_godot/src/fgd/func_godot_fgd_file.gd\" id=\"1_fgd\"]\n";
    for (i, class) in ADDON_ENTITIES.iter().enumerate() {
        file += &format!("[ext_resource type=\"Resource\" path=\"res://addons/func_godot/fgd/godottrench/{class}.tres\" id=\"{}_{class}\"]\n", i + 2);
    }

    let refs: Vec<String> = ADDON_ENTITIES.iter().enumerate().map(|(i, class)| format!("ExtResource(\"{}_{class}\")", i + 2)).collect();
    file +=
        &format!("\n[resource]\nscript = ExtResource(\"1_fgd\")\nfgd_name = \"GodotTrench\"\nentity_definitions = Array[Resource]([{}])\n", refs.join(", "));
    out.push(("godottrench_fgd.tres".into(), file));
    out
}

/// C# helper that exposes the GDScript I/O runtime and property parsing to C# entities.
pub const CSHARP_HELPER: &str = r#"using Godot;
using Godot.Collections;

/// <summary>Bridge from C# to the GodotTrench runtime of the FuncGodot fork.</summary>
public static class GodotTrench
{
    private static GDScript _io;
    private static GDScript IO => _io ??= GD.Load<GDScript>("res://addons/func_godot/src/godottrench/runtime/godottrench_io.gd");

    public static Array<Node> FindTargets(Node from, string target, Node activator = null)
    {
        var found = new Array<Node>();
        foreach (var n in (Array)IO.Call("find_targets", from, target, activator))
            found.Add(n.AsGodotObject() as Node);
        return found;
    }

    public static void Invoke(Node node, string input, string parameter = "", Node activator = null) =>
        IO.Call("invoke", node, input, parameter, activator);

    public static void FireOutput(Node source, string output, Node activator = null) =>
        IO.Call("fire_output", source, output, activator);

    public static string String(Dictionary props, string key, string fallback) =>
        props.TryGetValue(key, out var v) ? v.ToString() : fallback;

    public static int Int(Dictionary props, string key, int fallback) =>
        props.TryGetValue(key, out var v) && int.TryParse(v.ToString(), out var i) ? i : fallback;

    public static float Float(Dictionary props, string key, float fallback) =>
        props.TryGetValue(key, out var v) && float.TryParse(v.ToString(), System.Globalization.NumberStyles.Float, System.Globalization.CultureInfo.InvariantCulture, out var f) ? f : fallback;

    public static bool Bool(Dictionary props, string key, bool fallback) =>
        props.TryGetValue(key, out var v) ? (bool)IO.Call("to_bool", v) : fallback;

    public static Vector3 Vector3(Dictionary props, string key, Vector3 fallback) =>
        props.TryGetValue(key, out var v) ? (Vector3)IO.Call("to_vector3", v) : fallback;

    public static Color Color(Dictionary props, string key, Color fallback) =>
        props.TryGetValue(key, out var v) ? (Color)IO.Call("to_color", v) : fallback;
}

/// <summary>
/// Declares a C# node class as a map entity without an FGD resource. The FuncGodot fork scans the source folders listed in
/// GodotTrenchGameConfig.csharp_source_dirs: [Export] members become properties, [Signal] delegates outputs and
/// [GodotTrenchInput] methods inputs. Size is "minX minY minZ maxX maxY maxZ" in map units.
/// </summary>
[System.AttributeUsage(System.AttributeTargets.Class)]
public sealed class GodotTrenchEntityAttribute : System.Attribute
{
    public GodotTrenchEntityAttribute(string classname) { Classname = classname; }
    public string Classname { get; }
    public bool Solid { get; set; }
    public string Description { get; set; } = "";
    public string Color { get; set; } = "";
    public string Size { get; set; } = "";
}

/// <summary>Marks a method as an I/O input that map outputs can call.</summary>
[System.AttributeUsage(System.AttributeTargets.Method)]
public sealed class GodotTrenchInputAttribute : System.Attribute { }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_code_mentions_every_io_and_property() {
        let cfg = gt_formats::GameConfig::builtin();
        let door = cfg.entity("func_door_rotating").unwrap();
        let gd = gdscript_class(door);
        assert!(gd.contains("class_name FuncDoorRotating extends AnimatableBody3D"));
        assert!(gd.contains("signal locked_use(activator: Variant)"));
        assert!(gd.contains("@export var open_angle: float = 95.0"));
        assert!(gd.contains("func use(activator: Variant = null) -> void:"));
        let cs = csharp_class(door);
        assert!(cs.contains("public partial class FuncDoorRotating : AnimatableBody3D"));
        assert!(cs.contains("[Signal] public delegate void LockedUseEventHandler(Variant activator);"));
        assert!(cs.contains("OpenAngle = GodotTrench.Float(props, \"open_angle\", OpenAngle);"));
        let tres = fgd_resource(door);
        assert!(tres.contains("classname = \"func_door_rotating\"") && tres.contains("gt_door_rotating.gd"));
        assert!(gdscript_usage(door).contains("GodotTrenchIO.invoke(node, &\"open\""));
        assert_eq!(CodeKind::from_name("c#"), Some(CodeKind::CsharpClass));
    }

    #[test]
    fn generates_code_for_the_new_entities() {
        let cfg = gt_formats::GameConfig::builtin();
        let barrel = cfg.entity("prop_physics").unwrap();
        let cs = csharp_class(barrel);
        assert!(cs.contains("public partial class PropPhysics : RigidBody3D"), "{cs}");
        assert!(cs.contains("[Signal] public delegate void BrokenEventHandler();"));
        assert!(csharp_usage(barrel).contains("GodotTrench.Invoke(node, \"smash\""), "C# reaches the new inputs");

        let script = cfg.entity("logic_script").unwrap();
        assert!(csharp_class(script).contains("public partial class LogicScript : Node3D"));
        assert!(gdscript_usage(script).contains("GodotTrenchIO.invoke(node, &\"run\""));
    }
}
