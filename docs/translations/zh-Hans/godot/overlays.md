---
source: 5a39bcf2beab2f567018f04963913fd06af5df16
---

# 叠加层 { #overlays }

每次构建都会替换地图生成的内容，所以你手动加在地图节点下面的任何东西，在下次保存时都会消失。叠加层就是用来放那些属于这张地图、又必须经得起每次重新构建的 Godot 内容：带自己脚本的自动售货机、在 Godot 里调好的粒子效果、带 AnimationPlayer 的灯。

![夜晚街区的庭院：来自叠加层的彩灯串、萤火虫和招牌，以及来自地图的灯](../assets/overlays/night-district-courtyard.jpg)

## 添加一个 { #adding-one }

1. 添加叠加层节点

    在 Godot 里，添加一个 `TrenchDotOverlay` 节点，作为 `FuncGodotMap` 的直接子节点。

2. 把你的节点放在它下面

    构建、实时模式和热重载都不会碰它，所以它的节点会保持自己的状态和信号连接，并随你的场景一起保存。

（只有一个节点的话，可以不用叠加层：属于 `trenchdot_keep` 组的、地图的直接子节点，同样能经受住构建。场景里别处的叠加层可以改用 *Map Path* 指向它的地图，这样它就会跟随地图的变换。）

叠加层的内容使用地图的局部空间，单位是米。按每米 32 个单位，位于 `(2, 0, -3)` 的子节点，就落在 TrenchDot 显示为 `(64, 0, -96)` 的位置。

## 放在自己场景里的叠加层 { #an-overlay-in-its-own-scene }

叠加层也可以是一个独立的场景。当几个场景用同一张地图、应该共用同一个叠加层时，当一个人做关卡场景、另一个人做叠加层时，或者当关卡场景由脚本生成时（比如演示里），这样做就很划算。

1. 做一个根节点是 `TrenchDotOverlay` 的场景。
2. 把 `.tscn` 拖到 `FuncGodotMap` 节点上，让它作为地图的直接子节点被实例化。

Godot 把实例化的场景显示成单个节点，没有展开它的箭头。要进到它的内容里：

| 你想要 | 这样做 |
| --- | --- |
| 在所有用到叠加层的地方编辑它 | 点击场景面板里节点旁边的 *Open in Editor* 图标，或者在文件系统面板里双击那个 `.tscn` |
| 只在这个场景里查看或修改它的节点 | 右键点击节点，打开 *Editable Children* |

单独打开时，叠加层场景会在它的根节点上显示一个警告，因为它不在任何地图之下。这是正常的。TrenchDot 里的虚线框会在下一次构建或保存带有地图的场景时跟上，因为只有在那里，叠加层才知道它的地图。

## 连接叠加层 { #wiring-overlays }

叠加层节点和地图实体，可以通过地图实体彼此之间用的同一套输出和输入，互相触发。

| 你想要 | 这样做 |
| --- | --- |
| 让地图的输出到达某个叠加层节点 | 给这个节点一个 `TrenchDotOverlayIO` 子节点，并设置它的 *Targetname* |
| 让叠加层节点向地图实体触发 | 添加一个 `TrenchDotOutput` 子节点，在 *Output* 里选择信号，并填写目标、输入和参数 |
| 让内容跟着一个移动的实体走 | 把它放在 `TrenchDotAnchor` 下面，并把 *Target* 设为那个实体的 targetname |

发送给 `TrenchDotOverlayIO` 的输入，会调用节点里同名的方法，设置它同名的属性，或者运行 `show`、`hide` 或 `toggle`。所以没有脚本的节点也能用：输入 `emitting`，参数 `true`，就会启动一个 `GPUParticles3D`。每个输入还会发出 `input_received(input, parameter, activator)`。

`TrenchDotOutput` 会监听它父节点的任何信号，所以 `Area3D` 不用写代码就能在 `body_entered` 时触发。对于你自己的事件，在父节点的脚本里添加一个信号，再选择它：

```gdscript
extends StaticBody3D

signal flipped_on(activator: Node)
signal flipped_off(activator: Node)

var on := false

func use(activator: Node = null) -> void:
	on = not on
	if on:
		flipped_on.emit(activator)
	else:
		flipped_off.emit(activator)
```

锚点会保持你放置它时的偏移，并在重新构建、实时编辑和移动中跟随实体，所以警示牌会随着卷帘门一起升起。

> [!WARNING]
> 连线能经受住重新构建，状态则不能。地图实体会以起始状态回来，所以请连接叠加层的 `map_rebuilt(map)` 信号，把你的状态再推回去，比如把灯重新打开。

## 在演示里看看 { #see-it-in-the-demo }

演示项目的夜晚街区把这些全都用上了。它的场景 `res://demo/showcase/night_district.scn` 在它的地图下实例化了叠加层场景 `res://demo/overlays/night_district_overlay.tscn`，因为展示场景是由脚本重新构建的。打开叠加层场景，你会找到一个控制庭院灯的配电箱、由地图触发器打开的彩灯串，以及一块跟着车库卷帘门走的招牌。
