# Streamed sections

> **Note:** These entities come with the Gameplay entities pack, see
> [Installing and customizing](README.md#installing-and-customizing).

Entities for playing several maps as one world with no loading screen. Each map is a section, and sections meet in a
connector room that both maps hold an identical copy of. [Streaming sections](../../godot/streaming.md#streaming-sections)
walks through setting it up.

## info_landmark

The shared point of a connector room. Put one in the middle of the room, with the same name in both copies. When the
next section comes in, it is placed so its landmark lands exactly where the old one was.

* **Keys:** `targetname`

## func_section_stream

One half of a connector room, an invisible volume. While the player stands in it, the section it names is the one
loaded. A connector needs two, one per side, and the line where they meet is where the sections swap, so put it where
neither end of the room can be seen, for example in the middle of a Z shaped corridor.

* **Keys:** `section` (the section on this side: its scene path, like `res://levels/pools_2.tscn`, or a name found in
  the streamer's `sections_dir`), `landmark` (the `targetname` of the room's `info_landmark`)

## GTSectionStreamer

Not an entity but the node that runs the streaming. Add it to your world scene and set:

| Property | What it is |
| --- | --- |
| `first_section` | The section the game starts in, a scene path or a name |
| `sections_dir` | Folder short names are found in, as `name.tscn` or `name.scn` |
| `player_scene` | The player to spawn at the first section's `info_player_start` |
| `player_path` | Or a player already in the scene. With neither, the first node in the `player` group a section brings is kept |
| `recenter_distance` | Meters a loop of sections may drift from the origin before the whole world moves back, 8000 |

It emits `section_changed(from, to)` after every swap.
