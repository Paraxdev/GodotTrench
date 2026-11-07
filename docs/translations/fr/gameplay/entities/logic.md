---
source: 5168ac40e48ba63850a1be1795dae420bfe6bf5f
---

# Logique { #logic }

> [!NOTE]
> Ces entités viennent avec le pack d'entités Gameplay, voyez
> [Installer et personnaliser](README.md#installing-and-customizing).

Des entités point invisibles qui décident quand les choses arrivent. Elles se placent entre un déclencheur ou un bouton
et ce qu'il commande, et façonnent le signal au passage. Les entités qui travaillent avec des nombres sont dans
[Maths et valeurs](math.md), et celles qui lancent du code ou des animations dans [Scripts et appels](scripting.md).

## logic_relay { #logic_relay }

Transmet `trigger` sous la forme `triggered` tant qu'il est activé. Reliez plusieurs choses à un même relais, puis
coupez-le avec `disable` pour arrêter toute la chaîne d'un coup.

* **Entrées :** `trigger(activator)`, `enable`, `disable`, `toggle`
* **Sorties :** `triggered(activator)`
* **Clés :** `start_disabled` 0

## logic_branch { #logic_branch }

Retient vrai ou faux et lance `on_true` ou `on_false` sur `test`, pour des vérifications comme « la porte ne s'ouvre que
si le courant est allumé ». Le générateur le met à vrai, le bouton de la porte le teste.

* **Entrées :** `set_true`, `set_false`, `toggle`, `test`, `set_and_test(value)`
* **Sorties :** `on_true`, `on_false`
* **Clés :** `start_value` 0

## logic_gate { #logic_gate }

Garde deux valeurs vrai ou faux, `a` et `b`, et les combine avec une porte, pour des vérifications comme « la porte
s'ouvre quand le courant est allumé **et** que le levier est tiré ». Chaque changement d'entrée qui fait basculer le
résultat lance `changed(on)` puis `on_true` ou `on_false`, donc personne n'a besoin de poser la question. `test` relance
le résultat actuel, pour quand autre chose a besoin de le savoir tout de suite.

* **Entrées :** `set_a(value)`, `set_b(value)`, `toggle_a`, `toggle_b`, `test`
* **Sorties :** `on_true`, `on_false`, `changed(on)`
* **Clés :** `gate` and, `start_a` 0, `start_b` 0

| `gate` | Vrai quand |
| --- | --- |
| `and` | les deux sont vraies |
| `or` | au moins une est vraie |
| `xor` | exactement une est vraie |
| `nand` | les deux ne sont pas vraies |
| `nor` | aucune n'est vraie |
| `xnor` | les deux sont identiques |
| `not` | `a` est faux, `b` est ignoré |

`set_a` et `set_b` sans valeur mettent à vrai, donc la sortie `entered` d'un déclencheur reliée à `set_a` active `a`. Un
nombre est vrai sauf s'il vaut 0, et un texte n'est vrai que pour `true`, `yes` ou `1`. Les valeurs de départ ne lancent
rien, pas plus qu'un changement qui laisse le résultat tel qu'il était.

## logic_flipflop { #logic_flipflop }

Alterne à chaque déclenchement, donc un seul bouton peut allumer puis éteindre quelque chose. `on_a` se lance quand il
s'allume et `on_b` quand il s'éteint. `reset` le remet à `start_on` sans lancer ni l'un ni l'autre, et ne lance que
`changed` si cela a changé quelque chose.

* **Entrées :** `trigger`, `reset`
* **Sorties :** `on_a`, `on_b`, `changed(on)`
* **Clés :** `start_on` 0 (allumé, donc le premier déclenchement lance `on_b`)

## logic_case { #logic_case }

Compare la valeur qu'il reçoit à huit cas au plus et lance la sortie du premier qui correspond, ou `on_default` quand
aucun ne correspond. Il transforme un nombre, comme l'indice d'un `logic_random` ou le score d'un `math_value`, en
événements différents.

* **Entrées :** `in_value(value)`
* **Sorties :** `on_case_1` à `on_case_8`, `on_default`
* **Clés :** `case_1` à `case_8`, vides

Les nombres correspondent par valeur, donc le cas `2` correspond à une valeur de 2 comme de `2.0`. Les textes doivent
correspondre exactement, `Red` n'est pas `red`, et les espaces autour d'un cas sont ignorés. Les cas vides sont sautés,
la première correspondance l'emporte, et une valeur absente va à `on_default`.

## logic_random { #logic_random }

Choisit au hasard une sortie parmi huit au plus, ou tire une chance. Reliez `out_1` à `out_3` à trois lumières et `pick`
en allume une. `roll` décide plutôt au hasard, par exemple si un coffre contient du butin.

* **Entrées :** `pick`, `roll`
* **Sorties :** `out_1` à `out_8`, `picked(index)`, `on_success`, `on_fail`
* **Clés :** `count` 3 (1 à 8), `no_repeat` 0, `chance` 50 (pourcent)

`pick` lance `picked` avec le numéro de la sortie choisie puis cette sortie, donc `picked` peut alimenter un
[`logic_case`](#logic_case). Avec `no_repeat`, il ne choisit jamais la sortie choisie la fois d'avant, sauf si `count`
vaut 1. `roll` lance `on_success` avec une probabilité de `chance` pourcent et `on_fail` sinon, donc 0 ne réussit jamais
et 100 réussit toujours.

## logic_counter { #logic_counter }

Compte entre `min` et `max`, pour des énigmes comme « appuyez sur les trois boutons » : chaque bouton envoie `add`, et
`hit_max` ouvre la porte. La valeur est bornée et rien ne se lance si elle ne change pas. `hit_max` et `hit_min` se
lancent quand un changement tombe sur une limite, `reset` ne lance que `changed`.

* **Entrées :** `add(amount)` (1 par défaut), `subtract(amount)`, `set_value(value)`, `reset`
* **Sorties :** `hit_max`, `hit_min`, `changed(value)`
* **Clés :** `min` 0, `max` 3, `start_value` 0

## logic_timer { #logic_timer }

Lance `timer` encore et encore, pour tout ce qui se répète, comme une lumière qui vacille ou des vagues d'ennemis. Il
attend `interval` secondes entre deux lancements et tourne dès le chargement de la carte, sauf si `start_on` vaut 0. Avec
`random_max` au-dessus de `random_min`, chaque attente est tirée au hasard dans cette plage, pour que les répétitions
paraissent moins mécaniques. `once` l'arrête après le premier lancement, ce qui en fait un simple délai.

* **Entrées :** `start`, `stop`, `toggle`, `fire` (une fois, tout de suite)
* **Sorties :** `timer`
* **Clés :** `interval` 1.0, `random_min` 0, `random_max` 0, `start_on` 1, `once` 0

## logic_auto { #logic_auto }

Lance `map_spawn` une fois, quand toutes les entités de la carte sont prêtes. Reliez ici tout ce qui part dès le début,
comme la musique ou un texte d'introduction.

* **Sorties :** `map_spawn`

## logic_sequence { #logic_sequence }

Une chronologie de cinématique. Elle lance huit étapes numérotées au plus, dans l'ordre, et attend avant chacune, la
première comprise. Reliez chaque étape à ce qui se passe à ce moment, une porte qui s'ouvre, une ligne de texte, un
acteur qui marche.

* **Entrées :** `start`, `stop`, `reset`
* **Sorties :** `step_1` à `step_8`, `step(index)`, `finished`
* **Clés :** `steps` 3 (1 à 8), `interval` 1.0, `times` vide, `start_active` 0 (tourne dès le chargement de la carte),
  `loop` 0

`times` liste une attente par étape, comme `0 1 0.5`, et les étapes qui n'en ont pas utilisent `interval`. Une séquence
en boucle ne lance jamais `finished`. `start` est ignoré pendant qu'elle tourne, et après `stop` elle repart à l'étape 1.
Voyez le [tutoriel de cinématique](../../tutorials/cutscene.md).
