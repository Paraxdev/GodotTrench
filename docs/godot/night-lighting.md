# Night lighting

For a night map, turn the global light down so the lamps you place do the work. It is set on worldspawn: select
nothing and edit it in the Inspector.

| Key | Default | What to do for night |
| --- | --- | --- |
| `sun_energy`, `sun_color` | 1.1, `255 245 224` | Lower the energy and pick a pale blue, and the sun reads as a moon |
| `ambient_energy`, `sky_energy` | 1.0, 1.0 | Lower both, so unlit areas and the sky go dark |
| `glow_intensity` | 0 | Above 0, lamps and [glowing materials](materials.md#glowing-materials) bloom in Godot |
| `ssr` | 0 | 1 turns on screen space reflections, so wet streets and glossy floors mirror the lamps |

The editor's Lit Preview shows all of these except glow and reflections, so check those in Godot.

## Lamp fixtures

A lamp is a `light` or `light_spot` entity plus the geometry of the lamp with its glowing bulb. Put the targetname of
that geometry in the light's `fixture` key, a trailing `*` matches a prefix. The geometry then follows the light with
no wiring: shown while the light is on, hidden while it is off. Set `fixture_off` to `dark` to keep it visible with
its glow switched off instead. See [Props, lights and effects](../gameplay/entities/effects.md) for the other light
keys.

{% mcp %}

## MCP

The showcase maps are built by [MCP scripts](../mcp-scripts.md) that drive the editor through its
[MCP server](../mcp.md). `examples/mcp/night_district.json` and `examples/mcp/withered_city.json` are complete night
maps built this way.

{% endmcp %}
