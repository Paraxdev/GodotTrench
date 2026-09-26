# Project setup

The addon's defaults are enough for a first map, but updating the addon replaces them. Once you write your own
entities or want another texture folder, give the project its own resources.

## Installing and updating the addon

The editor checks the addon each time it opens a project. When `addons/func_godot` is missing, or its `plugin.cfg`
version differs from the editor's, the *Set up this project* window says so once, and the status bar keeps a yellow
note you can click. *Godot > Install or Update Addon…* opens the same window at any time.

| The project has | The window offers |
| --- | --- |
| No addon | **Install the addon**, then **Enable the plugin** |
| An older or newer addon | **Update the addon to** the editor's version |
| The addon, not enabled | **Enable the plugin**, which adds it to `[editor_plugins]` in `project.godot` and changes nothing else |
| The addon as a git checkout or submodule | Nothing. Update it with git, the editor never replaces it |

An update downloads the whole addon, unpacks it beside the project and only then swaps the folders, so a failed or
cancelled download leaves the old addon in place. The old one moves to `.godottrench/addon-backups/func_godot-<version>`
in the project. Godot skips folders whose names start with a dot, so it neither imports the backup nor lists it as a
second plugin. Delete it once the new addon works.

Close Godot before updating. It keeps running the old addon until it restarts, and the window waits while the live link
shows Godot with the project open. Godot also writes its own copy of `project.godot` back, so enable the plugin with
Godot closed too, or tick it in Godot under *Project > Project Settings > Plugins*.

If a download fails, the window links to the [releases](https://github.com/Paraxdev/GodotTrench/releases) and
[itch.io](https://paraxdev.itch.io/godottrench), which carry the same `func_godot-godottrench-addon.zip`. Extract it
into the project folder so it becomes `addons/func_godot`, replacing the old folder rather than unpacking over it.

## Core entities and the Gameplay entities pack

The addon ships a small core that every project has: worldspawn, `func_detail`, `func_illusionary`, `func_geo`,
`light`, `light_spot` and `info_player_start`. Doors, buttons, lifts, trains, triggers, logic, spawners, teleports,
path corners, props, effects and the rest of the [entity reference](../gameplay/entities/README.md) are the
Gameplay entities pack, which a project installs when it wants them.

Installing copies the pack's definitions and scripts from the addon into `res://godottrench/entities`:

| Path | Holds |
| --- | --- |
| `gameplay_fgd.tres` | The pack's FGD file, listing every definition |
| `definitions/` | One definition per entity: its keys, inputs, outputs and editor gizmos |
| `scripts/` | The GDScript each entity runs |
| `pack.json` | What was installed, so updates can tell your edits apart. Leave it as it is |

The copies belong to the project. Change a script or a definition, delete the entities you do not use, or add your
own definitions to `gameplay_fgd.tres`. The addon itself never defines these classnames or script classes, so nothing
you do there clashes with it, and an addon update does not undo it.

### Installing and updating the pack

1. Press **Install Gameplay Entities** in the *Set up this project* window, below the addon. *Gameplay > Install
   Gameplay Entities* and the button in the Entities panel do the same while the project only has the core.
2. The editor offers the new entities right away. Godot picks up the new files and exports the game config again on its
   own.
3. When an addon update brings a newer pack, the window and the Gameplay menu offer **Update Gameplay Entities**. The
   update adds new files and replaces the ones you did not change. Files you edited stay as they are and the editor names
   them, and files you deleted stay deleted. Nothing is updated until you ask.

Maps made before the entities left the addon keep working. When a project's maps use entities of the pack that nothing
in the project defines, the pack is installed by itself, when the editor opens the project and when Godot starts with
the updated addon. A `.gtm` file keeps every entity either way, with or without a definition.

> **Note:** The template in the addon, `addons/func_godot/gameplay_pack`, has a `.gdignore` file so that Godot skips
> it. Keep that file, or Godot finds every script class of the pack twice once it is installed.

### Which definition wins

The core FGD, `res://addons/func_godot/fgd/godottrench/godottrench_fgd.tres`, reads the installed pack by itself, so
every FGD built on the addon's defaults gets the pack without listing it. When two definitions share a classname, the
later one wins, in this order:

1. FuncGodot's classes and the addon's core
2. The installed pack in `res://godottrench/entities`
3. The entities of your own FGD file

To change an entity, edit its copy in the pack, or define the same classname in your own FGD. Both work, but a
classname defined twice is easy to forget, so the editor tells you. When the project opens, the status bar names the
definition that wins, and the Issues panel warns about each doubly defined class a map uses. Delete one of the two
definitions to settle it.

## Your own resources

Three resources work together. The FGD file lists the entities your maps can use. The map settings tell a build where
textures are and which FGD to read. The game config tells the GodotTrench editor about both, so it offers the same
entities and textures while you edit.

The quickest route is to copy `demo_fgd.tres`, `demo_map_settings.tres` and `demo_game_config.tres` from `godot/demo`
into your project and adjust the paths. To make them from scratch:

1. Create a `FuncGodotFGDFile`. Set *Base Fgd Files* to `res://addons/func_godot/fgd/godottrench_default_fgd.tres`,
   so you keep the core and the installed pack, and add your own entities under *Entity Definitions*.
2. Create a `FuncGodotMapSettings`. Set *Entity Fgd* to the FGD from step 1 and *Base Texture Dir* to your texture
   folder (default `res://textures`).
3. Create a `GodotTrenchGameConfig`. Set its *Fgd File* and *Map Settings* to the two resources above.
4. In *Project Settings*, point `func_godot/default_map_settings` at your map settings and `godottrench/game_config`
   at your game config.

> **Tip:** Set *Entity Name Property* in the map settings to `targetname`, as the demo does. Built nodes are then named
> after their targetnames, which keeps the scene tree readable.

## The game config

The editor learns your entities, texture folders and unit scale from `res://godottrench_game.json`, written by the
game config. The addon exports it again whenever Godot notices files changing. You can also export by hand from
*Project > Tools*, or headless, for example in CI:

```sh
godot --headless --path <project> --script res://addons/func_godot/src/godottrench/export_game_config_cli.gd
```

## Project settings

Most projects only change the first and third setting. The live link ones matter when another program already uses the
port.

| Setting | Default | Purpose |
| --- | --- | --- |
| `func_godot/default_map_settings` | the addon's defaults | Map settings for every map that does not set its own |
| `func_godot/default_inverse_scale_factor` | 32 | Map units per meter |
| `godottrench/game_config` | the addon's config | The config that writes `godottrench_game.json` |
| `godottrench/auto_export_game_config` | true | Export the game config again when files change |
| `godottrench/live_link_enabled` | true | Run the live link server that GodotTrench talks to |
| `godottrench/live_link_port` | 7842 | Must match *Godot live link* in the editor's preferences. Changes apply right away. Between 1024 and 65535, anything else falls back to 7842 with a warning |
| `godottrench/threaded_build` | true | Convert brushes and terrain chunks on the WorkerThreadPool, so big maps build faster |
| `godottrench/live_chunk_size` | 16.0 | In live mode, worldspawn geometry is split into chunks of this size in meters, so an edit only rebuilds its chunk |
| `godottrench/csharp_entity_dirs` | `res://` | Where C# entities are found at build time |
