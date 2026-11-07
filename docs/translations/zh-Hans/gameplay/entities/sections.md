---
source: 721efa7c5ececa362093af607d2e2c2faa92ad9b
---

# 流式分区 { #streamed-sections }

> [!NOTE]
> 这些实体来自玩法实体包，见[安装与自定义](README.md#installing-and-customizing)。

用来把多张地图当成一个世界来玩、没有加载画面的实体。每张地图是一个分区，分区在一个连接房间里相接，两张地图里都有这个房间的一份完全相同的副本。[流式加载分区](../../godot/streaming.md#streaming-sections)会带你一步步设置。

## info_landmark { #info_landmark }

连接房间的共享点。在房间中央放一个，两份副本里用同一个名字。下一个分区进来时，会把它放在让自己的地标正好落在旧地标位置上的地方。

* **键：** `targetname`

## func_section_stream { #func_section_stream }

连接房间的一半，是一个不可见的体积。玩家站在里面时，它指名的那个分区就是被加载的分区。一个连接房间需要两个，每一侧一个，它们相接的那条线就是分区交换的地方，所以要把它放在房间两端都看不到的位置，比如一条 Z 字形走廊的中间。

* **键：** `section`（这一侧的分区：它的场景路径，比如 `res://levels/pools_2.tscn`，或者在流式加载器的 `sections_dir` 里能找到的名字），`landmark`（这个房间里 `info_landmark` 的 `targetname`）

## TDSectionStreamer { #tdsectionstreamer }

它不是实体，而是负责流式加载的节点。把它加到你的世界场景里，然后设置：

| 属性 | 是什么 |
| --- | --- |
| `first_section` | 游戏开始所在的分区，一个场景路径或一个名字 |
| `sections_dir` | 查找短名字的文件夹，找的是 `name.tscn` 或 `name.scn` |
| `player_scene` | 要在第一个分区的 `info_player_start` 处生成的玩家 |
| `player_path` | 或者场景里已有的玩家。两者都没有时，会保留某个分区带来的 `player` 组里的第一个节点 |
| `recenter_distance` | 一圈分区循环可以偏离原点多少米，整个世界才会移回来，8000 |

每次交换之后，它都会发出 `section_changed(from, to)`。
