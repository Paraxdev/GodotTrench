---
source: 63266bb349fb8b02f543d43498bfdee3d8c3f3cd
---

# Portes, mouvements et boutons { #doors-movers-and-buttons }

> [!NOTE]
> Ces entités viennent avec le pack d'entités Gameplay, voyez
> [Installer et personnaliser](README.md#installing-and-customizing).

Des entités brush qui bougent d'un seul bloc et emportent ce qui se tient dessus, donc un ascenseur emmène le joueur avec
lui.

> [!NOTE]
> Les portes et les boutons ne réagissent pas d'eux-mêmes au joueur. Le script de votre joueur appelle `use(activator)`
> dessus, par exemple depuis un rayon d'interaction, voyez [Un bouton qui ouvre une porte](../../tutorials/button-door.md).
> Mettez `interact` à 0 pour une entité que seules les E/S ou un script doivent bouger.

## func_door { #func_door }

Une porte coulissante, une trappe ou un volet. Elle se déplace de `travel` à l'ouverture et revient à la fermeture,
tirez la poignée dans la vue pour le régler. Une porte verrouillée ignore `open` et `use` et lance `locked_use` à la
place, pour un message « verrouillé ».

| Clé | Par défaut | Ce qu'elle fait |
| --- | --- | --- |
| `travel` | 0 64 0 | Décalage de coulissement, en unités de carte |
| `speed` | 2.0 | Mètres par seconde |
| `wait` | -1 | -1 reste ouverte, une valeur positive la referme après ce nombre de secondes |
| `locked` | 0 | Démarre verrouillée |
| `start_open` | 0 | Démarre en position ouverte |
| `interact` | 1 | S'ouvre quand le joueur appelle `use` dessus |

* **Entrées :** `open`, `close`, `toggle`, `lock`, `unlock`, `use(activator)`
* **Sorties :** `opened`, `closed`, `started_opening`, `started_closing`, `locked_use(activator)`

## func_door_rotating { #func_door_rotating }

Une porte ou un portail qui pivote autour d'une charnière que vous tirez en place dans la vue. `open` prend l'activateur
pour que `open_away` puisse l'ouvrir en l'éloignant de lui plutôt que dans sa figure.

| Clé | Par défaut | Ce qu'elle fait |
| --- | --- | --- |
| `hinge` | 0 0 0 | Position de la charnière relative au centre de la porte |
| `axis` | y | Axe de rotation. `y` est une charnière verticale, `x` ou `z` font une trappe ou un pont-levis |
| `open_angle` | 95 | Degrés de rotation, une valeur négative tourne dans l'autre sens |
| `speed` | 120 | Degrés par seconde |
| `open_away` | 1 | S'ouvre en s'éloignant de celui qui l'ouvre |
| `wait`, `locked`, `start_open`, `interact` | -1, 0, 0, 1 | Comme sur `func_door` |

* **Entrées :** `open(activator)`, `close`, `toggle(activator)`, `lock`, `unlock`, `use(activator)`
* **Sorties :** les mêmes que `func_door`

`func_gate` est un ancien alias qui n'a pour clés que `hinge`, `open_angle` 100 et `speed` 90. Utilisez
`func_door_rotating`.

## func_platform { #func_platform }

Un ascenseur ou une plateforme mobile entre son départ et `start + travel`, pour des ascenseurs ou des sols qui
traversent une fosse.

* **Entrées :** `start`, `stop`, `toggle`, `go_to_end`, `go_to_start`
* **Sorties :** `reached_end`, `reached_start`, `started`
* **Clés :** `travel` 0 128 0, `speed` 2.0, `wait` 1.0, `mode` 0, `start_active` 0 (bouge dès le chargement de la carte)

| `mode` | Ce que fait `start` | Usage typique |
| --- | --- | --- |
| 0, toggle | Va à l'autre extrémité | Un ascenseur appelé par un bouton |
| 1, ping pong | Va et vient jusqu'à `stop`, en attendant `wait` secondes à chaque bout | Une plateforme qui tourne sans cesse |
| 2, once | Va jusqu'au bout et y reste | Un pont qui se déploie une seule fois |

## func_train { #func_train }

Suit une chaîne d'entités `path_corner` à partir de `target`, pour tout ce qui parcourt un trajet à plusieurs arrêts,
comme un tram ou un chariot sur un rail. Le `wait` et le `speed` de chaque corner s'appliquent à l'arrivée. Il se met en
route au chargement de la carte et boucle, donc mettez `start_active` ou `loop` à 0 pour un trajet unique.

* **Entrées :** `start`, `stop`, `toggle`
* **Sorties :** `arrived(corner)`, `finished` (seulement sans boucle)
* **Clés :** `target`, `speed` 3.0, `loop` 1, `start_active` 1, `orient` 0 (1 le tourne vers l'endroit où il va)

## func_button { #func_button }

S'enfonce de `travel` tant qu'il est pressé et lance `pressed`. Il ressort après `wait` secondes, alors que -1 le garde
enfoncé pour un interrupteur à usage unique. Un bouton verrouillé lance `locked_use` à la place, pratique pour un panneau
qui a d'abord besoin de courant.

* **Entrées :** `press(activator)`, `release`, `lock`, `unlock`, `use(activator)`
* **Sorties :** `pressed(activator)`, `released`, `locked_use(activator)`
* **Clés :** `travel` 0 -2 0, `wait` 1.0, `locked` 0, `interact` 1
