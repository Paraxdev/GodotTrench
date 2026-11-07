---
source: ba7a98139fdce301c2c6a3f48c26f62837b5d180
---

# Scatter { #scatter }

L'outil **Scatter** (B) peint des arbres, des rochers et de l'herbe sur les surfaces que vous choisissez, de quoi couvrir
un versant en quelques secondes au lieu de placer chaque modèle à la main. Le [tutoriel de l'île marine](../tutorials/sea-island-scatter.md)
déroule une mise en place complète.

![Peindre une bande de forêt à travers une île avec l'outil Scatter](../assets/scatter/scatter-paint.gif)

## Ensembles de scatter { #scatter-sets }

Tout ce que vous éparpillez vit dans un **ensemble de scatter** sur son propre calque, par exemple un ensemble pour les
arbres et un pour l'herbe. Appuyez sur B et le panneau Scatter s'ouvre à droite, avec une carte pour chaque modèle de
l'ensemble actif.

![Le panneau Scatter avec l'ensemble bushes actif : le menu des ensembles, +, Préréglage, Renommer et une carte par modèle](../assets/scatter/scatter-panel.png)

| Pour | Faites ceci |
| --- | --- |
| Démarrer un ensemble vide | **+** à côté du menu des ensembles |
| Partir d'un préréglage | **Préréglage**, puis choisissez l'un des dix ensembles intégrés |
| Ajouter des modèles | Faites-les glisser depuis le panneau Modèles, ou **Ajouter des modèles…**. **Ajouter les props sélectionnés** prend les modèles des entités sélectionnées |
| Arrêter de peindre un modèle | Décochez **peindre** sur sa carte. Ce qu'il a déjà placé reste |
| Régler un modèle | Cliquez sur la flèche à droite de sa carte |

![Le menu Préréglage ouvert avec forest, pines, mixed woodland, detailed forest, low-poly trees, bushes, undergrowth, rocks, boulders et grass, rocks survolé](../assets/scatter/preset-menu.png)

Les préréglages rochers, blocs, herbe, sous-bois et arbres low-poly utilisent des modèles Blockbench intégrés à
l'éditeur. En choisir un ajoute les modèles dont il a besoin dans `res://trenchdot/nature`, hors ligne. Les préréglages
forêt, pins, bois mixte, forêt détaillée et buissons utilisent les arbres et buissons glTF du pack nature, un
téléchargement d'environ 23 Mo. Dans un projet qui ne l'a pas, en choisir un ouvre *Godot > Ajouter du contenu au
projet…* sur les modèles de nature plutôt que de créer un ensemble dont les modèles manquent. *Terrain > Scatter >
Installer les modèles de nature…*, ou **Installer les modèles de nature…** en bas du panneau, l'ouvre aussi. Les
fichiers déjà présents dans le projet ne sont jamais écrasés.

### Cartes de modèle { #model-cards }

Ouvrez une carte avec sa flèche pour régler ce modèle précis.

![Une carte ouverte pour bush_round avec Échelle, Espacement, Aligner, Inclinaison, Enfoncement, Rotation aléatoire et Matériau](../assets/scatter/model-card.png)

| Réglage de carte | Ce qu'il fait |
| --- | --- |
| poids | La fréquence à laquelle ce modèle est choisi par rapport aux autres de l'ensemble |
| Échelle | Une taille aléatoire entre les deux valeurs |
| Espacement | La plus petite distance, en unités de carte, jusqu'à toute autre instance |
| Aligner | 0 dresse le modèle à la verticale, 1 l'incline pour suivre la surface |
| Inclinaison | La plus grande inclinaison aléatoire en degrés |
| Enfoncement | De combien le modèle est enfoncé dans le sol, pour cacher le bas d'un tronc ou d'un rocher |
| Rotation aléatoire | Tourne chaque instance dans une direction aléatoire |
| Matériau | Dessiné à la place des matériaux propres au modèle. Laissé vide, le matériau de l'ensemble s'applique, s'il en a un |

## Cibles { #targets }

Un ensemble ne se pose que sur ses **cibles**, si bien qu'une forêt peinte sur un terrain ne recouvre jamais la maison qui
s'y trouve. Alt+clic dans une vue, ou cliquez sur la pipette à droite de *Cibles* puis cliquez dans une vue, pour ajouter
ou retirer une cible. Un ensemble sans cibles prend ce que son premier trait touche. Un autre ensemble peut aussi être une
cible, c'est ainsi que l'herbe pousse sur des blocs de roche.

![La section Cibles du panneau Scatter avec le terrain pour cible, la pipette en haut à droite](../assets/scatter/targets.png)

Cocher **Suivre le curseur** ignore les cibles et peint sur n'importe quelle surface ou prop éparpillé sous le brush.

## Peindre { #painting }

Les réglages rapides se trouvent dans la barre d'options de l'outil en haut à gauche : l'ensemble, **Peindre** ou
**Effacer**, le rayon, la densité, la pente max, **Suivre le curseur**, la pipette et **Remplir**. **Panneau scatter**
ramène le panneau si vous l'avez fermé.

![La barre d'options de l'outil Scatter avec l'ensemble bushes, Peindre, rayon 512 u, densité 1.00 et pente max 40°](../assets/scatter/scatter-toolbar.png)

| Entrée | Action |
| --- | --- |
| Glisser avec le clic gauche | Peindre |
| Shift+glisser avec le clic gauche | Effacer, dans l'ensemble actif seulement |
| Ctrl+molette | Redimensionner le brush |
| **Remplir les cibles** | Couvrir chaque cible d'un coup, dans les limites de pente et de hauteur |

La section *Brush* du panneau contient le reste.

![La section Brush du panneau Scatter : Rayon, Densité, Pente, Hauteur, Atténuation, Seulement exposé, Quantité d'effacement, Graine, Sortie et les cases à cocher en dessous](../assets/scatter/brush-settings.png)

**Densité** est le nombre de placements que le brush tente par zone de 64 sur 64 unités, et l'espacement de chaque modèle
limite combien tiennent réellement. **Seulement exposé** saute les points qui ont de la géométrie au-dessus, ce qui évite
l'herbe sous les balcons et les toits. **Garder l'espacement avec les autres ensembles**, activé par défaut, empêche un
buisson de pousser à travers le tronc d'un autre ensemble.

## Dans Godot { #in-godot }

Le **Type** d'un ensemble décide de ce que Godot construit.

| Type | Construit comme |
| --- | --- |
| props | Des MultiMeshes avec collision et ombres. Les modèles dont les scènes portent des scripts sont instanciés un par un pour que les scripts s'exécutent |
| foliage | Des MultiMeshes sans collision ni ombres, masqués au-delà de 2400 unités par défaut. Prévu pour l'herbe et les petites plantes |

Sélectionnez l'ensemble dans l'Outliner et sa ligne **Collision** dans l'Inspecteur choisit la forme avec laquelle chaque
instance entre en collision. Les nouveaux ensembles props démarrent sur convexe et les ensembles foliage sur aucun.

| Collision | Forme par instance |
| --- | --- |
| aucun | Pas de collision |
| boîte | Une boîte autour de l'instance |
| convexe | Une enveloppe convexe autour de l'instance |
| décomposition | Plusieurs enveloppes convexes qui suivent les modèles concaves, comme une arche, plus longues à construire |
| trimesh | Chaque face du modèle, exact mais le plus coûteux |

Quand vous devez modifier des props isolés, **En entités** en bas du panneau Scatter, ou *Terrain > Scatter > Sets scatter
en entités* pour les ensembles sélectionnés, remplace un ensemble par une entité `prop_model` par instance.

![Le bas du panneau Scatter avec Type, Peindre, Effacer, Remplir les cibles, Effacer les instances, En entités et Installer les modèles de nature…](../assets/scatter/panel-buttons.png)
