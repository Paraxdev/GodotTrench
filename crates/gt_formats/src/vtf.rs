//! Valve texture (`.vtf`) reader, versions 7.0 to 7.5. Decodes the largest mipmap of the first frame to RGBA8.

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum VtfError {
    #[error("not a VTF file")]
    NotVtf,
    #[error("VTF version 7.{0} is not supported, only 7.0 to 7.5")]
    Version(u32),
    #[error("VTF image format {0} is not supported")]
    Format(String),
    #[error("VTF file is truncated")]
    Truncated,
    #[error("VTF has no image data")]
    NoImage,
}

/// An 8 bit RGBA image, rows top to bottom.
#[derive(Clone, Debug, PartialEq)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Rgba8888,
    Abgr8888,
    Rgb888,
    Bgr888,
    Rgb565,
    I8,
    Ia88,
    A8,
    Rgb888Bluescreen,
    Bgr888Bluescreen,
    Argb8888,
    Bgra8888,
    Dxt1,
    Dxt3,
    Dxt5,
    Bgrx8888,
    Bgr565,
    Bgrx5551,
    Bgra4444,
    Dxt1OneBitAlpha,
    Bgra5551,
    Uv88,
    Uvwq8888,
    Rgba16161616F,
    Rgba16161616,
    Uvlx8888,
}

const NAMES: [&str; 27] = [
    "RGBA8888",
    "ABGR8888",
    "RGB888",
    "BGR888",
    "RGB565",
    "I8",
    "IA88",
    "P8",
    "A8",
    "RGB888_BLUESCREEN",
    "BGR888_BLUESCREEN",
    "ARGB8888",
    "BGRA8888",
    "DXT1",
    "DXT3",
    "DXT5",
    "BGRX8888",
    "BGR565",
    "BGRX5551",
    "BGRA4444",
    "DXT1_ONEBITALPHA",
    "BGRA5551",
    "UV88",
    "UVWQ8888",
    "RGBA16161616F",
    "RGBA16161616",
    "UVLX8888",
];

impl Format {
    pub fn from_id(id: i32) -> Result<Format, VtfError> {
        use Format::*;
        let all = [
            Some(Rgba8888),
            Some(Abgr8888),
            Some(Rgb888),
            Some(Bgr888),
            Some(Rgb565),
            Some(I8),
            Some(Ia88),
            None,
            Some(A8),
            Some(Rgb888Bluescreen),
            Some(Bgr888Bluescreen),
            Some(Argb8888),
            Some(Bgra8888),
            Some(Dxt1),
            Some(Dxt3),
            Some(Dxt5),
            Some(Bgrx8888),
            Some(Bgr565),
            Some(Bgrx5551),
            Some(Bgra4444),
            Some(Dxt1OneBitAlpha),
            Some(Bgra5551),
            Some(Uv88),
            Some(Uvwq8888),
            Some(Rgba16161616F),
            Some(Rgba16161616),
            Some(Uvlx8888),
        ];
        let name = usize::try_from(id).ok().and_then(|i| NAMES.get(i)).map(|n| format!("{n} ({id})")).unwrap_or_else(|| format!("{id}"));
        usize::try_from(id).ok().and_then(|i| all.get(i).copied().flatten()).ok_or(VtfError::Format(name))
    }

    /// Bytes of one `w` x `h` image.
    pub fn size(self, w: usize, h: usize) -> usize {
        use Format::*;
        match self {
            Dxt1 | Dxt1OneBitAlpha => w.div_ceil(4).max(1) * h.div_ceil(4).max(1) * 8,
            Dxt3 | Dxt5 => w.div_ceil(4).max(1) * h.div_ceil(4).max(1) * 16,
            I8 | A8 => w * h,
            Ia88 | Rgb565 | Bgr565 | Bgrx5551 | Bgra4444 | Bgra5551 | Uv88 => w * h * 2,
            Rgb888 | Bgr888 | Rgb888Bluescreen | Bgr888Bluescreen => w * h * 3,
            Rgba16161616F | Rgba16161616 => w * h * 8,
            _ => w * h * 4,
        }
    }
}

