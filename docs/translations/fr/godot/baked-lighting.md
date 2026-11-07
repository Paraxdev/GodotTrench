---
source: 5d04cb3d9a7e005f4a371160c23d337e8ee707a7
---

# Éclairage précalculé dans Godot { #baked-lighting-in-godot }

Quand vous [précalculez une carte dans l'éditeur](../editor/light-baking.md), la lightmap est enregistrée dans le `.tdm`.
Godot ne précalcule rien lui-même, chaque construction transforme simplement le précalcul enregistré en nœud de lightmap
propre à Godot.

## Ce que la construction ajoute { #what-the-build-adds }

| Élément | Ce qu'il reçoit |
| --- | --- |
| Un `LightmapGI` nommé `baked_lighting` | Un enfant du `FuncGodotMap`, qui contient la lightmap, le masque d'ombre du soleil et les sondes de lumière |
| Les maillages précalculés | Leurs coordonnées de lightmap comme UV2 et un mode GI statique, chunks streamés compris |
| Les lumières et le soleil | Le mode de précalcul de Godot tiré de leur `bake_mode` ou `sun_bake_mode`, voir [Clés de lumière](../editor/light-baking.md#light-keys) |

Avec le soleil sur `bounce`, le soleil reste en temps réel et le masque d'ombre précalculé ombre ce qui se trouve au-delà de
sa distance d'ombre.

Les faces que vous avez modifiées après le précalcul retombent sur la lumière moyenne de la carte, donc elles paraissent
plausibles mais plates jusqu'à ce que vous précalculiez de nouveau.

## Sondes de lumière { #light-probes }

Les objets mobiles, comme le joueur, les props et les portes, prennent leur lumière aux sondes de lumière autour d'eux, donc
ils reçoivent la lumière rebondie précalculée et se fondent dans la scène.

Le précalcul place des sondes sur une grille de 2 mètres au-dessus de la boîte qui entoure la géométrie précalculée et
supprime celles qui sont dans des solides. Là où la grille est trop grossière, comme un couloir étroit ou un escalier,
ajoutez une entité `light_probe` pour en avoir une de plus. Une carte reçoit environ 1000 sondes au maximum, celles placées
à la main comprises, donc sur une plus grande carte la grille s'espace.

## Entités de sonde { #probe-entities }

| Entité | Type | Nœud Godot | À utiliser pour |
| --- | --- | --- | --- |
| `light_probe` | Point | `LightmapProbe` | Une sonde de lumière en plus pour le précalcul, voir ci-dessus |
| `env_reflection_probe` | Volume brush | `ReflectionProbe` dimensionné sur le brush | Des reflets qui correspondent à une pièce sur les sols brillants et le métal |
| `env_voxel_gi` | Volume brush | `VoxelGI` dimensionné sur le brush | Une lumière rebondie en temps réel qui suit les lumières et objets mobiles |

Vous trouvez les trois dans l'onglet Entités. Glissez `light_probe` dans une vue. Pour les deux volumes, dessinez une boîte
autour de la pièce, puis faites un clic droit sur la carte et choisissez **Y convertir les brushes sélectionnés**.

![L'onglet Entités filtré sur env_, avec la carte env_voxel_gi cliquée avec le bouton droit et Y convertir les brushes sélectionnés en surbrillance](../assets/baked-lighting/entities-turn-brushes-into.png)

Le brush ne donne que la boîte, la construction le supprime. Les clés des trois sont listées dans
[Props, lumières et effets](../gameplay/entities/effects.md#light_probe-env_reflection_probe-and-env_voxel_gi).

Un `env_voxel_gi` est précalculé dans l'éditeur Godot à chaque construction de la carte. Sur un grand volume, réglez
`bake_on_build` sur 0 et précalculez-le à la main avec le bouton **Précalculer** du nœud `VoxelGI` à la place. Les cartes
construites pendant l'exécution du jeu sautent ce précalcul.

## Utiliser un autre lightmapper { #using-another-lightmapper }

Vous voulez précalculer avec le `LightmapGI` propre à Godot ? Décochez **Use Baked Lighting** sur le `FuncGodotMap` pour
laisser de côté le précalcul enregistré. La construction n'ajoute alors pas de nœud `baked_lighting` et laisse tranquilles
les modes de précalcul des lumières. Cochez *Unwrap UV2* dans *Build Flags* pour que les maillages reçoivent l'UV2 dont a
besoin un précalcul Godot.
