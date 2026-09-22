# Level design tools and controls

## Controls (TrenchBroom style)

* 3D: hold right mouse to look, WASD to fly (Q/E down/up while looking), middle mouse pans, wheel dollies, Alt+left drag orbits.
* 2D: right or middle mouse pans, wheel zooms.
* Drag the point where the view separators cross to resize all four views at once.
* Click selects the object under the cursor, also inside groups, double click selects the whole group. Linked groups are always
  selected as a whole. The outliner expands to and scrolls to whatever you pick in a view.
* The 3D view skips faces pressed fully against another solid, and where two faces overlap on the same plane only one draws the
  shared area, so touching or overlapping brushes and meshes do not flicker. Open meshes such as a blend sheet laid on a floor
  draw over brushes, brushes over closed meshes. The Godot build hides the same faces when they are fully covered, a face only
  partly covered by another facing the same way still draws twice in Godot.
* Left drag on empty space draws a brush, drag the selection to move it (Alt: vertical, Ctrl: duplicate, Shift: lock axis).
* The selection shows a gizmo in the 3D view: arrows move along an axis, squares move in a plane, the center dot moves freely,
  rings rotate (15° steps, Shift for 1°) and the boxes scale along an axis. *View > Transform Gizmo* hides it.
* Shift+click selects faces, Shift+drag a face resizes the brush, Ctrl+Shift+drag extrudes. In 2D views drag selection edges to resize.
* Tools: `C` clip (Tab changes kept side, Enter applies), `V` vertex, `R` rotate, `T` scale, `G` sculpt, `P` paint,
  `B` scatter, `Shift+G` blend, `Shift+E` volume, `M` measure, `Shift+P` path, `Shift+T` texture, `Tab` edit mesh,
  `Q`/`Esc` back to select. Ctrl+wheel resizes the scatter, blend and sculpt brushes.
* `[` / `]` grid, `Ctrl+D` duplicate, `Ctrl+G` group, `Ctrl+K` subtract, `Ctrl+M` merge, `Ctrl+H` hide, `Ctrl+J` isolate, `F` focus.
* `Ctrl+Shift+E` convert to mesh, `Ctrl+Shift+J` join meshes, `Ctrl+Shift+N` new tab, `Ctrl+Tab` next tab, `Ctrl+W` close tab.
* `Ctrl+=` / `Ctrl+-` / `Ctrl+0` scale the interface. The scale is saved and also set in Preferences or *View > Interface Scale*.
  It multiplies the monitor's display scaling, turn that off in Preferences if your monitor reports the wrong scaling.
* Autosaves are written next to the map as `level.gtm.autosave`, so Godot does not import them. Opening one (pick the
  *Autosave* filter in the Open dialog) recovers the map it belongs to, and saving writes the real `level.gtm`.
* Every binding can be changed in the *Keyboard Shortcuts* window, starting from the TrenchBroom, Hammer or Blender preset
  chosen in Preferences.

## Tools

* **Scatter** (`B`): paints trees, rocks or foliage into a scatter set that lives on its own layer. A set only lands on its
  target surfaces (Alt+click a brush, mesh or terrain to add or remove it as a target), so painting a forest on a terrain never
  covers the house on top of it. The palette window has presets (forest, pines, undergrowth, rocks, grass) and takes your own
  models (`.bbmodel`, `.glb`, `.gltf`, `.tscn`), each with a weight, spacing (spread), scale range, normal alignment, tilt and sink.
  LMB paints, Shift+LMB erases (optionally only chosen items), Ctrl+wheel resizes the brush, *Fill* covers the targets with slope
  and height limits. Foliage sets skip collision and shadows and fade out by distance. In Godot every set becomes MultiMeshes with
  shared collision shapes, or instanced scenes when the model has scripts. *Scatter Sets to Entities* (Terrain menu) turns a set into `prop_model` entities.
* **Blend** (`Shift+G`): paints terrain splat layers, displacement alpha and a second material on brush and mesh faces
  (Terrain > Blend, *Set Blend Material*). Modes: paint, erase, smooth, sharpen, noise, slope and height masks, with smooth, linear,
  constant and spray falloffs. Godot uses the same blend through `gt_blend.gdshader`.
* **Volume** (`Shift+E`): drag out a `trigger_once`, `trigger_multiple`, `trigger_call`, spawn area, hurt, teleport, push or plain area volume in one step.
* **Gameplay wizards** (Gameplay menu and the selection panel): turn brushes into a door that swings on its left or right hinge or
  slides in any direction (optionally with a walk-up trigger), a lift, a button that fires a target, a trigger or spawn area around the
  selection, and *Link…* to connect any output to any input.
* **Gizmos**: selected entities show draggable handles for hinges, travel offsets, radii, spot cones, spawn areas and
  target points. Entity definitions declare them (`"gizmos"` in the game config) or they are inferred from property names.
* **Entities** tab: cards for every entity class. Drag one into a view to place it on the surface under the pointer,
  Ctrl or Shift click selects several and dragging any of them drops them all in a row. Brush entities arrive with a box brush,
  double clicking a brush entity with brushes selected wraps those brushes instead.
* **Reference** tab: for the selected entity class, GDScript and C# classes you can drop into your project, usage snippets,
  the FGD resource, and the C# helper with the attributes described in [gameplay.md](gameplay.md). Drag the divider between
  the class list and the details to resize them, double click it to fit the list to the names.
