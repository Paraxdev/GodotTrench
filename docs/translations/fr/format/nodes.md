---
source: d21533e024f6fb1ea2fed11362601fee5c4efd4d
---

# Nœuds { #nodes }

Le contenu d'une carte est un arbre de nœuds. Chaque nœud a la même forme extérieure, avec ses clés dans cet ordre :

| Clé | Type | Écrite | Signification |
| --- | --- | --- | --- |
| `id` | entier | toujours | Unique dans la carte |
| `type` | chaîne | toujours | `layer`, `group`, `entity`, `brush`, `mesh`, `terrain`, `scatter` ou `instance` |
| champs du type | | | Les clés de ce type de nœud, voir plus bas, [Géométrie](geometry.md) et [Terrain et scatter](terrain-scatter.md) |
| `label` | chaîne | si défini | Nom donné avec *Renommer*, voir plus bas |
| `graph` | tableau de 2 nombres | si défini | Où se trouve le nœud d'une entité dans le [graphe Logic](../gameplay/logic-graph.md) |
| `hidden` | booléen | si `true` | Masqué dans l'éditeur |
| `locked` | booléen | si `true` | Ne peut pas être sélectionné ni modifié dans l'éditeur |
| `children` | tableau de nœuds | si non vide | Nœuds enfants, dans l'ordre |

`hidden` et `locked` ne sont qu'un état de l'éditeur, Godot construit donc quand même un nœud masqué. Pour garder
quelque chose hors de la construction, placez-le dans un calque avec `omit_from_export`.

`label` est le nom que vous avez donné à un brush, un mesh, une entité, une instance ou un terrain dans l'éditeur,
affiché à la place d'un nom par défaut comme `brush12`. Les calques, les groupes et les ensembles de scatter n'en ont
jamais, ils gardent le leur dans `name`.

Godot n'utilise `label` que pour les terrains, il nomme le nœud `terrain_<id>_<label>`. Les brushes et les meshes
fusionnent dans leur entité, et une entité tire le nom de son nœud de `targetname`. Les éditeurs plus anciens que cette
clé lisent quand même les fichiers qui l'ont, mais l'abandonnent quand ils enregistrent.

`graph` est aussi un état de l'éditeur. C'est l'endroit où se trouve le nœud d'une entité dans le graphe Logic, en
points de graphe, et seuls les nœuds que quelqu'un a déplacés l'ont. Godot l'ignore, et une valeur qui n'est pas faite
de deux nombres est abandonnée au chargement.

## Ids { #ids }

Les ids sont des entiers positifs qui restent les mêmes d'un enregistrement à l'autre, pour que d'autres données
puissent désigner un nœud. Le live link envoie les modifications par id, les ensembles de scatter nomment leurs surfaces
par id, et Godot stocke l'id de chaque nœud généré dans sa métadonnée `_td_id`. Un id de `0` ou en double, ce que seul
un fichier modifié à la main possède, reçoit un nouvel id au chargement.

