# Terrain layers

A terrain has one to four layers, edited in the Inspector with the terrain selected. Each layer is a material, a
**tile** size and two de-tiling settings. Paint is stored as one weight per layer on every vertex, and the weights
always add up to one, so painting sand onto grass takes weight away from grass.

![The sea island's four layers in the Inspector](../assets/terrain-painting/layers-inspector.png)

| To | Do this |
| --- | --- |
| Add a layer | *+ layer (current material)* |
| Remove the last layer | *- last layer* |
| Change a layer's material | Type its name, or drop a material on the terrain. With the Blend tool active it fills the layer the tool paints |

**Layer 0 is the base ground.** Unpainted ground shows layer 0 and erasing paints back to it, so put your most common
ground there, usually grass.

A new terrain gets the layers named in the *Create Terrain* dialog, four by default. Dropping a material for a layer the terrain
lacks adds it as the next layer, and painting a missing layer paints the last one instead. The status bar says so in
both cases.

## Tile and de-tiling

| Setting | Meaning |
| --- | --- |
| tile | Map units one repeat covers. 128 to 384 works well for walkable ground |
| detile | 0 to 1. Turns and shifts every repeat randomly so the grid stops showing |
| sharpen | 0 to 1, shown once detile is above 0. 0 mixes the repeats evenly and looks soft, 1 mixes only where they join |

A small tile is sharp up close but repeats visibly from afar, a large one hides the repeat but blurs at your feet.
Detile fixes the repeat without a bigger tile. It costs four texture reads per projection, so leave it at 0 where the
repeat does not show.

![Tile 256 with detile 0 on the left and 0.7 on the right](../assets/terrain-painting/detile-compare.jpg)

The tile is independent of the texture's pixel size and of the material's `metadata/texture_size`, those only apply to brush
and mesh faces.

## How it looks in Godot

Each layer is projected from above and from the two sides and blended by the surface angle (triplanar mapping), so
cliffs show rock instead of a smear. The projection is anchored to the world and matches the editor.

Only each material's colour texture and emission are used, normal and roughness maps are not. Pick textures with some
light and shadow baked in.

![The painted cliffs of the sea island in Godot](../assets/terrain-painting/godot-cliffs.jpg)