/// Header fields the reader needs.
#[derive(Clone, Debug)]
pub struct Header {
    pub version: u32,
    pub width: u32,
    pub height: u32,
    pub flags: u32,
    pub frames: u32,
    pub format: i32,
    pub mipmaps: u32,
    pub depth: u32,
}

const FLAG_ENVMAP: u32 = 0x4000;

fn u16_at(b: &[u8], o: usize) -> Result<u16, VtfError> {
    b.get(o..o + 2).map(|s| u16::from_le_bytes([s[0], s[1]])).ok_or(VtfError::Truncated)
}

fn u32_at(b: &[u8], o: usize) -> Result<u32, VtfError> {
    b.get(o..o + 4).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]])).ok_or(VtfError::Truncated)
}

pub fn header(b: &[u8]) -> Result<Header, VtfError> {
    if b.get(0..4) != Some(b"VTF\0") {
        return Err(VtfError::NotVtf);
    }

    let (major, minor) = (u32_at(b, 4)?, u32_at(b, 8)?);
    if major != 7 || minor > 5 {
        return Err(VtfError::Version(if major == 7 { minor } else { major * 100 + minor }));
    }

    Ok(Header {
        version: minor,
        width: u16_at(b, 16)? as u32,
        height: u16_at(b, 18)? as u32,
        flags: u32_at(b, 20)?,
        frames: (u16_at(b, 24)? as u32).max(1),
        format: u32_at(b, 52)? as i32,
        mipmaps: (*b.get(56).ok_or(VtfError::Truncated)? as u32).max(1),
        depth: if minor >= 2 { (u16_at(b, 63)? as u32).max(1) } else { 1 },
    })
}

/// Decodes the largest mipmap of the first frame, first face and first slice.
pub fn decode(b: &[u8]) -> Result<Image, VtfError> {
    let h = header(b)?;
    let format = Format::from_id(h.format)?;
    let first_frame = u16_at(b, 26)?;
    let faces = if h.flags & FLAG_ENVMAP == 0 {
        1
    } else if h.version < 5 && first_frame != 0xffff {
        7
    } else {
        6
    };

    // 7.3 and later list their data blocks in a resource directory, older files put the thumbnail and then the
    // mipmaps (smallest first) right after the header.
    let start = if h.version >= 3 {
        let count = u32_at(b, 68)? as usize;
        (0..count.min(32)).find_map(|i| {
            let entry = 80 + i * 8;
            (b.get(entry..entry + 3) == Some(&[0x30, 0, 0])).then(|| u32_at(b, entry + 4).map(|o| o as usize))
        })
    } else {
        let header_size = u32_at(b, 12)? as usize;
        let low_format = u32_at(b, 57)? as i32;
        let low = match low_format {
            -1 => 0,
            id => Format::from_id(id)?.size(*b.get(61).ok_or(VtfError::Truncated)? as usize, *b.get(62).ok_or(VtfError::Truncated)? as usize),
        };
        Some(Ok(header_size + low))
    };
    let mut offset = start.ok_or(VtfError::NoImage)??;
    let (w, hgt) = (h.width as usize, h.height as usize);
    for mip in (1..h.mipmaps).rev() {
        let (mw, mh, md) = ((w >> mip).max(1), (hgt >> mip).max(1), ((h.depth as usize) >> mip).max(1));
        offset += format.size(mw, mh) * md * h.frames as usize * faces;
    }

    let data = b.get(offset..offset + format.size(w, hgt)).ok_or(VtfError::Truncated)?;
    Ok(Image { width: h.width, height: h.height, rgba: to_rgba(format, data, w, hgt) })
}

fn expand5(v: u16) -> u8 {
    ((v << 3) | (v >> 2)) as u8
}

fn expand6(v: u16) -> u8 {
    ((v << 2) | (v >> 4)) as u8
}

