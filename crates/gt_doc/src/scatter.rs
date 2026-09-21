//! Scatter sets: many model instances (trees, rocks, foliage) painted onto target surfaces and stored as one node.

use std::collections::HashMap;

use gt_core::{Aabb, DMat4, DQuat, DVec3, NodeId};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScatterKind {
    /// Scene instances that keep their scripts and collision: trees, rocks, props.
    #[default]
    Props,
    /// MultiMesh instances without collision: grass, flowers, small plants.
    Foliage,
}

impl ScatterKind {
    pub const ALL: [ScatterKind; 2] = [ScatterKind::Props, ScatterKind::Foliage];

    pub fn label(&self) -> &'static str {
        match self {
            ScatterKind::Props => "props",
            ScatterKind::Foliage => "foliage",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScatterCollision {
    None,
    #[default]
    Convex,
    Trimesh,
}

impl ScatterCollision {
    pub const ALL: [ScatterCollision; 3] = [ScatterCollision::None, ScatterCollision::Convex, ScatterCollision::Trimesh];

    pub fn label(&self) -> &'static str {
        match self {
            ScatterCollision::None => "none",
            ScatterCollision::Convex => "convex",
            ScatterCollision::Trimesh => "trimesh",
        }
    }
}

fn one() -> f64 {
    1.0
}

fn yes() -> bool {
    true
}

fn default_scale() -> [f64; 2] {
    [0.8, 1.2]
}

fn default_spacing() -> f64 {
    48.0
}

/// Cell new sets are chunked into: 64 m at the default 32 units per meter, small enough that the camera
/// usually sees a handful of cells and large enough that the draw call count stays low.
pub const DEFAULT_CHUNK_SIZE: f64 = 2048.0;

/// One entry of a scatter palette.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScatterItem {
    /// res:// model or scene (.bbmodel, .glb, .gltf, .tscn).
    pub source: String,
    /// Relative chance of being picked.
    #[serde(default = "one")]
    pub weight: f64,
    #[serde(default = "default_scale")]
    pub scale: [f64; 2],
    /// Spread: minimum distance in map units to every other instance.
    #[serde(default = "default_spacing")]
    pub spacing: f64,
    /// 0 keeps instances upright, 1 aligns them to the surface normal.
    #[serde(default)]
    pub align: f64,
    #[serde(default = "yes")]
    pub random_yaw: bool,
    /// Largest random lean in degrees.
    #[serde(default)]
    pub tilt: f64,
    /// Map units pushed into the surface, so trunks do not float on slopes.
    #[serde(default)]
    pub sink: f64,
    /// Material drawn instead of the ones the model ships with. Wins over the set's own override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material: Option<String>,
}

impl ScatterItem {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            weight: 1.0,
            scale: default_scale(),
            spacing: default_spacing(),
            align: 0.0,
            random_yaw: true,
            tilt: 0.0,
            sink: 0.0,
            material: None,
        }
    }

    /// File name without extension, for labels.
    pub fn label(&self) -> &str {
        let file = self.source.rsplit(['/', '\\']).next().unwrap_or(&self.source);
        file.split('.').next().unwrap_or(file)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScatterInstance {
    /// Index into the set's items.
    pub item: u32,
    pub position: DVec3,
    /// Degrees, pitch yaw roll in Godot's YXZ order like entity angles.
    pub angles: DVec3,
    pub scale: f64,
}

impl ScatterInstance {
    pub fn rotation(&self) -> DQuat {
        DQuat::from_euler(gt_core::EulerRot::YXZ, self.angles.y.to_radians(), self.angles.x.to_radians(), self.angles.z.to_radians())
    }

    pub fn transform(&self) -> DMat4 {
        DMat4::from_scale_rotation_translation(DVec3::splat(self.scale), self.rotation(), self.position)
    }
}

/// Stored as one compact array per instance so large sets stay small and diffable.
impl Serialize for ScatterInstance {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let round = |v: f64, k: f64| {
            let r = (v * k).round() / k;
            if r == 0.0 { 0.0 } else { r }
        };
        let p = self.position;
        let a = self.angles;
        [
            self.item as f64,
            round(p.x, 100.0),
            round(p.y, 100.0),
            round(p.z, 100.0),
            round(a.x, 10.0),
            round(a.y, 10.0),
            round(a.z, 10.0),
            round(self.scale, 1000.0),
        ]
        .serialize(s)
    }
}

