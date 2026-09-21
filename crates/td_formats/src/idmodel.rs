//! id Software model formats: Quake II `.md2` and Quake III `.md3`. Only the first animation frame is
//! read, which is what the editor needs to preview and place a static prop. Both are parsed into the
//! same [`IdModel`] of surfaces with positions, normals and UVs.

/// One expanded vertex, ready for the renderer.
#[derive(Clone, Copy, Debug)]
pub struct IdVertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

/// One drawable surface (md3 has several, md2 has one), with its skin path if the file names one.
pub struct IdSurface {
    pub skin: Option<String>,
    pub vertices: Vec<IdVertex>,
    pub indices: Vec<u32>,
}

pub struct IdModel {
    pub surfaces: Vec<IdSurface>,
}

/// Little-endian reader that returns an error instead of panicking on a short or malformed file.
struct Reader<'a> {
    data: &'a [u8],
}

impl<'a> Reader<'a> {
    fn slice(&self, at: usize, len: usize) -> Result<&'a [u8], String> {
        self.data.get(at..at + len).ok_or_else(|| "unexpected end of model file".to_string())
    }
    fn i32(&self, at: usize) -> Result<i32, String> {
        Ok(i32::from_le_bytes(self.slice(at, 4)?.try_into().unwrap()))
    }
    fn u32(&self, at: usize) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.slice(at, 4)?.try_into().unwrap()))
    }
    fn f32(&self, at: usize) -> Result<f32, String> {
        Ok(f32::from_le_bytes(self.slice(at, 4)?.try_into().unwrap()))
    }
    fn i16(&self, at: usize) -> Result<i16, String> {
        Ok(i16::from_le_bytes(self.slice(at, 2)?.try_into().unwrap()))
    }
    fn u16(&self, at: usize) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.slice(at, 2)?.try_into().unwrap()))
    }

    /// A fixed-length, nul-terminated name.
    fn name(&self, at: usize, len: usize) -> Result<String, String> {
        let raw = self.slice(at, len)?;
        let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
        Ok(String::from_utf8_lossy(&raw[..end]).into_owned())
    }
}

fn count(n: i32, what: &str) -> Result<usize, String> {
    if n < 0 { Err(format!("negative {what} count")) } else { Ok(n as usize) }
}

// --------------------------------------------------------------------------- md2

const MD2_MAGIC: u32 = u32::from_le_bytes(*b"IDP2");
const MD3_MAGIC: u32 = u32::from_le_bytes(*b"IDP3");

/// Parses a Quake II `.md2`, expanding its triangles into a single surface with computed normals.
pub fn parse_md2(data: &[u8]) -> Result<IdModel, String> {
    let r = Reader { data };
    if r.u32(0)? != MD2_MAGIC {
        return Err("not an md2 file (bad magic)".into());
    }

    if r.i32(4)? != 8 {
        return Err("unsupported md2 version".into());
    }

    let skinwidth = r.i32(8)?.max(1) as f32;
    let skinheight = r.i32(12)?.max(1) as f32;
    let num_skins = count(r.i32(24)?, "skin")?;
    let num_xyz = count(r.i32(28)?, "vertex")?;
    let num_st = count(r.i32(32)?, "texcoord")?;
    let num_tris = count(r.i32(36)?, "triangle")?;
    let num_frames = count(r.i32(44)?, "frame")?;
    let ofs_skins = r.i32(48)? as usize;
    let ofs_st = r.i32(52)? as usize;
    let ofs_tris = r.i32(56)? as usize;
    let ofs_frames = r.i32(60)? as usize;
    let framesize = r.i32(20)? as usize;
    if num_frames == 0 {
        return Err("md2 has no frames".into());
    }

    let skin = (num_skins > 0).then(|| r.name(ofs_skins, 64)).transpose()?.filter(|s| !s.is_empty());

    // Texture coordinates, stored as skin-space integers.
    let mut st = Vec::with_capacity(num_st);
    for i in 0..num_st {
        let o = ofs_st + i * 4;
        st.push([r.i16(o)? as f32 / skinwidth, r.i16(o + 2)? as f32 / skinheight]);
    }

    // Frame 0 vertices: each is a byte position scaled and translated by the frame header.
    let sc = |k: usize| r.f32(ofs_frames + k * 4);
    let (scale, translate) = ([sc(0)?, sc(1)?, sc(2)?], [sc(3)?, sc(4)?, sc(5)?]);
    let verts_at = ofs_frames + 40; // 6 floats + 16-byte name
    let mut positions = Vec::with_capacity(num_xyz);
    for i in 0..num_xyz {
        let o = verts_at + i * 4;
        let b = r.slice(o, 3)?;
        positions.push([b[0] as f32 * scale[0] + translate[0], b[1] as f32 * scale[1] + translate[1], b[2] as f32 * scale[2] + translate[2]]);
    }

    let _ = framesize;

    let mut vertices = Vec::with_capacity(num_tris * 3);
    let mut indices = Vec::with_capacity(num_tris * 3);
    for i in 0..num_tris {
        let o = ofs_tris + i * 12;
        let vi = [r.u16(o)? as usize, r.u16(o + 2)? as usize, r.u16(o + 4)? as usize];
        let sti = [r.u16(o + 6)? as usize, r.u16(o + 8)? as usize, r.u16(o + 10)? as usize];
        let p: Vec<[f32; 3]> = vi.iter().map(|&k| *positions.get(k).unwrap_or(&[0.0; 3])).collect();
        let n = face_normal(p[0], p[1], p[2]);
        for k in 0..3 {
            indices.push(vertices.len() as u32);
            vertices.push(IdVertex { pos: p[k], normal: n, uv: *st.get(sti[k]).unwrap_or(&[0.0, 0.0]) });
        }
    }

    Ok(IdModel { surfaces: vec![IdSurface { skin, vertices, indices }] })
}

