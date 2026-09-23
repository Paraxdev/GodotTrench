# Decals

Decals put grime, stains, cracks, posters and signs on a surface without flickering.

| To | Do |
| --- | --- |
| Place a decal | Hold Alt while you drop a material on a face |
| Place one over [MCP](../mcp.md) | `run_action` `create_decal` with `material`, `at`, `normal` and `size` `[width, height]` |
| Turn a thin mesh into a decal | Tick **Decal** in the mesh inspector |

A decal is a normal mesh, so move, rotate and scale it like one. Soft edges stay soft. Use a material in alpha scissor
mode for a hard cut, see [Transparency](../godot/materials.md#transparency).

> **Warning:** Do not fake decals with thin brushes a few millimeters off a wall. They flicker.

For a stain that wraps over a step or a pipe, use a projected decal instead: an entity with the `godottrench_decal.gd`
script, like the demo's `infodecal`. Set its `material` key to a material name like `base/stain`. It also paints props
inside its box and costs more, so stick to regular decals on flat surfaces.
