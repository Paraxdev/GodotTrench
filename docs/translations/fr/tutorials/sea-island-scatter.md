---
source: 4d58791b780474701e360cec324c41c44739f30d
---

# Île 4 : scatter { #sea-island-4-scatter }

Le scatter place plein d'arbres, de buissons, de rochers ou d'herbe avec un brush au lieu de les poser un par un.
Chaque sorte de plantation ci-dessous est son propre ensemble de scatter, et un ensemble ne pousse que sur ses cibles,
les surfaces que vous choisissez pour lui.

## Préparer un ensemble { #set-up-a-set }

Chaque ensemble de cette page commence de la même façon.

1. Choisir l'outil Scatter

    Appuyez sur **B**, ou cliquez sur le bouton Scatter de la barre d'outils. L'onglet Scatter s'ouvre à droite, à côté de
    l'Inspecteur. (Si vous le fermez, **Panneau scatter** dans la barre d'options de l'outil le ramène.)

    ![La barre d'outils avec le bouton Outil scatter survolé et son infobulle : glissez pour peindre les modèles de l'ensemble de scatter actif](../assets/sea-island/toolbar-scatter.png)

2. Partir d'un préréglage

    Cliquez sur **Préréglage** en haut du panneau Scatter et choisissez le préréglage que nomme l'étape.

    ![Le panneau Scatter avec le menu Préréglage ouvert et forêt mixte survolé](../assets/sea-island/scatter-preset-menu.png)

3. Cibler le terrain

    Cliquez sur la pipette à côté de **Cibles**, puis cliquez sur le terrain dans la vue 3D. Vous devriez le voir listé
    sous Cibles, comme `terrain #2` ou similaire.

    ![La ligne Cibles du panneau Scatter avec la pipette survolée et son infobulle](../assets/sea-island/scatter-eyedropper.png)

Laissez **Suivre le curseur** décoché, car il peint sur tout ce qui est sous le brush et ignore les cibles. Laissez
**Garder l'espacement avec les autres ensembles** coché, pour que les ensembles gardent leurs distances et que rien ne
pousse à travers un tronc.

## Forêt { #woodland }

1. **Préréglage** > *forêt mixte*, puis cliquez sur **Renommer** en haut et appelez-le `woodland`. Ciblez le terrain.
2. Dans la section Brush réglez **Densité** 0,16 et **Pente** 0 à 30, et cochez **Hauteur**, 120 à 2000. La plage de
    pente garde les arbres loin des falaises, et la plage de hauteur les garde loin de la plage.

    ![Le bas du panneau Scatter : terrain #2 comme cible, Suivre le curseur décoché, Densité 0,16, Pente 0 à 30, Hauteur cochée de 120 à 2000, Garder l'espacement avec les autres ensembles coché et le bouton Remplir les cibles survolé](../assets/sea-island/scatter-woodland-settings.png)

3. Cliquez sur **Remplir les cibles** en bas du panneau pour planter tout le terrain d'un coup.

La densité compte les tentatives par 64 x 64 unités, donc 0,16 fait à peu près un arbre tous les 5 m.

![La forêt après Remplir les cibles](../assets/sea-island/scatter-woodland.jpg)

Maintenez **Shift** et faites glisser le long de chaque chemin avec un rayon de 260 pour y effacer les arbres. Effacez une
clairière au sommet de la colline principale (rayon d'environ 520) et une prairie derrière la plage (environ 700).

![Effacer les arbres sur le chemin](../assets/sea-island/scatter-erase-paths.jpg)

## Arbres vedettes, buissons et blocs rocheux { #hero-trees-bushes-and-boulders }

Chaque ligne est un nouvel ensemble issu du préréglage nommé, ciblé sur le terrain comme la forêt. Les arbres vedettes
sont quelques grands arbres détaillés placés là où le joueur regarde de près.

| Ensemble | Préréglage | Comment le placer | Rayon | Densité | Pente | Hauteur |
| --- | --- | --- | --- | --- | --- | --- |
| Arbres vedettes | forêt détaillée | Des clics simples là où les chemins quittent la plage et autour de la clairière du sommet | 260 | 0,08 | 0 à 30 | désactivée |
| Buissons | buissons | **Remplir les cibles**, puis effacez le long des chemins avec un rayon de 190 | | 0,35 | 0 à 32 | 45 à 200 |
| Blocs rocheux | blocs rocheux | Un trait le long du haut des falaises | 1100 | 0,035 | 12 à 50 | 20 à 2000 |

![Des arbres vedettes au pied du chemin](../assets/sea-island/scatter-hero-trees.jpg)

![Des buissons en bande au-dessus du sable](../assets/sea-island/scatter-bushes.jpg)

![Des blocs rocheux le long des falaises](../assets/sea-island/scatter-boulders.jpg)

## Herbe { #grass }

1. Démarrez un ensemble depuis le préréglage *herbe*. L'herbe est du feuillage, dessiné sans collision ni ombres, donc
   elle peut être dense sans ralentir le jeu.
2. Peignez les prairies ouvertes avec un rayon de 650, une densité de 2,5, une pente de 0 à 28, une hauteur de 45 à 2000.
    Effacez-la sur les chemins avec un rayon de 170.

    ![De l'herbe de prairie derrière la plage](../assets/sea-island/scatter-meadow.jpg)

3. Pour de l'herbe sur les blocs rocheux, démarrez encore un ensemble *herbe*. Avec la pipette, cliquez sur un bloc
    plutôt que sur le terrain, pour que l'ensemble de blocs devienne sa cible. Vous devriez le voir sous Cibles avec
    `(set)` après son nom. Puis **Remplir les cibles** avec une densité de 6 et une pente de 0 à 45.

    ![La ligne Cibles listant cliff boulders (set) comme cible](../assets/sea-island/scatter-target-boulders.png)

> [!NOTE]
> Remplir place l'herbe sur les blocs tels qu'ils sont à ce moment, donc finissez d'abord les blocs rocheux.

![De l'herbe qui pousse sur un bloc rocheux](../assets/sea-island/scatter-boulder-grass.jpg)

Suite : [Île 5 : la jetée et Godot](sea-island-finish.md).
