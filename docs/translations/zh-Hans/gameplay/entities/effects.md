---
source: 80f0288013e71dba5bd67c791a5e3e3272a71653
---

# 道具、灯光与特效 { #props-lights-and-effects }

> [!NOTE]
> 这些实体来自玩法实体包，见[安装与自定义](README.md#installing-and-customizing)。`light`、`light_spot` 和探针（`light_probe`、`env_reflection_probe` 和 `env_voxel_gi`）属于插件的核心，不需要这个包也能用。

## prop_physics { #prop_physics }

一个可以被扔出去、受伤、被打碎的箱子或桶。生命值降到 0、受到比 `impact_speed` 更快的撞击、收到 `smash`，或者在 `ignite` 之后过 `fuse` 秒，它就会碎。爆炸性的道具碎掉时会爆炸。

| 键 | 默认值 | 作用 |
| --- | --- | --- |
| `model` | | 它看起来像什么的场景，只管外观 |
| `size` | 16 16 16 | 它盒形碰撞的半边长，所以默认值是一米宽 |
| `collision` | default | default 和 box 用 `size`，convex hull 和 convex decomposition 跟随模型，见[碰撞](collision.md) |
| `collision_layer`、`collision_mask` | 空 | 像 `1, 3` 这样的层编号，留空则保持项目默认值 |
| `mass` | 5.0 | 质量，单位是千克 |
| `health` | 30 | 碎掉之前能承受的伤害 |
| `explosive` | 0 | 碎掉时爆炸 |
| `explosion_radius`、`explosion_damage` | 192、40 | 那次爆炸的大小和威力 |
| `fuse` | 0.6 | 被点燃的道具在碎掉之前燃烧的秒数 |
| `impact_speed` | 0 | 撞上东西的速度比这个（米每秒）更快时碎掉。0 表示撞击永远不会让它碎 |
| `debris_scene` | | 在它碎掉的地方生成的场景，比如木板 |

* **输入：** `smash`、`ignite(activator)`、`push(direction)`、`take_damage(amount, source)`
* **输出：** `damaged(hp)`，带着剩余的生命值，`broken`

`push` 接受一个以地图单位每秒计的速度。`broken` 在爆炸之后触发，所以它触发的东西看到的已经是受过伤害的结果。

这个道具在每次场景加载时构建它的模型和碰撞盒。Godot 编辑器会把它们显示成预览，这个预览不会随场景保存。

## prop_model { #prop_model }

带可选碰撞的静态模型，用于家具、岩石和其他不该动的道具。

* **键：** `model`（.bbmodel、.glb、.gltf 或 .tscn）、`model_node`、`collision`、`collision_layer`、`collision_mask`、`scale` 1.0

`collision` 为 default 时，每个网格得到一个凸包，适合方方正正的形状。玩家要从中穿过或走在上面的形状，比如拱门，就选 convex decomposition 或 trimesh，想要最便宜的形状就选 box，见[碰撞](collision.md)。只有模型自己没有碰撞时才会生成碰撞。用*模型道具*放置的模型和预制体里的模型都是 prop_model 实体，所以它们的碰撞方式相同。如果一个文件里并排放着好几个变体，`model_node` 只保留指定名字的那个节点，并把它移到道具的原点。

## env_explosion { #env_explosion }

一次受脚本控制的爆炸，比如墙上的炸药。收到 `explode` 时，它会对 `radius` 范围内的每个节点调用 `damage_method`，伤害量是 `damage * (1 - distance / radius)`，把刚体推开，并生成 `effect_scene` 来做闪光和巨响。

* **输入：** `explode(activator)`
* **输出：** `exploded`
* **键：** `radius` 256，`damage` 50，`force` 8（中心处的米每秒，不管质量多大），`damage_method` take_damage，`effect_scene`

距离是量到每个节点的原点，没有视线检查，所以隔着墙也会受伤。

## game_text { #game_text }

在世界里、HUD 上或两处都显示一行文字，用于提示、告示牌或过场字幕。

| 键 | 默认值 | 作用 |
| --- | --- | --- |
| `text` | | 要显示的文字 |
| `place` | world | `world` 漂浮在实体处，`hud` 显示在屏幕上，`both` 两者都做 |
| `start_visible` | 0 | 从地图加载起就显示 |
| `duration` | 0 | 过这么多秒后隐藏它，0 表示一直显示到 `hide` |
| `text_color` | 255 255 255 | 文字颜色 |
| `world_size` | 0.01 | 世界标签的大小，单位是米每像素 |
| `billboard` | 1 | 世界标签朝向相机，并且会穿过墙壁绘制。告示牌请把它关掉，这样它会保持实体的角度，并被墙挡住 |

* **输入：** `show`、`hide`、`set_text(text)`、`flash(seconds)`
* **输出：** `shown`、`hidden`

`show` 和 `hide` 切换的是标签，不是节点。`flash` 把它显示它的参数那么多秒，没有参数就用 `duration`，再没有就是 2 秒。

## light 和 light_spot { #light-and-light_spot }

可以用输入开关的全向灯和聚光灯，用于电灯开关、警报和闪烁的灯。`switched` 只在状态真的变了时才触发。它们属于插件的核心，所以每个项目都有。

* **输入：** `turn_on`、`turn_off`、`toggle`
* **输出：** `switched(on)`
* **键：** `light_energy` 1.0（聚光灯是 1.5），`light_color` 255 255 255，`start_on` 1，`shadows` 0（渲染开销更大），`fixture`，`fixture_off` hide，以及 `omni_range` 10，或者 `spot_range` 15 和 `spot_angle` 35（光锥的半角，单位是度），`bake_mode` auto，`light_size` 0，`light_indirect_energy` 1，以及 `omni_attenuation` 1，或者 `spot_attenuation` 1 和 `spot_angle_attenuation` 1。最后这几个键设置的是同名的 Godot 灯光属性，烘焙时也会用到它们

**灯具**让灯的模型和光保持同步。给灯里发光的几何体起一个 targetname，填进 `fixture`，结尾的 `*` 匹配前缀，所以 `hall_tube_*` 能覆盖每一根灯管。灯亮时显示它，灯灭时隐藏它，不用接线。把 `fixture_off` 设为 `dark`，它会保持可见，只是关掉自发光。`fixture` 也接受其他[目标](../parameters.md#targets)写法，用 `@lamps` 表示节点分组，用 `/root/...` 表示节点路径。编辑器会预览除节点路径之外的每一种写法，节点路径只有 Godot 才能解析。

`bake_mode` 和 `light_size` 决定[烘焙光照](../../editor/light-baking.md#light-keys)怎样对待这盏灯。在 `auto` 下，能被 I/O 开关的灯，也就是有 targetname 或 `start_on` 为 0 的灯，保持实时。

## env_sound { #env_sound }

从关卡里一个点发出的声音，用于警报、机器运转，或者接到某个事件上的一声巨响。

* **输入：** `play`、`stop`、`toggle`
* **输出：** `finished`
* **键：** `sound`（一个 `res://` 音频文件），`volume_db` 0，`max_distance` 0（不限），`autoplay_on` 0（从地图加载起就播放），`loop_sound` 0

## env_particles { #env_particles }

用于火、烟、火花、灰尘和雨的粒子效果，自带几种外观，所以你不必自己搭一个。`burst` 发射一次，之后正在运行的效果继续进行。

* **输入：** `start`、`stop`、`toggle`、`burst`
* **输出：** `finished`
* **键：** `amount` 32（同时存活的粒子数），`lifetime` 2.0，`one_shot` 0，`start_emitting` 0，`effect` rise，`color` 255 255 255，`size` 0，`blend` auto，`area` 10 1 10

`effect` 可以是 `rise`（发光的灰尘或余烬）、`fire`、`smoke`、`sparks` 或 `rain`（从一个宽、高、深各为 `area` 米的盒子里落下）。`color` 给它染色，白色则保持效果自己的颜色。`size` 设置粒子的大小，单位是米，0 就用效果自己的。`blend` 为 `add` 时发光，为 `mix` 时是烟。每种效果的默认值本身就好看，所以先只设 `effect` 就好。

## infodecal { #infodecal }

投射贴花，一个把材质照到盒子里一切东西上的盒子，用于需要包住台阶、管道或凹凸不平地面的污渍。[投射贴花](../../editor/projected-decals.md)会告诉你怎样放置一个，以及要注意什么。

* **键：** `material`（像 `base/stain` 这样的名字或 `res://` 路径），`texture`，`normal_texture`，`size` 64 32 64，`modulate` 255 255 255

`material` 会带上它的纹理和颜色，`texture` 和 `normal_texture` 是没有材质时使用的 `res://` 图片。`size` 是盒子的宽、深和高，单位是地图单位。保持 64 32 64 时，改由材质的[纹理尺寸](../../godot/materials.md#texture-size)来决定宽和高。`modulate` 给贴花染色。

## light_probe、env_reflection_probe 和 env_voxel_gi { #light_probe-env_reflection_probe-and-env_voxel_gi }

给运动中的物体提供光照和反射的探针，见 [Godot 中的烘焙光照](../../godot/baked-lighting.md)。它们都没有输入或输出。

| 实体 | 是什么 | 键 |
| --- | --- | --- |
| `light_probe` | 一个点，往烘焙里加一个光照探针，用于探针网格够不到的地方 | 无 |
| `env_reflection_probe` | 一个笔刷体积，变成和笔刷一样大的 `ReflectionProbe` | `intensity` 1.0，`update` once，`box_projection` 1，`interior` 0，`max_distance` 0 |
| `env_voxel_gi` | 一个笔刷体积，变成和笔刷一样大的 `VoxelGI`，每次构建时在 Godot 编辑器里烘焙 | `subdiv` 128，`energy` 1.0，`bake_on_build` 1 |

把 `update` 设为 `always`，会让反射探针每一帧都捕捉一次，给运动的物体用，开销很大。`box_projection` 让反射和房间的墙壁对齐，`interior` 让没有窗户的房间不带天空，`max_distance` 是探针能看到盒子之外多少米。`subdiv` 是最长一边上的体素数，可选 64、128、256 或 512，越多越清晰，烘焙也越慢。
