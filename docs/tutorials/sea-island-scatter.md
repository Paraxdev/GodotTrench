# Sea island 4: scatter

Scattering places many copies of trees, bushes, rocks or grass with a brush instead of one by one. Each kind of
planting is a set: a list of models plus rules such as density and allowed slope, and the set only grows on its
targets, the surfaces you pick for it.

Press **B** for the Scatter tool. Start each set below from **Preset** in the Scatter panel, then click the eyedropper
next to **Targets** and click the terrain, so the set only grows on it. Leave **Follow cursor** unticked, since it
paints on whatever is under the brush and ignores the targets. Leave **Keep spacing to other sets** ticked, so the
sets keep their distance from each other and nothing grows through a trunk.

## Woodland

1. **Preset** > *mixed woodland*, **Rename** it `woodland` and target the terrain.
2. In the Brush section set **Density** 0.16, **Slope** 0 to 30 and tick **Height**, 120 to 2000. The slope range
   keeps trees off the cliffs, and the height range keeps them off the beach.
3. Click **Fill targets** to plant the whole terrain at once.

Density counts attempts per 64 x 64 units, so 0.16 is roughly one tree every 5 m.

![The woodland after Fill targets](../assets/sea-island/scatter-woodland.jpg)

Hold **Shift** and drag along each path with radius 260 to erase the trees there. Erase a clearing on the main
hilltop (radius about 520) and a meadow behind the beach (about 700).

![Erasing trees off the path](../assets/sea-island/scatter-erase-paths.jpg)

## Hero trees, bushes and boulders

Each row is a new set from the named preset, targeted at the terrain like the woodland. Hero trees are a few large,
detailed trees placed where the player will look closely.

| Set | Preset | How to place it | Radius | Density | Slope | Height |
| --- | --- | --- | --- | --- | --- | --- |
| Hero trees | detailed forest | Single clicks where the paths leave the beach and around the hilltop clearing | 260 | 0.08 | 0 to 30 | off |
| Bushes | bushes | **Fill targets**, then erase along the paths with radius 190 | | 0.35 | 0 to 32 | 45 to 200 |
| Boulders | boulders | One stroke along the top of the cliffs | 1100 | 0.035 | 12 to 50 | 20 to 2000 |

![Hero trees at the foot of the path](../assets/sea-island/scatter-hero-trees.jpg)

![Bushes in a band above the sand](../assets/sea-island/scatter-bushes.jpg)

![Boulders along the cliffs](../assets/sea-island/scatter-boulders.jpg)

## Grass

1. Start a set from the *grass* preset. It is foliage, drawn without collision or shadows, so it can be dense without
   slowing the game down.
2. Paint the open meadows with radius 650, density 2.5, slope 0 to 28, height 45 to 2000. Erase it off the paths with
   radius 170.
3. For grass on the boulders, start one more *grass* set. With the eyedropper, click a boulder instead of the
   terrain, so the boulder set becomes its target. **Fill targets** with density 6 and slope 0 to 45.

![Meadow grass behind the beach](../assets/sea-island/scatter-meadow.jpg)

> **Note:** Fill places grass on the boulders as they are at that moment, so finish the boulders first.

![Grass growing on a boulder](../assets/sea-island/scatter-boulder-grass.jpg)

Next: [Sea island 5: pier and Godot](sea-island-finish.md).
