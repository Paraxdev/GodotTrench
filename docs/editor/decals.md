# Decals

A decal lays a texture over a surface: grime, stains, cracks, posters, signs and road markings.

## Placing a decal

| Do this | Result |
| --- | --- |
| Alt while dropping a material on a face | A decal sheet one meter square on that face |
| `run_action` `create_decal` over [MCP](../mcp.md) | The same, with `size` `[width, height]` in map units |
| **Decal** in the mesh inspector | Turns any thin mesh into a decal sheet, or back |

A decal sheet is an ordinary mesh flagged `decal`. Move, rotate and scale it with the usual tools, and fit its texture
in the [UV Editor](uv-editor.md).

## How Godot draws it

The addon blends the sheet over what is behind it without writing depth, and draws it a little closer to the camera,
0.5 mm per meter of distance. The pull runs along the view ray, so nothing shifts on screen. A decal therefore never
z-fights with its wall at any distance, and overlapping decals layer instead of flickering.

Soft alpha stays soft, so soot and stains fade out at their edges. A material in alpha scissor mode keeps its hard cut,
see [Transparency](../godot/materials.md#transparency). Decals cast no shadows.

> **Warning:** Do not fake decals with thin brushes or `func_detail_illusionary` boxes a few millimeters off a wall.
> Their faces share depth with the wall and with each other, and they flicker.

## Projected decals

A sheet is flat. For a stain that wraps over a step or a pipe, project the texture with a Godot `Decal` node instead:
an entity whose script is `godottrench_decal.gd`, like the demo project's `infodecal`. Its `material` key takes a
material name such as `base/stain`, with its normal map, ORM and emission textures, and its `texture_size` sets the
footprint unless `size` says otherwise.

Projected decals cost more per pixel and paint everything inside their box, props included, so prefer sheets on flat
surfaces.
