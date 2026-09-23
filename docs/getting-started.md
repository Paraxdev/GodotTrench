# Getting started

This takes you from an empty Godot project to a room built in GodotTrench, standing in a Godot scene. Steps 1 to 3 are
done once per project.

## What you need

Godot 4.5 or newer. The addon is developed and tested on 4.7.2. Use the .NET build if you want to write entities in C#.
You also need the editor and the addon, both are in the
[rolling alpha](https://github.com/Paraxdev/GodotTrench/releases/tag/alpha).

## 1. Install the addon

Copy the `func_godot` folder into your project's `addons` folder, then enable **FuncGodot (GodotTrench)** under
*Project > Project Settings > Plugins*.

The addon's defaults already include the GodotTrench entity library, so doors, triggers and lights work without any
resources of your own.

## 2. Export the game config

Run *Project > Tools > GodotTrench: Export Game Config*. This writes `godottrench_game.json` to the project root, which
is how the editor learns about your entities and textures. From then on the addon exports again by itself whenever
files change.

> **Note:** The editor reads the game config when it opens the project and does not watch it. If you add an entity while
> the editor is running, use *Godot > Reload > Game Config*.

## 3. Open the project in the editor

Choose *Godot > Open Godot Project...*, or the **Open Godot project...** button at the right end of the toolbar, and
pick the project folder. Any folder inside the project works. The editor reopens it on the next start.

It worked when the toolbar button shows your game's name and the Entities panel lists the GodotTrench classes. If the
status bar says the project has no `godottrench_game.json`, step 2 has not run yet.

## 4. Build a room

The editor opens with four views: 3D, Top, Front and Side. The world is Y-up like Godot, so the Top view looks down
the Y axis.

1. **Draw a box.** With the Select tool (Q), left drag on empty space in the Top view. It snaps to the grid, 16 units
   to start with.
2. **Hollow it.** Press Ctrl+Shift+K. The box becomes walls 16 units thick.
3. **Texture it.** Click a texture in the Materials panel. To texture one wall only, Shift+click that face first.
4. **Save.** Press Ctrl+S and save inside the Godot project, the addon loads maps by `res://` path.

Map units are 32 per meter, so a 128 unit tall room is 4 m.

## 5. See it in Godot

Add a `FuncGodotMap` node to a scene, set its *Local Map File* to your `.gtm` and press **Build Map** in the Inspector.
Brushes become meshes with collision and entities become nodes with their scripts. The result is saved with the scene,
so nothing rebuilds at runtime.

Leave that scene open in Godot. Every save in GodotTrench now rebuilds it, and the editor's status bar tells you how
many map nodes were rebuilt. Save the scene in Godot to keep the result.

> **Warning:** A build deletes every child of the `FuncGodotMap` first. Put your own nodes next to the map node, or in
> an [overlay](godot/overlays.md).

## Try the demo project

The repository's `godot` folder is a complete project with the addon installed. Its main scene plays the showcase maps.

| Key | Action |
| --- | --- |
| WASD, Space, Shift | Walk, jump, run |
| E | Use doors and buttons |
| 1 to 5 | Switch between the showcase maps |
| F3 | Show the I/O debug overlay |

The maps are saved in `godot/demo/maps/showcase`. Opening one in the editor is a good way to see how a finished map is
put together.

## Next

Learn the interface in [Interface and navigation](editor/interface.md). To make the room do something, read
[How entity I/O works](gameplay/io.md) and build [Button opens a door](tutorials/button-door.md).
