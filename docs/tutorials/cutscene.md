# Scripted cutscene

The player walks into a room, a sign appears, a guide walks off, a barrel explodes, a lamp comes on and the exit
opens. Nothing here needs code of its own: a timer entity fires one output per beat, and each output is wired to the
entity that should act. It is the `scripted_scene` map in the demo project.

## The timeline

1. Cover the entrance with a `trigger_once` named `start_zone`. It fires `triggered` the first time the player walks
   into it, then switches itself off, so the scene only plays once.
2. Add a `logic_sequence` named `cutscene` with `steps` 5 and `interval` 1.5. When started it fires `step_1`, `step_2`
   and so on with 1.5 seconds between them.
3. Link `start_zone`'s `triggered` output to `cutscene.start`.
4. Link each step output of `cutscene` to the entity that acts in that beat:

| Output | Target and input | What happens |
| --- | --- | --- |
| `step_1` | `hint.show` | A `game_text` reads "Follow the guide" |
| `step_2` | `guide.start` | An `npc_walker` sets off along its `path_corner` chain, a row of markers each naming the next |
| `step_3` | `barrel.ignite` | An explosive `prop_physics` burns its fuse, then blasts and breaks |
| `step_4` | `lamp.turn_on` | The lamp switches on |
| `step_5` | `exit_door.open` | The exit door slides up |

The sequence waits before every step, the first one included, so step 1 fires at 1.5 s and step 5 at 7.5 s. For
uneven timing, set `times` to one wait per step, such as `0.5 2 1 1 3`.

## Reading the player's health

Some reactions need a little logic that no built in input covers. A `logic_script` runs a short GDScript snippet when
fired. Link the barrel's `broken` output to `hp_readout.run`, a `logic_script` whose `source` writes the player's
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

Use `!player` here, not `!activator`. `!activator` is whoever set the chain off, and the step outputs and `broken`
carry none. `!player` always finds the node in the `player` group. `broken` fires after the blast has done its
damage, so the health is already current.

## Playing it

The scene script,
[godot/demo/scenes/scripted_scene.gd](https://github.com/Paraxdev/GodotTrench/blob/main/godot/demo/scenes/scripted_scene.gd),
only builds the map, spawns the player at the `info_player_start` and adds a `GodotTrenchDebugOverlay`. Press **F3**
in game to watch each output fire, which is the quickest way to see a step that is wired to the wrong name.

{% mcp %}

## MCP

The demo map is built by driving the editor through the [MCP server](../mcp.md). The calls are in
[examples/mcp/scripted_scene.json](https://github.com/Paraxdev/GodotTrench/blob/main/examples/mcp/scripted_scene.json).

{% endmcp %}
