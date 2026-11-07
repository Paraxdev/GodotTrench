---
source: 6da371c3430389f428469b408b32808d1d2b0af6
---

# Île 3 : peindre { #sea-island-3-painting }

Passez à l'outil Mélange avec **Shift+G**, ou avec le bouton Mélange juste à côté de Sculpter dans la barre d'outils.
Comme Sculpter, il agit sur le terrain sélectionné ou sur tout quand rien n'est sélectionné. Ses réglages sont dans la
barre d'options de l'outil, en haut à gauche.

![La barre d'outils avec le bouton Outil mélange survolé et son infobulle : glissez pour peindre des mélanges sur des calques de terrain, des displacements et des faces avec un matériau de mélange](../assets/sea-island/toolbar-blend.png)

Les modes **pente** et **hauteur** ne peignent que là où le sol est dans une plage de raideur ou de hauteur. Un rayon de
12000 couvre toute l'île, donc un seul clic dans ces modes peint d'un coup tous les rivages ou toutes les falaises.

## Sable et roche { #sand-and-rock }

| Réglage | Sable | Roche |
| --- | --- | --- |
| Mode | hauteur, de `-3000` à `40` | pente, de `32` à `90` |
| Atténuation | constant, pleine force sur tout le brush | constant |
| Calque | 3 | 1 |
| Rayon, force | 12000, 1 | 12000, 1 |

Réglez cela, puis cliquez une fois au milieu de l'île pour chacun. Vous devriez voir tout ce qui est sous 40 passer au
sable, et chaque pente raide passer à la roche :

![Mode hauteur : tout ce qui est sous 40 unités devient du sable](../assets/sea-island/sand-height.jpg)

![Mode pente : tout ce qui est plus raide que 32° devient de la roche](../assets/sea-island/rock-slope.jpg)

Pour élargir la plage, peignez le calque 3 le long de la côte sud en mode **peindre**, atténuation doux, rayon 700, force
0,7. L'atténuation douce s'estompe vers le bord du brush, donc le bord du sable reste doux.

## Chemins { #paths }

Mode **peindre**, atténuation doux, calque 2, rayon 140, force 0,8. Faites glisser :

1. Du haut de la plage jusqu'à la colline principale en longues courbes.
2. Du sommet de la colline principale le long de la crête jusqu'au haut de la falaise.
3. De l'extrémité est de la plage jusqu'à la seconde colline.

![Peindre le chemin qui monte la colline principale](../assets/sea-island/paths.jpg)

![Le chemin de près](../assets/sea-island/path-close.jpg)

## Casser la répétition { #break-up-the-repeat }

De loin, une texture qui se répète se voit comme une grille. Sélectionnez le terrain dans l'Outliner (un clic dans une vue
peindrait avec l'outil Mélange), puis réglez **détuilage** et **netteté** sur chaque ligne Couche de l'Inspecteur. La
netteté garde la texture détuilée nette plutôt que floue.

| Calque | Détuilage | Netteté |
| --- | --- | --- |
| 0 herbe | 0,5 | 0,5 |
| 1 falaise | 0,7 | 0,4 |
| 2 terre | 0,5 | 0,5 |
| 3 sable | 0,4 | 0,5 |

![L'Inspecteur du terrain avec les quatre lignes Couche, chacune avec son matériau, sa tuile, son détuilage et sa netteté](../assets/sea-island/inspector-terrain-layers.png)

Suite : [Île 4 : scatter](sea-island-scatter.md).
