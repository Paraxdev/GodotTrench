---
source: 59a13276848e83b80ec8fffb5e9295c0444e1a52
---

# Terrain et scatter { #terrain-and-scatter }

## Terrain { #terrain }

Une heightmap sur une grille régulière, une hauteur par sommet de la grille, peinte avec jusqu'à quatre calques de
texture. Les données volumineuses sont du texte base64 en JSON et des octets bruts dans le
[conteneur binaire](container.md#value-encoding).

| Clé | Type | Par défaut | Signification |
| --- | --- | --- | --- |
| `origin` | `[x, y, z]` | obligatoire | Position dans le monde du premier sommet |
| `resolution` | `[nx, nz]` | obligatoire | Nombre de sommets le long de X et de Z, au moins 2 chacun |
| `cell_size` | nombre | obligatoire | Unités de carte entre sommets voisins |
| `heights` | base64 | obligatoire | Flottants 32 bits little endian, une hauteur par sommet |
| `layers` | tableau | obligatoire | Jusqu'à quatre calques de texture, voir plus bas |
| `splat` | base64 | omis | Quatre octets de poids par sommet, un par calque, qui disent dans quelle mesure chaque calque apparaît à cet endroit |
| `holes` | base64 | omis | Un octet par cellule, une valeur non nulle découpe la cellule hors du terrain |
| `chunk_cells` | entier | `32` | Toujours écrit. Cellules par côté de bloc. Le terrain est découpé en blocs de cette taille pour le rendu et les collisions |

Le sommet `(i, j)` se trouve à `origin + (i * cell_size, heights[j * nx + i], j * cell_size)`. `splat` suit le même
ordre, ses poids sont normalisés à la lecture, et une entrée entièrement nulle signifie le calque 0. `holes` couvre les
`(nx - 1) * (nz - 1)` cellules dans le même ordre.

Chaque calque :

| Clé | Type | Par défaut | Signification |
| --- | --- | --- | --- |
| `material` | chaîne | obligatoire | Matériau peint par ce calque |
| `tile` | nombre | `256` | Unités de carte par répétition de texture |
| `detile` | nombre | `0` | De 0 à 1, à quel point la répétition est brisée pour que le motif de grille cesse de se voir |
| `detile_sharpen` | nombre | `0.5` | De 0 à 1, à quel point le résultat détuilé reste net |

Voir [Calques de terrain](../editor/terrain-layers.md) pour ce que ces réglages donnent en pratique.

Les terrains restent alignés sur les axes. Une instance déplace le centre d'un terrain avec toute sa transformation,
rotation comprise, mais ne fait jamais tourner la grille elle-même.

## Scatter { #scatter }

Un ensemble de scatter garde dans un seul nœud de nombreuses copies de quelques modèles, comme des arbres, des rochers
ou de l'herbe peints sur des surfaces. Voir [Scatter](../editor/scatter.md) pour le côté éditeur. Chaque clé sauf
`targets` et `material` est toujours écrite.

| Clé | Type | Par défaut | Signification |
| --- | --- | --- | --- |
| `name` | chaîne | obligatoire | Nom affiché dans l'éditeur |
| `kind` | chaîne | `props` | Ce que Godot construit. `props` garde les scripts et les collisions, `foliage` est un MultiMesh sans collision |
| `targets` | tableau d'ids de nœuds | omis | Les surfaces sur lesquelles l'ensemble est peint |
| `items` | tableau | obligatoire | Les modèles qu'il disperse, voir plus bas |
| `collision` | chaîne | `none` pour le feuillage, sinon `convex` | Forme de collision par instance : `none`, `box`, `convex`, `decomposition` ou `trimesh` |
| `cast_shadows` | booléen | `true` | Si les instances projettent des ombres |
| `visibility_range` | nombre | `0` | Masque les instances au-delà de cette distance, 0 signifie jamais |
| `chunk_size` | nombre | `0` | Découpe l'ensemble en cellules de cette taille pour que Godot puisse éliminer ce qui est hors écran, 0 donne une seule cellule. Les nouveaux ensembles utilisent 2048 |
| `static_props_multimesh` | booléen | `false` | Dessine aussi en MultiMesh les scènes de props qui portent des scripts. Moins coûteux pour beaucoup de props, mais leurs scripts sont abandonnés |
| `material` | chaîne | omis | Matériau de remplacement pour tout l'ensemble |
| `instances` | tableau | vide | Les instances placées, voir plus bas |

Copiez un ensemble avec une surface qu'il cible, et la copie cible la surface copiée.

Chaque item est un modèle que l'ensemble peut placer :

| Clé | Type | Par défaut | Signification |
| --- | --- | --- | --- |
| `source` | chaîne | obligatoire | Modèle ou scène `res://` |
| `weight` | nombre | `1` | Chance relative que cet item soit choisi |
| `scale` | `[min, max]` | `[0.8, 1.2]` | Plage d'une échelle uniforme aléatoire |
| `spacing` | nombre | `48` | Distance minimale à chaque autre instance |
| `align` | nombre | `0` | 0 le tient droit, 1 suit la normale de la surface, les valeurs intermédiaires l'inclinent en partie |
| `random_yaw` | booléen | `true` | Oriente chaque instance dans une direction aléatoire |
| `tilt` | nombre | `0` | Plus grande inclinaison aléatoire en degrés |
| `sink` | nombre | `0` | Unités de carte enfoncées dans la surface, pour que les racines et les bases de rochers ne flottent pas |
| `material` | chaîne | omis | Matériau de remplacement pour cet item. Il l'emporte sur celui de l'ensemble |
| `enabled` | booléen | `true`, omis | Si le pinceau peint cet item. Les instances déjà placées restent |

Chaque instance est un tableau plat `[item, x, y, z, pitch, yaw, roll, scale]`, où `item` est un indice dans `items` et
les angles sont en degrés, dans l'ordre YXZ. À l'enregistrement, la position est arrondie à 0,01, les angles à 0,1 et
l'échelle à 0,001, ce qui permet au conteneur de les stocker en [colonnes d'entiers](container.md#value-encoding).
