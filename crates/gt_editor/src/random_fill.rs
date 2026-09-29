//! Random fill: covers a grid of cells with pieces from the prefab library, each cell's piece picked by weight. Build
//! a kit of pieces that each fill one cell, centered on the origin, and the fill lays out a floor plan from them, a
//! different one per seed.
//!
//! Clustering makes pieces come in patches rather than evenly mixed, the way real buildings have a wing of offices
//! and a hall of pillars: every piece gets a smooth noise field over the grid, and its weight in a cell rises and falls
//! with that field. The result is plain copies of the pieces, placed like exploded prefab instances, so it bakes and
//! builds like hand made content and each copy keeps its entity names apart.

use gt_core::{DVec3, NodeId};
use gt_doc::NodeKind;
use gt_doc::scatter::Rng;

#[derive(Clone, Debug, PartialEq)]
pub struct FillPiece {
    /// A prefab library entry, category/name.
    pub name: String,
    pub weight: f64,
    /// Turns the piece by a random multiple of 90 degrees.
    pub rotate: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FillOptions {
    pub pieces: Vec<FillPiece>,
    /// Weight of leaving a cell empty.
    pub empty: f64,
    /// Corner of the grid, the floor height is its y.
    pub min: DVec3,
    pub cell: f64,
    pub cells: [usize; 2],
    pub seed: u64,
    /// 0 mixes the pieces evenly, 1 gathers each into patches.
    pub cluster: f64,
    /// Patch size in cells.
    pub cluster_size: f64,
}

impl Default for FillOptions {
    fn default() -> Self {
        Self { pieces: Vec::new(), empty: 0.0, min: DVec3::ZERO, cell: 128.0, cells: [8, 8], seed: 1, cluster: 0.5, cluster_size: 4.0 }
    }
}

/// One filled cell: which piece, where its center is, and how far it is turned.
#[derive(Clone, Debug, PartialEq)]
pub struct Placement {
    pub piece: usize,
    pub cell: [usize; 2],
    pub center: DVec3,
    pub yaw: f64,
}

fn hash(a: u64, b: u64, c: u64) -> f64 {
    let mut h = a.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ b.wrapping_mul(0xC2B2_AE3D_27D4_EB4F) ^ c.wrapping_mul(0x1656_67B1_9E37_79F9);
    h ^= h >> 31;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 29;
    (h >> 11) as f64 / (1u64 << 53) as f64
}

/// Smooth value noise in 0..1 over cell coordinates, one field per `salt`.
fn noise(x: f64, z: f64, size: f64, seed: u64, salt: u64) -> f64 {
    let (fx, fz) = (x / size.max(0.5), z / size.max(0.5));
    let (ix, iz) = (fx.floor(), fz.floor());
    let (tx, tz) = (fx - ix, fz - iz);
    let smooth = |t: f64| t * t * (3.0 - 2.0 * t);
    let at = |dx: f64, dz: f64| hash((ix + dx) as i64 as u64, (iz + dz) as i64 as u64, seed.wrapping_mul(31).wrapping_add(salt));
    let top = at(0.0, 0.0) + (at(1.0, 0.0) - at(0.0, 0.0)) * smooth(tx);
    let bottom = at(0.0, 1.0) + (at(1.0, 1.0) - at(0.0, 1.0)) * smooth(tx);
    top + (bottom - top) * smooth(tz)
}

/// The layout for `opts`, the same for the same options. Empty cells are left out.
pub fn plan(opts: &FillOptions) -> Vec<Placement> {
    let mut rng = Rng::new(opts.seed.max(1));
    let cluster = opts.cluster.clamp(0.0, 1.0);
    let mut out = Vec::new();
    for j in 0..opts.cells[1] {
        for i in 0..opts.cells[0] {
            // Each weight is multiplied or divided by up to about six as the piece's field rises and falls. Value noise
            // stays near its middle most of the time, so a gentler swing would hardly gather anything.
            let weights: Vec<f64> = opts
                .pieces
                .iter()
                .enumerate()
                .map(|(k, p)| {
                    let field = noise(i as f64, j as f64, opts.cluster_size, opts.seed, k as u64 + 1);
                    p.weight.max(0.0) * (cluster * 6.0 * (field - 0.5)).exp()
                })
                .collect();
            let total = weights.iter().sum::<f64>() + opts.empty.max(0.0);
            let roll = rng.next_f64() * total;
            let turn = (rng.next_f64() * 4.0).floor().min(3.0);
            let mut acc = 0.0;
            let picked = weights.iter().position(|w| {
                acc += w;
                roll < acc
            });
            let Some(piece) = picked.filter(|_| total > 0.0) else { continue };
            let center = opts.min + DVec3::new((i as f64 + 0.5) * opts.cell, 0.0, (j as f64 + 0.5) * opts.cell);
            let yaw = if opts.pieces[piece].rotate { turn * 90.0 } else { 0.0 };
            out.push(Placement { piece, cell: [i, j], center, yaw });
        }
    }

    out
}

/// The layout as rows of letters, one per piece in order (a, b, c...) and . for empty, for a quick look.
pub fn sketch(opts: &FillOptions, placements: &[Placement]) -> Vec<String> {
    let mut rows = vec![vec!['.'; opts.cells[0]]; opts.cells[1]];
    for p in placements {
        rows[p.cell[1]][p.cell[0]] = (b'a' + (p.piece % 26) as u8) as char;
    }

    rows.into_iter().map(|r| r.into_iter().collect()).collect()
}

pub struct Filled {
    pub group: NodeId,
    pub placements: Vec<Placement>,
}

/// Fills the grid in one undo step: a group holding a copy of the picked piece per cell, each under its own name
/// prefix. `replace` removes an earlier fill first, for a reroll.
pub fn apply(state: &mut crate::state::EditorState, opts: &FillOptions, replace: Option<NodeId>) -> Result<Filled, String> {
    if opts.pieces.is_empty() {
        return Err("Add pieces from the prefab library to fill with".into());
    }

    let mut texts = Vec::new();
    for p in &opts.pieces {
        let entry = state.prefab_library.find(&p.name).cloned().ok_or_else(|| format!("No prefab {} in the library", p.name))?;
        texts.push(state.prefab_library.text(&entry)?);
    }

    let placements = plan(opts);
    let parent = state.insert_parent();
    let game = &state.game;
    let group = state.doc.edit("Random Fill", |m, s| {
        if let Some(old) = replace.filter(|id| m.contains(*id)) {
            m.remove(old);
        }

        let group = m.insert(parent, NodeKind::Group(gt_doc::map::Group::new(format!("Random fill {}", opts.seed))));
        for (n, p) in placements.iter().enumerate() {
            let inst =
                gt_doc::map::Instance { path: String::new(), origin: p.center, angles: DVec3::new(0.0, p.yaw, 0.0), fixup: format!("rf{}_{n}", opts.seed) };
            crate::commands::place_copy(m, game, group, &texts[p.piece], &inst);
        }

        s.clear();
        s.select_node(group);
        group
    });
    state.set_status(format!("Filled {} of {} cells", placements.len(), opts.cells[0] * opts.cells[1]));
    Ok(Filled { group, placements })
}

/// Brush > Random Fill: the pieces and grid of the last fill, kept while the editor runs.
#[derive(Default)]
pub struct RandomFillDialog {
    pub open: bool,
    opts: FillOptions,
    last: Option<NodeId>,
}

fn piece_color(k: usize) -> egui::Color32 {
    let h = hash(k as u64, 17, 3);
    egui::ecolor::Hsva::new(h as f32, 0.55, 0.85, 1.0).into()
}

impl RandomFillDialog {
    /// The grid over the selection's footprint, standing on its floor: the top of a flat selection, like a floor
    /// brush, else its bottom.
    fn fit_selection(&mut self, state: &crate::state::EditorState) {
        let ids = gt_doc::ops::selection_roots(&state.doc.map, &state.doc.selection);
        let b = state.doc.map.bounds_of(ids);
        if b.is_empty() {
            return;
        }

        let size = b.size();
        let cell = self.opts.cell.max(1.0);
        self.opts.cells = [((size.x / cell).floor() as usize).max(1), ((size.z / cell).floor() as usize).max(1)];
        let used = DVec3::new(self.opts.cells[0] as f64 * cell, 0.0, self.opts.cells[1] as f64 * cell);
        let y = if size.y < cell * 0.25 { b.max.y } else { b.min.y };
        self.opts.min = DVec3::new(b.min.x + (size.x - used.x) * 0.5, y, b.min.z + (size.z - used.z) * 0.5);
    }

