# Scatter

The **Scatter** tool (B) paints trees, rocks and grass onto surfaces you choose, so you can cover a hillside in seconds
instead of placing every model by hand. The [sea island tutorial](../tutorials/sea-island-scatter.md) walks through a
full setup.

## Scatter sets

Everything you scatter lives in a **scatter set** on its own layer, such as one set for trees and one for grass.
The set's models show as cards in the Scatter panel, which opens with the tool.

| To | Do this |
| --- | --- |
| Start an empty set | *+* |
| Start from a preset | *Preset*, then pick forest, pines, mixed woodland, detailed forest, low-poly trees, bushes, undergrowth, rocks, boulders or grass |
| Add models | Drag them in from the Models panel, or *Add models…* |
| Stop painting one model | Untick *paint* on its card. What it already placed stays |
| Tune a model | Open its card, see the table below |

| Card setting | What it does |
| --- | --- |
| weight | How often this model is picked compared to the others in the set |
| scale | A random size between the two values |
| spacing | The smallest distance in map units to any other instance |
| align | 0 stands the model upright, 1 tilts it to follow the surface |
| tilt | The largest random lean in degrees |
| sink | How far the model is pushed into the ground, to hide the bottom of a trunk or rock |
| random yaw | Turns each instance to a random direction |
| material | Drawn instead of the model's own materials. Left empty, the set's material applies, if it has one |

No foliage in the project yet? *Terrain > Scatter > Install Nature Models* copies the bundled trees and rocks into
`res://godottrench/nature`.

## Targets

A set only lands on its **targets**, so a forest painted on a terrain never covers the house on it. Alt+click in a view,
or use the eyedropper in the panel, to add or remove a target. A set without targets takes whatever its first stroke
lands on. Another set can be a target too, which is how grass grows on boulders.

Ticking *Follow cursor* ignores the targets and paints on any surface or scattered prop under the brush.

## Painting

| Input | Action |
| --- | --- |
| Left drag | Paint |
| Shift+left drag | Erase, from the active set only |
| Ctrl+wheel | Resize the brush |
| **Fill targets** | Cover every target in one go, within the slope and height limits |

**Density** is how many placements the brush tries per 64 by 64 units, and each model's spacing limits how many
actually fit. **Exposed only** skips points with geometry above them, which keeps grass out from under balconies and
roofs. **Keep spacing to other sets**, on by default, stops a bush growing through another set's trunk.

## In Godot

A set's **Kind** decides what Godot builds.

| Kind | Built as |
| --- | --- |
| props | MultiMeshes with collision and shadows. Models whose scenes carry scripts are instanced one by one so the scripts run |
| foliage | MultiMeshes without collision or shadows, hidden beyond 2400 units by default. Meant for grass and small plants |

*Bake to entities* replaces a set with one `prop_model` entity per instance, for when you need to edit single props.
