# Working with Godot

GodotTrench works on its own. A Godot editor with the GodotTrench addon (the FuncGodot fork in `godot/addons/func_godot`)
adds rebuilds on save, live mode and a one click way to open the project.

## Finding Godot

At startup GodotTrench looks for the Godot executable in this order:

1. *Godot executable* in File > Preferences
2. the `GODOT` environment variable
3. `PATH`: `godot`, `godot4` and downloaded builds like `Godot_v4.7.2-stable_win64.exe` (console builds are skipped, mono
   builds are preferred when the project has a `.csproj`)
4. common install folders: WinGet links, Scoop shims, Steam and `Programs/Godot`

When nothing is found the status bar says so and *Run Project* and *Open Project in Godot Editor* are disabled. Everything
else keeps working, including the live link to a Godot editor that was started some other way. Preferences shows the
executable in use.

## The Godot button

The Godot robot at the right end of the toolbar:

| State | Robot | Click |
|---|---|---|
| no project open | grey | nothing |
| Godot not found and no editor connected | grey, the tooltip explains how to set the path | nothing |
| Godot found | normal | opens the project in the Godot editor |
| a Godot editor has this project open | blue | brings that editor to the front |

The link button next to it switches live mode, a spinner shows while Godot is building.

## Saving and building

* **Save** writes the map and Godot does a full rebuild of every `FuncGodotMap` in the edited scene that uses it, as before.
  A running game with the `GodotTrenchHotReload` autoload reloads it too.
* **Godot > Build in Godot** does a full build of the map as it is in GodotTrench, saved or not, without writing the file.
* **Build Map** on the `FuncGodotMap` node in Godot does a full build of the saved file.

Builds only replace the children of the `FuncGodotMap` node. Nodes you add next to it in the scene are never touched.

Full builds read `.gtm` files in one go, convert brushes and terrain chunks on the WorkerThreadPool and create meshes and
shapes on the main thread. The project setting `godottrench/threaded_build` turns the threaded steps off.
`godot --headless --path godot --script res://tests/bench_build.gd -- runs=5 threaded=1` times every build step of the
showcase maps.

## Materials and texture size

A face's material name is a path under the texture folder without an extension, `showcase/planks` for example.
FuncGodot uses `showcase/planks.tres` when that material file exists and otherwise builds a material from the image
with the same name. The editor previews the same file, reading its albedo, normal and emission maps, transparency,
culling and filtering.

Face UVs are measured in texels, and by default a texel is one pixel of the albedo image, so a 64 pixel texture repeats
every 64 map units. That suits pixel art but not photos: a 1024 pixel scan of a two meter wall would stretch over 32
meters. A material can say how much of the world one repeat covers instead:

```
[resource]
albedo_texture = ExtResource("1_albedo")
metadata/texture_size = Vector2(64, 64)
```

The value is in map units (32 per meter in the demo), so this material repeats every two meters whatever its
resolution. It also accepts a `Vector2i` or a single number for a square size, and it can be set in Godot's inspector
under *Metadata*. The editor and the addon both honour it everywhere they derive UVs from a texture's size: brush and
mesh faces, displacements, decals, the Texture tool and UV editor, fit and justify, hotspots (still drawn in pixels,
scaled to the world size) and the MCP texture operations. On a face with a blend material the painted texture keeps
its own world size. Terrain layers are not affected, their `tile` value already sets the repeat in map units. Swapping
an image for one with more pixels then leaves every map looking the same, only sharper.

The demo textures are CC0 photo scans from Poly Haven and ambientCG, listed with their sources in
`godot/demo/textures/CREDITS.md`. `python tools/fetch_demo_textures.py` downloads them again, writes each material with
the real size of its scan as `texture_size`, and sets the import options photos need (mipmaps, VRAM compression and
normal map compression), since Godot only picks those by itself once its editor sees a texture used in 3D.

## Emissive materials and night maps

A material glows when its file enables emission. Godot computes the glow as the emission color plus the emission
texture, times the energy, or the color times the texture when `emission_operator = 1`. With the default black color
an emission texture therefore glows exactly as painted:

```
[resource]
albedo_texture = ExtResource("1_albedo")
emission_enabled = true
emission = Color(0, 0, 0, 1)
emission_energy_multiplier = 2.5
emission_texture = ExtResource("2_emission")
```

Without a material file, an image named like the albedo with `_emission` added (`night/sign_emission.png` next to
`night/sign.png`) becomes the emission map of the material the addon generates, following the map settings'
`emission_map_pattern`. The editor treats it as a companion map: it is hidden in the Materials panel and the albedo
gets a *glow* badge, as every emissive material does. The MCP `texture` op `material_info` returns the emission color,
energy, texture and operator.

