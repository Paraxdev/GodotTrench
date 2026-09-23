# Materials

A face's material is a path under the map settings' *Base Texture Dir* without an extension, for example
`showcase/planks`. If `showcase/planks.tres` exists there, or in *Base Material Dir* when that is set, the build uses
that material as it is. Otherwise the addon generates a plain material from the image. The editor previews the same
`.tres`, so both sides look alike. A `.tres` with only an `albedo_color` and no texture is a valid material too, the
editor draws it in that color.

New textures and materials show up in the editor within a couple of seconds, no reload needed.

## Texture size

By default one texture pixel covers one map unit, so a 64 pixel texture repeats every two meters. That suits pixel art,
but a 1024 pixel photo of a two meter wall would stretch over 32 meters. Tell the material how many map units one
repeat covers:

```
[resource]
albedo_texture = ExtResource("1_albedo")
metadata/texture_size = Vector2(64, 64)
```

You can also add it in Godot's inspector under *Metadata*, as a `Vector2` or a single number. Swapping in a sharper
image then no longer changes the map. Terrain layers ignore it, their `tile` setting already sets the repeat.

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
`night/sign.png`, becomes the glow map.

## Transparency

| `transparency` | Use it for | Watch out |
| --- | --- | --- |
| `1` blend | Glass, soft stains, smoke | Casts no shadows |
| `2` scissor | Hard cutouts up close, like grates and leaves | Thin parts vanish at a distance |
| `3` hash | Thin cutouts seen from afar, like wire fences | Soft edges turn into static |