fn rgb565(v: u16) -> [u8; 3] {
    [expand5(v >> 11), expand6((v >> 5) & 63), expand5(v & 31)]
}

fn half(bits: u16) -> f32 {
    let sign = if bits & 0x8000 != 0 { -1.0 } else { 1.0 };
    let exp = ((bits >> 10) & 31) as i32;
    let frac = (bits & 1023) as f32;
    sign * match exp {
        0 => frac / 1024.0 * 2f32.powi(-14),
        31 => f32::INFINITY,
        e => (1.0 + frac / 1024.0) * 2f32.powi(e - 15),
    }
}

fn to_rgba(format: Format, d: &[u8], w: usize, h: usize) -> Vec<u8> {
    use Format::*;
    let mut out = vec![255u8; w * h * 4];
    let px = |i: usize, n: usize| &d[i * n..i * n + n];
    match format {
        Dxt1 | Dxt1OneBitAlpha | Dxt3 | Dxt5 => decode_dxt(format, d, w, h, &mut out),
        _ => {
            for i in 0..w * h {
                let rgba: [u8; 4] = match format {
                    Rgba8888 | Uvwq8888 | Uvlx8888 => px(i, 4).try_into().unwrap_or_default(),
                    Abgr8888 => {
                        let p = px(i, 4);
                        [p[3], p[2], p[1], p[0]]
                    }

                    // Despite the name, files from Valve's tools store ARGB8888 as G, B, A, R.
                    Argb8888 => {
                        let p = px(i, 4);
                        [p[3], p[0], p[1], p[2]]
                    }
                    Bgra8888 => {
                        let p = px(i, 4);
                        [p[2], p[1], p[0], p[3]]
                    }
                    Bgrx8888 => {
                        let p = px(i, 4);
                        [p[2], p[1], p[0], 255]
                    }
                    Rgb888 | Rgb888Bluescreen => {
                        let p = px(i, 3);
                        [p[0], p[1], p[2], 255]
                    }
                    Bgr888 | Bgr888Bluescreen => {
                        let p = px(i, 3);
                        [p[2], p[1], p[0], 255]
                    }
                    I8 => [d[i], d[i], d[i], 255],
                    A8 => [0, 0, 0, d[i]],
                    Ia88 => [d[i * 2], d[i * 2], d[i * 2], d[i * 2 + 1]],
                    Uv88 => [d[i * 2], d[i * 2 + 1], 0, 255],
                    Rgb565 | Bgr565 => {
                        let [a, b, c] = rgb565(u16::from_le_bytes([d[i * 2], d[i * 2 + 1]]));
                        if format == Rgb565 { [c, b, a, 255] } else { [a, b, c, 255] }
                    }
                    Bgrx5551 | Bgra5551 => {
                        let v = u16::from_le_bytes([d[i * 2], d[i * 2 + 1]]);
                        let a = if format == Bgra5551 && v & 0x8000 == 0 { 0 } else { 255 };
                        [expand5((v >> 10) & 31), expand5((v >> 5) & 31), expand5(v & 31), a]
                    }
                    Bgra4444 => {
                        let v = u16::from_le_bytes([d[i * 2], d[i * 2 + 1]]);
                        let n = |s: u16| (((v >> s) & 15) * 17) as u8;
                        [n(8), n(4), n(0), n(12)]
                    }
                    Rgba16161616 => {
                        let c = |k: usize| d[i * 8 + k * 2 + 1];
                        [c(0), c(1), c(2), c(3)]
                    }
                    Rgba16161616F => {
                        let c = |k: usize| {
                            let v = half(u16::from_le_bytes([d[i * 8 + k * 2], d[i * 8 + k * 2 + 1]]));
                            if k == 3 { (v.clamp(0.0, 1.0) * 255.0).round() as u8 } else { (v.clamp(0.0, 1.0).powf(1.0 / 2.2) * 255.0).round() as u8 }
                        };
                        [c(0), c(1), c(2), c(3)]
                    }
                    Dxt1 | Dxt1OneBitAlpha | Dxt3 | Dxt5 => unreachable!(),
                };
                let rgba = if matches!(format, Rgb888Bluescreen | Bgr888Bluescreen) && rgba[..3] == [0, 0, 255] { [0, 0, 0, 0] } else { rgba };
                out[i * 4..i * 4 + 4].copy_from_slice(&rgba);
            }
        }
    }

    out
}

