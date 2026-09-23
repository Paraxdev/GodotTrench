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

## The Texture tool

**Shift+T** edits face alignment in place, like Hammer's face edit mode. It works on the faces you select with it.

| Input | Result |
| --- | --- |
| Click, Ctrl+click | Select a face, add or remove one |
| Drag a selected face | Slide the texture, the pixel under the cursor stays under it |
| Arrow keys | Slide by the grid size in pixels, with Shift by one pixel |
| Ctrl+wheel | Scale by 2, with Shift in fine steps |
| Alt+wheel | Rotate by 15°, with Shift by 1° |
| Alt+click | Pick up a face's material and alignment |
| Right click | Apply the current material |
| Ctrl+right click | Apply it and reset the alignment |
| Shift+right click | Paste the picked material and alignment |
| Alt+right click | Apply the current material, continuing the selected face's alignment across the edge |

Alt+right click is how a texture runs cleanly around a corner. In the tool options bar, *Treat as one* justifies
several faces as a single area.

For faces with explicit UVs, and for meshes, use the [UV Editor](uv-editor.md).

## Texture scale

A texture repeats every as many map units as it has pixels, which is too small for high resolution photos. Set
`metadata/texture_size` on the material to the world size one repeat should cover, see
[Materials](../godot/materials.md).

## Hotspots

*Texture > Hotspot Fit* (Alt+H) fits the selected faces to the best matching rectangle of a trim sheet, read from
`<texture>.hotspots.json`. *Texture > Hotspot Editor* draws those rectangles.

## Vertex paint

The **Paint** tool (P) paints vertex colours onto brush faces, for dirt, tint and fake lighting. Pick the colour in the
tool options bar, Ctrl+wheel resizes the brush.