impl<'de> Deserialize<'de> for ScatterInstance {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v: Vec<f64> = Vec::deserialize(d)?;
        if v.len() < 8 {
            return Err(serde::de::Error::custom("scatter instance needs [item, x, y, z, pitch, yaw, roll, scale]"));
        }

        Ok(ScatterInstance { item: v[0].max(0.0) as u32, position: DVec3::new(v[1], v[2], v[3]), angles: DVec3::new(v[4], v[5], v[6]), scale: v[7] })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scatter {
    pub name: String,
    #[serde(default)]
    pub kind: ScatterKind,
    /// Surfaces this set is painted on. Painting and erasing only touch these when the tool restricts to targets.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<NodeId>,
    pub items: Vec<ScatterItem>,
    #[serde(default)]
    pub collision: ScatterCollision,
    #[serde(default = "yes")]
    pub cast_shadows: bool,
    /// Map units beyond which instances are hidden in Godot, 0 shows them at any distance.
    #[serde(default)]
    pub visibility_range: f64,
    /// Grid cell in map units the instances are split into, so Godot frustum culls each cell on its own.
    /// 0 keeps one MultiMesh for the whole set.
    #[serde(default)]
    pub chunk_size: f64,
    /// Render prop scenes that carry scripts as MultiMesh visuals too, dropping their scripts. Off keeps them
    /// as one node each so their behaviour survives.
    #[serde(default)]
    pub static_props_multimesh: bool,
    /// Material drawn instead of the ones the models ship with, for the whole set. A palette entry's own
    /// material wins over it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material: Option<String>,
    #[serde(default)]
    pub instances: Vec<ScatterInstance>,
}

impl Scatter {
    pub fn new(name: impl Into<String>, kind: ScatterKind, items: Vec<ScatterItem>) -> Self {
        let collision = if kind == ScatterKind::Foliage { ScatterCollision::None } else { ScatterCollision::Convex };
        Self {
            name: name.into(),
            kind,
            targets: Vec::new(),
            items,
            collision,
            cast_shadows: kind == ScatterKind::Props,
            visibility_range: if kind == ScatterKind::Foliage { 2400.0 } else { 0.0 },
            chunk_size: DEFAULT_CHUNK_SIZE,
            static_props_multimesh: false,
            material: None,
            instances: Vec::new(),
        }
    }

    /// Material a palette entry draws with: its own override, else the set's, else the model's own materials.
    pub fn item_material(&self, item: usize) -> Option<&str> {
        self.items.get(item).and_then(|i| i.material.as_deref()).or(self.material.as_deref())
    }

    /// Instance indices grouped by `chunk_size` cell, so each cell becomes its own MultiMesh that Godot can
    /// frustum cull. One group holding everything when chunking is off. Groups are ordered so a set always
    /// builds the same way.
    pub fn chunks(&self) -> Vec<((i64, i64, i64), Vec<usize>)> {
        if self.chunk_size <= 0.0 || self.instances.is_empty() {
            return vec![((0, 0, 0), (0..self.instances.len()).collect())];
        }

        let cell = self.chunk_size;
        let mut groups: HashMap<(i64, i64, i64), Vec<usize>> = HashMap::new();
        for (k, i) in self.instances.iter().enumerate() {
            let p = i.position;
            let key = ((p.x / cell).floor() as i64, (p.y / cell).floor() as i64, (p.z / cell).floor() as i64);
            groups.entry(key).or_default().push(k);
        }

        let mut out: Vec<((i64, i64, i64), Vec<usize>)> = groups.into_iter().collect();
        out.sort_by_key(|(key, _)| *key);
        out
    }

    pub fn bounds(&self) -> Aabb {
        let mut b = Aabb::EMPTY;
        for i in &self.instances {
            b.include(&Aabb::from_center_size(i.position + DVec3::Y * 16.0 * i.scale, DVec3::splat(32.0 * i.scale.max(0.1))));
        }

        b
    }

    pub fn counts(&self) -> Vec<usize> {
        let mut out = vec![0; self.items.len()];
        for i in &self.instances {
            if let Some(c) = out.get_mut(i.item as usize) {
                *c += 1;
            }
        }

        out
    }

