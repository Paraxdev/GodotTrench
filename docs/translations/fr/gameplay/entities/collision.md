---
source: d6310713b2a9d65be25e506f0179132b8e0e10fd
---

# Collision { #collision }

Chaque entité brush du pack Gameplay et chaque entité modèle entre en collision sans que vous ayez rien à régler. Les
portes, boutons, ascenseurs, trains et déclencheurs reçoivent une enveloppe convexe par brush, et `prop_model`,
`prop_physics`, `npc_walker` et les props de scatter reçoivent une forme construite à partir de leur modèle. Vous ne
touchez aux clés ci-dessous que quand la valeur par défaut ne convient pas.

## Les clés { #the-keys }

Sélectionnez l'entité et trouvez ces lignes dans l'Inspecteur. Survolez une ligne pour lire ce que fait chaque choix.

![L'Inspecteur d'un func_door avec le menu de collision ouvert sur default, none, box, convex hull, convex decomposition et trimesh, les lignes collision_layer et collision_mask en dessous](../../assets/collision/inspector.png)

| Clé | Par défaut | Ce qu'elle fait |
| --- | --- | --- |
| `collision` | default | La forme : default, none, box, convex hull, convex decomposition ou trimesh |
| `collision_layer` | vide | Les couches physiques où se trouve le corps, sous forme de numéros de couche comme `1, 3`. `none` le met sur aucune couche |
| `collision_mask` | vide | Les couches physiques que le corps détecte, sous la même forme. `none` ne détecte rien |

Une couche ou un masque vide garde la valeur par défaut : la couche et le masque de la définition d'entité pour les
entités brush, les déclencheurs prenant *Trigger Collision Mask* dans les réglages de carte, et les réglages du projet
ci-dessous pour les entités modèle.

## Choisir une forme { #picking-a-shape }

| Forme | Entités brush | Modèles |
| --- | --- | --- |
| default | La forme de la classe, une enveloppe convexe par brush | Le réglage du projet, une enveloppe convexe par mesh sauf changement. `prop_physics` utilise une boîte tirée de `size` |
| none | Aucune collision | Aucune collision |
| box | Une boîte autour de tous les brushes | Une boîte autour du modèle, la forme la moins chère |
| convex hull | Une enveloppe par brush | Une enveloppe par mesh, correct pour les modèles en blocs |
| convex decomposition | Comme convex hull, un brush est déjà convexe | Plusieurs enveloppes qui suivent un modèle concave comme une arche, plus lent à construire |
| trimesh | Chaque face, pour les corps statiques seulement | Chaque triangle, exact mais le plus coûteux |

Trimesh ne marche que sur les corps statiques. Les portes, boutons, ascenseurs et trains sont des corps animables, qui
comptent comme statiques, donc ils peuvent l'utiliser. Les déclencheurs, `prop_physics` et `npc_walker` construisent des
enveloppes convexes à la place, et un déclencheur réglé sur trimesh signale un avertissement à la construction de la
carte.

Un `prop_model` dont le modèle contient déjà un corps de collision garde ce corps et ne construit pas de forme à lui.

## Valeurs par défaut du projet { #project-defaults }

Les entités modèle dont les clés sont vides suivent trois réglages des *Project Settings* de Godot, sous *Trenchdot >
Collision*. Ils apparaissent une fois le plugin TrenchDot activé.

| Réglage | Par défaut | Ce qu'il fait |
| --- | --- | --- |
| *Model Shape* | convex | La forme que représente default sur `prop_model` et `npc_walker` |
| *Model Layer* | 1 | La couche où se trouvent les corps de modèle |
| *Model Mask* | 1 | Les couches que les corps de modèle détectent |

Les ensembles de scatter choisissent leur forme dans leur propre ligne **Collision**, voyez [Scatter](../../editor/scatter.md).

## Vos propres entités brush { #your-own-brush-entities }

Une entité brush que vous définissez vous-même entre en collision comme le corps que nomme son *Node Class*. Avec *Node
Class* vide, elle prend le corps que son script étend, et sans script, ou avec un script qui étend `Node3D`, elle devient
un `StaticBody3D`. Seul un script qui étend un autre nœud, comme un `Marker3D`, la laisse sans collision, et la
construction avertit. Voyez [Entités personnalisées en GDScript](../custom-entities.md).

La vue de l'éditeur ne dessine pas les formes de collision. L'entrée *Debug > Visible Collision Shapes* de Godot les
montre pendant que le jeu tourne, aussi bien dans la session live que dans une construction complète. Les entités brush
et `prop_model` enregistrent leurs formes dans la scène construite, `prop_physics` et `npc_walker` construisent les leurs
à chaque chargement de la scène, comme ils chargent leur modèle. Un `prop_model` ou un `npc_walker` dont la scène de
modèle contient déjà un corps physique garde ce corps et ne construit pas de formes à lui.
