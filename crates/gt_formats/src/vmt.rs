//! Valve material (`.vmt`) reader: the shader and the parameters a Godot material can use. `patch` materials are
//! resolved through their `include`.

use std::collections::BTreeMap;

use crate::vmf::{Block, VmfError, parse_keyvalues};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Vmt {
    /// Lowercased, for example `lightmappedgeneric`.
    pub shader: String,
    /// Keys lowercased (`$basetexture`), values as written.
    pub params: BTreeMap<String, String>,
}

#[derive(Debug, thiserror::Error)]
pub enum VmtError {
    #[error(transparent)]
    Syntax(#[from] VmfError),
    #[error("empty material")]
    Empty,
    #[error("patch material includes {0}, which was not found")]
    MissingInclude(String),
    #[error("patch materials nest too deep")]
    TooDeep,
}

fn collect(block: &Block, params: &mut BTreeMap<String, String>) {
    for (k, v) in &block.props {
        params.insert(k.to_ascii_lowercase(), v.clone());
    }
}

/// Parses a material. `include` loads the material a `patch` names, given its path as written, such as
/// `materials/brick/brickwall001a.vmt`.
pub fn parse(text: &str, include: &mut dyn FnMut(&str) -> Option<String>) -> Result<Vmt, VmtError> {
    parse_depth(text, include, 0)
}

fn parse_depth(text: &str, include: &mut dyn FnMut(&str) -> Option<String>, depth: usize) -> Result<Vmt, VmtError> {
    let root = parse_keyvalues(text)?.into_iter().next().ok_or(VmtError::Empty)?;
    if !root.name.eq_ignore_ascii_case("patch") {
        let mut params = BTreeMap::new();
        collect(&root, &mut params);
        return Ok(Vmt { shader: root.name.to_ascii_lowercase(), params });
    }

    if depth > 8 {
        return Err(VmtError::TooDeep);
    }

    let target = root.get("include").unwrap_or_default().to_string();
    let base = include(&target).ok_or_else(|| VmtError::MissingInclude(target.clone()))?;
    let mut vmt = parse_depth(&base, include, depth + 1)?;
    for name in ["insert", "replace"] {
        for b in root.children_named(name) {
            collect(b, &mut vmt.params);
        }
    }

    Ok(vmt)
}

/// A material path as Source stores it: lowercase, forward slashes, no `materials/` prefix and no extension.
pub fn normalize(path: &str) -> String {
    let p = path.trim().replace('\\', "/").to_ascii_lowercase();
    let p = p.trim_start_matches('/');
    let p = p.strip_prefix("materials/").unwrap_or(p);
    p.strip_suffix(".vmt").or_else(|| p.strip_suffix(".vtf")).unwrap_or(p).to_string()
}

impl Vmt {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(|s| s.as_str()).filter(|s| !s.trim().is_empty())
    }

    fn flag(&self, key: &str) -> bool {
        self.get(key).and_then(|v| v.trim().parse::<f32>().ok()).is_some_and(|v| v != 0.0)
    }

    pub fn base_texture(&self) -> Option<String> {
        self.get("$basetexture").map(normalize)
    }

    /// `$bumpmap`, or `$normalmap`, which water prefers since its `$bumpmap` is a refraction offset map.
    pub fn normal_map(&self) -> Option<String> {
        let (first, second) = if self.water() { ("$normalmap", "$bumpmap") } else { ("$bumpmap", "$normalmap") };
        self.get(first).or(self.get(second)).map(normalize)
    }

    pub fn translucent(&self) -> bool {
        self.flag("$translucent")
    }

    pub fn additive(&self) -> bool {
        self.flag("$additive")
    }

    pub fn alpha_test(&self) -> bool {
        self.flag("$alphatest")
    }

    /// `$alphatestreference`, Source's default is 0.5.
    pub fn alpha_test_reference(&self) -> f32 {
        self.get("$alphatestreference").and_then(|v| v.trim().parse().ok()).unwrap_or(0.5)
    }

    /// The base texture's alpha, or `$selfillummask`, marks the glowing parts.
    pub fn self_illum(&self) -> bool {
        self.flag("$selfillum")
    }

