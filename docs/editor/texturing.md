# Texturing

## Applying materials

| Do this | Result |
| --- | --- |
| Click a thumbnail in Materials | Apply it to the selection, and use it for new brushes |
| Drag a thumbnail onto a face | Apply it to that face |
| Shift while dropping | Apply it to the whole brush or mesh |
| Alt while dropping | Place it as a [decal](decals.md) |

Dropping a material on a terrain sets a terrain layer instead, see [Terrain layers](terrain-layers.md).

UV lock (Ctrl+Shift+U) is on by default, so textures stay attached to a brush while you move it.

## Built in colours

The Materials panel always offers `dev/grey`, `dev/dark`, `dev/orange`, `dev/blue` and `dev/green`, grid textures
for blocking out a level before it has art. Color coding rooms with them survives the build, since the addon ships the
same grids and Godot uses them when the project has no texture of that name. A texture of your own at, for example,
`res://textures/dev/grey.png` replaces one.

## The Texture tool

A face's alignment is where its texture sits on it: the offset, scale and rotation. **Shift+T** switches to the
Texture tool, which edits alignment in place like Hammer's face edit mode. It works on the faces you select with it.

| Input | Result |
| --- | --- |
| Click, Ctrl+click | Click selects a face, Ctrl+click adds or removes one |
| Drag a selected face | Slide the texture, the pixel under the cursor stays under it |
| Arrow keys | Slide by the grid size in pixels, with Shift by one pixel |
| Ctrl+wheel | Scale by 2, with Shift in fine steps |
| Alt+wheel | Rotate by 15°, with Shift by 1° |
| Alt+click | Pick up a face's material and alignment |
| Right click | Apply the current material |
| Ctrl+right click | Apply it and reset the alignment |
| Shift+right click | Paste the picked material and alignment |
| Alt+right click | Apply the current material, continuing the selected face's alignment across the edge |

Alt+right click is how a texture, like a trim or a row of bricks, runs cleanly around a corner.

The tool options bar has **Justify** buttons that push the texture to a face's left, right, top, bottom or center, or
fit it to the face. With *Treat as one* ticked they justify several faces as a single area, so one texture spans
all of them. **Align to View** projects the texture along the 3D camera.

For faces with explicit UVs, and for meshes, use the [UV Editor](uv-editor.md).

## Texture scale

By default one texture pixel covers one map unit, so a 64 pixel texture repeats every two meters. That suits low
resolution textures, but a 1024 pixel photo would stretch over 32 meters. Set `metadata/texture_size` on the material to
the world size one repeat should cover, see [Texture size](../godot/materials.md#texture-size).

## Hotspots

A trim sheet is one texture holding many strips and panels, like trims, vents and door frames. *Texture > Hotspot Fit*
(Alt+H) fits each selected face to the rectangle whose shape matches it best, preferring one that keeps the texture
close to its normal scale. The rectangles are read from `<texture>.hotspots.json`, and
*Texture > Hotspot Editor* draws them.

## Vertex paint

The **Paint** tool (P) paints vertex colours onto brush faces, for dirt, tint and fake lighting. Pick the colour in the
tool options bar, next to the brush radius and strength. Ctrl+wheel resizes the brush.
