//! Paletted id Tech textures: Quake `WAD2` and Half-Life `WAD3` texture packs, Quake 2 `.wal` (and the Daikatana
//! variant with its own palette).

use crate::vtf::Image;

/// Quake's palette, from LibreQuake (BSD-3-Clause), the same colors as the original.
pub const QUAKE_PALETTE: &[u8; 768] = include_bytes!("../data/quake_palette.lmp");
/// Quake 2's palette, from FuncGodot (MIT).
pub const QUAKE2_PALETTE: &[u8; 768] = include_bytes!("../data/quake2_palette.lmp");

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum QuakeTextureError {
    #[error("not a WAD2 or WAD3 file")]
    NotWad,
    #[error("not a WAL texture")]
    NotWal,
    #[error("texture data is truncated")]
    Truncated,
}

/// Quake 2 surface flags a `.wal` carries as defaults for the faces using it.
pub const SURF_SKY: u32 = 0x4;
pub const SURF_TRANS33: u32 = 0x10;
pub const SURF_TRANS66: u32 = 0x20;

#[derive(Clone, Debug, PartialEq)]
pub struct Paletted {
    /// As stored, for a `.wal` the path below the textures folder without extension.
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub indices: Vec<u8>,
    pub palette: Vec<[u8; 3]>,
    /// Quake 2 surface flags, 0 for WAD textures.
    pub flags: u32,
}

fn u32_at(b: &[u8], o: usize) -> Result<u32, QuakeTextureError> {
    b.get(o..o + 4).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]])).ok_or(QuakeTextureError::Truncated)
}

fn c_string(b: &[u8]) -> String {
    let end = b.iter().position(|c| *c == 0).unwrap_or(b.len());
    b[..end].iter().map(|c| *c as char).collect()
}

fn palette_of(b: &[u8]) -> Vec<[u8; 3]> {
    b.as_chunks::<3>().0.to_vec()
}

/// Every mip texture of a WAD. Other lumps (status bar pictures, palettes) are skipped.
pub fn read_wad(b: &[u8]) -> Result<Vec<Paletted>, QuakeTextureError> {
    let half_life = match b.get(0..4) {
        Some(b"WAD2") => false,
        Some(b"WAD3") => true,
        _ => return Err(QuakeTextureError::NotWad),
    };
    let (count, dir) = (u32_at(b, 4)? as usize, u32_at(b, 8)? as usize);
    let mut out = Vec::new();
    for i in 0..count {
        let entry = b.get(dir + i * 32..dir + i * 32 + 32).ok_or(QuakeTextureError::Truncated)?;
        let (pos, kind, compression) = (u32_at(entry, 0)? as usize, entry[12], entry[13]);
        if !matches!(kind, 0x43 | 0x44) || compression != 0 {
            continue;
        }

        let name = c_string(&entry[16..32]);
        let (w, h, offset) = (u32_at(b, pos + 16)?, u32_at(b, pos + 20)?, u32_at(b, pos + 24)? as usize);
        let size = w as usize * h as usize;
        let indices = b.get(pos + offset..pos + offset + size).ok_or(QuakeTextureError::Truncated)?.to_vec();
        let palette = if half_life {
            // The palette follows the fourth mip level, prefixed by its color count.
            let at = pos + u32_at(b, pos + 36)? as usize + size / 64 + 2;
            palette_of(b.get(at..at + 768).ok_or(QuakeTextureError::Truncated)?)
        } else {
            palette_of(QUAKE_PALETTE)
        };
        out.push(Paletted { name: if name.is_empty() { c_string(&b[pos..pos + 16]) } else { name }, width: w, height: h, indices, palette, flags: 0 });
    }

    Ok(out)
}

/// A Quake 2 `.wal`, or a Daikatana one, which starts with version 3 and brings its own palette.
pub fn read_wal(b: &[u8], name: &str) -> Result<Paletted, QuakeTextureError> {
    let (w, h, offset, flags, palette) = if b.first() == Some(&3) && b.len() > 892 {
        let pal = palette_of(b.get(120..888).ok_or(QuakeTextureError::Truncated)?);
        (u32_at(b, 36)?, u32_at(b, 40)?, u32_at(b, 44)? as usize, u32_at(b, 112)?, pal)
    } else {
        (u32_at(b, 32)?, u32_at(b, 36)?, u32_at(b, 40)? as usize, u32_at(b, 88)?, palette_of(QUAKE2_PALETTE))
    };
    if w == 0 || h == 0 || w > 8192 || h > 8192 {
        return Err(QuakeTextureError::NotWal);
    }

    let indices = b.get(offset..offset + (w * h) as usize).ok_or(QuakeTextureError::Truncated)?.to_vec();
    Ok(Paletted { name: name.to_string(), width: w, height: h, indices, palette, flags })
}

impl Paletted {
    /// Names starting with `{` cut out their last palette index, the way Half-Life and Quake source ports do.
    pub fn alpha_test(&self) -> bool {
        self.name.starts_with('{')
    }

    pub fn is_sky(&self) -> bool {
        self.flags & SURF_SKY != 0 || self.name.to_ascii_lowercase().starts_with("sky")
    }

    pub fn to_rgba(&self) -> Image {
        let cut = self.alpha_test();
        let mut rgba = Vec::with_capacity(self.indices.len() * 4);
        for &i in &self.indices {
            let [r, g, b] = self.palette.get(i as usize).copied().unwrap_or_default();
            if cut && i == 255 {
                rgba.extend([0, 0, 0, 0]);
            } else {
                rgba.extend([r, g, b, 255]);
            }
        }

        Image { width: self.width, height: self.height, rgba }
    }