    pub fn show(&mut self, ctx: &egui::Context, state: &mut crate::state::EditorState) {
        if !self.open {
            return;
        }

        let mut open = self.open;
        egui::Window::new("Random Fill").open(&mut open).resizable(false).default_width(360.0).show(ctx, |ui| {
            ui.label("Covers a grid with pieces from the prefab library, picked by weight. Build pieces one cell wide, centered on the origin.");
            ui.separator();
            let mut remove = None;
            egui::Grid::new("fill_pieces").num_columns(4).show(ui, |ui| {
                for (k, p) in self.opts.pieces.iter_mut().enumerate() {
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 2.0, piece_color(k));
                    ui.label(&p.name);
                    ui.add(egui::DragValue::new(&mut p.weight).range(0.0..=1000.0).speed(0.1).prefix("weight "));
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut p.rotate, "turn").on_hover_text("Turns the piece by a random multiple of 90 degrees");
                        if ui.small_button("✕").on_hover_text("Remove").clicked() {
                            remove = Some(k);
                        }
                    });
                    ui.end_row();
                }
            });
            if let Some(k) = remove {
                self.opts.pieces.remove(k);
            }

            egui::ComboBox::from_id_salt("fill_add").selected_text("Add piece…").show_ui(ui, |ui| {
                for e in &state.prefab_library.entries {
                    if ui.selectable_label(false, e.key()).clicked() {
                        self.opts.pieces.push(FillPiece { name: e.key(), weight: 1.0, rotate: true });
                    }
                }
            });
            ui.add(egui::DragValue::new(&mut self.opts.empty).range(0.0..=1000.0).speed(0.1).prefix("empty weight "))
                .on_hover_text("How likely a cell stays empty, against the pieces' weights");
            ui.separator();
            egui::Grid::new("fill_grid").num_columns(2).show(ui, |ui| {
                ui.label("Cell");
                ui.add(egui::DragValue::new(&mut self.opts.cell).range(8.0..=4096.0).suffix(" u"));
                ui.end_row();
                ui.label("Cells");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut self.opts.cells[0]).range(1..=256));
                    ui.label("×");
                    ui.add(egui::DragValue::new(&mut self.opts.cells[1]).range(1..=256));
                });
                ui.end_row();
                ui.label("Corner");
                ui.horizontal(|ui| {
                    for v in [&mut self.opts.min.x, &mut self.opts.min.y, &mut self.opts.min.z] {
                        ui.add(egui::DragValue::new(v).speed(1.0));
                    }
                });
                ui.end_row();
                ui.label("Seed");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut self.opts.seed).range(1..=u32::MAX as u64));
                    if ui.small_button("🎲").on_hover_text("A new random seed").clicked() {
                        self.opts.seed = crate::commands::time_seed().max(1) % u32::MAX as u64;
                    }
                });
                ui.end_row();
                ui.label("Clustering");
                ui.add(egui::Slider::new(&mut self.opts.cluster, 0.0..=1.0)).on_hover_text("0 mixes the pieces evenly, 1 gathers each into patches");
                ui.end_row();
                ui.label("Patch size");
                ui.add(egui::DragValue::new(&mut self.opts.cluster_size).range(1.0..=64.0).speed(0.1).suffix(" cells"));
                ui.end_row();
            });
            if ui.button("Fit to selection").on_hover_text("The grid over the selection's footprint, as many whole cells as fit").clicked() {
                self.fit_selection(state);
            }

            // The layout as colored cells, so a seed can be judged before anything is placed.
            let placements = plan(&self.opts);
            let [nx, nz] = self.opts.cells;
            let px = (320.0 / nx.max(nz).max(1) as f32).clamp(2.0, 16.0);
            let (rect, _) = ui.allocate_exact_size(egui::vec2(px * nx as f32, px * nz as f32), egui::Sense::hover());
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 0.0, ui.visuals().extreme_bg_color);
            for p in &placements {
                let min = rect.min + egui::vec2(p.cell[0] as f32 * px, p.cell[1] as f32 * px);
                painter.rect_filled(egui::Rect::from_min_size(min, egui::vec2(px - 1.0, px - 1.0)), 0.0, piece_color(p.piece));
            }

            ui.horizontal(|ui| {
                if ui.button("Fill").on_hover_text("Adds a new fill").clicked() {
                    match apply(state, &self.opts, None) {
                        Ok(filled) => self.last = Some(filled.group),
                        Err(e) => state.set_status(e),
                    }
                }

                let reroll = ui.add_enabled(self.last.is_some(), egui::Button::new("Reroll")).on_hover_text("A new seed, replacing the last fill");
                if reroll.clicked() {
                    self.opts.seed = self.opts.seed % (u32::MAX as u64 - 1) + 1;
                    match apply(state, &self.opts, self.last) {
                        Ok(filled) => self.last = Some(filled.group),
                        Err(e) => state.set_status(e),
                    }
                }
            });
        });
        self.open = open;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pieces(weights: &[f64]) -> Vec<FillPiece> {
        weights.iter().enumerate().map(|(k, w)| FillPiece { name: format!("p{k}"), weight: *w, rotate: true }).collect()
    }

    #[test]
    fn the_same_seed_gives_the_same_layout_and_weights_decide_the_share() {
        let opts = FillOptions { pieces: pieces(&[6.0, 3.0, 1.0]), cells: [40, 40], cluster: 0.0, seed: 7, ..Default::default() };
        let a = plan(&opts);
        assert_eq!(a, plan(&opts));
        assert_ne!(a, plan(&FillOptions { seed: 8, ..opts.clone() }));
        assert_eq!(a.len(), 1600, "no empty weight, every cell is filled");
        let share = |k: usize| a.iter().filter(|p| p.piece == k).count() as f64 / a.len() as f64;
        assert!((share(0) - 0.6).abs() < 0.05 && (share(1) - 0.3).abs() < 0.05 && (share(2) - 0.1).abs() < 0.04, "{} {} {}", share(0), share(1), share(2));
        assert!(a.iter().all(|p| [0.0, 90.0, 180.0, 270.0].contains(&p.yaw)));
        assert!(a.iter().any(|p| p.yaw != 0.0));
        let first = &a[0];
        assert_eq!(first.center, DVec3::new(64.0, 0.0, 64.0), "cell centers, a cell is 128 by default");

        let sparse = plan(&FillOptions { empty: 10.0, ..opts.clone() });
        assert!(sparse.len() < 1000, "an empty weight as large as all pieces leaves about half the cells open, {}", sparse.len());
    }

    #[test]
    fn a_fill_places_copies_on_the_cell_centers_with_their_names_apart() {
        use gt_core::Aabb;
        let root = std::env::temp_dir().join(format!("gt_random_fill_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let mut state = crate::state::EditorState::new(crate::state::Prefs::default());
        // A piece: a 32 unit block with a named entity wired to itself, like a flickering lamp's timer.
        let mut piece = gt_doc::Map::new();
        let layer = piece.default_layer();
        let block = piece.insert(
            layer,
            NodeKind::Brush(gt_geom::Brush::from_aabb(&Aabb::new(DVec3::new(-16.0, 0.0, -16.0), DVec3::new(16.0, 32.0, 16.0)), "dev/grey").unwrap()),
        );
        let mut lamp = gt_doc::Entity::new("info_target");
        lamp.properties.insert("targetname".into(), "lamp".into());
        let lamp = piece.insert(layer, NodeKind::Entity(lamp));
        state.prefab_library.add(&piece, &[block, lamp], "kit", "block", &root, false).unwrap();

        let opts = FillOptions {
            pieces: vec![FillPiece { name: "kit/block".into(), weight: 1.0, rotate: false }],
            cells: [2, 1],
            cell: 64.0,
            seed: 5,
            ..Default::default()
        };
        let filled = apply(&mut state, &opts, None).unwrap();
        let map = &state.doc.map;
        let names: Vec<String> = map.walk().into_iter().filter_map(|id| map.entity(id)).filter_map(|e| e.properties.get("targetname").cloned()).collect();
        assert_eq!(names, ["rf5_0-lamp", "rf5_1-lamp"]);
        let blocks: Vec<Aabb> =
            map.walk().into_iter().filter(|id| matches!(map.get(*id).map(|n| &n.kind), Some(NodeKind::Brush(_)))).map(|id| map.bounds(id)).collect();
        assert_eq!(blocks.iter().map(|b| b.center().x).collect::<Vec<_>>(), [32.0, 96.0], "one per cell, on its center");

        let again = apply(&mut state, &FillOptions { seed: 6, ..opts }, Some(filled.group)).unwrap();
        assert!(!state.doc.map.contains(filled.group) && state.doc.map.contains(again.group), "a reroll replaces the old fill");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn clustering_gathers_each_piece_into_patches() {
        let base = FillOptions { pieces: pieces(&[1.0, 1.0]), cells: [48, 48], seed: 3, cluster_size: 6.0, ..Default::default() };
        // How often a cell's right hand neighbour holds the same piece.
        let alike = |cluster: f64| {
            let opts = FillOptions { cluster, ..base.clone() };
            let grid = sketch(&opts, &plan(&opts));
            let pairs: Vec<bool> = grid.iter().flat_map(|row| row.as_bytes().windows(2).map(|w| w[0] == w[1]).collect::<Vec<_>>()).collect();
            pairs.iter().filter(|x| **x).count() as f64 / pairs.len() as f64
        };
        let (mixed, patched) = (alike(0.0), alike(1.0));
        assert!((mixed - 0.5).abs() < 0.05, "two even pieces match their neighbour about half the time, {mixed}");
        assert!(patched > mixed + 0.08, "clustered layouts put like next to like, {patched} against {mixed}");
    }
}