Un prefab est un fichier `.tdm` à part avec ses propres ids, donc deux copies de lui se heurteraient. Pour l'éviter, la
construction Godot ajoute un million de fois un numéro d'occurrence à l'id de chaque nœud qu'apporte une
[instance](#instance). Deux copies d'un même prefab ne partagent jamais d'ids, quelle que soit la profondeur de leur
imbrication.

## Où vont les nœuds { #where-nodes-go }

| Nœud | Parent autorisé |
| --- | --- |
| Calque | Niveau supérieur uniquement |
| Groupe | Calque ou groupe |
| Brush, mesh, terrain, ensemble de scatter, entité point, instance | Calque ou groupe |
| Brush ou mesh d'une entité brush | Cette entité |

Une entité qui a des enfants est une entité brush, par exemple une porte faite de brushes. Les brushes et les meshes
hors de toute entité appartiennent au worldspawn.

## Calque { #layer }

Une partie de niveau supérieur de la carte, affichée dans l'Outliner de l'éditeur.

| Clé | Type | Par défaut | Signification |
| --- | --- | --- | --- |
| `name` | chaîne | obligatoire | Nom affiché dans l'Outliner |
| `color` | chaîne | obligatoire | `#rrggbbaa`. `#rrggbb` est accepté à la lecture |
| `omit_from_export` | booléen | `false` | Toujours écrit. Quand il vaut true, la construction saute tout le calque, pour de la géométrie de référence qui ne doit jamais arriver dans Godot |

## Groupe { #group }

Des nœuds rassemblés pour que vous les sélectionniez et les déplaciez ensemble.

| Clé | Type | Par défaut | Signification |
| --- | --- | --- | --- |
| `name` | chaîne | obligatoire | Nom affiché dans l'Outliner |
| `link_id` | entier | omis | Les groupes qui partagent un id de lien sont des copies liées, faites avec *Dupliquer en lien* |
| `transform` | 16 nombres | identité, omis | Le placement de cette copie par rapport aux autres, une matrice 4x4 en ordre colonne principale |

Le contenu d'un groupe est stocké dans l'espace du monde, déjà en place. Une [copie liée](../editor/organizing.md#groups)
répète dans toutes les autres une modification faite dans l'une, et `transform` est simplement ce que l'éditeur utilise
pour porter la modification d'une copie à la suivante. La construction Godot ignore à la fois `transform` et `link_id`.

## Entité { #entity }

| Clé | Type | Par défaut | Signification |
| --- | --- | --- | --- |
| `classname` | chaîne | obligatoire | Classe d'entité de la config de jeu, par exemple `prop_physics` |
| `origin` | `[x, y, z]` | `[0, 0, 0]` | Toujours écrit. Position d'une entité point |
| `angles` | `[pitch, yaw, roll]` | `[0, 0, 0]` | Toujours écrit. Degrés, ordre YXZ comme `rotation_degrees` |
| `properties` | objet de chaîne vers chaîne | omis | Les clés de l'entité, triées par nom |
| `outputs` | tableau | omis | Connexions d'E/S, voir plus bas |

Les valeurs de propriété sont toujours des chaînes, quel que soit leur type FGD, par exemple `"travel": "0 168 0"`.

Une propriété `angles`, `angle` ou `mangle` l'emporte sur les `angles` du nœud et se lit à la manière de Quake : pitch et
roll inversés, yaw tourné de 180 degrés. Sur les classes dont le nom commence par `light`, `mangle` signifie plutôt yaw,
pitch et roll. Un `angle` de -1 ou -2 pointe droit vers le haut ou vers le bas. L'éditeur dessine l'entité tournée dans
le même sens, et la tourner à cet endroit déplace la propriété dans les `angles` du nœud.

La construction ignore `origin` et `angles` d'une entité brush, puisque ses brushes sont déjà là où ils doivent être.

Chaque output est une [connexion d'E/S](../gameplay/io.md) : quand cette entité émet un signal, appeler une méthode sur
une cible.

| Clé | Type | Par défaut | Signification |
| --- | --- | --- | --- |
| `output` | chaîne | obligatoire | Signal de cette entité qui déclenche la connexion |
| `target` | chaîne | obligatoire | Qui reçoit l'appel, voir [Cibles](../gameplay/parameters.md#targets) |
| `input` | chaîne | obligatoire | Méthode à appeler sur la cible |
| `parameter` | chaîne | `""`, omis | Valeur transmise à l'input |
| `delay` | nombre | `0`, omis | Secondes à attendre avant l'appel |
| `times` | entier | `-1`, omis | Combien de fois elle peut se déclencher, `-1` pour chaque fois |

## Instance { #instance }

Une référence à un autre fichier `.tdm`, placée avec une position et une rotation, comme le `func_instance` de Hammer.
C'est ainsi que les prefabs sont placés : le contenu vit une seule fois dans son propre fichier, et chaque instance le
montre.

| Clé | Type | Par défaut | Signification |
| --- | --- | --- | --- |
| `path` | chaîne | obligatoire | La carte référencée, relative à la carte qui la référence, absolue, ou `res://` |
| `origin` | `[x, y, z]` | obligatoire | Où l'instance est placée |
| `angles` | `[pitch, yaw, roll]` | obligatoire | Degrés, ordre YXZ |
| `fixup` | chaîne | `""`, omis | Préfixe de nom du contenu de l'instance, voir plus bas |

La construction prend les enfants de chaque calque de la carte référencée qui n'est pas omis de l'export, et les
déplace selon `origin` et `angles`. Les projections de texture suivent la géométrie. Le worldspawn de la carte
référencée n'est pas utilisé. Les instances s'imbriquent sur huit niveaux au plus.

Une construction Godot lit chaque carte référencée une seule fois, et toutes les instances d'une carte ensemble peuvent
ajouter 2 097 152 valeurs, une valeur étant un élément de tableau ou une entrée de dictionnaire de la carte référencée,
chaque tranche de 8 caractères de texte comptant pour une de plus. Au-delà, la construction laisse de côté les
instances restantes et signale une erreur. Les cartes ordinaires restent très en dessous, seules des instances
d'instances multipliées de nombreuses fois l'atteignent.

Avec un fixup `p1`, `door` devient `p1-door` dans `targetname`, `target`, `destination`, `call_target`, chaque propriété
que le FGD déclare comme `target_source` ou `target_destination`, et chaque `target` d'output. Les valeurs qui
commencent par `!`, `@` ou `/` sont laissées telles quelles. Les fixups imbriqués s'additionnent de l'extérieur vers
l'intérieur, donc `b` dans `a` donne `a-b-door`, et *Éclater l'instance* dans l'éditeur donne les mêmes noms.
