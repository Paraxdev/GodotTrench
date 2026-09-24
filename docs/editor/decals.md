# Decals

Decals put grime, stains, cracks, posters and signs on a surface. A decal is a flat mesh drawn a hair closer to the
camera than the wall behind it and blended over it, so it never flickers against the wall or against other decals.

| To | Do |
| --- | --- |
| Place a decal | Hold Alt while you drop a material on a face. It lands as a one meter square |
| Place one over [MCP](../mcp.md)<span class="gt-mcp-row"></span> | `run_action` `create_decal` with `material`, `at`, `normal` and `size` `[width, height]` |
| Turn a thin mesh into a decal | Tick **Decal** in the mesh inspector, for a decal with a custom outline |

A decal is a normal mesh, so move, rotate and scale it like one. Soft edges stay soft. Use a material in alpha scissor
mode for a hard cut, see [Transparency](../godot/materials.md#transparency).

> **Warning:** Do not fake decals with thin brushes a few millimeters off a wall. They flicker.

A regular decal is flat. For a stain that wraps over a step or a pipe, use a projected decal instead, an entity with the
`godottrench_decal.gd` script like the demo's `infodecal`. Set its `material` key to a material name like `base/stain`.
It shines the material onto everything inside its box, props included, and costs more, so stick to regular decals on
flat surfaces.
