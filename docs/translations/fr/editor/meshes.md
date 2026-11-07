---
source: d32761c1e423392e984f1e0d2875aadc77d595ce
---

# Maillages { #meshes }

Contrairement aux brushes, les maillages peuvent avoir des creux, des trous et des faces courbes, utilisez-les donc pour
des rochers, des tuyaux, des arches lisses et d'autres formes que les brushes ne savent pas faire.

## Modifier un maillage { #editing-a-mesh }

Sélectionnez les brushes que vous voulez remodeler et choisissez *Maillage > Éditer le maillage*. Il les convertit en
maillage et commence à éditer tout de suite.

![Le menu Maillage avec Éditer le maillage survolé, puis Convertir les brushes en maillage (Ctrl+Shift+E), Convertir les maillages en brushes, Joindre les maillages (Ctrl+Shift+J) et les sous-menus Topologie, Composants, Normales et ombrage et Transformer](../assets/meshes/menu-edit-mesh.png)

Avec un maillage sélectionné, **Tab** fait la même chose, et Tab encore vous ramène aux objets. Tab sur des brushes ouvre
plutôt l'outil Sommet, pour qu'une touche pressée par erreur ne transforme jamais un brush en maillage. Le bouton de
l'outil Maillage dans la barre d'outils commence l'édition sans rien convertir. Ctrl+Shift+E ne fait que convertir,
Ctrl+Shift+J joint des maillages en un seul.

Pendant l'édition, la barre d'options de l'outil en haut passe entre **Sommet**, **Arête** et **Face**, et propose des
boutons pour les opérations courantes.

![La barre d'options de l'outil Maillage avec Sommet, Arête et Face, puis Subdiviser, Fusionner, Remplir, Supprimer, Lisser et Plat](../assets/meshes/mesh-tool-options.png)

## Raccourcis { #keys }

Les raccourcis suivent Blender. Par exemple, appuyez sur 3, cliquez sur la face supérieure d'une boîte et appuyez sur E,
puis montez la souris et cliquez pour l'extruder :

![Une boîte en maillage en mode face, sa face supérieure extrudée vers le haut par pas de grille](../assets/meshes/extrude-face.gif)

| Touche | Action |
| --- | --- |
| 1, 2, 3 | Travailler sur les sommets, les arêtes ou les faces |
| A, Alt+A | Tout sélectionner, ne rien sélectionner |
| Alt+clic, Ctrl+Alt+clic | Sélectionner la boucle d'arêtes passant par l'arête cliquée, ou l'anneau qui la traverse |
| Ctrl+L | Sélectionner tout ce qui est connecté à la sélection |
| G, R, S | Déplacer, tourner, mettre à l'échelle |
| E | Extruder de la nouvelle géométrie à partir de la sélection |
| I | Insérer, une face plus petite à l'intérieur de chaque face sélectionnée |
| Ctrl+B, Ctrl+Shift+B | Biseauter des arêtes ou des sommets, en coupant le coin |
| Ctrl+R | Coupe en boucle, un nouvel anneau d'arêtes à travers le maillage. La molette ou + et - règlent le nombre de coupes |
| K | Couteau, cliquez deux points pour couper le long de cette ligne |
| M | Fusionner les sommets sélectionnés en leur centre |
| F | Remplir, une nouvelle face à travers les sommets sélectionnés |
| X ou Delete | Supprimer |
| Shift+D | Dupliquer et déplacer |
| P | Séparer la sélection en un nouveau maillage |
| Alt+F | Inverser les normales, pour les faces qui s'affichent du mauvais côté |

Pendant un déplacement, une rotation ou une mise à l'échelle, appuyez sur X, Y ou Z pour verrouiller sur cet axe, ou sur
Shift avec la lettre pour l'exclure. Saisissez un nombre pour une valeur exacte. Un clic ou Enter valide, un clic droit ou
Esc annule. La ligne d'aide en bas de la vue les liste au fur et à mesure.

## Gizmo { #gizmo }

Double-cliquez sur un élément pour y placer un gizmo de déplacement, de rotation et d'échelle. C'est la façon propre de
faire glisser un sommet le long d'un seul axe, et cela marche aussi dans les vues 2D. Esc masque le gizmo et garde la
sélection.

Le menu *Maillage* contient le reste, comme la subdivision, la triangulation, l'épaississement, le miroir et l'ombrage
lisse.
