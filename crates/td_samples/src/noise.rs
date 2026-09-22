//! Tiling hash and value noise for the pixel art atlases of the demo Blockbench models.

pub const SIZE: u32 = 64;

/// Integer hash noise in 0..1, wrapping at `SIZE` so textures tile.
pub fn hash(x: i64, y: i64, seed: u32) -> f32 {
    let x = x.rem_euclid(SIZE as i64) as u32;
    let y = y.rem_euclid(SIZE as i64) as u32;
    let mut h = x.wrapping_mul(0x27d4_eb2d) ^ y.wrapping_mul(0x1656_67b1) ^ seed.wrapping_mul(0x9e37_79b9);
    h ^= h >> 15;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    (h & 0xffff) as f32 / 65535.0
}

/// Smooth value noise with `cells` cells across the texture, tiling.
pub fn value_noise(x: u32, y: u32, cells: u32, seed: u32) -> f32 {
    let scale = SIZE as f32 / cells as f32;
    let fx = x as f32 / scale;
    let fy = y as f32 / scale;
    let (ix, iy) = (fx.floor() as i64, fy.floor() as i64);
    let (tx, ty) = (fx - ix as f32, fy - iy as f32);
    let cell = |cx: i64, cy: i64| hash(cx.rem_euclid(cells as i64) * 7, cy.rem_euclid(cells as i64) * 13, seed);
    let s = |t: f32| t * t * (3.0 - 2.0 * t);
    let a = cell(ix, iy) + (cell(ix + 1, iy) - cell(ix, iy)) * s(tx);
    let b = cell(ix, iy + 1) + (cell(ix + 1, iy + 1) - cell(ix, iy + 1)) * s(tx);
    a + (b - a) * s(ty)
}