fn decode_dxt(format: Format, d: &[u8], w: usize, h: usize, out: &mut [u8]) {
    let block_bytes = if matches!(format, Format::Dxt1 | Format::Dxt1OneBitAlpha) { 8 } else { 16 };
    let (bw, bh) = (w.div_ceil(4).max(1), h.div_ceil(4).max(1));
    for by in 0..bh {
        for bx in 0..bw {
            let block = &d[(by * bw + bx) * block_bytes..][..block_bytes];
            let (alpha, color) = if block_bytes == 16 { (Some(&block[..8]), &block[8..]) } else { (None, block) };
            let c0 = u16::from_le_bytes([color[0], color[1]]);
            let c1 = u16::from_le_bytes([color[2], color[3]]);
            let (p0, p1) = (rgb565(c0), rgb565(c1));
            let mix = |a: u8, b: u8, wa: u16, wb: u16| ((a as u16 * wa + b as u16 * wb) / (wa + wb)) as u8;
            let mut palette = [[p0[0], p0[1], p0[2], 255], [p1[0], p1[1], p1[2], 255], [0; 4], [0; 4]];
            // DXT3 and DXT5 always use the four color mode, DXT1 switches to three colors and transparent black.
            if c0 > c1 || block_bytes == 16 {
                palette[2] = [mix(p0[0], p1[0], 2, 1), mix(p0[1], p1[1], 2, 1), mix(p0[2], p1[2], 2, 1), 255];
                palette[3] = [mix(p0[0], p1[0], 1, 2), mix(p0[1], p1[1], 1, 2), mix(p0[2], p1[2], 1, 2), 255];
            } else {
                palette[2] = [mix(p0[0], p1[0], 1, 1), mix(p0[1], p1[1], 1, 1), mix(p0[2], p1[2], 1, 1), 255];
            }

            let bits = u32::from_le_bytes([color[4], color[5], color[6], color[7]]);
            let alphas: Option<[u8; 16]> = alpha.map(|a| match format {
                Format::Dxt3 => std::array::from_fn(|i| ((a[i / 2] >> ((i % 2) * 4)) & 15) * 17),
                _ => {
                    let (a0, a1) = (a[0] as u16, a[1] as u16);
                    let table: [u8; 8] = std::array::from_fn(|i| match i {
                        0 => a0 as u8,
                        1 => a1 as u8,
                        _ if a0 > a1 => ((a0 * (8 - i as u16) + a1 * (i as u16 - 1)) / 7) as u8,
                        6 => 0,
                        7 => 255,
                        _ => ((a0 * (6 - i as u16) + a1 * (i as u16 - 1)) / 5) as u8,
                    });
                    let idx = u64::from_le_bytes([a[2], a[3], a[4], a[5], a[6], a[7], 0, 0]);
                    std::array::from_fn(|i| table[((idx >> (3 * i)) & 7) as usize])
                }
            });
            for py in 0..4 {
                for px in 0..4 {
                    let (x, y) = (bx * 4 + px, by * 4 + py);
                    if x >= w || y >= h {
                        continue;
                    }

                    let i = py * 4 + px;
                    let mut c = palette[((bits >> (2 * i)) & 3) as usize];
                    if let Some(a) = &alphas {
                        c[3] = a[i];
                    }

                    out[(y * w + x) * 4..][..4].copy_from_slice(&c);
                }
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Writes a VTF with every mip level and frame, each image filled by `fill(mip, frame)` bytes.
    pub fn build(version: u32, format: i32, w: u16, h: u16, frames: u16, mips: u8, fill: impl Fn(usize, usize, usize) -> Vec<u8>) -> Vec<u8> {
        let header_size = if version >= 3 { 80 + 2 * 8 } else { 80 };
        let mut b = vec![0u8; header_size];
        b[0..4].copy_from_slice(b"VTF\0");
        b[4..8].copy_from_slice(&7u32.to_le_bytes());
        b[8..12].copy_from_slice(&version.to_le_bytes());
        b[12..16].copy_from_slice(&(header_size as u32).to_le_bytes());
        b[16..18].copy_from_slice(&w.to_le_bytes());
        b[18..20].copy_from_slice(&h.to_le_bytes());
        b[24..26].copy_from_slice(&frames.to_le_bytes());
        b[52..56].copy_from_slice(&format.to_le_bytes());
        b[56] = mips;
        // A 4 x 4 DXT1 thumbnail, which the reader has to step over.
        b[57..61].copy_from_slice(&13i32.to_le_bytes());
        b[61] = 4;
        b[62] = 4;
        b[63..65].copy_from_slice(&1u16.to_le_bytes());
        let thumb_at = header_size;
        let data_at = thumb_at + 8;
        if version >= 3 {
            b[68..72].copy_from_slice(&2u32.to_le_bytes());
            b[80..84].copy_from_slice(&[0x01, 0, 0, 0]);
            b[84..88].copy_from_slice(&(thumb_at as u32).to_le_bytes());
            b[88..92].copy_from_slice(&[0x30, 0, 0, 0]);
            b[92..96].copy_from_slice(&(data_at as u32).to_le_bytes());
        }

        b.extend([0xAA; 8]);
        let fmt = Format::from_id(format).unwrap();
        for mip in (0..mips as usize).rev() {
            let (mw, mh) = ((w as usize >> mip).max(1), (h as usize >> mip).max(1));
            for frame in 0..frames as usize {
                let data = fill(mip, frame, fmt.size(mw, mh));
                assert_eq!(data.len(), fmt.size(mw, mh));
                b.extend(data);
            }
        }

        b
    }

    #[test]
    fn takes_the_largest_mip_of_the_first_frame() {
        for version in [0, 2, 3, 5] {
            // BGRA8888, the value tells mip and frame apart.
            let vtf = build(version, 12, 8, 4, 3, 4, |mip, frame, n| [10 * mip as u8, 20 + frame as u8, 200, 128].repeat(n / 4));
            let img = decode(&vtf).unwrap_or_else(|e| panic!("7.{version}: {e}"));
            assert_eq!((img.width, img.height), (8, 4));
            assert_eq!(&img.rgba[..4], &[200, 20, 0, 128], "7.{version}: BGRA swizzled, mip 0, frame 0");
        }
    }

    #[test]
    fn decodes_uncompressed_formats() {
        let one = |format: i32, px: &[u8]| {
            let vtf = build(2, format, 1, 1, 1, 1, |_, _, _| px.to_vec());
            decode(&vtf).unwrap().rgba
        };
        assert_eq!(one(0, &[1, 2, 3, 4]), [1, 2, 3, 4]);
        assert_eq!(one(1, &[4, 3, 2, 1]), [1, 2, 3, 4]);
        assert_eq!(one(2, &[1, 2, 3]), [1, 2, 3, 255]);
        assert_eq!(one(3, &[3, 2, 1]), [1, 2, 3, 255]);
        assert_eq!(one(5, &[9]), [9, 9, 9, 255]);
        assert_eq!(one(6, &[9, 7]), [9, 9, 9, 7]);
        assert_eq!(one(8, &[7]), [0, 0, 0, 7]);
        assert_eq!(one(11, &[2, 3, 4, 1]), [1, 2, 3, 4]);
        assert_eq!(one(16, &[3, 2, 1, 0]), [1, 2, 3, 255]);
        assert_eq!(one(10, &[255, 0, 0]), [0, 0, 0, 0], "blue screen is transparent");
        assert_eq!(one(17, &0xF800u16.to_le_bytes()), [255, 0, 0, 255], "BGR565 is D3D's R5G6B5, red in the high bits");
        assert_eq!(one(4, &0xF800u16.to_le_bytes()), [0, 0, 255, 255]);
        assert_eq!(one(19, &0xF0F0u16.to_le_bytes()), [0, 255, 0, 255]);
    }

    #[test]
    fn decodes_dxt_blocks() {
        // Pure red and pure blue endpoints, pixel 0 is color 0, pixel 1 color 1, pixel 2 the 2/3 blend.
        let red = 0xF800u16.to_le_bytes();
        let blue = 0x001Fu16.to_le_bytes();
        let dxt1 = [red[0], red[1], blue[0], blue[1], 0b10_01_00, 0, 0, 0];
        let img = decode(&build(5, 13, 4, 4, 1, 1, |_, _, _| dxt1.to_vec())).unwrap();
        assert_eq!(&img.rgba[0..4], &[255, 0, 0, 255]);
        assert_eq!(&img.rgba[4..8], &[0, 0, 255, 255]);
        assert_eq!(&img.rgba[8..12], &[170, 0, 85, 255]);

        // c0 <= c1 selects three colors and a transparent index 3, the one bit alpha of DXT1.
        let cut = [blue[0], blue[1], red[0], red[1], 0b11, 0, 0, 0];
        let img = decode(&build(5, 20, 4, 4, 1, 1, |_, _, _| cut.to_vec())).unwrap();
        assert_eq!(&img.rgba[0..4], &[0, 0, 0, 0]);

        // DXT5 alpha: endpoints 255 and 0, pixel 0 uses a0, pixel 1 a1, pixel 2 index 2 = (6 * 255 + 0) / 7.
        let mut dxt5 = vec![255, 0, 0b10_001_000, 0, 0, 0, 0, 0];
        dxt5.extend(dxt1);
        let img = decode(&build(5, 15, 4, 4, 1, 1, |_, _, _| dxt5.clone())).unwrap();
        assert_eq!([img.rgba[3], img.rgba[7], img.rgba[11]], [255, 0, 218]);

        // DXT3: 4 bit explicit alpha, 0xF then 0x0.
        let mut dxt3 = vec![0x0F, 0, 0, 0, 0, 0, 0, 0];
        dxt3.extend(dxt1);
        let img = decode(&build(5, 14, 4, 4, 1, 1, |_, _, _| dxt3.clone())).unwrap();
        assert_eq!([img.rgba[3], img.rgba[7]], [255, 0]);
    }

    #[test]
    fn non_power_of_two_dxt_edges_are_clipped() {
        let img = decode(&build(2, 13, 6, 2, 1, 1, |_, _, n| vec![0; n])).unwrap();
        assert_eq!(img.rgba.len(), 6 * 2 * 4);
    }

    #[test]
    fn refuses_what_it_cannot_read() {
        assert_eq!(decode(b"DDS "), Err(VtfError::NotVtf));
        let mut newer = build(5, 13, 4, 4, 1, 1, |_, _, n| vec![0; n]);
        newer[8] = 6;
        assert_eq!(decode(&newer), Err(VtfError::Version(6)));
        let p8 = build(2, 0, 4, 4, 1, 1, |_, _, n| vec![0; n]);
        let mut p8 = p8;
        p8[52..56].copy_from_slice(&7i32.to_le_bytes());
        assert_eq!(decode(&p8), Err(VtfError::Format("P8 (7)".into())));
        let mut unknown = build(2, 0, 4, 4, 1, 1, |_, _, n| vec![0; n]);
        unknown[52..56].copy_from_slice(&70i32.to_le_bytes());
        assert_eq!(decode(&unknown), Err(VtfError::Format("70".into())));
        let whole = build(2, 12, 4, 4, 1, 1, |_, _, n| vec![0; n]);
        assert_eq!(decode(&whole[..whole.len() - 1]), Err(VtfError::Truncated));
    }
}
