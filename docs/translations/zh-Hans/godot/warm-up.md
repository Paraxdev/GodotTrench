---
source: 70bacdb1e7200d36894611dfa449da2b79911911
---

# 着色器预热 { #shader-warm-up }

Godot 在第一次绘制某样东西时才编译着色器，所以游玩的第一帧会卡顿。预热就是把这部分工作挪到你的加载画面后面去做。

如果你的游戏就只有地图，什么都不用做。每张地图在游戏运行中构建或加载完成后，都会自己预热，做完时发出 `warmed_up`。

不过，地图自带的预热只覆盖地图本身。如果你的游戏还会加入自己的内容，比如玩家、敌人或武器，它们第一次出现时还是会卡。这种情况下，在 Godot 的检查器里取消勾选 `FuncGodotMap` 上的 **Warm Up Shaders**，等所有内容都就位之后，再一次性预热全部：

```gdscript
loading.show()
await build_maps()
spawn_player()
await TrenchDotWarmUp.warm_shaders([get_tree().current_scene, preload("res://enemies/crawler.tscn")]).finished
loading.hide()
```

`warm_shaders` 接受节点、场景、网格和材质。像上面的敌人那样，把还没生成的场景也传进去，这样它们出现时就已经准备好了。它不会在屏幕上画任何东西，也不会运行它们的脚本。把它的 `progress(done, total)` 信号接上去，就能驱动加载条。

在一个有五张地图的测试游戏里，它让游玩第一帧的耗时从 360 毫秒降到了 22 毫秒，代价是半秒钟的预热。

> [!TIP]
> 也请在导出预设里打开 **Shader Baker**，这样玩家首次启动时就不用再编译着色器了。
