---
source: 7732f7c6b2b4601df9d9b8913237f6914fb6ca69
---

# Calques de terrain { #terrain-layers }

Un terrain a de un à quatre calques, chacun étant un matériau plus quelques réglages pour sa répétition. Sélectionnez le
terrain et vous les trouvez dans l'Inspecteur. Les calques se partagent le sol, si bien que peindre du sable sur de
l'herbe retire du poids à l'herbe.

![L'Inspecteur d'un terrain avec quatre calques, chacun avec son matériau, sa tuile, son détuilage et sa netteté](../assets/terrain-layers/layers-inspector.png)

| Pour | Faites ceci |
| --- | --- |
| Ajouter un calque | **+ couche (matériau actuel)** sous les calques, qui utilise le matériau sur lequel vous avez cliqué en dernier dans le panneau Matériaux. Il s'affiche tant que le terrain en a moins de quatre |
| Retirer le dernier calque | **- dernière couche** |
| Changer le matériau d'un calque | Tapez son nom dans le champ du calque, ou faites glisser un matériau du panneau Matériaux sur le terrain. La ligne en bas de l'Inspecteur dit à quel calque va un dépôt, et avec l'outil Mélange actif c'est le calque que l'outil peint |

**Le calque 0 est le sol de base.** Le sol non peint montre le calque 0 et l'effacement repeint avec lui, mettez donc là
votre sol le plus courant, en général de l'herbe.

Un nouveau terrain reçoit les calques nommés dans la boîte de dialogue *Créer un terrain*, quatre par défaut. Déposer un
matériau pour un calque que le terrain n'a pas encore l'ajoute comme calque suivant.

## Tuile et détuilage { #tile-and-de-tiling }

Toute texture répétée sur une grande zone montre un motif de quadrillage de loin. Ces réglages échangent la netteté de
près contre la dissimulation de ce motif.

| Réglage | Signification |
| --- | --- |
| tuile | Combien d'unités de carte couvre une répétition de la texture. De 128 à 384 convient bien pour un sol sur lequel on marche |
| détuilage | De 0 à 1. Fait tourner et décale chaque répétition au hasard pour que le quadrillage cesse de se voir |
| netteté | De 0 à 1, apparaît dès que le détuilage dépasse 0. 0 mélange uniformément les répétitions décalées, ce qui paraît doux. 1 ne les mélange que le long de leurs jointures, ce qui garde la texture nette |

Une petite tuile est nette de près mais se répète visiblement de loin, une grande devient floue à vos pieds. Le détuilage
cache la répétition sans agrandir la tuile, mais il coûte des lectures de texture en plus, laissez-le donc à 0 là où la
répétition ne se voit pas.

![Tuile 256 avec détuilage 0 à gauche et 0,7 à droite](../assets/terrain-painting/detile-compare.jpg)

> [!NOTE]
> Le `metadata/texture_size` du matériau ne s'applique pas au terrain, seule **tuile** compte.

## Le rendu dans Godot { #how-it-looks-in-godot }

Chaque calque est projeté d'en haut et par les côtés, si bien que les falaises montrent de la roche au lieu d'une bavure,
et cela correspond à ce que vous voyez dans l'éditeur. Chaque calque utilise la texture de couleur, la carte normale, la
rugosité et l'émission de son matériau. L'occlusion ambiante est utilisée quand elle se trouve dans la texture de
rugosité, comme dans une texture ORM, ou quand le calque n'a pas de texture de rugosité.

![Les falaises peintes de l'île marine dans Godot](../assets/terrain-painting/godot-cliffs.jpg)
