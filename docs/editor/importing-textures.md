# Converting textures

Godot cannot load `.vtf`, `.wad` or `.wal` files, so the editor converts them into PNGs, plus a `.tres` where a plain
image cannot carry the settings. Each one lands in the project's texture folder under the face's material name, so the
imported faces find it.

The import offers this for the textures a map uses. For a whole folder, use *File > Import > Convert Textures (VTF/VMT,
WAD, WAL)*, or `map_file {op: convert_textures, path}` over MCP.

## Where they are found

| Source | Looked for | Named after |
| --- | --- | --- |
| Valve `.vmt` and `.vtf` | a `materials` folder next to the map or up to four folders up | the path below `materials`, like `brick/brickwall001a` |
| Quake and Half-Life `.wad` | the worldspawn `wad` key, then the same file name in nearby folders | the texture name, like `{grate` |
| Quake 2 `.wal` | a `textures` folder nearby | the path below `textures`, like `e1u1/floor1_1` |
| Loose `.png`, `.jpg`, `.tga` | a `textures` folder nearby, as Qodot and FuncGodot projects ship them | the path below `textures`, copied as they are |

## Valve materials

| Valve material | Godot material |
| --- | --- |
| `$basetexture` | albedo, its alpha dropped unless the material is see through |
| `$bumpmap`, or `$normalmap` on water | normal map, green flipped from DirectX to Godot |
| `$translucent`, `$additive` | alpha blend, additive blend |
| `$alphatest`, `$alphatestreference` | alpha scissor at that threshold, 0.5 by default |
| `$selfillum`, `$selfillummask` | emission texture masked by the base alpha or the mask |
| `$nocull`, `UnlitGeneric` | double sided, unshaded |
| `$color` | albedo color |
| `$surfaceprop` | `metadata/surfaceprop` on the material |
| `patch` with `include` | the included material with `insert` and `replace` applied |

## Quake textures

* Names starting with `{` cut out their last palette color with alpha scissor.
* Skies keep their back layer, unshaded.
* `*water` loses the `*` in its file name.
* Quake 2 translucency flags become alpha blending.
* Animated textures (`+0lava`, multi frame VTFs) keep their first frame.

## Skies

Converting a map's textures also sets its sky, unless the map already has `sky_top_color`.

| Map | Sky |
| --- | --- |
| Source, with the `skyname` skybox VTFs in `materials/skybox` | The six faces become one panorama, `skybox/<skyname>_panorama.png` in the texture folder, set as `sky_panorama`. Its zenith and horizon give the colors |
| Quake and others, from the `sky_source` texture | The darker half of the texture, the back layer for Quake, becomes `sky_top_color` and the lighter half `sky_horizon_color` |

`GodotTrenchEnvironment` builds a WorldEnvironment from these keys when the scene has none of its own.

## VTF formats

VTF 7.0 to 7.5: DXT1 (with one bit alpha), DXT3, DXT5, the 8 bit RGB and RGBA orders, I8, IA88, A8, RGB565, BGR565,
BGRA4444, BGRA5551, BGRX5551, UV88, UVWQ8888, UVLX8888 and the 16 bit RGBA formats. Others, and VTF 7.6, are refused
with the format named.