    /// Moves, rotates and scales every instance.
    pub fn transform(&mut self, m: &DMat4) {
        let (scale, rot, _) = m.to_scale_rotation_translation();
        let uniform = (scale.x.abs() * scale.y.abs() * scale.z.abs()).cbrt();
        for inst in &mut self.instances {
            inst.position = m.transform_point3(inst.position);
            let (y, x, z) = (rot * inst.rotation()).normalize().to_euler(gt_core::EulerRot::YXZ);
            inst.angles = DVec3::new(x.to_degrees(), y.to_degrees(), z.to_degrees());
            inst.scale *= uniform;
        }

        for item in &mut self.items {
            item.spacing *= uniform;
            item.sink *= uniform;
        }
    }

    /// Removes a palette entry together with its instances.
    pub fn remove_item(&mut self, index: usize) {
        if index >= self.items.len() {
            return;
        }

        self.items.remove(index);
        self.instances.retain(|i| i.item as usize != index);
        for i in &mut self.instances {
            if i.item as usize > index {
                i.item -= 1;
            }
        }
    }

    /// Adds palette entries that are not present yet. Returns the index of every source.
    pub fn merge_items(&mut self, items: &[ScatterItem]) -> Vec<usize> {
        items
            .iter()
            .map(|item| match self.items.iter().position(|i| i.source == item.source) {
                Some(k) => {
                    self.items[k] = item.clone();
                    k
                }
                None => {
                    self.items.push(item.clone());
                    self.items.len() - 1
                }
            })
            .collect()
    }

    pub fn pick_item(&self, rng: &mut Rng, allowed: Option<&[usize]>) -> Option<usize> {
        let candidates: Vec<usize> = (0..self.items.len()).filter(|k| allowed.is_none_or(|a| a.contains(k)) && self.items[*k].weight > 0.0).collect();
        let total: f64 = candidates.iter().map(|k| self.items[*k].weight).sum();
        if total <= 0.0 {
            return None;
        }

        let mut roll = rng.next_f64() * total;
        for k in &candidates {
            roll -= self.items[*k].weight;
            if roll <= 0.0 {
                return Some(*k);
            }
        }

        candidates.last().copied()
    }

    /// Palette entries with their share of `total` samples, widest spacing first. Placing big items first leaves them
    /// room, otherwise densely packed small items starve items that need more space.
    fn passes(&self, allowed: Option<&[usize]>, total: usize) -> Vec<(usize, usize)> {
        let mut items: Vec<usize> = (0..self.items.len()).filter(|k| allowed.is_none_or(|a| a.contains(k)) && self.items[*k].weight > 0.0).collect();
        let weight: f64 = items.iter().map(|k| self.items[*k].weight).sum();
        items.sort_by(|a, b| self.items[*b].spacing.total_cmp(&self.items[*a].spacing));
        items.into_iter().map(|k| (k, ((total as f64) * self.items[k].weight / weight.max(1e-9)).ceil() as usize)).collect()
    }

    fn largest_spacing(&self) -> f64 {
        self.items.iter().map(|i| i.spacing).fold(1.0, f64::max)
    }
}

/// Deterministic xorshift generator, so scripted scatters repeat exactly.
#[derive(Clone, Copy, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    pub fn next_f64(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.next_f64()
    }
}

/// First surface below a scatter sample.
#[derive(Clone, Copy, Debug)]
pub struct SurfaceHit {
    pub point: DVec3,
    pub normal: DVec3,
    pub node: NodeId,
}

/// Placement rules shared by painting and filling.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ScatterRules {
    /// Attempts per 64 x 64 unit area. Spacing decides how many succeed.
    pub density: f64,
    /// Allowed surface slope in degrees.
    pub slope: [f64; 2],
    /// Allowed world height, unrestricted when None.
    pub height: Option<[f64; 2]>,
    /// Only place on the set's target surfaces.
    pub only_targets: bool,
    /// Palette entries to use, every entry when None.
    pub items: Option<Vec<usize>>,
    /// 0 fills the brush evenly, 1 thins instances out towards the rim.
    pub falloff: f64,
}

impl Default for ScatterRules {
    fn default() -> Self {
        Self { density: 1.0, slope: [0.0, 40.0], height: None, only_targets: true, items: None, falloff: 0.3 }
    }
}

/// Hash grid of placed instances for spacing checks.
pub struct SpacingGrid {
    cell: f64,
    reach: i64,
    cells: HashMap<(i64, i64, i64), Vec<(DVec3, f64)>>,
}

