---
source: 059c4546c8d47d8cdd4e33a900932ff4ff2b3833
---

# Placer et câbler { #placing-and-wiring }

## Placer des entités { #placing-entities }

Ouvrez l'onglet **Entités**, il se trouve à côté de l'Inspecteur sur la droite. Chaque classe que votre projet connaît y
a une carte, classée par catégorie. Survolez une carte pour voir ce qu'elle fait.

![Le panneau Entités avec la carte func_door survolée, son infobulle dit entité brush, porte coulissante](../assets/placing/entities-panel.png)

Faites glisser une carte dans une vue pour la placer, ou double-cliquez dessus pour la déposer au curseur 3D. (Ctrl ou
Shift + clic choisit plusieurs cartes, faites-les glisser ensemble et elles tombent en rangée.) La carte d'une entité brush
vient avec une boîte. Avec des brushes sélectionnés, double-cliquez dessus pour envelopper ces brushes dans l'entité à la
place.

L'entité se pose sur la première surface solide sous le pointeur. Les volumes déclencheurs, les brushes faits seulement de
textures d'outil et les autres entités sont ignorés. Pendant que vous faites glisser, un repère cyan montre où elle va se
poser. Une vue 2D ne peut pas montrer la profondeur, donc là le repère porte la hauteur (ou la profondeur) où elle se
pose. Dans la vue Dessus, le toit d'une pièce gêne, donc maintenez Alt en déposant et l'entité passe le toit et se pose
sur le sol à l'intérieur. Dans les vues Face et Côté, Alt passe le mur proche et met l'entité à mi-chemin dans la pièce.

### Entités brush { #brush-entities }

Les portes, boutons et triggers sont des entités brush, elles prennent leur forme des brushes que vous avez construits.

1. Construisez la forme, par exemple une boîte fine là où va la porte, et sélectionnez-la.
2. Faites un clic droit dans une vue et choisissez une classe sous *Créer une entité brush*. *Brush > Entité brush* a la
    même liste.

![Le menu du clic droit d'une vue avec Créer une entité brush ouvert et trigger_once survolé](../assets/placing/context-create-brush-entity.png)

Les plus courantes ont des chemins plus rapides :

| Outil ou menu | Ce qu'il fait |
| --- | --- |
| *Gameplay > Portes et mobiles* | Une porte à charnière ou coulissante avec son trigger, ou un ascenseur, à partir des brushes sélectionnés |
| *Gameplay > Déclencheurs* | Un volume déclencheur dimensionné autour de la sélection |
| Outil **Volume** (Shift+E) | Trace d'un coup n'importe quel volume déclencheur, sans brush au préalable |
| Outil **Chemin** (Shift+P) | Clic après clic, une chaîne d'entités `path_corner` que les trains et les marcheurs suivent, Enter termine |

![Le menu Gameplay avec Portes et mobiles ouvert : portes à charnière, portes coulissantes et Ascenseur, Plateforme mobile](../assets/placing/gameplay-doors-menu.png)

![La barre d'outils avec l'outil Volume survolé et son infobulle, Outil volume (Shift+E), tracez un volume de déclenchement, d'apparition, de dégâts, de téléportation ou de poussée](../assets/placing/toolbar-volume-tool.png)

### Poignées { #handles }

Sélectionnez une porte, un portail, une plateforme, un bouton, un trigger de poussée, un spawner, une explosion ou un
accessoire physique et il affiche des poignées dans les vues. Faites-les glisser pour régler sa charnière, la distance
dont il s'ouvre ou voyage, son rayon ou sa taille, au lieu de taper des nombres. Un angle de charnière s'aimante par pas
de 15 degrés, maintenez Shift pour des degrés unitaires.

![Glisser la poignée d'angle d'une porte à charnière dans la vue Dessus, l'étiquette passe de 95° à 30°](../assets/placing/door-hinge-handle.gif)

## Câbler { #wiring }

Une connexion vit sur l'entité qui l'active. Sélectionnez cette entité et regardez **Sorties** en bas de l'Inspecteur.
**+ Ajouter une sortie** ajoute une connexion, puis vous remplissez sortie, cible et entrée. Ce que fait chaque champ est
expliqué dans [Comment fonctionnent les E/S d'entité](io.md).

![La section Sorties de l'Inspecteur pour un bouton : sortie pressed, cible front_door, entrée open](../assets/io/inspector-outputs.png)

Pour relier deux entités plus vite, sélectionnez-les toutes les deux et choisissez *Gameplay > Logique > Lier deux
entités sélectionnées…*. L'Inspecteur affiche aussi un bouton **Lier…** tant que deux entités exactement sont
sélectionnées.

![Le menu Gameplay avec Logique ouvert et Lier deux entités sélectionnées… survolé](../assets/placing/menu-link-two-entities.png)

La fenêtre *Lier des entités* montre la source à gauche et la cible à droite. Choisissez la sortie sous *Quand*, l'entrée
sous *appeler*, et appuyez sur **Lier**.

![La fenêtre Lier des entités : func_button à gauche, light à droite, Quand pressed, appeler turn_on, et l'infobulle Inverser](../assets/placing/link-dialog.png)

> [!WARNING]
> La source n'est pas toujours l'entité sur laquelle vous avez cliqué en premier. Vérifiez les deux noms en haut, et
> appuyez sur le bouton **Inverser** entre eux s'ils sont dans le mauvais sens.

Le [Graphe logique](logic-graph.md) montre tout le câblage d'un coup et relie les entités en glissant d'une broche à
l'autre.

## Vérifier le câblage sans Godot { #checking-wiring-without-godot }

Sélectionnez une entité dans le [Graphe logique](logic-graph.md#simulate), appuyez sur **Simuler** et choisissez une
sortie. Le graphe met en évidence chaque étape `source.output → target.input` qui suivrait et les liste avec leurs
délais, pour que vous puissiez suivre une longue chaîne sans construire ni lancer le jeu. Le rouge marque une cible qui
n'existe pas, d'habitude une faute de frappe dans un targetname. Le bleu marque une cible d'exécution comme `!player` ou
`@enemies`, qui n'existe qu'en jeu et n'est pas suivie plus loin.

## Du code pour n'importe quelle entité { #code-for-any-entity }

Le panneau **Référence** montre du GDScript et du C# pour n'importe quelle classe d'entité. Il est pratique quand le code
du jeu doit parler à une entité de la carte, par exemple pour ouvrir une porte ou écouter un bouton. Ouvrez-le avec
*Affichage > Panneaux > Référence*, avec *Aide > Référence des entités et du code*, ou avec **Référence du code** dans
l'Inspecteur, qui saute directement à la classe de l'entité sélectionnée. Faites glisser le séparateur entre la liste des
classes et les détails pour redimensionner la liste, double-cliquez dessus pour l'ajuster aux noms.

![Le panneau Référence avec func_button choisi : ses entrées et sorties, et l'Utilisation en GDScript](../assets/placing/reference-panel.png)

Choisissez une classe à gauche, puis un onglet. *Utilisation en GDScript* et *Utilisation en C#* montrent comment appeler
l'entité, *Classe GDScript*, *Classe C#* et *FGD (.tres)* sont des squelettes pour écrire la vôtre. **Copier** met le code
dans le presse-papiers. Sur les onglets de squelette, **Créer dans le projet** écrit le fichier sous `res://entities/` et
laisse tranquille un fichier qui existe déjà.
