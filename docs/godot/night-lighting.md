# Night lighting

Lighting is set on worldspawn. Select nothing and edit it in the Inspector.

| Key | Effect |
| --- | --- |
| `sun_energy`, `sun_color` | A low, pale blue sun makes a moon |
| `ambient_energy`, `sky_energy` | Darken the ambient light and the sky |
| `glow_intensity` | Above 0, lamps and [glowing materials](materials.md#glowing-materials) bloom in Godot |
| `ssr` | 1 turns on screen space reflections, so wet streets mirror the lamps |

The editor's Lit Preview shows all of these except glow and reflections.

## Lamp fixtures

`light` and `light_spot` take a `fixture` key. Put the targetname of the lamp's glowing geometry in it, a trailing `*`
matches a prefix, and the geometry follows the light with no wiring: shown while the light is on, hidden while it is
off. Set `fixture_off` to `dark` to keep it visible with its glow switched off instead. See
[Props, lights and effects](../gameplay/entities/effects.md) for the other light keys.

`examples/mcp/night_district.json` and `examples/mcp/withered_city.json` are complete night maps built this way, see
[MCP scripts](../mcp-scripts.md).