// --------------------------------------------------------------------------- md3

const MD3_XYZ_SCALE: f32 = 1.0 / 64.0;

/// Parses a Quake III `.md3`, reading every surface's first frame with its per-vertex UVs and normals.
pub fn parse_md3(data: &[u8]) -> Result<IdModel, String> {
    let r = Reader { data };
    if r.u32(0)? != MD3_MAGIC {
        return Err("not an md3 file (bad magic)".into());
    }

    if r.i32(4)? != 15 {
        return Err("unsupported md3 version".into());
    }

    let num_surfaces = count(r.i32(84)?, "surface")?;
    let mut ofs = r.i32(96)? as usize; // ofs_surfaces
    let mut surfaces = Vec::with_capacity(num_surfaces);
    for _ in 0..num_surfaces {
        let s = Reader { data };
        if s.u32(ofs)? != MD3_MAGIC {
            return Err("bad md3 surface magic".into());
        }

        let num_frames = count(s.i32(ofs + 72)?, "surface frame")?;
        let num_shaders = count(s.i32(ofs + 76)?, "shader")?;
        let num_verts = count(s.i32(ofs + 80)?, "surface vertex")?;
        let num_tris = count(s.i32(ofs + 84)?, "surface triangle")?;
        let ofs_tris = ofs + s.i32(ofs + 88)? as usize;
        let ofs_shaders = ofs + s.i32(ofs + 92)? as usize;
        let ofs_st = ofs + s.i32(ofs + 96)? as usize;
        let ofs_xyz = ofs + s.i32(ofs + 100)? as usize;
        let ofs_end = s.i32(ofs + 104)? as usize;
        if num_frames == 0 {
            return Err("md3 surface has no frames".into());
        }

        let skin = (num_shaders > 0).then(|| s.name(ofs_shaders, 64)).transpose()?.filter(|n| !n.is_empty());

        let mut vertices = Vec::with_capacity(num_verts);
        for i in 0..num_verts {
            let st_o = ofs_st + i * 8;
            let uv = [s.f32(st_o)?, s.f32(st_o + 4)?];
            let xyz_o = ofs_xyz + i * 8; // frame 0
            let pos = [s.i16(xyz_o)? as f32 * MD3_XYZ_SCALE, s.i16(xyz_o + 2)? as f32 * MD3_XYZ_SCALE, s.i16(xyz_o + 4)? as f32 * MD3_XYZ_SCALE];
            let normal = decode_md3_normal(s.u16(xyz_o + 6)?);
            vertices.push(IdVertex { pos, normal, uv });
        }

        let mut indices = Vec::with_capacity(num_tris * 3);
        for i in 0..num_tris {
            let o = ofs_tris + i * 12;
            indices.extend([s.u32(o)?, s.u32(o + 4)?, s.u32(o + 8)?]);
        }

        surfaces.push(IdSurface { skin, vertices, indices });
        if ofs_end == 0 {
            break;
        }

        ofs += ofs_end;
    }

    Ok(IdModel { surfaces })
}

/// md3 packs a normal into two bytes as spherical latitude and longitude.
fn decode_md3_normal(n: u16) -> [f32; 3] {
    let lat = ((n >> 8) & 0xff) as f32 * std::f32::consts::TAU / 255.0;
    let lng = (n & 0xff) as f32 * std::f32::consts::TAU / 255.0;
    [lat.cos() * lng.sin(), lat.sin() * lng.sin(), lng.cos()]
}

