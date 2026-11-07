---
source: fb9984bb419921807428d1c050d90fa918ed7b18
---

# Texturage { #texturing }

## Appliquer des matériaux { #applying-materials }

Vos textures vivent dans le panneau **Matériaux**, en bas de la fenêtre sous les vues. La zone de recherche en haut à
gauche les filtre par nom, **Utilisés dans la carte** et **Favoris** restreignent la liste, et le curseur règle la taille
des vignettes.

![Le panneau Matériaux avec la zone de recherche, la liste des dossiers, Utilisés dans la carte, Favoris et le curseur de taille, en dessous les vignettes de matériaux avec dev/blue survolé qui montre son nom et sa taille](../assets/texturing/materials-panel.png)

| Faites ceci | Résultat |
| --- | --- |
| Cliquer sur une vignette | L'applique à la sélection, et l'utilise pour les nouveaux brushes |
| Faire glisser une vignette sur une face | L'applique à cette face |
| Shift en déposant | L'applique au brush ou au maillage entier |
| Alt en déposant | La place comme [decal](decals.md) |

Pendant que vous faites glisser, une petite fiche à côté du curseur vous rappelle les modificateurs, et la face change
dès que vous relâchez.

![Glisser withered/mossy_plaster sur le devant d'un mur de briques, puis le même mur avec l'enduit sur sa seule face avant](../assets/texturing/drop-on-face.png)

Pour texturer une face sans faire glisser, faites d'abord Shift+clic dessus dans une vue, puis cliquez sur la vignette.
Ctrl+Shift+clic ajoute ou retire d'autres faces.

Faites un clic droit sur une vignette pour en savoir plus : l'ajouter à vos favoris, sélectionner les faces qui
l'utilisent, l'échanger contre le matériau actuel dans toute la carte, ouvrir l'éditeur de hotspots, ou *Afficher dans
l'explorateur* pour ouvrir le dossier de sa texture avec le fichier sélectionné, par exemple pour le modifier.

