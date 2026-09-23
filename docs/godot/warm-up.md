# Shader warm-up

Godot compiles a render pipeline for each material, mesh format and render pass the first time it draws them. A built
map brings dozens of materials, so without a warm-up the first frame of play and the first look into a new area hitch.

## What Godot 4.7 does by itself

| Feature | What it covers | What it leaves |
| --- | --- | --- |
| Ubershaders and pipeline precompilation | Pipelines for meshes as they load and for objects when they are first drawn, compiled on worker threads | Everything found on the first frame a camera sees the world: the environment's effects, shadow types, reflection probes. Every surface compiles again then, and that frame waits |
| Shader cache, `rendering/shader_compiler/shader_cache/enabled`, on | Compiled shaders on disk, reused on later launches | Pipelines themselves |
| Pipeline cache, `rendering/rendering_device/pipeline_cache/enable`, on | The driver's pipelines on disk, so later launches build them much faster | The first launch, and the stall still happens during play, only shorter |
| Shader baker, export option `shader_baker/enabled`, off | Shaders compiled at export for the target driver, so a first launch skips that step | Pipelines |

Keep both caches on and turn on the shader baker in your export presets. The warm-up below moves what is left behind
the loading screen.

## The warm-up

`FuncGodotMap` warms its own map in the running game after it builds or loads. **Warm Up Shaders** turns that off,
and `warmed_up` fires when it is done. Headless runs skip it.

A map rarely comes alone: the game adds its player, its environment, a torch. For the smoothest start, turn the
map's own warm-up off and warm the whole scene once everything is in, behind your loading screen:

```gdscript
func _start_level() -> void:
	loading.show()
	await build_maps()
	spawn_player()
	var warm := GodotTrenchWarmUp.warm_shaders([get_tree().current_scene, preload("res://enemies/crawler.tscn")])
	warm.progress.connect(func(done: int, total: int) -> void: loading.bar.value = 100.0 * done / total)
	await warm.finished
	loading.hide()
```

`warm_shaders` takes nodes, scenes, meshes and materials. It draws one copy of every distinct mesh, material, particle
and sprite setup it finds, including skinned meshes, MultiMeshes and the scenes that scripts reference, such as a
spawner's template. The copies are built from scratch, so no scripts run and nothing plays. They are drawn in chunks
into a small hidden viewport that shares the game's world, environment and viewport settings, lit by the scene's
directional lights and a shadowed omni and spot light of its own, until Godot's pipeline counters settle. Nothing is
drawn on screen.

## Measured

A test game with five maps and 317 distinct draw setups, RTX 4060 Ti, 1600x900. Cold means Godot's shader and
pipeline caches were deleted first.

| Setup | Warm-up | First frame of play, cold | First frame of play, warm | Pipelines compiled on it |
| --- | --- | --- | --- | --- |
| No warm-up | none | 360 ms | 62 to 88 ms | 592 |
| The maps' own warm-up | during the build | 96 to 102 ms | 48 to 66 ms | 325 to 338 |
| `warm_shaders` on the whole scene | 0.6 to 0.8 s cold, 0.5 s warm | 22 ms | 21 ms | 0 |

The maps' own warm-up cannot cover what the game adds after them, here its player, environment and torch, which is
why a warm-up by hand after those pays off. Chunks of 64, 256 or all copies at once finish within 0.1 s of each other.
