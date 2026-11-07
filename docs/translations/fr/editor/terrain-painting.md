---
source: 7757565706801cf3511cdef72eaa79c5eb0dc3d0
---

# Peinture de terrain { #terrain-painting }

L'outil **Mélange** (Shift+G) peint les [calques](terrain-layers.md) d'un terrain, par exemple du sable le long d'une
plage ou de la roche sur une falaise. Comme Sculpter, il agit sur les terrains sélectionnés, ou sur tous quand rien n'est
sélectionné.

## L'outil Mélange { #the-blend-tool }

1. Choisir l'outil

    Appuyez sur Shift+G, ou utilisez *Terrain > Mélange > Outil mélange*. Ses options apparaissent dans la barre d'options
    de l'outil en haut à gauche.

    ![Les options de l'outil Mélange en mode pente : mode, atténuation, calque, rayon, force et la plage de pente](../assets/terrain-painting/blend-toolbar.png)

2. Choisir un mode

    Le premier menu de la barre décide de ce que peint le brush, les modes sont listés plus bas.

    ![Le menu des modes de l'outil Mélange ouvert avec pente survolé](../assets/terrain-painting/blend-modes.png)

3. Choisir le calque et peindre

    Réglez **calque** (de 0 à 3) sur le calque voulu, puis faites glisser sur le terrain. Plusieurs passes légères donnent
    des bords plus doux qu'une seule passe forte.

    ![Peindre un chemin de terre qui monte sur l'île avec le calque 2 en mode peindre](../assets/terrain-painting/blend-paint-path.gif)

| Mode | Ce qu'il peint |
| --- | --- |
| peindre | Ajoute le calque choisi sous le brush |
| effacer | Repeint vers le calque 0, le sol de base |
| lisser | Adoucit les frontières entre les calques |
| netteté | Laisse gagner le calque le plus fort, pour des frontières nettes |
| bruit | Seulement par plaques en forme de nuage de la **taille** donnée, pour de la mousse, de l'herbe sèche ou du gravier |
| pente | Seulement là où le sol est entre deux angles. `30` à `90` met de la roche sur tout ce qui est plus raide que 30° |
| hauteur | Seulement entre deux hauteurs du monde, pour du sable à la ligne d'eau ou de la neige au-dessus de la limite des arbres |

Pente et hauteur rendent les très grands brushes sûrs, un brush de la taille d'une colline en mode pente ne transforme en
roche que les falaises.

Le second menu de la barre est l'atténuation, la façon dont la force s'estompe du centre du brush jusqu'à son bord.

| Atténuation | Forme |
| --- | --- |
| lisser | Un fondu doux, le choix habituel |
| linéaire | Un fondu régulier du centre au bord |
| constant | Tout le disque à la même force, pour des bords durs |
| spray | Des mouchetures aléatoires, pour des cailloux et des fleurs |

Shift transforme peindre, bruit, pente et hauteur en effacer, et Ctrl lisse. Ctrl+molette redimensionne le brush.

(Remarque : **Définir le matériau de mélange**, **répétition**, **détuilage**, **netteté** et **Appliquer le carrelage**, plus à
droite dans la barre, servent à mélanger des faces de brush, un terrain prend ces réglages de ses calques.)

## Peinture auto { #auto-paint }

La Peinture auto repeint un terrain d'après sa forme, une première passe rapide avant de peindre à la main.

1. Sélectionner le terrain

    La Peinture auto ne touche que les terrains sélectionnés.

2. La lancer

    Allez dans *Terrain > Mélange > Peindre les calques automatiquement*, ou cliquez sur **Peinture auto** dans
    l'Inspecteur.

    ![Le menu Terrain > Mélange avec Peindre les calques automatiquement survolé et Bandes depuis le niveau de la mer coché à y 0](../assets/terrain-painting/menu-auto-paint.png)

| Calque | Va sur |
| --- | --- |
| 1 | Les pentes plus raides qu'environ 50° |
| 2 | Les 20 % du haut de la plage de hauteur |
| 3 | Les 8 % du bas de la plage de hauteur |

Le calque 0 couvre tout le reste. Un terrain avec moins de calques reçoit les bandes manquantes dans son dernier calque.

La plage commence au point le plus bas, qui sur une île est le fond marin. Cochez **Bandes depuis le niveau de la mer**
pour mesurer plutôt depuis une hauteur que vous donnez, comme la ligne d'eau à y 0 ci-dessus.

![L'île marine avant la Peinture auto, toute en herbe](../assets/terrain-painting/auto-paint-before.jpg)

![La même île après la Peinture auto : de la roche sur les falaises, de la terre sur le sommet et du sable le long des plages](../assets/terrain-painting/auto-paint-after.jpg)

> [!WARNING]
> La Peinture auto remplace toute la peinture, donc sculptez d'abord, peignez en auto, et peignez à la main en dernier.

## Quelle finesse de peinture { #how-narrow-you-can-paint }

La peinture vit sur les sommets, donc un trait demande un rayon d'environ deux cellules pour une bande nette. En dessous
d'une cellule, il se transforme en escalier. Pour une piste étroite, utilisez des cellules plus petites, ou posez un
maillage ou un [decal](decals.md) par-dessus.

![Des traits de rayon 40, 96 et 160 sur des cellules de 64 unités, à côté de la grille de sommets qui porte la peinture](../assets/terrain-painting/brush-width-grid.jpg)
