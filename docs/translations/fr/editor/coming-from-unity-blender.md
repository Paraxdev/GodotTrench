---
source: 84383292fb022d6e6369e9ad8b643a67a07d4a67
---

# Venir de Unity ou de Blender { #coming-from-unity-or-blender }

TrenchDot est un éditeur de brushes dans la lignée de TrenchBroom et de Hammer. Un niveau commence comme des blocs pleins
que vous dessinez sur une grille, pas comme des maillages importés. La plupart de ce que vous connaissez de Unity et de
Blender a toujours sa place ici, sous d'autres noms et avec d'autres raccourcis. Cette page fait la correspondance, et
[Premiers pas](../getting-started.md) vous guide dans une première pièce.

## Où sont les choses { #where-things-are }

| Dans Unity ou Blender | Dans TrenchDot |
| --- | --- |
| Hierarchy, Outliner | L'**Outliner**, avec des [calques et des groupes](organizing.md) à la place des objets parents |
| Inspector, panneau N | L'**Inspecteur**. Ses champs de position et de taille acceptent des valeurs saisies, Enter les applique |
| Scene view, viewport | La vue 3D plus les vues de dessus, de face et de côté. Dessiner et aligner est plus simple en 2D |
| Formes ProBuilder, menu Add | Faites glisser dans n'importe quelle vue pour dessiner un [brush](brushes.md) en forme de boîte, Ctrl+Shift+B pour des arches, des escaliers et des cylindres |
| Modificateur Boolean | *Brush > CSG* : soustraire (Ctrl+K), fusionner, intersecter et évider. Ils modifient les brushes tout de suite, il n'y a pas de pile de modificateurs |
| Edit mode | L'outil **Sommet** pour les brushes, l'outil **Maillage** pour les [maillages](meshes.md), tous deux sur Tab |
| Prefabs | Les [prefabs](organizing.md#prefabs), enregistrés comme leur propre carte et mis à jour dans chaque instance |
| Linked duplicate (Alt+D) | Dupliquer en lien, Ctrl+Shift+D, pour que les copies partagent leur géométrie |
| Unités, 1 m | 32 unités de carte font un mètre, le pas de grille par défaut est 16 |

Les nombres exacts vont dans l'Inspecteur à droite. Sélectionnez quelque chose, saisissez une nouvelle position ou une
nouvelle taille et appuyez sur Enter. La position est le coin le plus bas de la sélection, pas son centre ni un pivot.

![L'Inspecteur avec un brush sélectionné, sa position et sa taille sous forme de champs X, Y et Z au-dessus de la section Sélection](../assets/coming-from-unity-blender/inspector-position-size.png)

## Les raccourcis qui diffèrent { #keys-that-differ }

Les raccourcis par défaut suivent TrenchBroom, et trois d'entre eux surprennent les utilisateurs de Blender :

* **G** est l'outil de sculpture du terrain, pas le déplacement.
* **R** et **T** choisissent les outils Rotation et Échelle, qui agissent sur la sélection avec des poignées au lieu de
  suivre la souris.
* **Tab** modifie la sélection là où elle se trouve.

Pour déplacer quelque chose, faites-le glisser dans n'importe quelle vue ou utilisez le gizmo de la vue 3D, et passez par
l'Inspecteur pour les nombres exacts.

Vous préférez garder vos raccourcis Blender ? Passez *Fichier > Préférences > Préréglage des raccourcis* sur Blender.

![La liste déroulante Préréglage des raccourcis dans la fenêtre Préférences, ouverte avec TrenchBroom, Hammer et Blender](../assets/coming-from-unity-blender/keymap-preset.png)

Elle ajoute A pour tout sélectionner, X pour supprimer, Shift+D pour dupliquer, H et Alt+H pour masquer et afficher, S pour
l'outil Échelle et Shift+A pour le générateur de formes. G reste Sculpter, et saisir un nombre pendant un déplacement ne
marche que dans l'outil Maillage. La liste complète est sous [Autres préréglages](shortcuts.md#other-presets).

Il n'y a pas de vues du pavé numérique. Les vues de dessus, de face et de côté sont toujours ouvertes, Shift+Space agrandit
celle qui est sous le pointeur et *Affichage > Vues* passe de une à deux ou quatre vues.

## Les habitudes qui restent { #habits-that-carry-over }

Dans l'outil Maillage, les raccourcis suivent Blender : 1, 2 et 3 pour les sommets, les arêtes et les faces, G, R et S avec
X, Y ou Z pour verrouiller un axe, E pour extruder, Ctrl+R pour une coupe en boucle et Ctrl+B pour biseauter.

Un glisser avec le clic droit et WASD fait voler la caméra 3D comme la scene view de Unity, et F cadre la sélection.
Ctrl+Z et Ctrl+Shift+Z annulent et rétablissent, et F2 renomme.
