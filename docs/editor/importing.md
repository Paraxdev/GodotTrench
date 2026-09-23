# Importing Hammer and TrenchBroom maps

*File > Import* opens a Hammer `.vmf` or a TrenchBroom, Quake or Qodot `.map` as a new unsaved map. Save it as a
`.gtm` and Godot builds it like any other map, with the textures the editor converted for it.

## Steps

1. Open the Godot project the map is for, converted textures go into its texture folder.
2. *File > Import > .vmf (Hammer)* or *.map (TrenchBroom, Quake)* and pick the file.
3. When textures for the map are found next to it, the editor offers to convert the ones the project lacks. Say yes.
4. Check the Issues panel. `missing_material` lists every material the project still lacks, with how many faces use
   it. Those faces keep the name, so adding the texture later fixes them.
5. *File > Save As* a `.gtm` inside the project.

Textures kept elsewhere are converted with *File > Import > Convert Textures (VTF/VMT, WAD, WAL)*. Pick a folder, and
when the open map is missing some of its textures you can convert only those. Over MCP it is
`map_file {op: convert_textures, path}`, and `import_vmf` or `import_map` take `textures: "auto"` or a folder.

## Where textures are found

| Source | Looked for | Named after |
| --- | --- | --- |
| Valve `.vmt` and `.vtf` | a `materials` folder next to the map or up to four folders up | the path below `materials`, like `brick/brickwall001a` |
| Quake and Half-Life `.wad` | the worldspawn `wad` key, then the same file name in nearby folders | the texture name, like `{grate` |
| Quake 2 `.wal` | a `textures` folder nearby | the path below `textures`, like `e1u1/floor1_1` |
| Loose `.png`, `.jpg`, `.tga` | a `textures` folder nearby, as Qodot and FuncGodot projects ship them | the path below `textures`, copied as they are |

Every texture becomes a PNG under the project's texture folder with the face's material name. A `.tres` next to it
carries what a plain image cannot, so Godot and the editor show the same thing.

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

Quake textures starting with `{` cut out their last palette color with alpha scissor. A Quake sky keeps its back
layer, unshaded. `*water` loses the `*` in its file name, which the addon and the editor both look past. Quake 2
translucency flags on a `.wal` become alpha blending. Animated textures (`+0lava`, multi frame VTFs) keep their
first frame.

VTF versions 7.0 to 7.5 are read in these formats: DXT1 (with one bit alpha), DXT3, DXT5, RGBA8888, ABGR8888,
ARGB8888, BGRA8888, BGRX8888, RGB888, BGR888, the blue screen variants, I8, IA88, A8, RGB565, BGR565, BGRA4444,
BGRA5551, BGRX5551, UV88, UVWQ8888, UVLX8888 and the 16 bit RGBA formats. Others, and VTF 7.6, are refused with the
format named in the report.

## What the import keeps

| Hammer | In GodotTrench |
| --- | --- |
| Tool materials (`tools/toolsnodraw`, `toolsskybox`, `toolsclip`, `toolsplayerclip`, `toolstrigger`, `toolsorigin`, ...) | The project's skip, clip and origin textures and `special/trigger` |
| Displacements | Displacements with their heights and blend alphas |
| Displacements whose vertices move sideways | Smooth meshes with the exact positions, since a displacement here only moves along the face normal |
| `func_instance` | A group with the instance's contents, placed, renamed by its fixup and with `$` and `#` replacements applied. `instance:name;Input` wiring is resolved |
| Visgroups | Layers |
| Hidden objects | Hidden nodes |
| Entity keys and outputs | Entity properties and outputs |

TrenchBroom layers and groups stay layers and groups. Quake tool textures (`skip`, `clip`, `hint`, `trigger`,
`origin`, Quake 3 `common/caulk`) become the project's tool textures the same way.

> **Note:** Units are kept as they are. A Source map is built for about 40 units per meter and a Quake map for about
> 32, so a Source map comes out a little larger than in its game unless the project uses a different scale.

## Known limitations

| Limitation | What to do |
| --- | --- |
| Textures inside `.vpk` or `.pak` archives are not read | Extract them first, for example with VPKEdit |
| Instances are looked for next to the map and in the folders above it | Keep the game's `maps` folder layout, missing ones stay `func_instance` entities |
| Valve and Quake entity classes have no definitions in a new project | Add them to your FGD, or replace them, the Issues panel lists them |
| `$basetexture2` blends, detail textures, env maps and proxies are left out | Set them up in Godot |
| Quake 3 patches and brush primitives are skipped, the status bar counts them | Rebuild them as meshes |
| A displacement turned into a mesh keeps no blend alphas and no longer sculpts | Edit it as a mesh |
