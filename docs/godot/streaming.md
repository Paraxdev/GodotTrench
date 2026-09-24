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
