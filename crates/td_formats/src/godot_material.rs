//! Reads the parts of Godot `StandardMaterial3D` / `ORMMaterial3D` text resources that matter for the editor preview and
//! the glTF and OBJ exports.

use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Transparency {
    Opaque,
    Alpha,
    /// Alpha scissor with its threshold.
    Scissor(f32),
    /// Alpha hash: each pixel is cut against a noise threshold, so thin cutouts like wire fences fade out with
    /// distance instead of vanishing once their mipmaps average below a fixed threshold.
    Hash,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GodotMaterial {
    /// Color as written in the resource (sRGB), multiplied with the albedo texture.
    pub albedo_color: [f32; 4],
    pub albedo_texture: Option<String>,
    pub normal_texture: Option<String>,
    pub normal_scale: f32,
    pub emission: Option<[f32; 3]>,
    pub emission_energy: f32,
    pub emission_texture: Option<String>,
    /// `emission_operator = 1`: the emission texture is multiplied with the emission color instead of added to it.
    pub emission_multiply: bool,
    pub transparency: Transparency,
    pub double_sided: bool,
    pub unshaded: bool,
    /// Some(true) for nearest filtering, None when the resource keeps the project default.
    pub nearest: Option<bool>,
    pub roughness: f32,
    pub metallic: f32,
    /// Roughness map and the channel it is read from, 0 to 3 for red to alpha, 4 for grayscale.
    pub roughness_texture: Option<String>,
    pub roughness_channel: u8,
    pub metallic_texture: Option<String>,
    pub metallic_channel: u8,
    /// Ambient occlusion map, only when `ao_enabled` is set.
    pub ao_texture: Option<String>,
    /// Packed occlusion, roughness and metallic map of an `ORMMaterial3D`, in its red, green and blue channels.
    pub orm_texture: Option<String>,
    /// UV1 scale, applied on top of the face projection.
    pub uv_scale: [f32; 2],
    /// World size in map units that one repeat of the texture covers, from `metadata/texture_size`. Face UVs
    /// use it in place of the albedo's pixel size, so a high resolution photo keeps a believable scale.
    pub texture_size: Option<[f32; 2]>,
}

impl Default for GodotMaterial {
    fn default() -> Self {
        Self {
            albedo_color: [1.0; 4],
            albedo_texture: None,
            normal_texture: None,
            normal_scale: 1.0,
            emission: None,
            emission_energy: 1.0,
            emission_texture: None,
            emission_multiply: false,
            transparency: Transparency::Opaque,
            double_sided: false,
            unshaded: false,
            nearest: None,
            roughness: 1.0,
            metallic: 0.0,
            roughness_texture: None,
            roughness_channel: 0,
            metallic_texture: None,
            metallic_channel: 0,
            ao_texture: None,
            orm_texture: None,
            uv_scale: [1.0, 1.0],
            texture_size: None,
        }
    }
}

impl GodotMaterial {
    pub fn is_transparent(&self) -> bool {
        matches!(self.transparency, Transparency::Alpha)
    }

    /// Whether the material glows at all. Godot adds the emission texture to the color by default, so a black color
    /// with a texture still glows, while multiplying needs both.
    pub fn is_emissive(&self) -> bool {
        let Some(c) = self.emission else { return false };
        let lit = c.iter().any(|v| *v > 0.0);
        self.emission_energy > 0.0 && if self.emission_multiply { lit && self.emission_texture.is_some() } else { lit || self.emission_texture.is_some() }
    }
}

fn numbers(s: &str) -> Vec<f32> {
    let inner = s.find('(').and_then(|a| s.rfind(')').map(|b| &s[a + 1..b])).unwrap_or(s);
    inner.split(',').filter_map(|p| p.trim().parse().ok()).collect()
}

fn quoted(s: &str) -> Option<&str> {
    let a = s.find('"')?;
    let b = s[a + 1..].find('"')?;
    Some(&s[a + 1..a + 1 + b])
}

/// A `Vector2(w, h)`, `Vector2i(w, h)` or single number world size, positive and finite only.
fn parse_texture_size(v: &str) -> Option<[f32; 2]> {
    let n = numbers(v);
    let size = match n.as_slice() {
        [s] => [*s, *s],
        [w, h, ..] => [*w, *h],
        _ => return None,
    };
    size.iter().all(|s| s.is_finite() && *s > 0.0).then_some(size)
}

/// Parses a `.tres` or `.material` text resource. Returns None for other resource types.
pub fn parse(text: &str) -> Option<GodotMaterial> {
    let mut resources: HashMap<String, String> = HashMap::new();
    let mut section = String::new();
    let mut is_material = false;
    let mut values: HashMap<String, String> = HashMap::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }

        if line.starts_with('[') {
            section = line.trim_matches(|c| c == '[' || c == ']').split_whitespace().next().unwrap_or_default().to_string();
            match section.as_str() {
                "gd_resource" => {
                    let ty = line.split("type=").nth(1).and_then(quoted).unwrap_or_default();
                    is_material = matches!(ty, "StandardMaterial3D" | "ORMMaterial3D");
                }
                "ext_resource" => {
                    let path = line.split("path=").nth(1).and_then(quoted);
                    let id = line.split(" id=").nth(1).map(|s| s.trim_end_matches(']').trim().trim_matches('"').to_string());
                    if let (Some(path), Some(id)) = (path, id) {
                        resources.insert(id, path.to_string());
                    }
                }
                _ => {}
            }

            continue;
        }

        if section == "resource"
            && let Some((k, v)) = line.split_once('=')
        {
            values.insert(k.trim().to_string(), v.trim().to_string());
        }
    }

    if !is_material {
        return None;
    }

    let texture = |key: &str| -> Option<String> {
        let v = values.get(key)?;
        let inner = v.strip_prefix("ExtResource(")?.trim_end_matches(')').trim().trim_matches('"');
        resources.get(inner).cloned()
    };
    let float = |key: &str| values.get(key).and_then(|v| v.parse::<f32>().ok());
    let flag = |key: &str| values.get(key).map(|v| v == "true" || v == "1");
    let mut m = GodotMaterial { albedo_texture: texture("albedo_texture"), ..Default::default() };
    if let Some(c) = values.get("albedo_color").map(|v| numbers(v)).filter(|c| c.len() >= 3) {
        m.albedo_color = [c[0], c[1], c[2], c.get(3).copied().unwrap_or(1.0)];
    }

    if flag("normal_enabled") != Some(false) {
        m.normal_texture = texture("normal_texture");
    }

    m.normal_scale = float("normal_scale").unwrap_or(1.0);
    if flag("emission_enabled") == Some(true) {
        let c = values.get("emission").map(|v| numbers(v)).filter(|c| c.len() >= 3).unwrap_or(vec![0.0, 0.0, 0.0]);
        m.emission = Some([c[0], c[1], c[2]]);
        m.emission_energy = float("emission_energy_multiplier").or(float("emission_energy")).unwrap_or(1.0);
        m.emission_texture = texture("emission_texture");
        m.emission_multiply = values.get("emission_operator").is_some_and(|v| v == "1");
    }

    m.transparency = match values.get("transparency").map(String::as_str) {
        Some("1") | Some("4") => Transparency::Alpha,
        Some("2") => Transparency::Scissor(float("alpha_scissor_threshold").unwrap_or(0.5)),
        Some("3") => Transparency::Hash,
        _ => Transparency::Opaque,
    };
    m.double_sided = values.get("cull_mode").is_some_and(|v| v == "2");
    m.unshaded = values.get("shading_mode").is_some_and(|v| v == "0");
    m.nearest = values.get("texture_filter").and_then(|v| v.parse::<u32>().ok()).map(|f| f % 2 == 0);
    m.roughness = float("roughness").unwrap_or(1.0);
    m.metallic = float("metallic").unwrap_or(0.0);
    let channel = |key: &str| values.get(key).and_then(|v| v.parse::<u8>().ok()).filter(|c| *c <= 4).unwrap_or(0);
    m.roughness_texture = texture("roughness_texture");
    m.roughness_channel = channel("roughness_texture_channel");
    m.metallic_texture = texture("metallic_texture");
    m.metallic_channel = channel("metallic_texture_channel");
    if flag("ao_enabled") == Some(true) {
        m.ao_texture = texture("ao_texture");
    }

    m.orm_texture = texture("orm_texture");
    if let Some(s) = values.get("uv1_scale").map(|v| numbers(v)).filter(|s| s.len() >= 2) {
        m.uv_scale = [s[0], s[1]];
    }

    m.texture_size = values.get("metadata/texture_size").and_then(|v| parse_texture_size(v));
    Some(m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_transparency_emission_and_filter() {
        let glass = parse(
            "[gd_resource type=\"StandardMaterial3D\" load_steps=2 format=3]\n\n[ext_resource type=\"Texture2D\" path=\"res://t/glass.png\" id=\"1_tex\"]\n\n[resource]\nalbedo_texture = ExtResource(\"1_tex\")\ntexture_filter = 2\ntransparency = 1\nalbedo_color = Color(1, 1, 1, 0.35)\n",
        )
        .unwrap();
        assert_eq!(glass.albedo_texture.as_deref(), Some("res://t/glass.png"));
        assert_eq!(glass.transparency, Transparency::Alpha);
        assert!(glass.is_transparent());
        assert_eq!(glass.nearest, Some(true));
        assert!((glass.albedo_color[3] - 0.35).abs() < 1e-6);

        let lamp = parse(
            "[gd_resource type=\"StandardMaterial3D\" format=3]\n[ext_resource type=\"Texture2D\" path=\"res://lamp.png\" id=\"2\"]\n[ext_resource type=\"Texture2D\" path=\"res://lamp_n.png\" id=\"3\"]\n[resource]\nemission_enabled = true\nemission = Color(1, 0.9, 0.6, 1)\nemission_energy_multiplier = 3.0\nnormal_enabled = true\nnormal_texture = ExtResource(\"3\")\ncull_mode = 2\ntransparency = 2\nalpha_scissor_threshold = 0.4\ntexture_filter = 3\n",
        )
        .unwrap();
        assert_eq!(lamp.emission, Some([1.0, 0.9, 0.6]));
        assert_eq!(lamp.emission_energy, 3.0);
        assert_eq!(lamp.normal_texture.as_deref(), Some("res://lamp_n.png"));
        assert!(lamp.double_sided);
        assert_eq!(lamp.transparency, Transparency::Scissor(0.4));
        assert_eq!(lamp.nearest, Some(false));
        assert!(!lamp.is_transparent());
        assert!(lamp.is_emissive() && !lamp.emission_multiply);
        assert!(!glass.is_emissive());

        assert!(parse("[gd_resource type=\"ShaderMaterial\" format=3]\n[resource]\n").is_none());
        let fence = parse("[gd_resource type=\"StandardMaterial3D\" format=3]\n[resource]\ntransparency = 3\n").unwrap();
        assert_eq!(fence.transparency, Transparency::Hash);
        assert!(!fence.is_transparent(), "hashed pixels are opaque or cut, never blended");
    }

    #[test]
    fn reads_the_texture_size_override() {
        let text = |value: &str| format!("[gd_resource type=\"StandardMaterial3D\" format=3]\n[resource]\nmetadata/texture_size = {value}\n");
        assert_eq!(parse(&text("Vector2(128, 64)")).unwrap().texture_size, Some([128.0, 64.0]));
        assert_eq!(parse(&text("Vector2i(96, 96)")).unwrap().texture_size, Some([96.0, 96.0]));
        assert_eq!(parse(&text("48")).unwrap().texture_size, Some([48.0, 48.0]));
        assert_eq!(parse(&text("Vector2(0, 64)")).unwrap().texture_size, None, "a zero size would divide by zero");
        assert_eq!(parse(&text("Vector2(-8, 64)")).unwrap().texture_size, None);
        assert_eq!(parse("[gd_resource type=\"StandardMaterial3D\" format=3]\n[resource]\n").unwrap().texture_size, None);
    }

    #[test]
    fn reads_the_emission_texture_and_operator() {
        let text = |body: &str| {
            format!(
                "[gd_resource type=\"StandardMaterial3D\" format=3]\n[ext_resource type=\"Texture2D\" path=\"res://w.png\" id=\"1\"]\n[ext_resource type=\"Texture2D\" path=\"res://w_emission.png\" id=\"2\"]\n[resource]\n{body}"
            )
        };
        let added = parse(&text("emission_enabled = true\nemission_texture = ExtResource(\"2\")\nemission_energy_multiplier = 2.5\n")).unwrap();
        assert_eq!(added.emission, Some([0.0, 0.0, 0.0]), "Godot's default emission color is black");
        assert_eq!(added.emission_texture.as_deref(), Some("res://w_emission.png"));
        assert!(!added.emission_multiply);
        assert!(added.is_emissive(), "a black color plus a texture still glows with the add operator");

        let multiplied =
            parse(&text("emission_enabled = true\nemission = Color(1, 0.8, 0.5, 1)\nemission_operator = 1\nemission_texture = ExtResource(\"2\")\n")).unwrap();
        assert!(multiplied.emission_multiply && multiplied.is_emissive());
        let black_multiplied = parse(&text("emission_enabled = true\nemission_operator = 1\nemission_texture = ExtResource(\"2\")\n")).unwrap();
        assert!(!black_multiplied.is_emissive(), "black times anything is dark");

        let disabled = parse(&text("emission = Color(1, 1, 1, 1)\nemission_texture = ExtResource(\"2\")\n")).unwrap();
        assert_eq!(disabled.emission, None, "emission values without emission_enabled are ignored, like in Godot");
        assert!(!disabled.is_emissive());
        let zero_energy = parse(&text("emission_enabled = true\nemission = Color(1, 1, 1, 1)\nemission_energy_multiplier = 0.0\n")).unwrap();
        assert!(!zero_energy.is_emissive());
    }

    #[test]
    fn reads_the_pbr_maps() {
        let metal = parse(
            "[gd_resource type=\"StandardMaterial3D\" format=3]\n[ext_resource type=\"Texture2D\" path=\"res://m.png\" id=\"1\"]\n[ext_resource type=\"Texture2D\" path=\"res://m_roughness.jpg\" id=\"2\"]\n[ext_resource type=\"Texture2D\" path=\"res://m_ao.png\" id=\"3\"]\n[resource]\nalbedo_texture = ExtResource(\"1\")\nroughness_texture = ExtResource(\"2\")\nroughness_texture_channel = 1\nmetallic = 0.8\nao_texture = ExtResource(\"3\")\n",
        )
        .unwrap();
        assert_eq!(metal.roughness_texture.as_deref(), Some("res://m_roughness.jpg"));
        assert_eq!(metal.roughness_channel, 1);
        assert_eq!(metal.metallic_texture, None);
        assert_eq!(metal.ao_texture, None, "an occlusion map without ao_enabled is ignored, like in Godot");
        let orm = parse(
            "[gd_resource type=\"ORMMaterial3D\" format=3]\n[ext_resource type=\"Texture2D\" path=\"res://p_orm.png\" id=\"1\"]\n[resource]\norm_texture = ExtResource(\"1\")\n",
        )
        .unwrap();
        assert_eq!(orm.orm_texture.as_deref(), Some("res://p_orm.png"));
    }
}