impl SpacingGrid {
    pub fn new(largest_spacing: f64) -> Self {
        let cell = largest_spacing.max(1.0);
        Self { cell, reach: 1, cells: HashMap::new() }
    }

    fn key(&self, p: DVec3) -> (i64, i64, i64) {
        ((p.x / self.cell).floor() as i64, (p.y / self.cell).floor() as i64, (p.z / self.cell).floor() as i64)
    }

    pub fn insert(&mut self, p: DVec3, spacing: f64) {
        let k = self.key(p);
        self.cells.entry(k).or_default().push((p, spacing));
    }

    /// True when another point is closer than the larger of both spacings.
    pub fn blocked(&self, p: DVec3, spacing: f64) -> bool {
        let (x, y, z) = self.key(p);
        let r = self.reach;
        for dx in -r..=r {
            for dy in -r..=r {
                for dz in -r..=r {
                    if let Some(list) = self.cells.get(&(x + dx, y + dy, z + dz))
                        && list.iter().any(|(q, s)| (*q - p).length() < spacing.max(*s))
                    {
                        return true;
                    }
                }
            }
        }

        false
    }
}

fn slope_ok(normal: DVec3, rules: &ScatterRules) -> bool {
    let slope = normal.normalize_or(DVec3::Y).y.clamp(-1.0, 1.0).acos().to_degrees();
    slope >= rules.slope[0] - 1e-6 && slope <= rules.slope[1] + 1e-6
}

fn height_ok(p: DVec3, rules: &ScatterRules) -> bool {
    rules.height.is_none_or(|[lo, hi]| p.y >= lo && p.y <= hi)
}

impl Scatter {
    fn spacing_grid(&self, others: &[(DVec3, f64)]) -> SpacingGrid {
        let largest = others.iter().map(|o| o.1).fold(self.largest_spacing(), f64::max);
        let mut grid = SpacingGrid::new(largest);
        for i in &self.instances {
            let spacing = self.items.get(i.item as usize).map(|it| it.spacing).unwrap_or(0.0);
            grid.insert(i.position, spacing);
        }

        for (p, s) in others {
            grid.insert(*p, *s);
        }

        grid
    }

    fn accepts(&self, hit: &SurfaceHit, rules: &ScatterRules) -> bool {
        (!rules.only_targets || self.targets.is_empty() || self.targets.contains(&hit.node)) && slope_ok(hit.normal, rules) && height_ok(hit.point, rules)
    }

    fn make_instance(&self, item: usize, hit: &SurfaceHit, rng: &mut Rng) -> ScatterInstance {
        let it = &self.items[item];
        let yaw = if it.random_yaw { rng.range(0.0, 360.0) } else { 0.0 };
        let n = hit.normal.normalize_or(DVec3::Y);
        let up = DVec3::Y.lerp(n, it.align.clamp(0.0, 1.0)).normalize_or(DVec3::Y);
        let lean_dir = rng.range(0.0, std::f64::consts::TAU);
        let lean = DQuat::from_axis_angle(DVec3::new(lean_dir.cos(), 0.0, lean_dir.sin()), rng.range(0.0, it.tilt.max(0.0)).to_radians());
        let q = DQuat::from_rotation_arc(DVec3::Y, up) * lean * DQuat::from_rotation_y(yaw.to_radians());
        let (y, x, z) = q.normalize().to_euler(gt_core::EulerRot::YXZ);
        let scale = rng.range(it.scale[0].min(it.scale[1]), it.scale[0].max(it.scale[1])).max(0.01);
        ScatterInstance {
            item: item as u32,
            position: hit.point - up * it.sink * scale,
            angles: DVec3::new(x.to_degrees(), y.to_degrees(), z.to_degrees()),
            scale,
        }
    }