    pub fn self_illum_mask(&self) -> Option<String> {
        self.get("$selfillummask").map(normalize)
    }

    pub fn surface_prop(&self) -> Option<&str> {
        self.get("$surfaceprop")
    }

    pub fn no_cull(&self) -> bool {
        self.flag("$nocull")
    }

    pub fn unlit(&self) -> bool {
        self.shader == "unlitgeneric"
    }

    pub fn water(&self) -> bool {
        self.shader == "water"
    }

    /// `$color` as 0 to 1 RGB, written as `[r g b]` (0 to 1) or `{r g b}` (0 to 255).
    pub fn color(&self) -> Option<[f32; 3]> {
        let v = self.get("$color")?.trim();
        let scale = if v.starts_with('{') { 255.0 } else { 1.0 };
        let n: Vec<f32> = v.trim_matches(|c| matches!(c, '[' | ']' | '{' | '}')).split_whitespace().filter_map(|p| p.parse().ok()).collect();
        (n.len() >= 3).then(|| [n[0] / scale, n[1] / scale, n[2] / scale])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_parameters_a_material_needs() {
        let text = r#"
// comment
"LightmappedGeneric"
{
	"$basetexture" "Brick\BrickWall001a"
	$bumpmap "brick/brickwall001a_normal"
	"$surfaceprop" "brick"
	"$alphatest" "1"
	"$alphatestreference" ".3"
	"$nocull" 1
	"$color" "{255 128 0}"
	"Proxies" { "AnimatedTexture" { "animatedtexturevar" "$basetexture" } }
}
"#;
        let vmt = parse(text, &mut |_| None).unwrap();
        assert_eq!(vmt.shader, "lightmappedgeneric");
        assert_eq!(vmt.base_texture().as_deref(), Some("brick/brickwall001a"));
        assert_eq!(vmt.normal_map().as_deref(), Some("brick/brickwall001a_normal"));
        assert_eq!(vmt.surface_prop(), Some("brick"));
        assert!(vmt.alpha_test() && vmt.no_cull() && !vmt.translucent() && !vmt.self_illum());
        assert!((vmt.alpha_test_reference() - 0.3).abs() < 1e-6);
        let c = vmt.color().unwrap();
        assert!((c[0] - 1.0).abs() < 1e-6 && (c[1] - 128.0 / 255.0).abs() < 1e-6 && c[2] == 0.0);
    }

    #[test]
    fn water_uses_its_normal_map() {
        let vmt = parse("Water { $bumpmap \"dev/water_dudv\" $normalmap \"nature/water_dx70_normal\" $translucent 0 }", &mut |_| None).unwrap();
        assert!(vmt.water() && !vmt.translucent());
        assert_eq!(vmt.base_texture(), None);
        assert_eq!(vmt.normal_map().as_deref(), Some("nature/water_dx70_normal"));
    }

    #[test]
    fn patch_materials_include_and_override() {
        let base = "UnlitGeneric { \"$basetexture\" \"tools/toolstrigger\" \"$translucent\" \"1\" \"$surfaceprop\" \"default\" }";
        let patch = r#""Patch"
{
	include materials\tools\toolstrigger.vmt
	insert { "$selfillum" "1" }
	replace { "$basetexture" "tools/toolshurt" }
}"#;
        let mut asked = Vec::new();
        let vmt = parse(patch, &mut |p| {
            asked.push(p.to_string());
            Some(base.to_string())
        })
        .unwrap();
        assert_eq!(asked, ["materials\\tools\\toolstrigger.vmt"]);
        assert_eq!(normalize(&asked[0]), "tools/toolstrigger");
        assert!(vmt.unlit() && vmt.translucent() && vmt.self_illum());
        assert_eq!(vmt.base_texture().as_deref(), Some("tools/toolshurt"));
        assert!(matches!(parse(patch, &mut |_| None), Err(VmtError::MissingInclude(_))));
        assert!(matches!(parse(patch, &mut |_| Some(patch.to_string())), Err(VmtError::TooDeep)));
    }
}
