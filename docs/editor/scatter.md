# Scatter

The **Scatter** tool (B) paints trees, rocks and grass onto surfaces you choose.

## Scatter sets

Everything you scatter lives in a **scatter set** on its own layer. A set is a palette of models, shown as cards in the
Scatter panel that opens when you pick the tool.

| To | Do this |
| --- | --- |
| Start an empty set | *+* |
| Start from a preset | *Preset*: forest, pines, mixed woodland, detailed forest, bushes, low-poly trees, undergrowth, rocks, boulders or grass |
| Add models | Drag them in from the Models panel, or *Add models…* |
| Stop painting one model | Switch its card off. What it already placed stays |
| Tune a model | Open its card for weight, spacing, scale, alignment, tilt, sink and a material override |

No foliage in your project yet? *Terrain > Scatter > Install Nature Models* copies the bundled trees and rocks into
`res://godottrench/nature`.

## Targets

A set only lands on its **targets**, so painting a forest on a terrain never covers the house standing on it.

Add or remove a target with the eyedropper in the panel, or Alt+click in a view. A set without targets takes whatever
its first stroke starts on. Targets can be brushes, meshes, terrains or **another scatter set**, which is how you grow
grass on top of boulders.

*Follow cursor* ignores the targets and paints on whatever is under the brush.

## Painting

| Input | Action |
| --- | --- |
| Left drag | Paint |
| Shift+left drag | Erase, from the active set only |
| Ctrl+wheel | Resize the brush |
| **Fill targets** | Cover every target in one go, within the slope and height limits |

The Brush section of the panel holds density, slope and height limits and a seed. Density is instances per 64 by 64
units. **Exposed only** skips spots with geometry above them within the clearance height, so grass stays out from under
balconies and roofs.

Leave **Keep spacing to other sets** ticked so a new set keeps clear of what other sets placed, and no bush grows
through a trunk.

## In Godot

Every set becomes MultiMeshes with shared collision shapes, or instanced scenes when the model has scripts. Foliage sets
(like the grass preset) have no collision or shadows and fade out with distance.

*Bake to entities* turns a set into one `prop_model` entity per instance, when you need to edit single props.

The [sea island tutorial](../tutorials/sea-island.md) walks through a full scatter setup.
