---
source: c7fdbd8dcaa1ac8625c1d6e7014b686194c50f36
---

# Déclencheurs { #triggers }

> [!NOTE]
> Ces entités viennent avec le pack d'entités Gameplay, voyez
> [Installer et personnaliser](README.md#installing-and-customizing).

Des volumes brush invisibles qui réagissent quand un corps y entre. C'est ainsi qu'un niveau remarque le joueur : une
porte lance une cinématique, une fosse blesse, un tremplin propulse. Dessinez-en un avec l'outil **Volume** (Maj+E).

## Entrées, sorties et clés communes { #shared-inputs-outputs-and-keys }

Chaque déclencheur ci-dessous a ceci en plus de ce qui lui est propre.

* **Entrées :** `enable`, `disable`, `toggle`
* **Sorties :** `triggered(activator)`, `entered(activator)`, `exited(activator)`
* **Clés :** `filter_group` player, `start_disabled` 0

`filter_group` limite le déclencheur aux corps de ce groupe Godot, pour que les ennemis et les caisses ne
déclenchent pas un déclencheur de joueur. Laissez-le vide pour accepter n'importe quel corps.

| Sortie | Se lance |
| --- | --- |
| `entered` | À chaque entrée acceptée tant qu'il est activé |
| `triggered` | À une entrée acceptée, au plus une fois par `cooldown` pour chaque corps. La plupart du câblage utilise celle-ci |
| `exited` | À la sortie, même quand le déclencheur est désactivé, pour défaire quelque chose quand le corps part |

Les déclencheurs détectent les corps sur la couche physique 1. Si le joueur est sur une autre couche, ajoutez-la à
**Trigger Collision Mask** dans les réglages de carte (la ressource `FuncGodotMapSettings` dans Godot), sinon les
déclencheurs ne se lancent jamais.

## trigger_once { #trigger_once }

Se lance une fois puis se désactive, pour les événements uniques comme une cinématique ou un indice. `enable` le réarme.

* **Clés :** `once` 1

## trigger_multiple { #trigger_multiple }

Se lance à chaque entrée, au plus une fois par `cooldown` pour chaque corps, pour ce qui arrive à chaque fois, comme la
clochette d'une boutique.

* **Clés :** `cooldown` 0.5

## trigger_hurt { #trigger_hurt }

Blesse les corps tant qu'ils restent à l'intérieur, pour de la lave, des pointes ou une zone mortelle sous la carte. Il
appelle `damage_method(amount, source)` sur eux toutes les `interval` secondes, avec `damage * interval` comme quantité.

* **Sorties :** `hurt(activator)`, par corps blessé et par tick
* **Clés :** `damage` 10 (par seconde), `damage_method` take_damage, `interval` 0.5, `filter_group` vide

Avec le `filter_group` vide par défaut, il blesse tout ce qui a la méthode, pas seulement le joueur. Avec un groupe
réglé, `hurt` se lance pour chaque corps du groupe, même un corps sans la méthode.

## trigger_teleport { #trigger_teleport }

Déplace les corps vers l'entité nommée dans `destination`, d'habitude un
[`info_teleport_destination`](actors.md#info_teleport_destination), et les tourne dans la direction de son yaw. Il les
arrête sauf si `keep_velocity` vaut 1, pour un portail que le joueur doit traverser en tombant ou en courant sans perdre
sa vitesse.

* **Sorties :** `teleported(activator)`
* **Clés :** `destination`, `keep_velocity` 0, `cooldown` 0.5

## trigger_push { #trigger_push }

Pousse les corps avec `push`, une vitesse en unités de carte par seconde que vous pouvez orienter avec sa poignée dans la
vue. Avec `once` à 1, chaque entrée reçoit une seule impulsion, comme un tremplin. À 0, il pousse en continu comme une
soufflerie ou un tapis roulant, et un corps rigide cesse d'accélérer à la vitesse `push`.

* **Sorties :** `pushed(activator)`, une fois par entrée
* **Clés :** `push` 0 384 0, `filter_group` vide, `once` 1

## trigger_call { #trigger_call }

Appelle une méthode quand il est déclenché, sur `!activator` par défaut, donc un déclencheur peut appeler directement le
script du joueur, par exemple pour sauvegarder un point de contrôle. `call_target` et `arguments` fonctionnent comme dans
[`logic_call`](scripting.md#logic_call).

* **Entrées :** `trigger(activator)`
* **Sorties :** `called(result)`
* **Clés :** `call_target` `!activator`, `method`, `arguments` `[]`, `once` 0, `cooldown` 0.5

## trigger_spawn_area { #trigger_spawn_area }

Fait apparaître des scènes à des endroits au hasard dans son volume, par défaut quand le joueur entre, pour une embuscade
quand le joueur entre dans une salle. Ses clés fonctionnent comme sur [`info_spawner`](actors.md#info_spawner). `start`
lance un minuteur d'apparition quand `interval` est supérieur à 0, et `toggle` bascule ici ce minuteur plutôt que le
déclencheur.

* **Entrées :** `spawn`, `start`, `stop`, `toggle`, `kill_all`
* **Sorties :** `spawned(node)`, `all_dead`, `exhausted` (une fois, la première fois que `total` est atteint)
* **Clés :** `scene`, `count` 3, `max_alive` 10, `total` 0, `interval` 0, `spawn_on_enter` 1, `spawn_group` enemies,
  `snap_to_ground` 1, `cooldown` 0.5

## trigger_area { #trigger_area }

Un simple volume `Area3D` sans script, pour relier directement les événements d'entrée et de sortie d'une salle à
d'autres entités. Il n'a aucune des clés communes ci-dessus et se lance pour chaque corps, donc filtrez dans l'entité
qu'il appelle quand cela compte.

* **Sorties :** `body_entered(body)`, `body_exited(body)`
* **Clés :** `targetname`
