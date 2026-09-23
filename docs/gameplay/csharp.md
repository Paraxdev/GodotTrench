# C# entities

A C# class can be a placeable entity without a `.tres` definition. Signals become outputs, exported members become keys
and marked methods become inputs.

## 1. Add the helper

The attributes live in a helper file that does not ship with the addon. In the **Reference** panel
(*View > Panels > Reference*), press **Add C# helper to project**. It writes
`res://addons/func_godot/csharp/GodotTrench.cs`, and without it code using the attributes does not compile.

Running C# entities needs the .NET build of Godot. The addon finds them by reading the `.cs` files as text, so the
editor sees them either way.

## 2. Declare an entity

```csharp
[GlobalClass]
[GodotTrenchEntity("npc_guard", Description = "Patrolling guard",
    Color = "#ff4040ff", Size = "-16 0 -16 16 64 16")]
public partial class NpcGuard : CharacterBody3D
{
    [Signal] public delegate void AlertedEventHandler(Node activator, int level); // output "alerted"
    [Export] public float WalkSpeed { get; set; } = 3.5f;                          // key "walk_speed"
    [GodotTrenchInput] public void Alert(Node activator) { }                        // input "alert"
}
```

| Part | Notes |
| --- | --- |
| `[GlobalClass]` | Required, the map build creates the node by its class name |
| `Size` | Bounding box as mins then maxs, in map units |
| `Solid = true` | Makes a brush entity, like a door, instead of a point entity |
| `[GodotTrenchInput]` | Marks inputs. If no method has it, every public `void` method with a capitalized name is an input |

Names are snake_case in the map and PascalCase in C#: the input `alert` calls `Alert`, the key `walk_speed` sets
`WalkSpeed`. To read keys yourself instead, implement `_func_godot_apply_properties(Dictionary props)`, the helper has
readers like `GodotTrench.Float(props, "walk_speed", 3.5f)`.

## Where the addon looks

| Used by | Setting |
| --- | --- |
| The editor (game config export) | *Csharp Source Dirs* on the game config resource |
| The map build | Project setting `godottrench/csharp_entity_dirs` |

Both default to `res://` and skip `addons`, `bin`, `obj` and hidden folders. A C# class only defines classnames the FGD
does not, so it cannot replace a library entity by using the same name.

## I/O from C#

```csharp
var guide = GodotTrench.FindTargets(this, "guide")[0];
guide.Connect("finished", Callable.From(() => GD.Print("cutscene over")));

foreach (Node barrel in GodotTrench.FindTargets(this, "barrel"))
    GodotTrench.Invoke(barrel, "ignite", "", this);

GodotTrench.FireOutput(this, "alerted", player);
```

The Reference panel generates this code for any entity, and the MCP server's `code_reference` tool returns the same.
