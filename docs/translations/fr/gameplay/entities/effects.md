---
source: 80f0288013e71dba5bd67c791a5e3e3272a71653
---

# Props, lumières et effets { #props-lights-and-effects }

> [!NOTE]
> Ces entités viennent avec le pack d'entités Gameplay, voyez
> [Installer et personnaliser](README.md#installing-and-customizing). `light`, `light_spot` et les sondes
> (`light_probe`, `env_reflection_probe` et `env_voxel_gi`) font partie du socle de l'addon et fonctionnent sans lui.

## prop_physics { #prop_physics }

Une caisse ou un tonneau qu'on peut lancer, abîmer et casser. Il casse à 0 de santé, sur un impact plus rapide que
`impact_speed`, sur `smash`, ou `fuse` secondes après `ignite`. Un prop explosif explose en cassant.

| Clé | Par défaut | Ce qu'elle fait |
| --- | --- | --- |
| `model` | | La scène à laquelle il ressemble, seulement pour l'aspect |
| `size` | 16 16 16 | Demi-dimensions de sa boîte de collision, donc la valeur par défaut fait un mètre de large |
| `collision` | default | default et box utilisent `size`, convex hull et convex decomposition suivent le modèle, voyez [Collision](collision.md) |
| `collision_layer`, `collision_mask` | vide | Des numéros de couche comme `1, 3`, vide garde la valeur par défaut du projet |
| `mass` | 5.0 | Masse en kilogrammes |
| `health` | 30 | Dégâts qu'il encaisse avant de casser |
| `explosive` | 0 | Explose en cassant |
| `explosion_radius`, `explosion_damage` | 192, 40 | Taille et puissance de cette explosion |
| `fuse` | 0.6 | Secondes qu'un prop allumé brûle avant de casser |
| `impact_speed` | 0 | Casse quand il heurte quelque chose plus vite que ça, en m/s. 0 ne casse jamais à l'impact |
| `debris_scene` | | Une scène créée là où il a cassé, comme des planches |

* **Entrées :** `smash`, `ignite(activator)`, `push(direction)`, `take_damage(amount, source)`
* **Sorties :** `damaged(hp)` avec la santé restante, `broken`

`push` prend une vitesse en unités de carte par seconde. `broken` se lance après l'explosion, donc ce qu'il déclenche voit
déjà les dégâts.

Le prop construit son modèle et sa boîte de collision à chaque chargement de la scène. L'éditeur de Godot les montre en
aperçu, qui n'est pas enregistré avec la scène.

## prop_model { #prop_model }

Un modèle statique avec une collision facultative, pour des meubles, des rochers et d'autres props qui ne doivent pas
bouger.

* **Clés :** `model` (.bbmodel, .glb, .gltf ou .tscn), `model_node`, `collision`, `collision_layer`,
  `collision_mask`, `scale` 1.0

`collision` default donne à chaque mesh une enveloppe convexe, correct pour des formes en blocs. Choisissez convex
decomposition ou trimesh pour les formes que le joueur traverse ou sur lesquelles il marche, comme une arche, ou box pour
la forme la moins chère, voyez [Collision](collision.md). La collision n'est générée que si le modèle n'en a pas déjà
une. Les modèles placés avec *Prop modèle* et les modèles dans des prefabs sont des entités prop_model, donc ils entrent
en collision de la même façon. Pour un fichier qui contient plusieurs variantes côte à côte, `model_node` ne garde que le
nœud nommé et le déplace à l'origine du prop.

## env_explosion { #env_explosion }

Une explosion scriptée, comme une charge contre un mur. Sur `explode`, elle appelle `damage_method` sur chaque nœud à
moins de `radius` avec `damage * (1 - distance / radius)`, repousse les corps rigides et crée `effect_scene` pour
l'éclair et la détonation.

* **Entrées :** `explode(activator)`
* **Sorties :** `exploded`
* **Clés :** `radius` 256, `damage` 50, `force` 8 (m/s au centre, quelle que soit la masse), `damage_method` take_damage,
  `effect_scene`

La distance est mesurée jusqu'à l'origine de chaque nœud sans vérifier la ligne de vue, donc elle blesse à travers les
murs.

## game_text { #game_text }

Affiche une ligne de texte dans le monde, sur le HUD ou les deux, pour des indices, des panneaux ou des sous-titres de
cinématique.

| Clé | Par défaut | Ce qu'elle fait |
| --- | --- | --- |
| `text` | | Le texte à afficher |
| `place` | world | `world` flotte à l'entité, `hud` l'affiche à l'écran, `both` fait les deux |
| `start_visible` | 0 | Affiché dès le chargement de la carte |
| `duration` | 0 | Le cache après ce nombre de secondes, 0 le garde jusqu'à `hide` |
| `text_color` | 255 255 255 | Couleur du texte |
| `world_size` | 0.01 | Taille de l'étiquette dans le monde, en mètres par pixel |
| `billboard` | 1 | L'étiquette du monde fait face à la caméra et se dessine à travers les murs. Désactivez-le pour des panneaux, qui gardent alors les angles de l'entité et se cachent derrière les murs |

* **Entrées :** `show`, `hide`, `set_text(text)`, `flash(seconds)`
* **Sorties :** `shown`, `hidden`

`show` et `hide` basculent l'étiquette, pas le nœud. `flash` l'affiche pendant son paramètre en secondes, sinon
`duration`, sinon 2.

## light et light_spot { #light-and-light_spot }

Des lumières omni et spot que des entrées allument et éteignent, pour des interrupteurs, des alarmes et des lampes qui
vacillent. `switched` ne se lance que quand l'état change vraiment. Elles font partie du socle de l'addon, donc tous les
projets les ont.

* **Entrées :** `turn_on`, `turn_off`, `toggle`
* **Sorties :** `switched(on)`
* **Clés :** `light_energy` 1.0 (1.5 pour les spots), `light_color` 255 255 255, `start_on` 1, `shadows` 0 (coûte plus
  cher à rendre), `fixture`, `fixture_off` hide, et `omni_range` 10, ou `spot_range` 15 et `spot_angle` 35 (le demi-angle
  du cône en degrés), `bake_mode` auto, `light_size` 0, `light_indirect_energy` 1, et `omni_attenuation` 1, ou
  `spot_attenuation` 1 et `spot_angle_attenuation` 1. Ces dernières clés règlent les propriétés de lumière de Godot du
  même nom, et le précalcul les utilise aussi

Les **fixtures** gardent le modèle de la lampe en accord avec la lumière. Donnez un targetname à la géométrie lumineuse
de la lampe et mettez-le dans `fixture`, une `*` finale correspond à un préfixe donc `hall_tube_*` couvre tous les
tubes. Elle est affichée quand la lumière est allumée et cachée quand elle est éteinte, sans câblage. Avec `fixture_off`
sur `dark`, elle reste visible avec son émission coupée. `fixture` accepte aussi les autres formes de
[cible](../parameters.md#targets), `@lamps` pour un groupe de nœuds et `/root/...` pour un chemin de nœud. L'éditeur
prévisualise toutes les formes sauf le chemin de nœud, que seul Godot peut résoudre.

`bake_mode` et `light_size` décident comment [Précalculer l'éclairage](../../editor/light-baking.md#light-keys) traite la
lumière. Avec `auto`, une lumière que les E/S peuvent commuter, c'est-à-dire avec un targetname ou `start_on` à 0, reste
en temps réel.

## env_sound { #env_sound }

Un son joué depuis un point du niveau, pour des alarmes, des machines ou un fracas relié à un événement.

* **Entrées :** `play`, `stop`, `toggle`
* **Sorties :** `finished`
* **Clés :** `sound` (un fichier audio `res://`), `volume_db` 0, `max_distance` 0 (sans limite), `autoplay_on` 0 (joue dès
  le chargement de la carte), `loop_sound` 0

## env_particles { #env_particles }

Un effet de particules pour le feu, la fumée, les étincelles, la poussière et la pluie, avec des aspects intégrés pour
que vous n'ayez pas à en construire un vous-même. `burst` émet une salve, puis un effet en cours continue.

* **Entrées :** `start`, `stop`, `toggle`, `burst`
* **Sorties :** `finished`
* **Clés :** `amount` 32 (particules en vie à la fois), `lifetime` 2.0, `one_shot` 0, `start_emitting` 0, `effect` rise,
  `color` 255 255 255, `size` 0, `blend` auto, `area` 10 1 10

`effect` vaut `rise` (poussière lumineuse ou braises), `fire`, `smoke`, `sparks` ou `rain` (tombe d'une boîte `area` de
cette largeur, hauteur et profondeur en mètres). `color` la teinte, et le blanc garde les couleurs propres de l'effet.
`size` règle la taille des particules en mètres, 0 utilise celle de l'effet. `blend` vaut `add` pour une lueur ou `mix`
pour de la fumée. Les valeurs par défaut de chaque effet rendent bien d'elles-mêmes, commencez donc par régler seulement
`effect`.

## infodecal { #infodecal }

Un Decal projeté, une boîte qui plaque un matériau sur tout ce qu'elle contient, pour des taches qui épousent des
marches, des tuyaux ou un sol irrégulier. [Decals projetés](../../editor/projected-decals.md) vous montre comment en
placer un et à quoi faire attention.

* **Clés :** `material` (un nom comme `base/stain` ou un chemin `res://`), `texture`, `normal_texture`, `size` 64 32 64,
  `modulate` 255 255 255

`material` apporte ses textures et sa couleur, `texture` et `normal_texture` sont des images `res://` utilisées sans
matériau. `size` est la largeur, la profondeur et la hauteur de la boîte en unités de carte. Laissée à 64 32 64, c'est la
[taille de texture](../../godot/materials.md#texture-size) du matériau qui fixe la largeur et la hauteur. `modulate`
teinte le decal.

## light_probe, env_reflection_probe et env_voxel_gi { #light_probe-env_reflection_probe-and-env_voxel_gi }

Des sondes qui éclairent et reflètent les objets mobiles, voyez
[Éclairage précalculé dans Godot](../../godot/baked-lighting.md). Aucune n'a d'entrées ni de sorties.

| Entité | Ce que c'est | Clés |
| --- | --- | --- |
| `light_probe` | Un point qui ajoute une sonde de lumière au précalcul, pour les endroits que la grille de sondes rate | Aucune |
| `env_reflection_probe` | Un volume brush qui devient un `ReflectionProbe` de la taille du brush | `intensity` 1.0, `update` once, `box_projection` 1, `interior` 0, `max_distance` 0 |
| `env_voxel_gi` | Un volume brush qui devient un `VoxelGI` de la taille du brush, précalculé dans l'éditeur de Godot à chaque construction | `subdiv` 128, `energy` 1.0, `bake_on_build` 1 |

`update` réglé sur `always` capture une sonde de réflexion à chaque image pour les objets mobiles, ce qui coûte cher.
`box_projection` aligne les reflets sur les murs d'une salle, `interior` laisse le ciel de côté pour les salles sans
fenêtres, et `max_distance` est le nombre de mètres que la sonde voit au-delà de sa boîte. `subdiv` vaut 64, 128, 256 ou
512 voxels sur le plus grand côté, plus c'est net et plus le précalcul est lent.