![Le menu du clic droit d'une vignette avec Appliquer à la sélection, Ajouter aux favoris, Sélectionner les faces qui l'utilisent, Remplacer showcase/red_bricks par lui dans la carte, Éditeur de hotspots, Copier le nom et Afficher dans l'explorateur](../assets/texturing/material-context-menu.png)

Déposer un matériau sur un terrain règle plutôt un calque de terrain, voir [Calques de terrain](terrain-layers.md).

Le Verrouillage UV (Ctrl+Shift+U) est activé par défaut, les textures restent donc attachées à un brush pendant que vous
le déplacez.

## Couleurs intégrées { #built-in-colors }

Le panneau Matériaux propose toujours `dev/grey`, `dev/dark`, `dev/orange`, `dev/blue` et `dev/green`, des textures de
grille pour dégrossir un niveau avant qu'il ait des visuels. Coder les pièces par couleur avec elles survit à la
construction, car l'addon fournit les mêmes grilles et Godot les utilise quand le projet n'a pas de texture de ce nom. Une
texture à vous placée par exemple dans `res://textures/dev/grey.png` en remplace une.

## Textures d'outil { #tool-textures }

L'éditeur dessine les textures d'outil en transparence avec des rayures, et la construction Godot ne leur donne aucun
maillage visible. Toute texture des dossiers `special/`, `tools/` ou `td/` compte, tout comme une texture portant le nom
de l'un des outils ci-dessous, telle que `common/caulk`.

![Un bloc dev/dark, un mur violet transparent special/clip devant lui et une boîte orange rayée special/trigger](../assets/texturing/tool-textures.png)

| Texture | Dans Godot |
| --- | --- |
| `special/clip`, `playerclip`, `nodraw`, `caulk`, `trigger`, `sky` et tout autre outil | Entre en collision mais ne dessine rien. Un trigger garde sa forme de trigger |
| `special/skip`, `origin`, `hint`, `hintskip`, `null`, `areaportal` | Ne construit rien du tout. Un brush fait uniquement de ces faces n'entre pas en collision |
| `special/occluder` | Ne dessine rien et ne cache pas les voisins, mais s'ajoute à l'occulteur d'une entité qui en construit un, comme `func_geo` |

Les réglages de carte nomment les textures clip, skip, origin et sky, `special/clip` et ainsi de suite par défaut. Un
brush origin ne fixe l'origine de son entité qu'avec la texture que nomment les réglages de carte.

## L'outil Texture { #the-texture-tool }

L'alignement d'une face est l'endroit où sa texture se place sur elle : le décalage, l'échelle et la rotation. Appuyez sur
**Shift+T** pour l'outil Texture, qui l'édite sur place, comme le mode d'édition de face de Hammer. Cliquez sur une face
pour la sélectionner. Elle est teintée de rouge et montre une boîte bleue autour d'une répétition de la texture, plus deux
petites lignes pour les directions U et V. La face sous le curseur reçoit un contour jaune.

![L'outil Texture avec la face avant d'un mur de briques sélectionnée, une boîte bleue marque une répétition de la texture](../assets/texturing/texture-tool-face.png)

| Action | Résultat |
| --- | --- |
| Clic, Ctrl+clic | Un clic sélectionne une face, Ctrl+clic en ajoute ou en retire une |
| Faire glisser une face sélectionnée | Fait glisser la texture, le pixel sous le curseur reste sous lui |
| Flèches | Décalent de la taille de la grille en pixels, avec Shift d'un pixel |
| Ctrl+molette | Met à l'échelle par 2, avec Shift par petits pas |
| Alt+molette | Tourne de 15°, avec Shift de 1° |
| Alt+clic | Prélève le matériau et l'alignement d'une face |
| Clic droit | Applique le matériau actuel |
| Ctrl+clic droit | L'applique et réinitialise l'alignement |
| Shift+clic droit | Colle le matériau et l'alignement prélevés |
| Alt+clic droit | Applique le matériau actuel en prolongeant l'alignement de la face sélectionnée de l'autre côté de l'arête |

Alt+clic droit est la façon de faire courir proprement une texture, comme un trim ou une rangée de briques, autour d'un
coin.

La barre d'options de l'outil en haut a les boutons **Justifier**. **Gauche**, **Droite**, **Haut**, **Bas** et
**Centre** poussent la texture de ce côté de la face, **Ajuster** l'étire sur toute la face, **Ajuster L** et **Ajuster H**
seulement en largeur ou seulement en hauteur. Avec **Traiter comme un seul** coché, ils agissent sur toutes les faces
sélectionnées comme sur une seule zone, si bien qu'une texture les couvre toutes. **Aligner sur la vue** projette la
texture selon la caméra 3D.

![La barre d'options de l'outil Texture avec Justifier Gauche, Droite, Haut, Bas, Centre, Ajuster, Ajuster L, Ajuster H, Traiter comme un seul, Aligner sur la vue et Ajustement au hotspot](../assets/texturing/texture-tool-options.png)

Si vous préférez les nombres, l'**Inspecteur** montre la même face sous **Alignement**, avec des champs de décalage,
d'échelle et de rotation et des boutons pour la retourner, la tourner et la réinitialiser. **Outils de texture**, en
dessous, règle la densité de texels et copie l'alignement d'une face à l'autre.

![L'Inspecteur avec une face sélectionnée, la section Alignement avec décalage, échelle et rotation, les boutons Justifier et la section Outils de texture](../assets/texturing/inspector-alignment.png)

Pour les faces avec des UV explicites, et pour les maillages, utilisez l'[Éditeur UV](uv-editor.md).

## Échelle de texture { #texture-scale }

Par défaut, un pixel de texture couvre une unité de carte, donc une texture de 64 pixels se répète tous les deux mètres.
Cela convient aux textures en basse résolution, mais une photo de 1024 pixels s'étirerait sur 32 mètres. Réglez
`metadata/texture_size` sur le matériau à la taille dans le monde que doit couvrir une répétition, voir
[Taille de texture](../godot/materials.md#texture-size). La ligne de l'Inspecteur sous le nom du matériau vous dit ce
qu'une face a reçu, comme « 1024 x 1024 px, 64 x 64 unités par répétition ».

Pour le régler pour une seule carte, ou pour qu'une texture se raccorde sans couture sur toutes les faces, utilisez les
[Réglages de texture](light-baking.md#texture-settings) dans l'Inspecteur. Leur taille de répétition l'emporte sur celle
du matériau.

## Hotspots { #hotspots }

Les hotspots permettent à une seule texture de trim sheet de couvrir plein de faces de formes différentes. Vous marquez
ses rectangles une fois, puis TrenchDot choisit le bon pour chaque face.

1. Faites un clic droit sur la trim sheet dans le panneau Matériaux et choisissez **Éditeur de hotspots**. *Texture >
    Éditeur de hotspots…* l'ouvre aussi, pour le matériau de la face sélectionnée ou sinon pour le matériau actuel.

    ![Le menu Texture avec Éditeur de hotspots… survolé, Ajustement au hotspot (Alt+H) en bas](../assets/texturing/menu-texture.png)

2. Faites glisser sur la texture pour dessiner un rectangle, ou réglez une grille et cliquez sur **Diviser en grille**.
   Faites glisser un rectangle pour le déplacer, sa poignée de coin pour le redimensionner, un clic droit le supprime.
3. Cliquez sur **Enregistrer**. Les rectangles vont dans `<texture>.hotspots.json` à côté du fichier de texture.
4. Sélectionnez les faces et utilisez *Texture > Ajustement au hotspot* (Alt+H), ou **Appliquer à la sélection** dans
   l'éditeur.

![L'Éditeur de hotspots sur withered/concrete_panels divisé en une grille de 4 sur 4, 16 rectangles, non enregistré](../assets/texturing/hotspot-editor.png)

L'Ajustement au hotspot donne à chaque face le rectangle dont la forme lui correspond le mieux, en préférant celui qui
garde la texture proche de son échelle normale.

## Peinture de sommets { #vertex-paint }

L'outil **Peindre** (P) peint des couleurs de sommet sur les faces des brushes, pour la saleté, la teinte et un faux
éclairage. Choisissez la couleur dans la barre d'options de l'outil, à côté du rayon et de la force du pinceau, puis
faites glisser sur les faces. Il peint les brushes sélectionnés, ou tous les brushes quand rien n'est sélectionné.
Ctrl+molette redimensionne le pinceau.

![La barre d'options de l'outil Peindre avec l'échantillon Couleur, le rayon 48 u et la force 4.0](../assets/texturing/paint-tool-options.png)