The lit view adds emission to the light a surface receives before tone mapping, so emissive surfaces stay bright in
the dark and saturate like a lamp instead of clipping. The textured view brightens them towards their glow color. In
the built scene the emission of a face also survives decals, which duplicate the material, and blends, which carry the
emission color, energy, texture and operator of both sides and mix them with the painted weight. Blockbench textures
set to the emissive render mode glow with their own image, and glTF props keep the emissive materials they ship with:
`emissiveFactor`, `emissiveTexture` and `KHR_materials_emissive_strength`, previewed the way Godot's glTF importer reads
them (with an emissive texture the texture alone glows). Their `alphaMode` carries over too: `BLEND` glass and flames
are see through and `MASK` cards cut out at `alphaCutoff`, in the editor as in Godot. The Models panel thumbnails show the glow too. A terrain layer
takes the emission of its material as well, weighted by how much of that layer is painted, in the editor and in the
built `GodotTrenchTerrain` alike, so a lava or neon layer glows where it is painted.

Thin cutouts such as wire fences read better with alpha hash (`transparency = 3`) than with an alpha scissor: their
mipmaps average the wires below any fixed threshold, so a scissored fence vanishes a few meters away, while a hashed
one fades into a dither. The editor previews both.

For a night map the worldspawn keys cover the sky and the ambient light: a low `sun_energy` with a pale blue
`sun_color` makes the sun a moon, `ambient_energy` and `sky_energy` scale the ambient light and the sky down, and
`glow_intensity` above zero turns on Godot's glow so lamps and emissive materials bloom. The editor's lit view
previews the same keys (glow excepted), fog and up to 64 point and spot lights, the brightest first. Lights that
start switched off stay out of the preview. `light` and `light_spot` take a `shadows` key for the lamps that should
cast shadows. A lamp fixture can follow its light: give the glowing brush a `func_illusionary` entity with a
targetname and put that name in the light's `fixture` key. The fixture hides while the light is off, or with
`fixture_off dark` stays visible with its emission off, and the lit view previews fixtures of lights that start off
the same way.
`examples/mcp/night_district.json` builds a whole night district this way.

## Live mode

Turn it on with the link button, Godot > Live Mode or *Godot live mode* in Preferences (off by default). While the Godot
editor shows a scene with a `FuncGodotMap` using the current map, edits appear in Godot right away, before you save:

* moving point entities, brush entities, terrains and scatter sets moves their node while you drag
* changing an entity, terrain or scatter set rebuilds just that node
* changing loose brushes splits the worldspawn into chunks of `godottrench/live_chunk_size` meters (16 by default) the
  first time, then only rebuilds the chunks a change touches, a dragged brush gets a node of its own until you let go
* layer omission, prefab instances and anything inside them rebuild the whole map once editing pauses

The preview is close to a full build but not always identical: faces covered by coplanar faces of brush entities or of
chunks that were not rebuilt may show until the next full build, and rebuilt nodes move to the end of their parent in
Godot's Scene dock (putting them back in place trips a Scene dock bug in Godot 4.7). Saving always does a full build. When you throw away unsaved changes
(Discard on quit) Godot goes back to the saved file. New and Open never replace a modified map, they open in a new tab instead.

Edits only go out when Godot has taken the previous batch, and a drag sends small `translate` messages instead of the
geometry, so a slow Godot editor never piles up work.

## Live link protocol

Newline delimited JSON over TCP on `127.0.0.1:7842` (`godottrench/live_link_port`). Every request carries a `seq` that the
reply echoes. The editor keeps one connection open and sends a `status` heartbeat every second.

| Event | Fields | Reply |
|---|---|---|
| `status` | | `project`, `godot`, `pid`, `maps: [{path, epoch}]` for the edited scene |
| `map_saved` | `path` | `rebuilt` |
| `build` | `path`, `text` | `rebuilt` |
| `focus` | | |
| `live_begin`, `live_resync` | `path`, `text` | `epoch` |
| `live_delta` | `path`, `ops` | `epoch`, or `ok: false, resync: true` when the ops do not fit |
| `live_end` | `path`, `revert` | |
| `inspect` | `path`, `ids` | generated node name, class and position per map node id |
| `export_game_config` | | |

`ops` are `set {id, parent, index, node}` (the node as stored in the `.gtm` file, without children), `remove {id}` (the top
most removed node), `translate {ids, offset}` and `properties {properties}`. The build `epoch` of a map goes up whenever it
is built outside a live session or its `FuncGodotMap` nodes change, and the editor then starts over with `live_begin`.
Generated nodes carry their map node id in the `_gt_id` metadata (`_gt_group` for layers and groups).
