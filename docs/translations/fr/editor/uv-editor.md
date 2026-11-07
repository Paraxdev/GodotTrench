---
source: 1cc5b4057e3a84e4d2bcf666b68e70eb94fb265b
---

# Éditeur UV { #uv-editor }

L'Éditeur UV dessine à plat les faces de brush et de maillage sélectionnées par-dessus leur texture répétée, pour que vous
voyiez quelle partie de la texture chaque face affiche. Servez-vous-en pour ce que l'[outil Texture](texturing.md#the-texture-tool)
ne sait pas faire dans la vue 3D, comme placer les UV d'un maillage au pixel près.

## L'ouvrir { #opening-it }

L'Éditeur UV est un onglet du panneau de droite, à côté de l'Inspecteur, et *Texture > Éditeur UV* l'affiche aussi.
Sélectionnez ensuite des faces : Shift+clic sur l'une d'elles dans une vue, ou utilisez l'outil Texture. La ligne du haut
donne le nom du matériau et le nombre de faces sélectionnées.

![L'Éditeur UV avec une face de mur sélectionnée, dessinée comme un rectangle orange sur la texture showcase/plaster répétée](../assets/uv-editor/uv-editor-face.png)

> [!TIP]
> Tant que l'Éditeur UV est l'onglet visible de son panneau, Ctrl+clic sur un brush ou un maillage sélectionne toutes ses
> faces, ce qui permet de texturer chaque côté d'un coup. Les brushes et les maillages déjà sélectionnés se transforment
> eux aussi en leurs faces. Quand la sélection contient autre chose, comme un terrain ou une entité, Ctrl+clic ajoute et
> retire des objets comme d'habitude.

Une face de brush reçoit normalement sa texture d'un seul décalage, d'une échelle et d'une rotation. Une face avec des
**UV explicites** stocke à la place une position de texture pour chaque coin, ce dont les maillages ont besoin. L'Éditeur
UV dessine ces faces en bleu, et vous pouvez choisir leurs coins un par un. Les faces orange n'ont pas d'UV explicites et
bougent d'un bloc.

## Commandes { #controls }

Le **?** au bout de la ligne du haut les affiche aussi dans l'éditeur.

| Entrée | Résultat |
| --- | --- |
| Faire glisser sur l'espace vide | Déplace la texture, ou les faces entières quand elles ont des UV explicites |
| Molette, glisser avec le clic droit | Mise à l'échelle, rotation |
| Clic sur un coin | Le sélectionne avec les coins des faces voisines qui lui sont cousus |
| Shift+clic, Ctrl+clic | Ajouter, retirer |
| Alt+clic | Sélectionne uniquement le coin de cette face, pour défaire une couture |
| Shift ou Ctrl+glisser sur l'espace vide | Sélection par rectangle, en ajoutant ou en retirant |
| Faire glisser un coin sélectionné | Déplace toute la sélection |
| Double clic sur un coin | Manipulateur qui déplace seulement selon U ou V, Échap le masque |
| Glisser avec le clic central, Ctrl+molette | Panoramique, zoom du canevas |

Avec des coins sélectionnés, la molette et le glisser avec le clic droit les mettent à l'échelle et les font tourner autour
de leur centre. Les champs U et V sous le canevas placent la sélection au pixel près.

Les petits boutons après les menus activent l'aimantation au pixel, affichent les hotspots de la texture, font d'un simple
glisser une sélection par rectangle, affichent le manipulateur, basculent le verrouillage UV et recadrent les faces. Passez
la souris sur l'un d'eux pour voir lequel est lequel.

## Projections UV de maillage { #mesh-uv-projections }

Avec des faces de maillage sélectionnées, le menu **Projection** crée des UV explicites pour vous. Il utilise la densité
de texels qu'une face de brush avec la même texture aurait, si bien que les maillages s'accordent avec les brushes autour
d'eux. Passez la souris sur une entrée pour une explication en une ligne.

![Le menu Projection de l'Éditeur UV ouvert, Cylindre Y survolé avec son infobulle, Enveloppe autour de cet axe du monde passant par le centre des faces, u tourne autour](../assets/uv-editor/projection-menu.png)

| Commande | Résultat |
| --- | --- |
| *Planaire*, *Planaire X*, *Y*, *Z* | Projette selon la direction vers laquelle les faces pointent le plus, ou selon un axe du monde, en les gardant d'un seul tenant |
| *Boîte* | Projette chaque face depuis son axe du monde le plus proche, pour les formes anguleuses |
| *Cylindre X*, *Y*, *Z*, *Sphère* | Enroulent la texture autour des faces, pour les tuyaux, les colonnes et les dômes |
| *Affichage* | Projette droit le long de la caméra 3D |
| *Dérouler* | Étale les faces à plat arête par arête, si bien que les bandes et les grilles restent continues |
| *Réinitialiser au monde* | La projection d'une face de brush qui vient d'être réinitialisée |
| *Précalculer le planaire* | Transforme la texture planaire en UV de coins modifiables, affichée tant que certaines faces n'en ont pas encore |
| *Effacer les UV* | Abandonne à nouveau les UV explicites |
| *Normaliser* | Met les UV à l'échelle pour qu'ils tiennent une fois dans la texture |
| *Pack* | Range les îlots dans la texture à une seule densité de texels |

Ici, une colonne de maillage à 12 côtés a reçu *Cylindre Y*, ses côtés se suivent maintenant en une seule bande autour de la
texture :

![L'Éditeur UV après Cylindre Y sur une colonne de maillage à 12 côtés, ses faces latérales étalées en une seule bande continue](../assets/uv-editor/uv-editor-cylinder.png)

Le menu **UV** retourne, fait tourner, ajuste, aligne, redresse et cale sur les pixels. Le menu **Hotspot** ajuste sur un
rectangle de hotspot, voir [Hotspots](texturing.md#hotspots).
