# GodotTrench

GodotTrench is a brush based level editor for Godot in the spirit of TrenchBroom and Hammer. You block out a level from
brushes, add meshes, terrain and props, and wire gameplay together with Hammer style inputs and outputs. The editor is a
standalone app, and the GodotTrench addon inside your Godot project turns the saved map into a scene.

![The GodotTrench editor](screenshots/editor.png)

## How the pieces fit

| Part | Role |
| --- | --- |
| The editor | Writes `.gtm` maps, compressed binary files made of independent chunks, so a damaged byte costs a few objects instead of the whole map |
| The addon | Reads `.gtm` files in Godot and builds meshes, collision and working entities |
| The game config | `godottrench_game.json`, written by the addon so the editor knows your entities and textures |

When something looks wrong on one side, one of these three is usually out of date.

## Where to go

| You want to | Read |
| --- | --- |
| Set up a project and build a first room | [Getting started](getting-started.md) |
| Find your way around the editor | [Interface and navigation](editor/interface.md), [Shortcuts](editor/shortcuts.md) |
| Understand what happens in Godot | [Building maps](godot/building.md) |
| See edits in Godot before saving | [Live mode and hot reload](godot/live.md) |
| Make doors, triggers and scripted scenes | [How entity I/O works](gameplay/io.md), then [Button opens a door](tutorials/button-door.md) |
| Build outdoor terrain | [Terrain](editor/terrain.md), then the [sea island tutorial](tutorials/sea-island.md) |
| Drive the editor from scripts or AI agents | [MCP server](mcp.md) |

## Download

The [latest release](https://github.com/Paraxdev/GodotTrench/releases/latest) has the editor for Linux, Windows and macOS, the addon zip and a demo project. The
[rolling beta](https://github.com/Paraxdev/GodotTrench/releases/tag/beta) has the newest changes from `main`. To build from source, see [Development](development.md).
