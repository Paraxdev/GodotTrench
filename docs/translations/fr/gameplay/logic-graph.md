---
source: 8586c5cf175329bb5dcb3e351f21f472800d70ab
---

# Le Graphe logique { #the-logic-graph }

Le panneau **Logique** montre les E/S d'entité de la carte sous forme de graphe. Chaque entité câblée et chaque entité
logique est un nœud, et chaque connexion est un fil allant d'un pin de sortie à droite d'un nœud à un pin d'entrée à
gauche d'un autre.

Le graphe montre les mêmes sorties que celles que modifie l'Inspecteur, donc un changement de l'un se voit aussitôt dans
l'autre. La carte, la construction Godot et le jeu marchent exactement pareil que vous utilisiez le graphe ou non.

## L'ouvrir { #opening-it }

Choisissez *Affichage > Panneaux > Logique*.

![Le menu Affichage avec Panneaux ouvert et Logique survolé](../assets/logic-graph/menu-panels-logic.png)

Il s'ouvre à côté de Matériaux sous les vues et agrandit ce dock jusqu'à environ la moitié du milieu de la fenêtre,
puisqu'un graphe a besoin de place. Faites glisser le séparateur ou l'onglet pour changer cela.

![Le panneau Logique de la carte de démo Night District : des boutons reliés à des portes, un trigger et un timer, avec des en-têtes colorés par catégorie, et un cadre intitulé Hall lights autour d'un trigger qui allume trois lumières avec des délais](../assets/logic-graph/graph.png)

## Lire le graphe { #reading-the-graph }

| Ce que vous voyez | Ce que ça veut dire |
| --- | --- |
| Titre du nœud | Le targetname de l'entité, sinon son nom donné avec *Renommer*, sinon son classname. Dessous viennent la catégorie et le classname |
| Lignes grises sous le titre | Les réglages qui diffèrent des valeurs par défaut de l'entité, comme `max 3`, `wait 2 s` ou `once`, coupés à deux courtes lignes. Un nœud sans elles est aux valeurs par défaut. L'Inspecteur les montre tous |
| Couleur d'en-tête | La catégorie : le groupe de la définition, sinon le premier mot du classname, donc `logic_relay` est *logic*. Voir [Couleurs](#colors) |
| Pins | Entrées à gauche, sorties à droite, d'après la [définition d'entité](entities/README.md) plus tout nom que la carte utilise déjà sur cette entité. Un pin plein a un fil. Survolez un pin pour son type et les valeurs qu'il passe |
| Nom de pin jaune | Le nom n'est pas dans la définition de l'entité, le panneau Problèmes vous en avertit aussi |
| Étiquette de fil | Le délai, `once` ou le nombre de fois où il peut s'activer, et le paramètre, quand ils sont réglés. Survolez un fil pour voir sa sortie, sa cible, son entrée et son délai en entier |
| Nœud et fil bleus | Une [cible](parameters.md#targets) que seul le jeu en cours résout, comme `!player`, `@enemies`, un chemin de nœud ou un joker comme `door_*` |
| Nœud rouge, fil rouge en tirets | Un targetname qu'aucune entité n'a, d'habitude une faute de frappe |

Seules les entités câblées et les entités logiques et mathématiques reçoivent un nœud. Sélectionnez n'importe quelle autre
entité, dans une vue ou l'Outliner, et elle apparaît aussi, prête à être câblée.

## Couleurs { #colors }

Les couleurs aident à parcourir un grand graphe, mais chaque couleur vient aussi avec un nom ou une forme, et les palettes
restent distinctes pour les formes courantes de daltonisme.

| Catégorie | Couleur |
| --- | --- |
| logic | bleu |
| math | orange |
| trigger | vert bleuté |
| func, portes, boutons et mobiles | vermillon |
| light | jaune |
| env | sarcelle |
| game | bleu ciel |
| prop | rose |
| npc | pourpre rougeâtre |
| info | gris |

| Type de pin | Couleur |
| --- | --- |
| pulse, un événement sans valeur | blanc |
| bool | vermillon |
| int | bleu ciel |
| float | vert bleuté |
| string | rose |
| vector3 | jaune |
| color | pourpre rougeâtre |
| node, comme l'activateur | bleu |
| variant, n'importe quelle valeur | gris |

Les pins qui passent une valeur sont des cercles, les pulses des triangles. Un fil prend la couleur de la sortie d'où il
part. Une catégorie à vous, réglée avec le `group` d'une définition, reçoit aussi sa propre couleur.

## Câbler { #wiring }

Faites glisser d'un pin de sortie vers un pin d'entrée. Pendant le glisser, les pins que le fil peut rejoindre reçoivent un
anneau, et le pin où il finirait se remplit. Relâchez et la connexion est ajoutée aux sorties de l'entité source.

![Glisser un fil depuis la sortie released de button_1 vers l'entrée turn_off de hall_light, les pins qu'il peut rejoindre montrent des anneaux](../assets/logic-graph/wire-drag.gif)

Un pulse convient à n'importe quel pin et une valeur convient à un pin de son propre type, donc un nœud `activator` ne
convient pas à une entrée `int`. (Un int convient aussi à un pin float ou bool et inversement, et un variant convient à
tout.) N'importe quel pin accepte tout de même le fil, les noms dans les E/S d'entité ne sont vérifiés que quand le jeu
tourne.

Une cible sans targetname en reçoit un dans la même étape : le nom que vous lui avez donné avec *Renommer* dans l'Outliner,
donc une lumière que vous avez appelée `lamp` devient la cible `lamp`, sinon un nom libre comme `door_2`. Un nom qu'une
autre entité a déjà reçoit un numéro derrière.

Vous pouvez aussi déposer le fil ailleurs :

* Sur le titre d'un nœud, pour choisir parmi toutes ses entrées ou taper un nom quelconque.
* Sur un espace vide, pour y ajouter une entité logique, déjà câblée au pin d'où vous avez glissé.

### Ajouter des entités logiques { #adding-logic-entities }

Faites un clic droit sur un espace vide, ou appuyez sur **Ajouter** en haut à gauche du panneau, pour ajouter une entité
logique sans fil.

![La liste Ajouter : un champ de recherche, puis le groupe logic avec logic_animate en surbrillance, logic_auto, logic_branch et d'autres](../assets/logic-graph/add-menu.png)

Sans recherche, la liste est classée par catégorie, logic et math d'abord, puis les autres entités qui ont des entrées ou
des sorties. Tapez pour chercher parmi toutes : les classes dont le nom commence par le texte ou le contient viennent en
premier, puis celles dont la catégorie ou la description le contient. Les flèches haut et bas déplacent la surbrillance et
Enter ajoute cette classe.

Les nouvelles entités sont des entités ponctuelles, et la construction Godot fait un nœud pour chacune, donc elles ont
besoin d'une place dans la carte. Elles vont sur un calque appelé *Logic*, à côté de l'entité à laquelle elles sont
câblées, ou devant la caméra de la vue 3D. Chacune reçoit aussitôt un targetname libre, comme `relay_2`, qui titre son
nœud. Changez-le dans l'Inspecteur, puisque *Renommer* ne règle que le nom dans l'Outliner.

> [!NOTE]
> Sans le pack d'entités Gameplay, il n'y a aucune entité logique à ajouter. Le menu Ajouter propose alors de
> l'installer.

### Modifier une connexion { #editing-a-connection }

Cliquez sur un fil et une boîte en haut à droite du graphe vous laisse modifier sa sortie, sa cible, son entrée, son
paramètre, son délai et ses fois. Son bouton **Supprimer**, ou la touche Delete pendant que le pointeur est sur le graphe,
le retire.

![Un fil sélectionné de button_1 vers hall_light et la boîte Connexion depuis button_1 en haut à droite avec sortie released, cible hall_light, entrée turn_off, délai 0,00 s, fois à chaque fois et Supprimer](../assets/logic-graph/connection-box.png)

Faites un clic droit n'importe où le long d'un fil, pas seulement sur son étiquette, pour **Modifier la connexion** et
**Supprimer la connexion**. Chaque changement est une étape de l'*Historique*, donc l'annulation marche comme partout
ailleurs.

### Le menu du nœud { #the-node-menu }

Faites un clic droit sur un nœud pour ce qu'on peut faire avec son entité.

![Le menu du clic droit du nœud hall_light : Cadrer dans les vues, Renommer, Dupliquer, Sélectionner les connectés, Tout déconnecter, Supprimer l'entité, et Simuler avec sa sortie switched](../assets/logic-graph/node-menu.png)

| Élément | Ce qu'il fait |
| --- | --- |
| Cadrer dans les vues | La sélectionne et y amène les vues, comme un double-clic |
| Renommer | Renomme l'entité dans l'Outliner. Le nœud est titré par le targetname quand il y en a un, donc le nom n'y apparaît que pour une entité sans |
| Dupliquer | Copie l'entité d'un pas de grille à côté, fils depuis elle compris |
| Sélectionner les connectés | Sélectionne le nœud et chaque nœud auquel un fil le relie |
| Tout déconnecter | Supprime tous les fils entrant dans le nœud ou en sortant, en une étape. Les entités restent |
| Supprimer l'entité | Retire l'entité de la carte, voir [Copier, coller et supprimer](#copy-paste-and-delete) |
| Simuler | Active une de ses sorties sur le papier, voir [Simuler](#simulate) |

Avec plusieurs nœuds sélectionnés, Dupliquer et Supprimer agissent sur tous quand vous faites un clic droit sur l'un des
nœuds sélectionnés.

## Copier, coller et supprimer { #copy-paste-and-delete }

Le graphe montre des entités de la carte, donc ces touches agissent sur les entités sélectionnées, brushes compris,
pendant que le pointeur est sur le graphe.

| Touche | Ce qu'elle fait |
| --- | --- |
| Ctrl+C, Ctrl+V | Copier et coller. La copie se pose d'un pas de grille de l'original, fils compris, puisqu'il n'y a pas de vue sous le pointeur où la mettre. Collez avec le pointeur sur une vue pour la placer là |
| Ctrl+D | Dupliquer, aussi d'un pas de grille de l'original |
| Delete | Retire le fil ou le cadre sélectionné. Sans rien de sélectionné, elle supprime les **entités sélectionnées de la carte**, pas seulement leurs nœuds. Les fils d'autres entités vers une entité supprimée deviennent rouges. Annuler restaure tout |
| Ctrl+A | Sélectionne tout dans la carte, pas seulement les nœuds affichés |

## Se déplacer { #moving-around }

| Pour | Faites |
| --- | --- |
| Zoomer | Tournez la molette |
| Déplacer la vue | Glissez avec le bouton du milieu ou le droit. Un clic droit sans bouger ouvre un menu, donc un déplacement hésitant peut le faire surgir, Escape le ferme |
| Sélectionner | Cliquez sur un nœud, Shift ou Ctrl + clic ajoute, glissez sur un espace vide pour une sélection par rectangle. Les entités sont sélectionnées aussi dans les vues et l'Outliner, et inversement |
| Cadrer dans les vues | Double-cliquez sur un nœud |
| Déplacer | Glissez des nœuds. Leurs positions sont enregistrées dans la carte |
| Ranger | **Organiser** dispose les nœuds sélectionnés selon le flux des signaux. Avec moins de deux sélectionnés, chaque nœud revient à la disposition automatique |
| Tout voir | **Ajuster**, ou Home avec le pointeur sur le graphe |
| Voir la sélection | F avec le pointeur sur le graphe cadre les nœuds sélectionnés |

Les nœuds que personne n'a déplacés suivent la disposition automatique, de gauche à droite selon le flux des signaux, et
la vue zoome pour tenir dans le panneau jusqu'à ce que vous zoomiez ou déplaciez vous-même. Une fois que vous déplacez un
nœud ou câblez quelque chose dans le graphe, les autres gardent leur place, et un nouveau nœud apparaît à côté de ce à quoi
il est câblé. Les nœuds repères bleus et rouges ne sont pas des entités, donc vous ne pouvez pas les déplacer. Ils se
tiennent à côté du nœud qui les cible.

## Cadres { #frames }

Un cadre est une boîte titrée derrière un groupe de nœuds, pour garder un grand graphe organisé : les portes d'un bâtiment,
une énigme, une cinématique. Sélectionnez les nœuds et appuyez sur **C** sur le graphe, ou sur **Cadrer** dans la barre
d'outils du panneau, puis tapez son titre. La boîte *Hall lights* de l'image du haut en est une.

| Pour | Faites |
| --- | --- |
| Déplacer un cadre avec tout son contenu | Glissez son titre |
| Le redimensionner | Glissez le coin en bas à droite |
| Le renommer | Double-cliquez sur son titre, ou sélectionnez-le et appuyez sur F2 |
| Le recolorer ou le supprimer | Faites un clic droit sur son titre. Supprimer un cadre garde ses nœuds |

Les cadres sont enregistrés dans la carte comme données d'éditeur, la construction Godot les ignore.

## Simuler { #simulate }

Sélectionnez une entité, appuyez sur **Simuler** et choisissez l'une de ses sorties, ou faites un clic droit sur un nœud.
Le graphe numérote en jaune chaque fil que la sortie déclencherait, et une liste sous le graphe montre chaque étape par les
titres des nœuds, dans l'ordre d'arrivée des étapes, avec le temps en secondes après les délais. **Effacer** termine
l'aperçu. Cliquez sur une étape pour sélectionner son fil.

![Simuler sur button_2 : les fils vers garage_door puis garage_beacon sont jaunes et numérotés de 1 à 3, et la liste sous le graphe nomme chaque étape](../assets/logic-graph/simulate.png)

Il suit les règles de l'exécution : les cibles sont trouvées par targetname, et les relays, timers, compteurs, portes
logiques, portes et les autres entités courantes passent la chaîne. Les cibles d'exécution comme `!player` sont montrées
mais pas suivies.

Une entité qui active l'une de plusieurs sorties montre tous les fils qu'elle pourrait déclencher. Ces étapes, et tout ce
qui vient après, finissent par `(maybe)` dans la liste, qui s'affiche `(peut-être)` en français, et par `?` sur le numéro du fil, et le survol dit ce qu'elles
attendent. Un compteur alimenté par `add` ou `subtract` continue vers `hit_max` et `hit_min` de cette façon, puisque
l'aperçu ne compte pas les pressions : l'étape est là, et elle a lieu quand le compteur atteint son `max` ou son `min`. Un
`logic_sequence` active ses étapes aux moments de ses propriétés `interval`, `times` et `steps`, donc une cinématique se
lit comme une frise chronologique.

Simuler marche sur le papier, sans Godot. Ce qui se passe vraiment en jeu est traité dans
[Déboguer le câblage](debugging.md).