    /// One brush dab of `radius` around `center` on the plane of `normal`. `cast(origin, dir)` returns the first surface
    /// along a ray. `others` are instances of other sets to keep apart from. Returns how many instances were added.
    #[allow(clippy::too_many_arguments)]
    pub fn paint(
        &mut self,
        center: DVec3,
        normal: DVec3,
        radius: f64,
        rules: &ScatterRules,
        rng: &mut Rng,
        others: &[(DVec3, f64)],
        cast: impl Fn(DVec3, DVec3) -> Option<SurfaceHit>,
    ) -> usize {
        if self.items.is_empty() || radius <= 0.0 {
            return 0;
        }

        let normal = normal.normalize_or(DVec3::Y);
        let area = std::f64::consts::PI * radius * radius;
        let count = ((area / 4096.0) * rules.density).round().max(1.0) as usize;
        let mut grid = self.spacing_grid(others);
        let basis = gt_core::Plane::from_point_normal(center, normal).basis();
        let mut placed = 0;
        for (item, share) in self.passes(rules.items.as_deref(), count) {
            let mut placed_item = 0;
            for _ in 0..share * 4 {
                if placed_item >= share {
                    break;
                }

                let angle = rng.range(0.0, std::f64::consts::TAU);
                let t = rng.next_f64().sqrt();
                if rules.falloff > 0.0 && rng.next_f64() > 1.0 - rules.falloff.clamp(0.0, 1.0) * t * t {
                    continue;
                }

                let offset = (basis.0 * angle.cos() + basis.1 * angle.sin()) * radius * t;
                let Some(hit) = cast(center + offset + normal * radius, -normal) else { continue };
                // A tilted or edge normal makes the cast skate off the brush and land far away, so drop hits that
                // fall outside the disc (or came back non-finite) rather than flinging an instance to a random spot.
                if !hit.point.is_finite() || (hit.point - center).reject_from(normal).length() > radius * 1.5 {
                    continue;
                }

                let spacing = self.items[item].spacing;
                if !self.accepts(&hit, rules) || grid.blocked(hit.point, spacing) {
                    continue;
                }

                grid.insert(hit.point, spacing);
                let inst = self.make_instance(item, &hit, rng);
                self.instances.push(inst);
                placed_item += 1;
            }

            placed += placed_item;
        }

        placed
    }

    /// Fills the XZ area of `bounds` from above with jittered samples. Returns how many instances were added.
    pub fn fill(
        &mut self,
        bounds: &Aabb,
        rules: &ScatterRules,
        rng: &mut Rng,
        others: &[(DVec3, f64)],
        cast: impl Fn(DVec3, DVec3) -> Option<SurfaceHit>,
    ) -> usize {
        if self.items.is_empty() || bounds.is_empty() || rules.density <= 0.0 {
            return 0;
        }

        let size = bounds.size();
        let samples = ((size.x.max(1.0) * size.z.max(1.0) / 4096.0) * rules.density).ceil() as usize;
        if samples > 4_000_000 {
            return 0;
        }

        let mut grid = self.spacing_grid(others);
        let top = bounds.max.y + 64.0;
        let mut placed = 0;
        for (item, share) in self.passes(rules.items.as_deref(), samples) {
            let spacing = self.items[item].spacing;
            for _ in 0..share {
                let x = bounds.min.x + rng.next_f64() * size.x;
                let z = bounds.min.z + rng.next_f64() * size.z;
                let Some(hit) = cast(DVec3::new(x, top, z), DVec3::NEG_Y) else { continue };
                if !hit.point.is_finite() || !self.accepts(&hit, rules) || grid.blocked(hit.point, spacing) {
                    continue;
                }

                grid.insert(hit.point, spacing);
                let inst = self.make_instance(item, &hit, rng);
                self.instances.push(inst);
                placed += 1;
            }
        }

        placed
    }

    /// Removes instances within `radius` of `center`, optionally only some palette entries and only a fraction.
    pub fn erase(&mut self, center: DVec3, radius: f64, items: Option<&[usize]>, amount: f64, rng: &mut Rng) -> usize {
        let before = self.instances.len();
        let amount = amount.clamp(0.0, 1.0);
        self.instances.retain(|i| {
            let inside = (i.position - center).length() <= radius && items.is_none_or(|a| a.contains(&(i.item as usize)));
            !(inside && (amount >= 1.0 || rng.next_f64() < amount))
        });
        before - self.instances.len()
    }

    /// Positions and spacings of every instance, for keeping other sets apart.
    pub fn footprints(&self) -> Vec<(DVec3, f64)> {
        self.instances.iter().map(|i| (i.position, self.items.get(i.item as usize).map(|it| it.spacing).unwrap_or(0.0))).collect()
    }
}

