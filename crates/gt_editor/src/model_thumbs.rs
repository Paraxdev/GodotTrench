//! Offscreen-rendered previews for the Models panel: a small three-quarter view of each model, cached
//! in memory and, when the editor has a cache directory, on disk too. Rendered a couple at a time so
//! opening the panel or scrolling a big library never stalls waiting on the GPU.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use glam::Vec3;
use gt_render::{Frame, FrameParams, Lighting, MeshBatch, MeshVertex, Renderer, ShadeMode, ViewTarget};

use crate::models::ModelCache;

const SIZE: u32 = 128;
/// Part of the disk cache name, bumped when thumbnails render differently (emission) so stale ones are redrawn.
const THUMB_VERSION: u32 = 2;
/// Cache misses rendered per egui frame, so scrolling past many never-seen models never stalls the UI.
const BUDGET_PER_FRAME: u32 = 2;

struct Entry {
    mtime: Option<SystemTime>,
    texture: Option<egui::TextureHandle>,
}

/// Renders and caches small offscreen previews of models, for the Models panel and anywhere else a
/// preview is useful (e.g. the scatter palette's item cards). One instance is meant to live for the
/// app's lifetime. Call `thumbnail` for each model to show, `clear` when the library rescans.
#[derive(Default)]
pub struct ModelThumbnails {
    entries: HashMap<PathBuf, Entry>,
    /// A small render target reused for every model, never exposed to callers, so a big library does
    /// not keep one GPU texture resident per thumbnail.
    scratch: Option<ViewTarget>,
    /// `None` until first resolved, then `Some(None)` when the editor has no cache directory.
    disk_dir: Option<Option<PathBuf>>,
    budget_frame: u64,
    budget_left: u32,
}

impl ModelThumbnails {
    /// Drops every cached thumbnail, so the next call to `thumbnail` re-renders. The cache already keys
    /// each entry on its model's mtime, so this is a safety net, e.g. a rescan, rather than the only
    /// invalidation path (a file edited and re-saved within the same mtime second is the case it covers).
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// A model's thumbnail texture, or `None` while it is still loading, if it failed to render, or if
    /// no `Renderer` is available (headless tests, or a disk cache miss before the app has a wgpu
    /// surface) and nothing is cached for it on disk either.
    ///
    /// `models` and `units_per_meter` are only touched on an actual render, never on a cache hit.
    pub fn thumbnail(
        &mut self,
        ctx: &egui::Context,
        renderer: Option<&mut Renderer>,
        models: &mut ModelCache,
        units_per_meter: f64,
        path: &Path,
    ) -> Option<egui::TextureId> {
        let mtime = std::fs::metadata(path).and_then(|m| m.modified()).ok();
        if let Some(e) = self.entries.get(path)
            && e.mtime == mtime
        {
            return e.texture.as_ref().map(|t| t.id());
        }

        let frame = ctx.cumulative_frame_nr();
        if frame != self.budget_frame {
            self.budget_frame = frame;
            self.budget_left = BUDGET_PER_FRAME;
        }

        let disk_path = self.disk_cache_path(path, mtime);
        if let Some(p) = &disk_path
            && let Ok(img) = image::open(p)
        {
            let texture = load_texture(ctx, path, &img.to_rgba8());
            let id = texture.id();
            self.entries.insert(path.to_path_buf(), Entry { mtime, texture: Some(texture) });
            return Some(id);
        }

        let renderer = renderer?;
        if self.budget_left == 0 {
            return None;
        }

        self.budget_left -= 1;
        let rendered = models.get(path, units_per_meter).and_then(|model| render_model(renderer, &mut self.scratch, &model));
        let texture = rendered.as_ref().map(|img| load_texture(ctx, path, img));
        if let (Some(img), Some(p)) = (&rendered, &disk_path) {
            if let Some(dir) = p.parent() {
                let _ = std::fs::create_dir_all(dir);
            }

            let _ = img.save(p);
        }

        let id = texture.as_ref().map(|t| t.id());
        self.entries.insert(path.to_path_buf(), Entry { mtime, texture });
        id
    }

    /// The on-disk cache file for `path` at `mtime`, or `None` when the editor has no cache directory.
    /// The mtime is baked into the filename, so a changed file misses instead of loading a stale image;
    /// nothing needs to scan for or clean up the file its old mtime left behind.
    fn disk_cache_path(&mut self, path: &Path, mtime: Option<SystemTime>) -> Option<PathBuf> {
        let dir = self.disk_dir.get_or_insert_with(|| eframe::storage_dir("GodotTrench").map(|d| d.join("cache").join("model_thumbs"))).clone()?;
        let secs = mtime.and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
        Some(dir.join(format!("{:016x}-{secs}-v{THUMB_VERSION}.png", fnv_hash(&path.to_string_lossy()))))
    }
}

fn load_texture(ctx: &egui::Context, path: &Path, img: &image::RgbaImage) -> egui::TextureHandle {
    let color = egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw());
    ctx.load_texture(format!("model_thumb:{}", path.display()), color, egui::TextureOptions::LINEAR)
}

fn fnv_hash(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }

    h
}

