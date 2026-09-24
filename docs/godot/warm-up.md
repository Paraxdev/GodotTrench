# Shader warm-up

Godot compiles shaders the first time it draws something, so the first frame of play stutters. A warm-up does that work
behind your loading screen instead.

Every map warms itself up after it builds or loads in the running game, so a game that is only the map needs nothing.

A map's own warm-up only covers the map. If your game adds its own content like a player, enemies or weapons, their
first appearance still stutters. Turn off **Warm Up Shaders** on the map and warm everything in one pass once it is all
in:

```gdscript
loading.show()
await build_maps()
spawn_player()
await GodotTrenchWarmUp.warm_shaders([get_tree().current_scene, preload("res://enemies/crawler.tscn")]).finished
loading.hide()
```

`warm_shaders` takes nodes, scenes, meshes and materials. Pass scenes that are not spawned yet, like the enemy above,
so they are ready when they appear. It draws nothing on screen and runs none of their scripts. Connect its
`progress(done, total)` signal to drive a loading bar.

On a test game with five maps, it took the first frame of play from 360 ms to 22 ms, for half a second of warm-up.

> **Tip:** Also turn on **Shader Baker** in your export presets, so players skip compiling shaders on first launch.
