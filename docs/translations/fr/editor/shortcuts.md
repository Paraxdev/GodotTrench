---
source: c1ad5f7b4051d96e108a3133de3b683670efbf64
---

# Raccourcis { #shortcuts }

Ce sont les touches par défaut, le préréglage TrenchBroom.

Vous voulez d'autres touches ? *Aide > Raccourcis clavier…* liste chaque raccourci. Tapez dans le filtre pour en trouver
un, appuyez sur **Définir** puis sur le nouveau raccourci. **Préréglage** en haut de cette fenêtre, ou *Fichier >
Préférences > Préréglage des raccourcis*, bascule vers Hammer ou Blender, voir [Autres préréglages](#other-presets).

![La fenêtre Raccourcis clavier filtrée sur csg, avec la liste Préréglage, Réinitialiser les remplacements et un bouton Définir et un bouton Effacer sur chaque ligne](../assets/shortcuts/keyboard-shortcuts-window.png)

Les raccourcis attendent pendant que vous tapez dans un champ ou qu'un menu est ouvert. Tant que le pointeur est sur une
fenêtre flottante, comme le Générateur de formes, seuls enregistrer, annuler et rétablir fonctionnent, si bien qu'une
touche destinée à la fenêtre ne supprime ni ne modifie jamais votre carte.

## Outils { #tools }

Appuyez de nouveau sur la touche d'un outil pour revenir à Sélectionner. Esc annule un glissement ou un trait, sinon il
revient à Sélectionner, et dans Sélectionner il efface la sélection.

| Touche | Outil |
| --- | --- |
| Q | Sélectionner, dessiner des brushes et déplacer des choses |
| C | Clip, pour couper des brushes le long d'un plan |
| V | Sommet, pour faire glisser les coins des brushes |
| R | Rotation |
| T | Échelle |
| Tab | Modifier la sélection là où elle se trouve : les maillages dans l'outil Maillage, les brushes dans l'outil Sommet. Il ne convertit jamais un brush, *Maillage > Éditer le maillage* le fait |
| Shift+T | Texture, pour aligner les textures sur les faces |
| P | Peindre, pour peindre des couleurs de sommet sur les faces |
| G | Sculpter le terrain et les displacements |
| Shift+G | Mélanger les calques de terrain |
| B | Scatter |
| Shift+E | Volume, pour tracer un volume déclencheur |
| Shift+P | Chemin, pour placer une chaîne d'entités `path_corner` |
| M | Mesurer des distances |

## Souris dans les vues { #mouse-in-the-views }

| Entrée | Résultat |
| --- | --- |
| Glisser hors de la sélection | Dessiner un brush |
| Glisser la sélection | Déplacer. Ctrl copie, Shift verrouille l'axe, Alt déplace à la verticale en 3D |
| Clic, double clic | Sélectionner, sélectionner tout le groupe |
| Cliquer de nouveau peu après au même endroit, ou Alt+clic (2D) | Sélectionner l'objet suivant sous le curseur, comme un mur sous son plafond dans la vue de dessus |
| Ctrl+clic | Ajouter à la sélection ou retirer, dans les vues et dans l'Outliner |
| Shift+clic dans l'Outliner | Sélectionner les lignes à partir de la dernière sur laquelle vous avez cliqué |
| Shift+clic, Ctrl+Shift+clic | Sélectionner une face, ajouter ou retirer une face |
| Shift+glisser une face sélectionnée (3D) | La pousser ou la tirer |
| Ctrl+Shift+glisser une face sélectionnée (3D) | Extruder un nouveau brush |
| Glisser avec le clic droit (3D) | Regarder autour. Maintenez-le et utilisez WASD pour voler, Q et E pour descendre et monter |
| Glisser avec le clic du milieu | Panoramique |
| Alt+glisser avec le clic gauche (3D) | Orbite. Dans l'outil Sélectionner, un glissement qui commence sur la sélection ou son gizmo la déplace plutôt à la verticale |
| Ctrl+molette (Sculpter, Mélange, Peindre, Scatter) | Redimensionner le pinceau |

## Édition { #editing }

| Touche | Action |
| --- | --- |
| Ctrl+Z, Ctrl+Shift+Z ou Ctrl+Y | Annuler, rétablir |
| Ctrl+X, Ctrl+C, Ctrl+V | Couper, copier, coller sous la souris |
| Ctrl+D, Ctrl+Shift+D | Dupliquer, dupliquer en lien |
| F2 | Renommer l'objet sélectionné dans l'Outliner |
| Ctrl+R | Répéter la dernière rotation, le dernier retournement, décalage ou la dernière duplication. En édition de maillage, il fait une coupe en boucle |
| Delete ou Backspace | Supprimer |
| Ctrl+A, Esc, Ctrl+I | Tout sélectionner, ne rien sélectionner, inverser |
| Ctrl+T, Ctrl+Shift+T | Utiliser les brushes sélectionnés comme pochoir et sélectionner ce qu'ils touchent, ou ce qui se trouve à l'intérieur. Les brushes du pochoir sont supprimés |
| Ctrl+G, Ctrl+Shift+G | Grouper, dégrouper |
| Ctrl+H, Ctrl+J, Ctrl+Shift+H | Masquer la sélection, l'isoler en masquant tout le reste, tout afficher |
| Flèches, Page Up, Page Down | Décaler d'un pas de grille |

## Brushes, maillages et textures { #brushes-meshes-and-textures }

| Touche | Action |
| --- | --- |
| Ctrl+K, Ctrl+M, Ctrl+L | CSG soustraire (creuser la sélection dans ce qu'elle touche), fusion convexe, intersection |
| Ctrl+Shift+K | Évider, transformer un brush plein en pièce avec des murs |
| Ctrl+Shift+B | Générateur de formes, pour des arches, des cylindres, des escaliers et d'autres formes toutes faites |
| Ctrl+Shift+E, Ctrl+Shift+J | Convertir les brushes en maillage, joindre les maillages |
| Ctrl+Shift+W | Déplacer les brushes vers le monde, en les sortant de leur entité pour qu'ils deviennent de la simple géométrie de niveau |
| Ctrl+Shift+U | Basculer le verrouillage UV, qui garde les textures attachées à un brush pendant qu'il bouge |
| Alt+H | Ajustement au hotspot, qui ajuste les faces sélectionnées au rectangle qui leur correspond le mieux dans une trim sheet |

## Vue et fichiers { #view-and-files }

| Touche | Action |
| --- | --- |
| F ou Ctrl+U | Cadrer la sélection, ou toute la carte |
| `[` et `]`, ou `-` et `=` | Diviser par deux ou doubler la grille |
| F4, F3 | Parcourir les ombrages (texturé, uni, éclairé, fil de fer), basculer l'Aperçu éclairé |
| Shift+Space | Agrandir la vue sous le pointeur |
| Ctrl+Shift+1 à 9, Ctrl+1 à 9 | Enregistrer, rappeler des signets de caméra |
| Ctrl+=, Ctrl+-, Ctrl+0 | Agrandir l'échelle de l'interface, la réduire, la réinitialiser |
| F1 | Ouvrir le [manuel](interface.md#help-and-the-manual) à l'outil, à l'onglet de panneau, à l'entrée de menu ou à la section de l'Inspecteur sous le pointeur, ailleurs la palette de commandes |
| Ctrl+clic sur un outil dans la barre d'outils, Cmd+clic sur macOS | Ouvrir sa page dans le manuel |
| Ctrl+Shift+P | Palette de commandes |
| Ctrl+N, Ctrl+O | Nouvelle carte, ouvrir une carte |
| Ctrl+S, Ctrl+Shift+S | Enregistrer, enregistrer sous |
| Ctrl+Shift+N, Ctrl+Tab, Ctrl+W | Nouvel onglet, onglet suivant, fermer l'onglet |
| F5 | Lancer le projet Godot |

> [!NOTE]
> F2 servait à parcourir les ombrages. Il renomme maintenant, comme dans Godot, Blender et la plupart des gestionnaires de
> fichiers, et l'ombrage est passé à F4. Un raccourci que vous avez défini vous-même dans la fenêtre des raccourcis reste
> tel qu'il était.

## Autres préréglages { #other-presets }

Un préréglage ne change que les touches ci-dessous, tout le reste reste comme plus haut. Vos propres raccourcis passent
par-dessus n'importe quel préréglage, **Réinitialiser les remplacements** dans la fenêtre des raccourcis les efface.

| Touche | Préréglage Hammer |
| --- | --- |
| Shift+S, Shift+X, Shift+V, Shift+A | Outil Sélectionner, Clip, Sommet, Texture |
| H, Ctrl+H, U | Masquer, isoler, tout afficher |
| Ctrl+W | Déplacer les brushes vers le monde |
| Ctrl+M | Générateur de formes |
| Ctrl+L | Rotation de 90° autour de l'axe vertical |
| Ctrl+Shift+M, Ctrl+Shift+L | CSG fusion convexe, intersection |
| Ctrl+F4 | Fermer l'onglet |

| Touche | Préréglage Blender |
| --- | --- |
| A, Alt+A | Tout sélectionner, ne rien sélectionner |
| X | Supprimer |
| Shift+D, Alt+D | Dupliquer, dupliquer en lien |
| S | Outil Échelle |
| H, Alt+H, / | Masquer, tout afficher, isoler |
| Ctrl+J | Joindre les maillages |
| Z | Parcourir les ombrages |
| Shift+A | Générateur de formes |
| F3 | Palette de commandes |
| F4 | Aperçu éclairé |
| Alt+Shift+H | Ajustement au hotspot |
