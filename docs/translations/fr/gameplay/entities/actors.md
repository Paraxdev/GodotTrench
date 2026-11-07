---
source: 978a2cda95dff6236396e32b6d494bed11f3f94b
---

# Spawners et chemins { #spawners-and-paths }

> [!NOTE]
> Ces entités viennent avec le pack d'entités Gameplay, voyez
> [Installer et personnaliser](README.md#installing-and-customizing).

## info_spawner { #info_spawner }

Fait apparaître une scène, comme un ennemi ou un objet à ramasser, à des endroits au hasard dans `radius`. Lancez `spawn`
depuis un déclencheur pour une embuscade, ou réglez un `interval` et faites `start` pour un flux régulier. Pour répartir
les apparitions dans une salle, utilisez [`trigger_spawn_area`](triggers.md#trigger_spawn_area).

| Clé | Par défaut | Ce qu'elle fait |
| --- | --- | --- |
| `scene` | | La scène `res://` à faire apparaître |
| `count` | 1 | Combien en faire apparaître à chaque fois |
| `max_alive` | 5 | Jamais plus en vie à la fois, les apparitions en trop sont sautées |
| `total` | 0 | S'arrête pour de bon après ce nombre, 0 veut dire sans limite |
| `interval` | 0 | Secondes entre deux apparitions tant que le minuteur tourne. 0 ne fait apparaître que sur `spawn` |
| `radius` | 64 | À quelle distance du spawner elles peuvent apparaître. Tirez sa poignée dans la vue |
| `start_active` | 0 | Lance le minuteur dès le chargement de la carte |
| `spawn_on_ready` | 0 | Fait apparaître une fois au chargement de la carte |
| `spawn_group` | enemies | Groupe Godot ajouté à chaque nœud créé |
| `snap_to_ground` | 1 | Pose chaque apparition sur le sol en dessous |

* **Entrées :** `spawn`, `start`, `stop`, `toggle`, `kill_all`
* **Sorties :** `spawned(node)`, `all_dead`, `exhausted` (une fois, la première fois que `total` est atteint)

`all_dead` se lance quand tous les nœuds créés ont disparu, pour « videz la salle pour ouvrir la porte ». Les nœuds créés
n'ont pas de targetname, atteignez-les par leur `spawn_group`, comme `@enemies`.

Avec les clés par défaut, il ne fait rien apparaître tant que quelque chose ne lui lance pas `spawn`. Pour des
apparitions sans déclencheur, réglez `spawn_on_ready`, ou un `interval` avec `start_active`. L'apparition n'a lieu que
dans le jeu lancé, dans l'éditeur de Godot le spawner reste un nœud vide.

## info_teleport_destination { #info_teleport_destination }

Marque l'endroit où `trigger_teleport` envoie les corps. Orientez-le comme le joueur doit regarder à l'arrivée, son yaw
devient leur direction. Sa seule clé est `targetname`.

## path_corner { #path_corner }

Un arrêt sur le trajet d'un `func_train` ou d'un `npc_walker`, avec `target` qui nomme le suivant. Posez une chaîne d'un
clic avec l'outil **Chemin** (Maj+P). Il lance `reached` quand un train ou un marcheur arrive, pour un son ou un
événement à cet arrêt.

* **Sorties :** `reached(train)`
* **Clés :** `target`, `wait` 0 (secondes d'attente ici), `speed` 0 (nouvelle vitesse du train à partir d'ici, 0 la garde)

`speed` n'affecte que les trains, les marcheurs gardent la leur.

## npc_walker { #npc_walker }

Un acteur scripté qui suit une chaîne de path corners en jouant des animations, pour des cinématiques plutôt que de l'IA,
comme un garde qui passe devant une fenêtre. Il avance en ligne droite, sans tenir compte de la gravité ni des murs,
donc posez les corners là où il peut marcher.

| Clé | Par défaut | Ce qu'elle fait |
| --- | --- | --- |
| `model` | | Une scène qui lui donne un mesh et un `AnimationPlayer` |
| `target` | | Le premier path corner où aller |
| `speed` | 2.0 | Mètres par seconde |
| `loop` | 0 | Retourne au premier corner après le dernier |
| `start_active` | 0 | Marche dès le chargement de la carte |
| `walk_anim`, `idle_anim` | walk, idle | Animations jouées pendant la marche et à l'arrêt |
| `face_travel` | 1 | Se tourne vers l'endroit où il marche |
| `collision` | default | La forme construite à partir de son modèle, pour que les autres corps le heurtent, voyez [Collision](collision.md) |
| `collision_layer`, `collision_mask` | vide | Des numéros de couche comme `1, 3`, vide garde la valeur par défaut du projet |

* **Entrées :** `start`, `stop`, `walk_to(corner)`, `play_anim(name)`, `face(target)`
* **Sorties :** `arrived(corner)`, `reached_goal`, `finished` (les deux au dernier corner sans boucle)

`face` et `walk_to` acceptent n'importe quelle cible, comme `!player`. `walk_to` abandonne le tronçon en cours et part
droit vers là.

Il ne charge son `model` que dans le jeu lancé, donc dans l'éditeur de Godot c'est un nœud vide.
