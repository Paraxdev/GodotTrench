# UV Editor

The UV Editor panel draws the selected brush and mesh faces flat over their tiled texture, so you can see which part of
the texture each face shows. Use it for what the [Texture tool](texturing.md#the-texture-tool) cannot do in the 3D view,
like placing mesh UVs to the pixel.

A brush face normally gets its texture from one offset, scale and rotation. A face with **explicit UVs** stores a
texture position for each corner instead, which is what meshes need.

> **Tip:** While the UV Editor is the visible tab in its panel, Ctrl+click on a brush or mesh selects all its faces, so
> every side can be textured at once.

## Controls

| Input | Result |
| --- | --- |
| Drag empty space | Move the texture, or the whole faces when they have explicit UVs |
| Wheel, right drag | Scale, rotate |
| Click a corner | Select it together with the corners of neighbouring faces stitched to it |
| Shift+click, Ctrl+click | Add, remove |
| Alt+click | Select only that face's corner, to split a stitch |
| Shift or Ctrl+drag on empty space | Box select, adding or removing |
| Drag a selected corner | Move the whole selection |
| Double click a corner | Gizmo that moves along U or V only, Esc hides it |
| Middle drag, Ctrl+wheel | Pan, zoom the canvas |

With corners selected, the wheel and right drag scale and rotate them around their center. The U and V fields under the
canvas place the selection to the pixel.

## Mesh UV projections

The **Projection** menu lays out explicit UVs on meshes, at the texel density a brush face with the same texture would
get, so meshes match the brushes around them.

| Command | Result |
| --- | --- |
| *Planar* | Projects along the direction the faces mostly point, keeping them in one piece |
| *Box* | Projects each face from its closest world axis, for blocky shapes |
| *Cylinder*, *Sphere* | Wrap the texture around the faces, for pipes, columns and domes |
| *View* | Projects straight along the 3D camera |
| *Unfold* | Lays the faces out flat edge by edge, so strips and grids stay continuous |
| *Reset to World* | The projection of a freshly reset brush face |
| *Bake Planar* | Turns the planar texture into editable corner UVs |
| *Clear UVs* | Drops the explicit UVs again |
| *Normalize* | Scales the UVs to fit the texture once |
| *Pack* | Packs the islands into the texture at one texel density |

The **UV** menu flips, turns, fits, aligns, straightens and snaps to pixels. The **Hotspot** menu fits to a hotspot
rectangle, see [Hotspots](texturing.md#hotspots).
