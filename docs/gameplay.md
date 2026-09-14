# Gameplay entities, I/O and C#

## Using the addon in your project

Add a `FuncGodotMap` node, point it at a `.gtm` file and press *Build Map*. Entity definitions are FuncGodot FGD resources;
the addon exports them to `godottrench_game.json`, which the editor reads when you open the project folder.
`godot/` is an example project with the addon and demo scenes, you may use the demo scenes in your project.

## Entity library

The fork ships a ready entity library (`godot/addons/func_godot/fgd/godottrench`): `func_door`, `func_door_rotating`,
`func_gate`, `func_platform`, `func_train` with `path_corner`, `func_button`, `trigger_once`, `trigger_multiple`,
`trigger_call`, `trigger_spawn_area`, `trigger_hurt`, `trigger_teleport`, `trigger_push`, `info_spawner`,
`info_teleport_destination`, `logic_call`, `logic_relay`, `logic_timer`, `logic_counter`, `logic_auto` and `logic_debug`.

## Inputs and outputs

Outputs target entities by targetname (with `*` wildcards), `@group`, a node path (`/root/Game/Score`), `!player`, `!activator`
or `!self`. Inputs call any method on the target, GDScript or C# (`add_score` also finds `AddScore`), and a JSON array parameter
is spread into arguments with `$activator`, `$self`, `$position` and `$caller_name` placeholders, converted to the declared types.
`trigger_call` and `logic_call` call a method directly, for example `call_target = "/root/Game"`, `method = "give_item"`,
`arguments = ["key_red", "$activator"]`. `GodotTrenchIO.events()` reports every fired output, and
`GodotTrenchDebugOverlay` shows them in game (F3) together with the trigger volumes.

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
