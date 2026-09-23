# Materials

A face's material is a path under the map settings' *Base Texture Dir* without an extension, for example
`showcase/planks`. If `showcase/planks.tres` exists there, or in *Base Material Dir* when that is set, the build uses
that material as it is. Otherwise the addon generates a plain material from the image. The editor previews the same `.tres`, including transparency, emission and normal maps, so both sides look
alike.

## Texture size

UVs count texels, and by default a texel is one pixel. A 64 pixel texture then repeats every 64 units, two meters. That
suits pixel art, but a 1024 pixel photo of a two meter wall would stretch over 32 meters.

Fix it by telling the material how many map units one repeat covers:

```
[resource]
albedo_texture = ExtResource("1_albedo")
metadata/texture_size = Vector2(64, 64)
```

You can also add it in Godot's inspector under *Metadata*, as a `Vector2` or a single number. The editor and the build
both use it for UVs, so swapping an image for a sharper one no longer changes the map. Terrain layers ignore it, their
`tile` setting already measures the repeat in map units.

> **Note:** `python tools/fetch_demo_textures.py` downloads the demo's CC0 scans from Poly Haven and ambientCG again and
> writes each material with its real `texture_size`.

## Glowing materials

A material glows when its file sets `emission_enabled`. With a black emission color, the emission texture glows exactly
as painted:

```
[resource]
albedo_texture = ExtResource("1_albedo")
emission_enabled = true
emission = Color(0, 0, 0, 1)
emission_energy_multiplier = 2.5
emission_texture = ExtResource("2_emission")
```

Without a material file, an image named like the albedo with `_emission` added, `night/sign_emission.png` next to
`night/sign.png`, becomes the glow map. Emissive materials carry a glow badge in the Materials panel, and blends and
terrain layers keep the glow of the materials they are made from.

## Transparency

| `transparency` | Mode | Use it for | Watch out |
| --- | --- | --- | --- |
| `1` | Alpha blend | Glass, soft decals, soot, stains, smoke cards | Sorted per object, and it neither writes depth nor casts shadows |
| `2` | Alpha scissor | Hard cutouts seen up close: grates, leaves, signs | Thin parts vanish a few meters away once the mipmaps average below the threshold |
| `3` | Alpha hash | Thin cutouts seen from afar, like wire fences | Soft alpha turns into dither that looks like TV static, so only use it on textures whose alpha is 0 or 1 |

Decal sheets blend by default whatever the mode, and keep a hard cut only in scissor mode, see [Decals](../editor/decals.md).
