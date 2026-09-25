# Getting started

Steps 1 to 3 connect a Godot project to the editor and are done once per project. Steps 4 and 5 build a first room and
load it in Godot.

## What you need

Godot 4.7, CI tests the addon on 4.7.2. Use the .NET build if you want to write entities in C#. The editor and the
addon are in the [latest release](https://github.com/Paraxdev/GodotTrench/releases/latest).

## 1. Install the addon

Extract `func_godot-godottrench-addon.zip` into your project folder so the addon ends up in `addons/func_godot`, then
enable **GodotTrench** under *Project > Project Settings > Plugins*.

Doors, triggers, lights and the other built in entities work out of the box. [Project setup](godot/project-setup.md)
covers replacing them with your own.

## 2. Export the game config

Run *Project > Tools > GodotTrench: Export Game Config*. It writes `godottrench_game.json` to the project root, which is
how the editor learns your entities and textures. From then on the addon exports again whenever files change, and the
editor reloads the file within a second of each export.

## 3. Open the project in the editor

Choose *Godot > Open Godot Project…*, or the **Open Godot project…** button in the toolbar, and pick the project
folder or any folder inside it. The editor reopens it on the next start.

If the status bar says there is no `godottrench_game.json` in the project, step 2 has not run and the editor falls back
to its built in entities.

## 4. Build a room

The editor opens with four views: a 3D view you fly around in, and Top, Front and Side views that look straight along
one axis. The world is Y-up like Godot, and map units are 32 per meter. Levels are built from brushes, solid convex
blocks such as boxes and wedges.

1. **Draw a box.** With the Select tool (Q), drag on empty space in the Top view, about 384 units wide and 256 deep.
   The Top view prints the size as you drag, and the box snaps to the grid, 16 units (half a meter) by default. The
   first box you draw is 128 units (4 m) tall.
2. **Hollow it.** Press Ctrl+Shift+K. The solid box turns into walls, floor and ceiling 16 units thick, with 96 units
   (3 m) of headroom inside, plenty for a player.
3. **Cut a doorway.** Click empty space outside the room so nothing is selected, since dragging a selection moves
   it. In the Front view, drag a box 64 units wide from the top of the floor up 80 units. A new box takes its depth
   from the last box you drew, so this one reaches through the whole room. In the Top view, drag the
   top edge of the new box down until it only crosses the bottom wall. Then press Ctrl+K, *Brush > CSG > Subtract*,
   which cuts the selected box out of every brush it overlaps and deletes it, leaving a hole in the wall.
4. **Texture it.** Click a texture in the Materials panel. To texture one wall only, Shift+click that face first.
5. **Save.** Press Ctrl+S and save inside the Godot project, since the addon loads maps by their `res://` path. The
   save dialog starts in the project folder.

> **Tip:** With the Select tool, clicking a brush selects it and Ctrl+click adds more. If a click picks the wrong
> brush, click it in the Outliner instead.

## 5. See it in Godot

Add a `FuncGodotMap` node to a scene, set its *Local Map File* to your `.gtm` and press **Build Map** in the Inspector.
Brushes become meshes with collision and entities become nodes with their scripts, saved with the scene.

Leave that scene open in Godot and every save in GodotTrench rebuilds it. Save the scene in Godot to keep the result.

> **Warning:** A build deletes the children of the `FuncGodotMap` first. Put your own nodes next to the map node, or in
> an [overlay](godot/overlays.md).

## Try the demo project

The repository's `godot` folder, also in the release as `godottrench-demo-project.zip`, is a project with the addon
installed. Its main scene plays the showcase maps, which are saved in `godot/demo/maps/showcase`. Open the project in
the editor and pick one under *File > Maps in Project* to see how a finished map is put together.

| Key | Action |
| --- | --- |
| WASD, Space, Shift | Walk, jump, run |
| E | Use doors and buttons |
| 1 to 5 | Switch between the showcase maps |
| F3 | Show the I/O debug overlay, a log of every input and output as it fires, with trigger volumes drawn in |

## Next

To make the room do something, read [How entity I/O works](gameplay/io.md) and build
[Button opens a door](tutorials/button-door.md).
