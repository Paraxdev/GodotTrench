# Streaming big maps

Large outdoor maps can hide their visuals by distance, so a map costs what the player can see rather than everything
it contains. Select nothing so the Inspector shows worldspawn, then set:

| Key | Default | Meaning |
| --- | --- | --- |
| `chunk_streaming` | 0 | Set to 1 to add a `GodotTrenchStreamer` on the next build |
| `chunk_size` | 2048 | Cell size in map units |
| `load_radius` | 8192 | How far from the camera cells stay visible, in map units |

Streaming only hides meshes. Collision and scripts keep running everywhere, so it saves rendering cost, not physics or
logic. It follows the viewport's current camera, or an `info_player_start` while there is no camera, and only runs in
the game, so the Godot editor always shows the whole map. Overlays and anything under a moving entity, like a door or a
train, are never hidden.

The streamer emits `area_loaded(key, bounds)` and `area_unloaded(key, bounds)` as cells come and go, so you can stream
your own content alongside:

```gdscript
streamer.area_loaded.connect(func(key, bounds): spawn_wildlife(bounds))
```