/// Built-in palettes. Sources point at the nature models the editor installs into `res://godottrench/nature/`.
pub fn preset(name: &str) -> Option<(ScatterKind, Vec<ScatterItem>)> {
    let item = |file: &str, weight: f64, scale: [f64; 2], spacing: f64, align: f64, tilt: f64, sink: f64| ScatterItem {
        source: format!("{NATURE_DIR}/{file}.bbmodel"),
        weight,
        scale,
        spacing,
        align,
        random_yaw: true,
        tilt,
        sink,
        material: None,
    };
    // The procedural rock and tree packs ship as glTF in their own subfolders.
    let glb = |file: &str, weight: f64, scale: [f64; 2], spacing: f64, align: f64, tilt: f64, sink: f64| ScatterItem {
        source: format!("{NATURE_DIR}/{file}.glb"),
        weight,
        scale,
        spacing,
        align,
        random_yaw: true,
        tilt,
        sink,
        material: None,
    };
    Some(match name {
        // The procedural glTF trees are authored larger than the map scale, so they sit around 0.5 (see the pack readme).
        "forest" => (
            ScatterKind::Props,
            vec![
                glb("trees/pine", 3.0, [0.45, 0.7], 120.0, 0.0, 3.0, 6.0),
                glb("trees/oak", 2.0, [0.5, 0.7], 150.0, 0.0, 2.0, 4.0),
                glb("trees/birch", 1.0, [0.45, 0.65], 100.0, 0.0, 4.0, 4.0),
                glb("trees/beech", 1.0, [0.45, 0.7], 130.0, 0.0, 2.0, 4.0),
            ],
        ),
        "pines" => (ScatterKind::Props, vec![glb("trees/pine", 1.0, [0.4, 0.75], 110.0, 0.0, 3.0, 6.0)]),
        // The original Blockbench trees, kept as a low poly option.
        "low-poly trees" => (
            ScatterKind::Props,
            vec![
                item("pine", 3.0, [0.8, 1.35], 120.0, 0.0, 3.0, 6.0),
                item("oak", 2.0, [0.85, 1.3], 150.0, 0.0, 2.0, 4.0),
                item("birch", 1.0, [0.8, 1.2], 100.0, 0.0, 4.0, 4.0),
            ],
        ),
        "undergrowth" => (ScatterKind::Props, vec![item("bush", 3.0, [0.7, 1.4], 60.0, 0.4, 0.0, 2.0), item("fern", 2.0, [0.8, 1.3], 40.0, 0.7, 6.0, 1.0)]),
        // The two Blockbench stones (the boulder is mossy) split into a small and a large size set.
        "rocks" => (ScatterKind::Props, vec![item("rock", 3.0, [0.8, 1.8], 80.0, 0.8, 12.0, 5.0), item("boulder", 1.0, [0.8, 1.3], 130.0, 0.6, 8.0, 8.0)]),
        "boulders" => (ScatterKind::Props, vec![item("boulder", 3.0, [1.6, 2.8], 240.0, 0.5, 8.0, 14.0), item("rock", 1.0, [1.4, 2.2], 150.0, 0.7, 10.0, 8.0)]),
        "grass" => (
            ScatterKind::Foliage,
            vec![
                item("grass", 6.0, [0.7, 1.3], 14.0, 0.8, 8.0, 0.5),
                item("flowers", 1.0, [0.8, 1.2], 24.0, 0.8, 6.0, 0.5),
                item("fern", 1.0, [0.5, 0.9], 30.0, 0.8, 6.0, 0.5),
            ],
        ),
        _ => return None,
    })
}

