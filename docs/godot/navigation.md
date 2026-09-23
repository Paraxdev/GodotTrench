# Navigation

`GodotTrenchNav` bakes a navigation mesh from a built map, so enemies and NPCs can find paths through it.

```gdscript
map.build()
var region := await GodotTrenchNav.bake_region(map)
```

`bake_region` bakes on a worker thread, adds a `NavigationRegion3D` named `navigation` under the map and returns once
paths over it work. Pass a node that holds several maps to bake them into one mesh. A rebuild frees the region, so bake
again after it.

## What counts

The bake reads the map's collision, not its meshes.

| In the map | In the navigation mesh |
| --- | --- |
| World brushes, `func_detail`, terrain and prop collision | Yes |
| Clip brushes | Yes, they block like walls |
| Illusionary brushes, triggers and physics props | No |
| Doors, buttons, platforms and trains | No, so a closed door does not seal its doorway |
| Bodies in the `navigation_ignore` group | No |

Paths lead through closed doors, so an agent has to open a door it walks into.

## Agent size

`GodotTrenchNav.profile(radius, height, max_climb, max_slope)` returns the settings for one agent size, in meters and
degrees. The default is 0.3, 1.8, 0.3 and 45 with 0.1 m cells. Pass it as the second argument:

```gdscript
var region := await GodotTrenchNav.bake_region(map, GodotTrenchNav.profile(0.5, 1.2))
```

`bake_region` sets the cell size of the navigation map to the bake's, which other regions on that map have to share.

## Caching

The first bake of a big map takes seconds. The result is cached in `user://godottrench_navigation`, keyed by the
collision and the settings, so the next start with the same map loads it in a fraction of a second. Pass
`false` as the third argument to bake without the cache.

## Checking a map

`GodotTrenchNav.bake(map)` bakes on the calling thread and returns the `NavigationMesh`, which suits tools and tests.
`GodotTrenchNav.islands(nav)` lists its unconnected parts as polygon indices, largest first. A room that should be
reachable but has an island of its own usually has a doorway blocked by something low, or a step higher than
`max_climb`. Wall tops and the floor inside thick walls show up as small islands too.

> **Warning:** Godot stops a path search after 4096 polygons and returns the path so far, which looks like an
> unreachable spot on a big map. `GodotTrenchNav.path(node, from, to)` has no limit. For agents set
> `path_search_max_polygons` on the `NavigationAgent3D` to 0.

## Seeing it in GodotTrench

GodotTrench can show where an agent walks without running the game. Open the project in the Godot editor with a scene
whose `FuncGodotMap` builds the map, then turn on **View > Walkable Area > Show Walkable Area**. Godot bakes the scene
over the [live link](live-link.md) and GodotTrench draws the result in the 3D and 2D views.

| Colour | Meaning |
| --- | --- |
| Bluish green | The biggest connected area |
| Vermillion | Islands cut off from it: behind a step higher than *Max climb*, a gap narrower than twice the *Radius* or a slope steeper than *Max slope* |

The agent size is set in the same menu. The overlay bakes again a moment after each edit. Without
[live mode](live.md) Godot builds the map as shown first, like *Build in Godot*. Agents call the `walkability` MCP tool,
which returns the islands with their areas and bounds in map units.

> **Tip:** Wall and roof tops show up as small vermillion islands. Look for vermillion floor next to green floor.
