# Scripted cutscene

The player walks into a room, a sign appears, a guide walks off, a barrel explodes, a lamp comes on and the exit
opens, all wired with I/O. It is the `scripted_scene` map in the demo project, built by
[examples/mcp/scripted_scene.json](https://github.com/Paraxdev/GodotTrench/blob/main/examples/mcp/scripted_scene.json).

## The timeline

A `trigger_once` named `start_zone` covers the entrance. Its `triggered` output starts a `logic_sequence` named
`cutscene` with `steps` 5 and `interval` 1.5.

The sequence waits before every step, the first one included, so step 1 fires 1.5 s after the player walks in and
step 5 at 7.5 s. For uneven timing, set `times` to one wait per step, such as `0.5 2 1 1 3`.

| Output | Target and input | What happens |
| --- | --- | --- |
| `step_1` | `hint.show` | A `game_text` reads "Follow the guide" |
| `step_2` | `guide.start` | An `npc_walker` sets off along its `path_corner` chain |
| `step_3` | `barrel.ignite` | An explosive `prop_physics` burns its fuse, then blasts and breaks |
| `step_4` | `lamp.turn_on` | The lamp switches on |
| `step_5` | `exit_door.open` | The exit door slides up |

## Reading the player's health

The barrel's `broken` output calls `run` on a `logic_script` named `hp_readout`. Its `source` writes the player's
health onto a HUD `game_text` named `hpmsg`:

```gdscript
var players = io.find_targets(this, "!player", null)
if players.is_empty():
	return
var hp = int(players[0].health)
for label in io.find_targets(this, "hpmsg", null):
	label.set_text("HP " + str(hp))
	label.show()
```

It looks the player up with `!player` because there is no activator to use: the sequence's step outputs and
`broken` carry none. No delay is needed either, since `prop_physics` fires `broken` after the blast has done its
damage.

## Playing it

The scene script,
[godot/demo/scenes/scripted_scene.gd](https://github.com/Paraxdev/GodotTrench/blob/main/godot/demo/scenes/scripted_scene.gd),
only builds the map, spawns the player at the `info_player_start` and adds a `GodotTrenchDebugOverlay`. Press **F3**
in game to watch each output fire.
