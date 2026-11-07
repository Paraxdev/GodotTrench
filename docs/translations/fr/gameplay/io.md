---
source: 757483e9586cbd895b01c35597f84b4f54089ad0
---

# Comment fonctionnent les E/S d'entité { #how-entity-io-works }

Les E/S d'entité sont la façon dont une carte fait arriver des choses sans écrire de code, câblées comme le fait Hammer
pour Source.

Chaque entité a des **sorties**, des événements qu'elle annonce comme un bouton qu'on presse, et des **entrées**, des
choses qu'on peut lui dire de faire comme l'ouverture d'une porte. Une connexion dit : quand cette sortie s'active,
appelle cette entrée sur une cible. Le câblage est enregistré dans la carte, donc vous reliez un bouton à une porte dans
l'éditeur et la porte s'ouvre en jeu.

## Une connexion { #a-connection }

Une sortie peut porter autant de connexions que vous voulez. Chacune a ces champs :

| Champ | Ce qu'il fait |
| --- | --- |
| Sortie | L'événement de cette entité qui lance la connexion, par exemple `pressed` sur un bouton |
| Cible | Quelles entités reçoivent l'appel, d'habitude un targetname, voir [Cibles](parameters.md#targets) |
| Entrée | Ce que les cibles doivent faire, par exemple `open` sur une porte |
| Paramètre | Une valeur facultative passée à l'entrée, comme une quantité de dégâts, voir [Paramètres](parameters.md#parameters) |
| Délai | Les secondes à attendre avant d'appeler l'entrée, 0 l'appelle tout de suite. Servez-vous-en pour échelonner des événements, comme des lumières qui s'allument l'une après l'autre |
| Fois | Combien de fois la connexion peut s'activer. -1 veut dire à chaque fois, 1 fait un événement à usage unique |

Vous les modifiez dans l'Inspecteur, sous **Sorties** de l'entité sélectionnée, ou sous forme de fils dans le
[Graphe logique](logic-graph.md). Les deux modifient les mêmes sorties. (L'Inspecteur n'a qu'une case **une seule fois**
pour *Fois*, pour tout autre nombre utilisez la boîte de connexion du Graphe logique.)

![La section Sorties de l'Inspecteur pour un bouton : sortie pressed, cible front_door, entrée open, un paramètre vide, délai 0,00 s et la case une seule fois](../assets/io/inspector-outputs.png)

Dans Godot, une sortie est un signal et une entrée est une méthode : `func_button` a `signal pressed(activator)` et
`func_door` a `func open()`. Vos propres scripts participent de la même façon, voir
[Entités personnalisées](custom-entities.md).

## Quand une sortie s'active { #when-an-output-fires }

Après *Délai* secondes, l'entrée est appelée sur chaque nœud que la cible trouve. Une cible qui ne trouve rien ne fait
rien. Pour un targetname ou un chemin de nœud, c'est presque toujours une faute de frappe, donc un avertissement
s'affiche la première fois qu'elle s'active. Un appel retardé est abandonné si son entité a quitté l'arbre entre-temps,
par exemple parce qu'elle a été tuée.

Sans délai, toute la chaîne s'exécute dans l'`emit` du signal, avant que le code qui l'a émis continue. Cela compte quand
votre script lit un état juste après l'émission : les portes et compteurs plus bas ont déjà réagi.

## L'activateur { #the-activator }

L'activateur est celui qui a lancé la chaîne, d'habitude le joueur qui a pressé le bouton ou est entré dans le trigger.
Plus bas dans la chaîne, `!activator` désigne ce nœud, donc un trigger peut blesser ou téléporter exactement celui qui l'a
touché.

Il est pris dans le premier argument `Node` du signal qui s'est activé, donc il ne survit que tant que chaque signal de la
chaîne en transmet un. Les signaux intégrés se comportent ainsi :

| Signaux | Ce qui arrive à l'activateur |
| --- | --- |
| `triggered`, `pressed`, `entered`, `exited`, `hurt`, `teleported`, `pushed` | Transmis tel quel, ces signaux portent le nœud qui les a causés |
| `arrived(corner)`, `spawned(node)`, `reached(train)` | Remplacé par le nœud entre parenthèses, donc plus bas `!activator` est ce corner, ce nœud créé ou ce train |
| `opened`, `timer`, `step_N`, `on_true`, `on_case_N`, `result`, `broken`, `switched`, `map_spawn` | Perdu. Ils ne portent aucun nœud, donc à partir de là `!activator` ne trouve rien |

> [!TIP]
> Besoin du joueur plusieurs étapes plus bas dans une chaîne qui a perdu l'activateur ? Ciblez plutôt `!player`, il ne
> dépend pas de la chaîne.
