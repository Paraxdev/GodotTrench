# Materials and lighting

## How a material is found

A face's material is a path under the texture folder without an extension, for example `showcase/planks`.

| If this exists | The build uses |
| --- | --- |
| `showcase/planks.tres` | That material file as it is |
| Only the image | A plain material the addon generates |

The editor previews the same `.tres`, including transparency, emission and normal maps, so both sides look alike.

## Texture size

UVs count texels, and by default a texel is one pixel. A 64 pixel texture then repeats every 64 units, two meters.
That suits pixel art, but a 1024 pixel photo of a two meter wall would stretch over 32 meters.

Fix it by telling the material how much world one repeat covers, in map units:

```
[resource]
albedo_texture = ExtResource("1_albedo")
metadata/texture_size = Vector2(64, 64)
```

You can also add it in Godot's inspector under *Metadata*, as a `Vector2` or a single number. The editor and the build
both honour it everywhere: faces, displacements, decals, the Texture tool, the UV Editor and hotspots. Swapping an image
for a sharper one then no longer changes the map.

Terrain layers ignore it, their `tile` setting already measures the repeat in map units.

> **Note:** The demo textures are CC0 scans from Poly Haven and ambientCG. `python tools/fetch_demo_textures.py`
> downloads them again and writes each material with its real `texture_size`.

## Glowing materials

A material glows when its file sets `emission_enabled`. With the default black emission color, an emission texture glows
exactly as painted:

```
[resource]
albedo_texture = ExtResource("1_albedo")
emission_enabled = true
emission = Color(0, 0, 0, 1)
emission_energy_multiplier = 2.5
emission_texture = ExtResource("2_emission")
```

Without a material file, an image named like the albedo with `_emission` added, `night/sign_emission.png` next to
`night/sign.png`, becomes the glow map.

Emissive materials carry a glow badge in the Materials panel. Decals, blends and terrain layers keep the glow of the
materials they are made from, and glTF props keep the emissive materials they ship with.

## Cutouts

For thin cutouts like wire fences, use alpha hash (`transparency = 3`) rather than alpha scissor. A scissored fence
vanishes a few meters away, a hashed one fades into a dither.

## Night maps

Night lighting is set on worldspawn. Select nothing and edit it in the Inspector.

| Key | Effect |
| --- | --- |
| `sun_energy`, `sun_color` | A low, pale blue sun makes a moon |
| `ambient_energy`, `sky_energy` | Darken the ambient light and the sky |
| `glow_intensity` | Above 0, lamps and emissive surfaces bloom in Godot |
| `ssr` | 1 turns on screen space reflections, so wet streets mirror the lamps |

The editor's Lit Preview shows all of these except glow and reflections.

## Lamp fixtures

`light` and `light_spot` take a `shadows` key and a `fixture` key. Put the targetname of the lamp's glowing geometry in
`fixture` (a trailing `*` matches a prefix) and it follows the light with no wiring at all: shown while on, hidden while
off. Set `fixture_off` to `dark` to keep it visible with its glow switched off instead.

`examples/mcp/night_district.json` and `examples/mcp/withered_city.json` are complete night maps built this way.