fn face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len > 1e-8 { [n[0] / len, n[1] / len, n[2] / len] } else { [0.0, 1.0, 0.0] }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put_i32(v: &mut [u8], at: usize, n: i32) {
        v[at..at + 4].copy_from_slice(&n.to_le_bytes());
    }

    #[test]
    fn md2_single_triangle_round_trips() {
        // One skin, three texcoords, three vertices, one triangle, one frame.
        let mut d = vec![0u8; 68];
        put_i32(&mut d, 0, MD2_MAGIC as i32);
        put_i32(&mut d, 4, 8);
        put_i32(&mut d, 8, 64); // skinwidth
        put_i32(&mut d, 12, 64); // skinheight
        put_i32(&mut d, 24, 1); // num_skins
        put_i32(&mut d, 28, 3); // num_xyz
        put_i32(&mut d, 32, 3); // num_st
        put_i32(&mut d, 36, 1); // num_tris
        put_i32(&mut d, 44, 1); // num_frames
        let ofs_skins = d.len();
        put_i32(&mut d, 48, ofs_skins as i32);
        d.extend(b"skins/hero.png");
        d.resize(ofs_skins + 64, 0);
        let ofs_st = d.len();
        put_i32(&mut d, 52, ofs_st as i32);
        for (s, t) in [(0i16, 0i16), (64, 0), (0, 64)] {
            d.extend(s.to_le_bytes());
            d.extend(t.to_le_bytes());
        }

        let ofs_tris = d.len();
        put_i32(&mut d, 56, ofs_tris as i32);
        for idx in [0u16, 1, 2, 0, 1, 2] {
            d.extend(idx.to_le_bytes());
        }

        let ofs_frames = d.len();
        put_i32(&mut d, 60, ofs_frames as i32);
        for f in [1.0f32, 1.0, 1.0, 0.0, 0.0, 0.0] {
            d.extend(f.to_le_bytes());
        }

        d.extend([0u8; 16]); // frame name
        d.extend([0, 0, 0, 0]); // vertex 0 (x,y,z,normal index)
        d.extend([10, 0, 0, 0]); // vertex 1
        d.extend([0, 10, 0, 0]); // vertex 2

        let model = parse_md2(&d).unwrap();
        assert_eq!(model.surfaces.len(), 1);
        let s = &model.surfaces[0];
        assert_eq!(s.skin.as_deref(), Some("skins/hero.png"));
        assert_eq!(s.vertices.len(), 3);
        assert_eq!(s.indices, vec![0, 1, 2]);
        assert_eq!(s.vertices[1].pos, [10.0, 0.0, 0.0]);
        assert_eq!(s.vertices[1].uv, [1.0, 0.0]);
        // The triangle lies in the XY plane, so its normal is ±Z.
        assert!((s.vertices[0].normal[2].abs() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn md3_single_triangle_round_trips() {
        let mut d = vec![0u8; 108];
        put_i32(&mut d, 0, MD3_MAGIC as i32);
        put_i32(&mut d, 4, 15);
        put_i32(&mut d, 84, 1); // num_surfaces
        let ofs_surface = 108;
        put_i32(&mut d, 96, ofs_surface);
        // Surface header is 108 bytes.
        let mut surf = vec![0u8; 108];
        put_i32(&mut surf, 0, MD3_MAGIC as i32);
        put_i32(&mut surf, 72, 1); // num_frames
        put_i32(&mut surf, 76, 1); // num_shaders
        put_i32(&mut surf, 80, 3); // num_verts
        put_i32(&mut surf, 84, 1); // num_tris
        // Layout after headers: shaders, triangles, st, xyz.
        let shaders_at = 108;
        let name = b"models/box.tga";
        let mut shader = vec![0u8; 68];
        shader[..name.len()].copy_from_slice(name);
        let tris_at = shaders_at + 68;
        let st_at = tris_at + 12;
        let xyz_at = st_at + 3 * 8;
        let end = xyz_at + 3 * 8;
        put_i32(&mut surf, 88, tris_at); // ofs_triangles (relative to surface)
        put_i32(&mut surf, 92, shaders_at);
        put_i32(&mut surf, 96, st_at);
        put_i32(&mut surf, 100, xyz_at);
        put_i32(&mut surf, 104, end);

        d.extend(&surf); // header
        d.extend(&shader); // shaders
        for idx in [0i32, 1, 2] {
            d.extend(idx.to_le_bytes());
        }

        for uv in [[0.0f32, 0.0], [1.0, 0.0], [0.0, 1.0]] {
            d.extend(uv[0].to_le_bytes());
            d.extend(uv[1].to_le_bytes());
        }

        for xyz in [[0i16, 0, 0], [64, 0, 0], [0, 64, 0]] {
            for c in xyz {
                d.extend(c.to_le_bytes());
            }

            d.extend(0u16.to_le_bytes()); // normal
        }

        let model = parse_md3(&d).unwrap();
        assert_eq!(model.surfaces.len(), 1);
        let s = &model.surfaces[0];
        assert_eq!(s.skin.as_deref(), Some("models/box.tga"));
        assert_eq!(s.vertices.len(), 3);
        assert_eq!(s.indices, vec![0, 1, 2]);
        assert_eq!(s.vertices[1].pos, [1.0, 0.0, 0.0]); // 64 * (1/64)
        assert_eq!(s.vertices[2].uv, [0.0, 1.0]);
    }
}