/// Renders one three-quarter view of `model` from slightly above, over a transparent background, and
/// reads it back to the CPU.
fn render_model(renderer: &mut Renderer, scratch: &mut Option<ViewTarget>, model: &crate::models::Model) -> Option<image::RgbaImage> {
    for (key, image, pixelated) in &model.textures {
        renderer.set_material_desc(key, &model.material_desc(key, image, *pixelated));
    }

    let mut batch = MeshBatch::default();
    for part in &model.parts {
        if part.indices.is_empty() {
            continue;
        }

        let verts: Vec<MeshVertex> =
            part.vertices.iter().map(|v| MeshVertex { pos: v.pos.to_array(), normal: v.normal.to_array(), uv: v.uv, color: [1.0; 4] }).collect();
        batch.add_triangles(&part.material, &verts, &part.indices);
    }

    let mesh = renderer.upload_mesh(&batch)?;

    let center = model.bounds.center().as_vec3();
    let radius = (model.bounds.size().length() as f32 * 0.5).max(1e-3);
    // Three-quarter view: to the side and in front, looking slightly down from above.
    let offset = Vec3::new(1.0, 0.7, 1.0).normalize();
    let eye = center + offset * radius * 2.4;
    let forward = (center - eye).normalize_or(Vec3::NEG_Z);
    let view = glam::camera::rh::view::look_to_mat4(eye, forward, Vec3::Y);
    let proj = glam::camera::rh::proj::directx::perspective_infinite_reverse(35f32.to_radians(), 1.0, (radius * 0.05).max(1e-3));

    renderer.ensure_target(scratch, [SIZE, SIZE]);
    let target = scratch.as_ref()?;
    let params = FrameParams {
        view_proj: proj * view,
        eye,
        grid_size: 0.0,
        grid_alpha: 0.0,
        shade: ShadeMode::Lit,
        orthographic: false,
        clear: [0.0, 0.0, 0.0, 0.0],
        line_width: 1.0,
    };
    let mut frame = Frame::default();
    frame.opaque.push(&mesh);
    renderer.render_isolated(target, &params, &frame, &Lighting::default());
    renderer.read_target(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disk_cache_path_is_stable_and_mtime_sensitive() {
        let mut thumbs = ModelThumbnails { disk_dir: Some(Some(PathBuf::from("/cache"))), ..Default::default() };
        let p = Path::new("/models/oak.bbmodel");
        let a = thumbs.disk_cache_path(p, Some(SystemTime::UNIX_EPOCH)).unwrap();
        let b = thumbs.disk_cache_path(p, Some(SystemTime::UNIX_EPOCH)).unwrap();
        assert_eq!(a, b, "the same path and mtime always name the same cache file");

        let later = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(5);
        let c = thumbs.disk_cache_path(p, Some(later)).unwrap();
        assert_ne!(a, c, "a changed mtime changes the cache filename, so a stale image is never loaded");
    }

    #[test]
    fn thumbnails_saved_before_emission_are_not_reused() {
        let mut thumbs = ModelThumbnails { disk_dir: Some(Some(PathBuf::from("/cache"))), ..Default::default() };
        let name = thumbs.disk_cache_path(Path::new("/models/lamp.gltf"), Some(SystemTime::UNIX_EPOCH)).unwrap();
        let name = name.file_name().unwrap().to_string_lossy().into_owned();
        assert!(name.ends_with(&format!("-v{THUMB_VERSION}.png")) && THUMB_VERSION >= 2, "{name} would load a thumbnail rendered without emission");
    }

    #[test]
    fn no_cache_dir_disables_disk_caching() {
        let mut thumbs = ModelThumbnails { disk_dir: Some(None), ..Default::default() };
        assert!(thumbs.disk_cache_path(Path::new("/models/oak.bbmodel"), None).is_none());
    }

    #[test]
    fn disk_cache_hit_needs_no_renderer() {
        // Headless contexts (kittest, or a disk hit before the app has a wgpu surface) can still show a
        // thumbnail that a previous, real-renderer session already saved to disk.
        let dir = std::env::temp_dir().join(format!("gt_model_thumbs_hit_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let model_path = dir.join("thing.glb");
        std::fs::write(&model_path, b"stub").unwrap();
        let mtime = std::fs::metadata(&model_path).unwrap().modified().unwrap();

        let mut thumbs = ModelThumbnails { disk_dir: Some(Some(dir.join("cache"))), ..Default::default() };
        let cached_file = thumbs.disk_cache_path(&model_path, Some(mtime)).unwrap();
        std::fs::create_dir_all(cached_file.parent().unwrap()).unwrap();
        image::RgbaImage::from_pixel(4, 4, image::Rgba([200, 10, 10, 255])).save(&cached_file).unwrap();

        let ctx = egui::Context::default();
        let mut models = ModelCache::default();
        let id = thumbs.thumbnail(&ctx, None, &mut models, 32.0, &model_path);
        assert!(id.is_some(), "a disk-cached thumbnail loads without a renderer");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_model_and_no_renderer_is_a_quiet_miss() {
        let ctx = egui::Context::default();
        let mut models = ModelCache::default();
        let mut thumbs = ModelThumbnails { disk_dir: Some(None), ..Default::default() };
        assert!(thumbs.thumbnail(&ctx, None, &mut models, 32.0, Path::new("/does/not/exist.glb")).is_none());
    }
}
