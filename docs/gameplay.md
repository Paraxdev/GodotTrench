# Gameplay entities, I/O and C#

## Using the addon in your project

Add a `FuncGodotMap` node, point it at a `.gtm` file and press *Build Map*. Entity definitions are FuncGodot FGD resources;
the addon exports them to `godottrench_game.json`, which the editor reads when you open the project folder.
`godot/` is an example project with the addon and demo scenes, you may use the demo scenes in your project.

## Entity library

The fork ships a ready entity library (`godot/addons/func_godot/fgd/godottrench`): `func_door`, `func_door_rotating`,
`func_gate`, `func_platform`, `func_train` with `path_corner`, `func_button`, `trigger_once`, `trigger_multiple`,
`trigger_call`, `trigger_spawn_area`, `trigger_hurt`, `trigger_teleport`, `trigger_push`, `info_spawner`,
`info_teleport_destination`, `light`, `logic_call`, `logic_relay`, `logic_timer`, `logic_counter`, `logic_auto`
and `logic_debug`.

A second set covers scripted scenes and props with little setup, so a map can spawn a character, walk it a path,
play animations, show text, break things and switch lights entirely through I/O:

* `npc_walker`: a character that walks a chain of `path_corner` entities playing animations, for cutscenes.
  Inputs `start`, `stop`, `walk_to(corner)`, `play_anim(name)`, `face(target)`. Outputs `arrived(corner)`,
  `reached_goal`, `finished`.
* `logic_sequence`: a timeline that fires `step_1` up to `step_8` in order with a delay between them, so a scene is
  scripted by wiring each step. Inputs `start`, `stop`, `reset`.
* `logic_animate`: plays animations on the `AnimationPlayer` under a target. Inputs `play(name)`, `stop`,
  `queue(name)`, `seek(time)`. Output `finished(name)`.
* `game_text`: shows a line in the world (a `Label3D`) and/or on the HUD. Inputs `show`, `hide`, `set_text(text)`,
  `flash(seconds)`. Outputs `shown`, `hidden`.
* `prop_physics`: a throwable, breakable crate or barrel. Damage, a hard impact (`impact_speed`) or a `smash` input
  breaks it, and an `explosive` prop blasts when broken. Inputs `smash`, `ignite`, `push(direction)`,
  `take_damage(amount, source)`. Outputs `damaged(hp)`, `broken`.
* `env_explosion`: on `explode` it pushes rigid bodies away and calls `take_damage` on nodes within `radius`, then
  optionally spawns an effect. Output `exploded`.
* `light` and `light_spot`: switchable omni and spot lights. Inputs `turn_on`, `turn_off`, `toggle`. Output
  `switched(on)`.
* `env_sound`: a positional sound. Inputs `play`, `stop`, `toggle`. Output `finished`.
* `env_particles`: a particle effect for smoke, fire, sparks or dust. Inputs `start`, `stop`, `toggle`, `burst`.
* `logic_branch`: stores a boolean and, on `test`, fires `on_true` or `on_false`, for conditional wiring. Inputs
  `set_true`, `set_false`, `toggle`, `test`, `set_and_test(value)`.

## Running a script when the built-in inputs are not enough

`logic_script` runs an inline GDScript snippet when fired, the deeper route for logic that reaching data like a
player's health needs. Its `source` is a function body with these names in scope: `this` (the node), `activator`
(the firing or `!activator` node), `parameter` (the incoming value), `io` (the `GodotTrenchIO` helpers) and `tree`
(the `SceneTree`). For a one liner set `expression` instead. It has inputs `run(activator)` and `run_with(parameter)`
and an output `ran(result)`. For example, `source = "activator.take_damage(25, this)\nreturn activator.health"` reads
and lowers the activator's health and passes the remaining value out through `ran`. Snippets are author provided
GDScript that run at the map's own trust level, the same as the entity scripts a map already ships.

## Inputs and outputs

Outputs target entities by targetname (with `*` wildcards), `@group`, a node path (`/root/Game/Score`), `!player`, `!activator`
or `!self`. Inputs call any method on the target, GDScript or C# (`add_score` also finds `AddScore`), and a JSON array parameter
is spread into arguments with `$activator`, `$self`, `$position` and `$caller_name` placeholders, converted to the declared types.
`trigger_call` and `logic_call` call a method directly, for example `call_target = "/root/Game"`, `method = "give_item"`,
`arguments = ["key_red", "$activator"]`. `GodotTrenchIO.events()` reports every fired output, and
`GodotTrenchDebugOverlay` shows them in game (F3) together with the trigger volumes.

The editor's **Logic** panel previews this wiring without launching Godot: pick a named entity and one of its outputs,
press *Fire*, and it follows the connections across the map, listing each `output > target.input` step in order and
flagging any target that resolves to nothing. Runtime targets like `!player` or `@group` are shown but not resolved,
since they only exist while the map plays.

## C# entities

C# classes can be entities without any `.tres` file. Add the helper from the Reference tab (it defines the attributes) and list your
source folders in `GodotTrenchGameConfig.csharp_source_dirs`:

```csharp
[GlobalClass]
[GodotTrenchEntity("npc_guard", Description = "Patrolling guard", Color = "#ff4040ff", Size = "-16 0 -16 16 64 16")]
public partial class NpcGuard : CharacterBody3D
{
    [Signal] public delegate void AlertedEventHandler(Node activator, int level);   // output "alerted"
    [Export] public float WalkSpeed { get; set; } = 3.5f;                           // property "walk_speed"
    [GodotTrenchInput] public void Alert(Node activator) { }                        // input "alert"
}
```

What the fork changes compared to upstream FuncGodot is listed in [FORK.md](../godot/addons/func_godot/FORK.md).
