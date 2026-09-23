# Texturing and UVs

Most texturing happens right in the 3D view. The UV Editor panel handles what the 3D view cannot.

## Applying materials

| Do this | Result |
| --- | --- |
| Click a thumbnail in Materials | Apply it to the selection, and use it for new brushes |
| Drag a thumbnail onto a face | Apply to that face |
| Shift while dropping | Apply to the whole brush |
| Alt while dropping | Place a decal |

UV lock (Ctrl+Shift+U) is on by default, so textures stay attached to a brush while you move it.

## The Texture tool

Shift+T is Hammer's face edit mode, in place.

| Input | Result |
| --- | --- |
| Drag | Slide the texture |
| Ctrl+wheel | Scale |
| Alt+wheel | Rotate |
| Alt+click | Eyedropper, picks up a face's material and its alignment |
| Right click | Apply the picked material to another face |
| Shift+right click | Apply material and alignment |

Shift+right click is how you make a texture run cleanly around a corner.

## The UV Editor

The UV Editor works on brush faces and meshes. It draws the selected faces over their tiled texture.

| Input | Result |
| --- | --- |
| Drag empty space | Move the texture |
| Wheel, right drag | Scale, rotate |
| Click a corner | Select it, together with the corners stitched to it |
| Shift+click, Ctrl+click | Add, remove |
| Alt+click | Select only that face's corner, to split a stitch |
| Shift or Ctrl + drag empty space | Box select, adding or removing |
| Double click a corner | Gizmo that moves along U or V only |
| Middle drag, Ctrl+wheel | Pan, zoom the canvas |

The U and V fields under the canvas place the selection to the pixel. Every drag is one undo step.

> **Tip:** While the UV Editor is the visible tab in its panel, Ctrl+click on a brush or mesh selects all its faces, so
> every side can be textured at once.

## Mesh UV projections

The UV Editor's **Projection** menu lays out explicit UVs on meshes: planar, box, cylinder, sphere, view and unfold.
They all use the texel density a brush face with the same texture would get.

| Command | Result |
| --- | --- |
| *Reset to World* | The projection of a freshly reset brush face |
| *Bake Planar* | Turns the planar texture into editable corner UVs |
| *Clear UVs* | Drops them again |
| *Normalize* | Scales the UVs to fit the texture once |
| *Pack* | Packs the islands into the texture at one texel density |

The **UV** menu flips, turns, fits, aligns, straightens and snaps to pixels. The **Hotspot** menu fits to a hotspot
(Alt+H).

## Texture scale

By default a texture repeats every as many map units as it has pixels, which is too small for photo textures. Give
the material a `texture_size` to fix that, see [Materials and lighting](../godot/materials.md).

## Vertex paint

The **Paint** tool (P) paints vertex colours onto faces, for dirt, tint and fake lighting.
