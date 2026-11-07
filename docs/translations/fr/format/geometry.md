---
source: d8695a266e0b8802237d74d9393aab6b51f516c4
---

# Géométrie { #geometry }

## Brush { #brush }

Un solide convexe, stocké sous la forme de ses coins exacts et d'une liste de faces qui les désignent par indice, vous
obtenez donc les coins sans avoir à intersecter des plans. Les plans des faces sont recalculés au chargement.

| Clé | Type | Signification |
| --- | --- | --- |
| `vertices` | tableau de `[x, y, z]` | Coins en unités de carte |
| `faces` | tableau de faces | Un polygone par face |

Chaque face :

| Clé | Type | Par défaut | Signification |
| --- | --- | --- | --- |
| `indices` | tableau d'entiers | obligatoire | Indices dans `vertices`, au moins trois, dans le sens antihoraire vu de l'extérieur |
| `material` | chaîne | `""` | Toujours écrit. Nom de texture, par exemple `base/wall` |
| `uv` | objet | paraxial | Toujours écrit. La manière dont la texture est posée sur la face, voir [Projection de texture](#texture-projection) |
| `props` | objet de chaîne vers chaîne | omis | Attributs de face libres, comme les clés de mélange ci-dessous |
| `disp` | objet | omis | Transforme la face en [displacement](#displacement) |
| `colors` | tableau de `[r, g, b, a]` | omis | [Peinture de sommets](../editor/texturing.md#vertex-paint), une couleur par indice |

La construction Godot saute un brush qui a moins de quatre sommets ou faces, puisqu'aucun solide fermé n'en a moins.

Une face peut aussi mélanger deux matériaux. Les props de face `blend_material`, `blend_detile`, `blend_uv_scale` et
`blend_detile_sharpen` le configurent, puis l'alpha de la couleur de sommet fait passer progressivement `material` vers
`blend_material` sur la face.

## Projection de texture { #texture-projection }

`uv` est une projection de style Valve 220 avec des axes explicites : deux directions du monde disent dans quel sens la
texture court sur la face, et un décalage et une échelle la placent.

| Clé | Type | Signification |
| --- | --- | --- |
| `u_axis`, `v_axis` | `[x, y, z]` | Direction dans le monde du u et du v de la texture |
| `offset` | `[u, v]` | Décalage en texels |
| `scale` | `[u, v]` | Unités de carte par texel |
| `rotation` | nombre | Facultatif et purement informatif, les axes contiennent déjà la rotation |

Un point `p` tombe sur le texel `dot(p, u_axis) / scale.x + offset.x`, et de même pour v. Pour l'UV final, divisez le
résultat par la taille de texture. C'est le `texture_size` du matériau quand il en a un, voir
[Matériaux](../godot/materials.md), sinon la taille en pixels de l'image.

Un `uv` absent signifie `u_axis = [1, 0, 0]` et `v_axis = [0, 0, 1]` à l'échelle 1. Quand `uv` est présent, toutes les
clés sauf `rotation` sont obligatoires.

## Réglages de texture { #texture-settings }

L'objet `textures` de niveau supérieur contient les réglages partagés par toutes les faces qui ont un matériau, indexés
par le nom du matériau. Seuls les réglages qui diffèrent de la valeur par défaut sont écrits, et un matériau qui n'en a
aucun est omis.

| Clé | Type | Par défaut | Signification |
| --- | --- | --- | --- |
| `bake` | booléen | `true` | Les faces obtiennent une place dans la lightmap. Désactivé, Godot leur donne la lumière précalculée moyenne |
| `casts` | booléen | `true` | Les faces bloquent la lumière dans le précalcul |
| `texel_scale` | nombre | `1` | Multiplie la taille des texels de la lightmap sur ces faces, de 0,125 à 16 |
| `size` | `[w, h]` | aucun | Unités de carte que couvre une répétition, utilisées à la place de la taille de texture ci-dessus |
| `projection` | chaîne | `"face"` | `"world"` ignore le `uv` de chaque face de brush et utilise la projection réinitialisée de sa normale, décalage 0 et échelle 1 |

`world` ne s'applique qu'aux faces de brush, les faces de mesh gardent leur projection.

## Displacement { #displacement }

Une face quadrilatère peut porter un displacement de style Hammer, une grille de sommets qui sont chacun poussés le long
de la normale de la face.

| Clé | Type | Par défaut | Signification |
| --- | --- | --- | --- |
| `power` | entier | obligatoire | De 1 à 4. La grille a `2^power + 1` sommets par côté |
| `heights` | tableau de nombres | obligatoire | De combien chaque sommet de la grille est poussé le long de la normale |
| `alphas` | tableau de nombres | omis | Poids de mélange par sommet de la grille, 0 montre le matériau de la face |

Les deux tableaux sont en ordre ligne principale avec `(2^power + 1)^2` entrées. Les lignes vont du premier coin de la
face vers son quatrième, les colonnes du premier coin vers le deuxième.

## Mesh { #mesh }

Un mesh polygonal modifiable. Contrairement à un brush, il peut être concave, ouvert, ou avoir des faces qui ne sont pas
planes.

| Clé | Type | Par défaut | Signification |
| --- | --- | --- | --- |
| `vertices` | tableau de `[x, y, z]` | obligatoire | Coins en unités de carte |
| `faces` | tableau de faces | obligatoire | Les mêmes clés que les faces de brush, plus `uvs` |
| `smooth_angle` | nombre | `0`, omis | Les faces qui se rencontrent à moins de ce nombre de degrés partagent leurs normales et s'ombrent en douceur. 0 donne un ombrage plat |
| `decal` | booléen | `false`, omis | Marque le mesh comme un [decal](../editor/decals.md) |

Le lissage compare chaque coin de face avec chaque face autour de son sommet. Un mesh qui demande plus de 64 comparaisons
par coin, ou 262 144 en tout, est ombré à plat à la place. Seul un sommet partagé par des centaines de faces, comme la
pointe d'un cône de plus de 650 côtés environ, y arrive.

Les `uvs` d'une face de mesh sont un tableau de `[u, v]`, un par indice, en largeurs de texture. Quand ils sont là, ils
remplacent `uv`, et un nombre qui ne correspond pas à `indices` fait refuser la carte par l'éditeur.