    /// What a static face shows of a Quake sky: the back layer, the right half of a 2:1 sky texture. Other skies are
    /// used whole.
    pub fn sky_image(&self) -> Image {
        let img = self.to_rgba();
        if self.width != self.height * 2 {
            return img;
        }

        let (w, h) = (self.width as usize, self.height as usize);
        let rgba = (0..h).flat_map(|y| img.rgba[(y * w + w / 2) * 4..(y * w + w) * 4].to_vec()).collect();
        Image { width: self.width / 2, height: self.height, rgba }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A WAD with one mip texture per `(name, width, height, fill index)`.
    pub fn wad(magic: &[u8; 4], textures: &[(&str, u32, u32, u8)]) -> Vec<u8> {
        let mut b = magic.to_vec();
        b.extend([0; 8]);
        let mut dir = Vec::new();
        for (name, w, h, fill) in textures {
            let pos = b.len() as u32;
            let mut header = [0u8; 40];
            header[..name.len()].copy_from_slice(name.as_bytes());
            header[16..20].copy_from_slice(&w.to_le_bytes());
            header[20..24].copy_from_slice(&h.to_le_bytes());
            let size = w * h;
            for (k, off) in [40, 40 + size, 40 + size + size / 4, 40 + size + size / 4 + size / 16].iter().enumerate() {
                header[24 + k * 4..28 + k * 4].copy_from_slice(&off.to_le_bytes());
            }

            b.extend(header);
            let mut pixels = vec![*fill; size as usize];
            pixels[0] = 255;
            b.extend(pixels);
            b.extend(vec![0; (size / 4 + size / 16 + size / 64) as usize]);
            if magic == b"WAD3" {
                b.extend(256u16.to_le_bytes());
                b.extend((0..=255u8).flat_map(|i| [i, 255 - i, 7]));
                b.extend([0, 0]);
            }

            let mut entry = [0u8; 32];
            entry[0..4].copy_from_slice(&pos.to_le_bytes());
            entry[12] = if magic == b"WAD3" { 0x43 } else { 0x44 };
            entry[16..16 + name.len()].copy_from_slice(name.as_bytes());
            dir.extend(entry);
        }

        let dir_pos = b.len() as u32;
        b[4..8].copy_from_slice(&(textures.len() as u32).to_le_bytes());
        b[8..12].copy_from_slice(&dir_pos.to_le_bytes());
        b.extend(dir);
        b
    }

    #[test]
    fn reads_quake_and_half_life_wads() {
        let q1 = read_wad(&wad(b"WAD2", &[("wall1", 16, 8, 3), ("{fence", 8, 8, 5)])).unwrap();
        assert_eq!(q1.len(), 2);
        assert_eq!((q1[0].name.as_str(), q1[0].width, q1[0].height), ("wall1", 16, 8));
        let img = q1[0].to_rgba();
        let q = |i: usize| [QUAKE_PALETTE[i * 3], QUAKE_PALETTE[i * 3 + 1], QUAKE_PALETTE[i * 3 + 2], 255];
        assert_eq!(&img.rgba[4..8], &q(3));
        assert_eq!(&img.rgba[0..4], &q(255), "index 255 is an ordinary color without the {{ prefix");
        let fence = q1[1].to_rgba();
        assert_eq!(fence.rgba[3], 0, "{{ cuts out index 255");
        assert_eq!(&fence.rgba[4..8], &q(5));

        let hl = read_wad(&wad(b"WAD3", &[("{grate", 8, 8, 9)])).unwrap();
        assert_eq!(&hl[0].to_rgba().rgba[4..8], &[9, 246, 7, 255], "the texture's own palette");
        assert!(hl[0].alpha_test());
        assert_eq!(read_wad(b"PACK...."), Err(QuakeTextureError::NotWad));
    }

    #[test]
    fn reads_wal_textures() {
        let mut wal = vec![0u8; 100];
        wal[..5].copy_from_slice(b"floor");
        wal[32..36].copy_from_slice(&4u32.to_le_bytes());
        wal[36..40].copy_from_slice(&2u32.to_le_bytes());
        wal[40..44].copy_from_slice(&100u32.to_le_bytes());
        wal[88..92].copy_from_slice(&(SURF_SKY | SURF_TRANS33).to_le_bytes());
        wal.extend([1, 2, 3, 4, 5, 6, 7, 8]);
        let t = read_wal(&wal, "e1u1/floor").unwrap();
        assert_eq!((t.width, t.height, t.flags), (4, 2, SURF_SKY | SURF_TRANS33));
        assert!(t.is_sky());
        assert_eq!(&t.to_rgba().rgba[..3], &QUAKE2_PALETTE[3..6]);
        assert_eq!(read_wal(&wal[..104], "x"), Err(QuakeTextureError::Truncated));
    }

    #[test]
    fn quake_skies_keep_their_back_layer() {
        let mut sky = read_wad(&wad(b"WAD2", &[("sky1", 4, 2, 0)])).unwrap().remove(0);
        sky.indices = vec![1, 1, 2, 2, 1, 1, 2, 2];
        let back = sky.sky_image();
        assert_eq!((back.width, back.height), (2, 2));
        assert_eq!(&back.rgba[..3], &QUAKE_PALETTE[6..9]);
    }
}
