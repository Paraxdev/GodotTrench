# Scripted cutscene

A small Half-Life style scene built entirely from I/O. The player walks into a room, a sign appears, a guide walks
off, a barrel explodes, a lamp comes on and the exit opens.

It is the `scripted_scene` map in the demo project. Its source is `examples/mcp/scripted_scene.json`.

## The timeline

A `trigger_once` named `start_zone` covers the entrance. Its `triggered` output starts a `logic_sequence` named
`cutscene`, with five steps and an `interval` of 1.5 seconds.

A sequence waits before every step, the first included, so step 1 fires 1.5 s after the player walks in and step 5 at
7.5 s.

| Step | Wiring | What happens |
| --- | --- | --- |
| 1 | `step_1 -> hint.show` | A `game_text` reads "Follow the guide" |
| 2 | `step_2 -> guide.start` | An `npc_walker` sets off along its `path_corner` chain |
| 3 | `step_3 -> barrel.ignite` | An explosive `prop_physics` burns its fuse, breaks and blasts |
| 4 | `step_4 -> lamp.turn_on` | The lamp switches on |
| 5 | `step_5 -> exit_door.open` | The exit door opens |

For uneven timing, set the sequence's `times` to one wait per step, for example `0.5 2 1 1 3`.

The relevant part of the map:

```json
{ "classname": "trigger_once",
  "properties": { "targetname": "start_zone" },
  "outputs": [ { "output": "triggered", "target": "cutscene", "input": "start" } ] }

{ "classname": "logic_sequence",
  "properties": { "targetname": "cutscene", "steps": "5", "interval": "1.5" },
  "outputs": [
    { "output": "step_1", "target": "hint",      "input": "show" },
    { "output": "step_2", "target": "guide",     "input": "start" },
    { "output": "step_3", "target": "barrel",    "input": "ignite" },
    { "output": "step_4", "target": "lamp",      "input": "turn_on" },
    { "output": "step_5", "target": "exit_door", "input": "open" } ] }
```

## Reading the player's health

The barrel's `broken` output runs a `logic_script` named `hp_readout`, which writes the player's health onto a HUD
`game_text` named `hpmsg`:

```python
var players = io.find_targets(this, "!player", null)
if players.is_empty(): return
var hp = int(players[0].health)
for label in io.find_targets(this, "hpmsg", null):
	label.set_text("HP " + str(hp))
	label.show()
```

Two details make this work:

* It uses `!player` because the activator is long gone by this point. `step_3` drops it.
* No delay is needed, `prop_physics` only fires `broken` after its blast is applied, so the health read is already
  lowered.

## The scene script

Since the wiring lives in the map, the scene script only builds the map, spawns the player at the `info_player_start`
and adds the debug overlay. Press F3 in game to watch each output fire.

```python
extends Node3D

const PLAYER := preload("res://demo/player.tscn")
const MAP := "res://demo/maps/scripted_scene.gtm"
const SETTINGS := "res://demo/demo_map_settings.tres"

func _ready() -> void:
	add_child(GodotTrenchDebugOverlay.new())
	var map := FuncGodotMap.new()
	map.map_settings = load(SETTINGS)
	map.local_map_file = MAP
	add_child(map)
	map.build()
	var player := PLAYER.instantiate()
	add_child(player)
	var spawn := find_spawn(map)
	if spawn:
		player.global_transform = Transform3D(Basis(Vector3.UP, spawn.global_rotation.y), spawn.global_position + Vector3.UP * 0.1)
```
