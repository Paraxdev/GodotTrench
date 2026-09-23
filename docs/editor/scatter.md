# Scatter

The **Scatter** tool (B) paints trees, rocks and grass onto surfaces you choose. The
[sea island tutorial](../tutorials/sea-island-scatter.md) walks through a full setup.

## Scatter sets

Everything you scatter lives in a **scatter set** on its own layer. Its models show as cards in the Scatter panel,
which opens with the tool.

| To | Do this |
| --- | --- |
| Start an empty set | *+* |
| Start from a preset | *Preset*: forest, pines, mixed woodland, detailed forest, low-poly trees, bushes, undergrowth, rocks, boulders or grass |
| Add models | Drag them in from the Models panel, or *Add models…* |
| Stop painting one model | Untick *paint* on its card. What it already placed stays |
| Tune a model | Open its card for weight, scale, spacing, align, tilt, sink, random yaw and a material override |

No foliage in the project yet? *Terrain > Scatter > Install Nature Models* copies the bundled trees and rocks into
`res://godottrench/nature`.

## Targets

A set only lands on its **targets**, so a forest painted on a terrain never covers the house on it. Alt+click in a view,
or use the eyedropper in the panel, to add or remove a target. A set without targets takes whatever its first stroke
lands on. Another set can be a target too, which is how grass grows on boulders. *Follow cursor* ignores the targets.

## Painting

| Input | Action |
| --- | --- |
| Left drag | Paint |
| Shift+left drag | Erase, from the active set only |
| Ctrl+wheel | Resize the brush |
| **Fill targets** | Cover every target in one go, within the slope and height limits |

Density is attempts per 64 by 64 units, and each model's spacing limits how many fit. **Exposed only** keeps grass out
from under balconies and roofs. **Keep spacing to other sets**, on by default, stops a bush growing through another
set's trunk.

## In Godot

A set's **Kind** decides what Godot builds.

| Kind | Built as |
| --- | --- |
| props | MultiMeshes with collision and shadows. Models whose scenes carry scripts are instanced one by one so the scripts run |
| foliage | MultiMeshes without collision or shadows, hidden beyond 2400 units by default |

*Bake to entities* replaces a set with one `prop_model` entity per instance, for when you need to edit single props.
