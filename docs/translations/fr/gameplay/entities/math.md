---
source: c6245792b90999ca4b48812eb3d96e7a9c2389d6
---

# Maths et valeurs { #math-and-values }

> [!NOTE]
> Ces entités viennent avec le pack d'entités Gameplay, voyez
> [Installer et personnaliser](README.md#installing-and-customizing).

Des entités point invisibles qui stockent, calculent et comparent des nombres, pour qu'un score, un compte ou un prix
puisse piloter la carte sans script. Elles se câblent comme toute entité. Un nombre voyage le long d'une connexion comme
valeur d'une sortie, comme `changed(value)` ou `result(value)`, et une entrée qui n'a pas de paramètre propre la reçoit,
voyez [Pas de paramètre](../parameters.md#no-parameter). Un paramètre saisi dans la connexion l'emporte sur la valeur.

Les nombres entiers restent entiers, `2 + 3` fait `5` et non `5.0`, et un nombre qui arrive sous forme de texte comme
`2.5` compte comme un nombre. Tout ce qui n'est pas un nombre compte comme du texte.

## math_value { #math_value }

Stocke un nombre ou un morceau de texte, comme le fait une variable. Les boutons y font un `add`, et `changed` apprend au
reste de la carte ce qu'il vaut maintenant. Il ne lance `changed` que si la valeur diffère de la précédente.

* **Entrées :** `set_value(value)`, `add(amount)` (1 par défaut), `get_value`, `reset`
* **Sorties :** `value(value)`, `changed(value)`
* **Clés :** `start_value` 0, un nombre ou du texte

`add` additionne deux nombres et concatène dès que l'un des deux est du texte, donc ajouter `!` à `hello` donne
`hello!`. `get_value` lance `value` avec ce qui est stocké, pour quand quelque chose a besoin du nombre sans attendre un
changement. `reset` revient à `start_value`. Un `set_value` sans valeur garde ce qui est stocké.

## math_calc { #math_calc }

Fait un calcul sur le nombre qu'il reçoit et lance la réponse. Il ne retient rien, donc la même entrée donne toujours la
même réponse. Enchaînez-en plusieurs pour calculer quelque chose par étapes.

* **Entrées :** `calculate(value)`, une valeur absente compte comme 0
* **Sorties :** `result(value)`
* **Clés :** `operation` add, `operand` 1

| `operation` | `result` |
| --- | --- |
| `add`, `subtract`, `multiply` | la valeur plus, moins ou fois `operand` |
| `divide` | la valeur divisée par `operand` |
| `modulo` | le reste de la division, jamais négatif pour un `operand` positif, donc `-1` modulo `3` fait `2` |
| `power` | la valeur élevée à la puissance `operand` |
| `min`, `max` | le plus petit ou le plus grand des deux |
| `abs`, `negate` | la valeur sans son signe, ou avec le signe inversé |
| `round`, `floor`, `ceil` | l'entier le plus proche (les demis s'éloignent de 0), arrondi vers le bas, arrondi vers le haut |

`abs`, `negate`, `round`, `floor` et `ceil` ignorent `operand`. Quand il n'y a pas de réponse, comme pour un `divide` ou
un `modulo` par 0 ou la racine d'un nombre négatif, rien ne se lance et Godot affiche un avertissement une seule fois.

## math_compare { #math_compare }

Compare une valeur stockée à `compare_value` et lance le côté où elle est tombée. `on_not_equal` se lance en même temps
que `on_less` et `on_greater`, pour quand seule une différence compte.

* **Entrées :** `set_value(value)`, `set_compare_value(value)`, `compare`, `set_and_compare(value)`
* **Sorties :** `on_less`, `on_equal`, `on_not_equal`, `on_greater`
* **Clés :** `compare_value` 0

`set_value` et `set_compare_value` ne font que stocker, `compare` compare ce qui est stocké, et `set_and_compare` fait les
deux, c'est donc l'entrée à relier à une sortie `changed(value)`. Les nombres se comparent par valeur, et `10` est plus
grand que `9`. Deux textes se comparent dans l'ordre du dictionnaire, les majuscules avant les minuscules. Des décimaux
presque égaux, comme `0.1 + 0.2` et `0.3`, comptent comme égaux.

## Exemple : un score qui ouvre une porte { #example-a-score-that-opens-a-door }

Trois boutons de pièces et un levier ouvrent la porte d'un coffre-fort dès que trois pièces au moins sont entrées.

1. Placez un `math_value` nommé `score`, un `math_compare` nommé `enough` avec `compare_value` 3, et un `logic_gate`
   nommé `door_gate` avec `gate` and.
2. Reliez le `pressed` de chaque bouton de pièce à `score`, entrée `add`. Chaque pression ajoute 1.
3. Reliez le `changed` de `score` à `enough`, entrée `set_and_compare`. Il lui transmet le nouveau score.
4. Reliez le `on_less` de `enough` à `door_gate`, entrée `set_a`, paramètre `false`. Reliez `on_equal` et `on_greater` de
   la même façon avec le paramètre `true`. Le `a` de la porte logique dit maintenant s'il y a assez de pièces.
5. Reliez le `pressed` du levier à `door_gate`, entrée `toggle_b`.
6. Reliez le `on_true` de `door_gate` à la porte avec l'entrée `open`, et `on_false` avec l'entrée `close`.

La porte s'ouvre quand la troisième pièce tombe alors que le levier est tiré, ou l'inverse. Remettre le levier en place
ou réinitialiser le score la ferme. Pour afficher le score, reliez le `changed` de `score` à un `game_text` avec l'entrée
`set_text`, ou intercalez un `math_calc` pour afficher des points au lieu de pièces.

La carte du terrain de jeu logique construit cette salle, et ajoute une loterie de lumières avec `logic_random` et
`logic_case` et un interrupteur de lampe avec `logic_flipflop`. Ouvrez `godot/demo/maps/logic_playground.tdm` dans le
projet de démo, ou lisez comment elle est faite dans
[`examples/mcp/logic_playground.json`](https://github.com/Paraxdev/TrenchDot/blob/main/examples/mcp/logic_playground.json).

<div class="td-mcp" markdown>

## MCP { #mcp }

Le script ci-dessus place les entités avec `gameplay place`, un appel chacune :

```json
{ "tool": "gameplay", "args": { "op": "place", "classname": "math_value", "origin": [-280, 40, 40],
  "properties": { "targetname": "score" },
  "outputs": [ { "output": "changed", "target": "enough", "input": "set_and_compare" } ] } }
{ "tool": "gameplay", "args": { "op": "place", "classname": "math_compare", "origin": [-280, 40, -40],
  "properties": { "targetname": "enough", "compare_value": "3" },
  "outputs": [ { "output": "on_greater", "target": "door_gate", "input": "set_a", "parameter": "true" } ] } }
```

</div>