pub const NATURE_DIR: &str = "res://godottrench/nature";
pub const PRESETS: [&str; 7] = ["forest", "pines", "low-poly trees", "undergrowth", "rocks", "boulders", "grass"];

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(node: NodeId, y: f64) -> impl Fn(DVec3, DVec3) -> Option<SurfaceHit> {
        move |origin: DVec3, dir: DVec3| {
            if dir.y >= 0.0 {
                return None;
            }

            let t = (origin.y - y) / -dir.y;
            (t >= 0.0).then(|| SurfaceHit { point: origin + dir * t, normal: DVec3::Y, node })
        }
    }

    #[test]
    fn paint_respects_spacing_targets_and_weights() {
        let (_, items) = preset("forest").unwrap();
        let mut set = Scatter::new("trees", ScatterKind::Props, items);
        set.targets = vec![NodeId(7)];
        let mut rng = Rng::new(3);
        let rules = ScatterRules { density: 4.0, falloff: 0.0, ..Default::default() };
        let placed = set.paint(DVec3::ZERO, DVec3::Y, 900.0, &rules, &mut rng, &[], flat(NodeId(7), 10.0));
        assert!(placed > 30, "placed {placed}");
        for (k, a) in set.instances.iter().enumerate() {
            assert!((a.position.y - 10.0 + set.items[a.item as usize].sink * a.scale).abs() < 1e-6);
            for b in &set.instances[k + 1..] {
                let s = set.items[a.item as usize].spacing.max(set.items[b.item as usize].spacing);
                assert!((a.position - b.position).length() >= s - 1e-6);
            }
        }

        let counts = set.counts();
        assert!(counts[0] > counts[2], "pines weigh three times the birches: {counts:?}");
        // A surface that is not a target gets nothing.
        let before = set.instances.len();
        assert_eq!(set.paint(DVec3::new(5000.0, 0.0, 0.0), DVec3::Y, 400.0, &rules, &mut rng, &[], flat(NodeId(8), 0.0)), 0);
        assert_eq!(set.instances.len(), before);
    }

    #[test]
    fn erase_only_touches_chosen_items_and_round_trips() {
        let (_, items) = preset("rocks").unwrap();
        let mut set = Scatter::new("rocks", ScatterKind::Props, items);
        let mut rng = Rng::new(9);
        let rules = ScatterRules { density: 6.0, slope: [0.0, 90.0], ..Default::default() };
        set.fill(&Aabb::new(DVec3::new(-1000.0, 0.0, -1000.0), DVec3::new(1000.0, 0.0, 1000.0)), &rules, &mut rng, &[], flat(NodeId(1), 0.0));
        let boulders = set.counts()[1];
        assert!(set.counts()[0] > 0 && boulders > 0, "{:?}", set.counts());
        let removed = set.erase(DVec3::ZERO, 5000.0, Some(&[0]), 1.0, &mut rng);
        assert!(removed > 0);
        assert_eq!(set.counts(), vec![0, boulders]);

        let text = serde_json::to_string(&set).unwrap();
        let back: Scatter = serde_json::from_str(&text).unwrap();
        assert_eq!(back.instances.len(), set.instances.len());
        assert!((back.instances[0].position - set.instances[0].position).length() < 0.01);
    }

    #[test]
    fn default_tree_presets_are_procedural_glb() {
        for name in ["forest", "pines"] {
            let (_, items) = preset(name).unwrap();
            assert!(items.iter().all(|i| i.source.ends_with(".glb") && i.source.contains("/trees/")), "{name}: {items:?}");
        }

        let (_, low) = preset("low-poly trees").unwrap();
        assert!(low.iter().all(|i| i.source.ends_with(".bbmodel")), "{low:?}");
        assert!(PRESETS.contains(&"low-poly trees"));
    }

    #[test]
    fn rocks_and_boulders_split_the_bbmodel_stones_by_size() {
        let (_, rocks) = preset("rocks").unwrap();
        let (_, boulders) = preset("boulders").unwrap();
        assert!(rocks.iter().chain(&boulders).all(|i| i.source.ends_with(".bbmodel")), "stones stay Blockbench");
        let biggest = |v: &[ScatterItem]| v.iter().map(|i| i.scale[1]).fold(0.0f64, f64::max);
        assert!(biggest(&boulders) > biggest(&rocks), "boulders are the larger size set");
    }

    #[test]
    fn paint_keeps_instances_inside_the_brush_disc() {
        // A narrow ledge under the brush: samples that clear it skim on and strike a wall far to the side. Without a
        // footprint guard those instances teleport to the wall; with it every placed instance stays within the disc.
        let ledge = flat(NodeId(1), 0.0);
        let cast = |origin: DVec3, dir: DVec3| match ledge(origin, dir) {
            Some(h) if h.point.length() <= 40.0 => Some(h),
            _ => Some(SurfaceHit { point: DVec3::new(600.0, 0.0, 600.0), normal: DVec3::Y, node: NodeId(2) }),
        };
        let mut set = Scatter::new("s", ScatterKind::Props, vec![ScatterItem { spacing: 8.0, ..ScatterItem::new("res://a.glb") }]);
        let mut rng = Rng::new(4);
        let radius = 200.0;
        let center = DVec3::ZERO;
        let rules = ScatterRules { density: 40.0, slope: [0.0, 90.0], only_targets: false, falloff: 0.0, ..Default::default() };
        let placed = set.paint(center, DVec3::Y, radius, &rules, &mut rng, &[], cast);
        assert!(placed > 0, "instances on the ledge still land");
        for i in &set.instances {
            let lateral = (i.position - center).reject_from(DVec3::Y).length();
            assert!(lateral <= radius * 1.5 + 1e-6, "instance flung to {:?}", i.position);
        }
    }

    #[test]
    fn chunks_partition_every_instance_by_cell() {
        let mut set = Scatter::new("s", ScatterKind::Props, vec![ScatterItem::new("res://a.glb")]);
        set.chunk_size = 100.0;
        for p in [DVec3::ZERO, DVec3::new(10.0, 0.0, 10.0), DVec3::new(150.0, 0.0, 0.0), DVec3::new(0.0, 0.0, -250.0)] {
            set.instances.push(ScatterInstance { item: 0, position: p, angles: DVec3::ZERO, scale: 1.0 });
        }

        let chunks = set.chunks();
        assert_eq!(chunks.len(), 3, "the two instances in the same cell share a chunk: {chunks:?}");
        let mut seen: Vec<usize> = chunks.iter().flat_map(|(_, list)| list.iter().copied()).collect();
        seen.sort();
        assert_eq!(seen, vec![0, 1, 2, 3], "every instance lands in exactly one chunk");
        // Instances of a chunk really sit inside its cell.
        for (key, list) in &chunks {
            for k in list {
                let p = set.instances[*k].position;
                assert_eq!(((p.x / 100.0).floor() as i64, (p.y / 100.0).floor() as i64, (p.z / 100.0).floor() as i64), *key);
            }
        }

        set.chunk_size = 0.0;
        assert_eq!(set.chunks(), vec![((0, 0, 0), vec![0, 1, 2, 3])], "chunking off keeps one MultiMesh");
    }

    #[test]
    fn item_material_falls_back_to_the_set_then_to_the_model() {
        let mut set = Scatter::new("s", ScatterKind::Props, vec![ScatterItem::new("res://a.glb"), ScatterItem::new("res://b.glb")]);
        assert_eq!(set.item_material(0), None, "no override means the model's own materials");
        set.material = Some("moss".into());
        assert_eq!(set.item_material(0), Some("moss"));
        assert_eq!(set.item_material(1), Some("moss"));
        set.items[1].material = Some("snow".into());
        assert_eq!(set.item_material(1), Some("snow"), "the entry wins over the set");
        assert_eq!(set.item_material(0), Some("moss"), "the other entry still follows the set");
        assert_eq!(set.item_material(9), Some("moss"), "a missing entry falls back to the set");

        // Overrides survive a save and stay out of the file when unset.
        let text = serde_json::to_string(&set).unwrap();
        assert!(!serde_json::to_string(&Scatter::new("s", ScatterKind::Props, vec![ScatterItem::new("res://a.glb")])).unwrap().contains("material"));
        let back: Scatter = serde_json::from_str(&text).unwrap();
        assert_eq!(back.material.as_deref(), Some("moss"));
        assert_eq!(back.items[1].material.as_deref(), Some("snow"));
    }

    #[test]
    fn slope_filter_and_transform() {
        let mut set = Scatter::new("s", ScatterKind::Foliage, vec![ScatterItem::new("res://a.glb")]);
        let steep = |origin: DVec3, dir: DVec3| Some(SurfaceHit { point: origin + dir * 10.0, normal: DVec3::new(1.0, 1.0, 0.0).normalize(), node: NodeId(1) });
        let mut rng = Rng::new(1);
        let rules = ScatterRules { slope: [0.0, 30.0], ..Default::default() };
        assert_eq!(set.paint(DVec3::ZERO, DVec3::Y, 200.0, &rules, &mut rng, &[], steep), 0, "45 degree slope is rejected");
        set.instances.push(ScatterInstance { item: 0, position: DVec3::new(10.0, 0.0, 0.0), angles: DVec3::ZERO, scale: 1.0 });
        set.transform(&(DMat4::from_rotation_y(std::f64::consts::FRAC_PI_2) * DMat4::from_scale(DVec3::splat(2.0))));
        let i = set.instances[0];
        assert!((i.position - DVec3::new(0.0, 0.0, -20.0)).length() < 1e-9);
        assert!((i.angles.y - 90.0).abs() < 1e-6 && (i.scale - 2.0).abs() < 1e-9);
    }
}
