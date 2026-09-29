# Streaming big maps

Large outdoor maps can hide their visuals by distance, so a map costs what the player can see rather than everything
it contains. The build sorts the map's meshes into chunks on a grid, and only the chunks near the camera are drawn.
Select nothing so the Inspector shows worldspawn, then set:

| Key | Default | Meaning |
| --- | --- | --- |
| `chunk_streaming` | 0 | Set to 1 to add a `GodotTrenchStreamer` on the next build |
| `chunk_size` | 2048 | Chunk size in map units, 64 m. Smaller chunks hide more precisely but cost more to check |
| `load_radius` | 8192 | How far from the camera chunks stay visible, in map units, 256 m |

Streaming only hides meshes. Collision and scripts keep running everywhere, so it saves rendering cost, not physics or
logic. Overlays, content spawned at runtime and anything on a moving door or train are never hidden.

The streamer follows the viewport's current camera, or an `info_player_start` while there is no camera. To pin it to
one camera, set its `camera_path` from code after the build, since a rebuild replaces the streamer. It only runs in the
game, so the Godot editor always shows the whole map.

The streamer emits `area_loaded(key, bounds)` and `area_unloaded(key, bounds)` as chunks come and go, so you can
stream your own content alongside:

```gdscript
streamer.area_loaded.connect(func(key, bounds): spawn_wildlife(bounds))
```

## Streaming sections

To play several maps as one world without loading screens, split it into sections and join them with connector
rooms. Only one section is in the scene tree at a time. The next one loads in the background while the player
explores, and swaps in while the player walks through a connector, lined up so the room around them stays exactly the
same. This needs the [Gameplay entities pack](../gameplay/entities/README.md#installing-and-customizing).

1. Build a connector room: a corridor with a bend on each side, so neither end can be seen from its middle. Put an
   `info_landmark` in the middle, and two `func_section_stream` volumes, one covering each half, meeting in the
   middle. Add it to the [prefab library](../editor/prefab-library.md) so every copy is the same.
2. In each map that meets another, paste the connector where the two meet, name its landmark, for example
   `lm_hall_cellar`, and set each half's `section` to the section on that side and `landmark` to the landmark's name.
   Both maps get the same copy with the same names, turned the same way.
3. Bake lighting in every section map. SDFGI only notices geometry a swap brings in once its cascades scroll, so a
   swapped in section would stay unlit, while baked lighting comes with the section.
4. Make a scene per section with its built map, the usual way.
5. Add a `GTSectionStreamer` node to your world scene, set its `first_section` and its player, see
   [GTSectionStreamer](../gameplay/entities/sections.md#gtsectionstreamer).

> **Tip:** Keep a landmark's position a multiple of 480 map units, or any size all the connector's textures repeat
> at, so the textures line up the same way in both copies.
