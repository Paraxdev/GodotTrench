---
source: c6245792b90999ca4b48812eb3d96e7a9c2389d6
---

# Mathe und Werte { #math-and-values }

> [!NOTE]
> Diese Entitäten gehören zum Gameplay-Entitätenpaket, siehe
> [Installieren und anpassen](README.md#installing-and-customizing).

Unsichtbare Punkt-Entitäten, die Zahlen speichern, berechnen und vergleichen, sodass ein Punktestand, eine Anzahl oder ein Preis die Karte ohne Skript steuern kann. Sie werden verdrahtet wie jede Entität. Eine Zahl wandert über eine Verbindung als Wert eines Outputs, etwa `changed(value)` oder `result(value)`, und ein Input ohne eigenen Parameter empfängt sie, siehe [Kein Parameter](../parameters.md#no-parameter). Ein Parameter, den du in die Verbindung tippst, gewinnt gegen den Wert.

Ganze Zahlen bleiben ganz, `2 + 3` ist `5` und nicht `5.0`, und eine Zahl, die als Text wie `2.5` ankommt, zählt als Zahl. Alles, was keine Zahl ist, zählt als Text.

## math_value { #math_value }

Speichert eine Zahl oder einen Text, so wie es eine Variable tut. Knöpfe können mit `add` darauf addieren, und `changed` sagt dem Rest der Karte, was jetzt gespeichert ist. Es löst `changed` nur aus, wenn sich der Wert vom vorherigen unterscheidet.

* **Inputs:** `set_value(value)`, `add(amount)` (standardmäßig 1), `get_value`, `reset`
* **Outputs:** `value(value)`, `changed(value)`
* **Schlüssel:** `start_value` 0, eine Zahl oder ein Text

`add` addiert zwei Zahlen und hängt an, sobald einer der beiden Text ist, `!` zu `hello` hinzuzufügen ergibt also `hello!`. `get_value` löst `value` mit dem gespeicherten Wert aus, falls etwas die Zahl braucht, ohne auf eine Änderung zu warten. `reset` geht zurück auf `start_value`. Ein `set_value` ohne Wert behält, was gespeichert ist.

## math_calc { #math_calc }

Führt eine Rechnung mit der empfangenen Zahl aus und löst die Antwort aus. Es speichert nichts, derselbe Input liefert also immer dieselbe Antwort. Verkette mehrere, um etwas in Schritten auszurechnen.

* **Inputs:** `calculate(value)`, ein fehlender Wert zählt als 0
* **Outputs:** `result(value)`
* **Schlüssel:** `operation` add, `operand` 1

| `operation` | `result` |
| --- | --- |
| `add`, `subtract`, `multiply` | der Wert plus, minus oder mal `operand` |
| `divide` | der Wert geteilt durch `operand` |
| `modulo` | der Rest der Division, bei positivem `operand` nie negativ, `-1` modulo `3` ist also `2` |
| `power` | der Wert hoch `operand` |
| `min`, `max` | die kleinere oder die größere der beiden Zahlen |
| `abs`, `negate` | der Wert ohne Vorzeichen, oder mit umgedrehtem Vorzeichen |
| `round`, `floor`, `ceil` | die nächste ganze Zahl (Hälften gehen weg von 0), abgerundet, aufgerundet |

`abs`, `negate`, `round`, `floor` und `ceil` ignorieren `operand`. Gibt es keine Antwort, etwa bei `divide` oder `modulo` durch 0 oder bei der Wurzel einer negativen Zahl, löst nichts aus, und Godot gibt einmal eine Warnung aus.

## math_compare { #math_compare }

Vergleicht einen gespeicherten Wert mit `compare_value` und löst aus, auf welcher Seite er gelandet ist. `on_not_equal` löst neben `on_less` und `on_greater` aus, für den Fall, dass nur ein Unterschied zählt.

* **Inputs:** `set_value(value)`, `set_compare_value(value)`, `compare`, `set_and_compare(value)`
* **Outputs:** `on_less`, `on_equal`, `on_not_equal`, `on_greater`
* **Schlüssel:** `compare_value` 0

`set_value` und `set_compare_value` speichern nur, `compare` vergleicht das Gespeicherte, und `set_and_compare` macht beides, es ist also der Input, den du mit einem Output `changed(value)` verdrahtest. Zahlen werden nach Wert verglichen, `10` ist also mehr als `9`. Zwei Texte werden in Wörterbuchreihenfolge verglichen, wobei Großbuchstaben vor Kleinbuchstaben kommen. Fast gleiche Dezimalzahlen wie `0.1 + 0.2` und `0.3` gelten als gleich.

## Beispiel: Ein Punktestand, der eine Tür öffnet { #example-a-score-that-opens-a-door }

Drei Münzknöpfe und ein Hebel öffnen eine Tresortür, sobald mindestens drei Münzen drin sind.

1. Platziere ein `math_value` mit dem Namen `score`, ein `math_compare` mit dem Namen `enough` und `compare_value` 3 sowie ein `logic_gate`
    mit dem Namen `door_gate` und `gate` and.
2. Verdrahte `pressed` jedes Münzknopfs mit `score`, Input `add`. Jeder Druck addiert 1.
3. Verdrahte `changed` von `score` mit `enough`, Input `set_and_compare`. Es gibt den neuen Punktestand weiter.
4. Verdrahte `on_less` von `enough` mit `door_gate`, Input `set_a`, Parameter `false`. Verdrahte `on_equal` und `on_greater` genauso
    mit Parameter `true`. Das `a` des Gatters sagt jetzt, ob genug Münzen da sind.
5. Verdrahte `pressed` des Hebels mit `door_gate`, Input `toggle_b`.
6. Verdrahte `on_true` von `door_gate` mit der Tür und dem Input `open`, und `on_false` mit dem Input `close`.

Die Tür öffnet sich, wenn die dritte Münze fällt, während der Hebel gezogen ist, oder umgekehrt. Ziehst du den Hebel zurück oder setzt den Punktestand zurück, schließt sie sich. Um den Punktestand anzuzeigen, verdrahte `changed` von `score` mit einem `game_text` mit dem Input `set_text`, oder setze ein `math_calc` dazwischen, um Punkte statt Münzen anzuzeigen.

Die Karte Logic Playground baut diese Halle und fügt eine Lichtlotterie mit `logic_random` und `logic_case` sowie einen Lampenschalter mit `logic_flipflop` hinzu. Öffne `godot/demo/maps/logic_playground.tdm` im Demoprojekt oder lies in [`examples/mcp/logic_playground.json`](https://github.com/Paraxdev/TrenchDot/blob/main/examples/mcp/logic_playground.json) nach, wie sie gemacht ist.

<div class="td-mcp" markdown>

## MCP { #mcp }

Das Skript oben platziert die Entitäten mit `gameplay place`, ein Aufruf pro Entität:

```json
{ "tool": "gameplay", "args": { "op": "place", "classname": "math_value", "origin": [-280, 40, 40],
  "properties": { "targetname": "score" },
  "outputs": [ { "output": "changed", "target": "enough", "input": "set_and_compare" } ] } }
{ "tool": "gameplay", "args": { "op": "place", "classname": "math_compare", "origin": [-280, 40, -40],
  "properties": { "targetname": "enough", "compare_value": "3" },
  "outputs": [ { "output": "on_greater", "target": "door_gate", "input": "set_a", "parameter": "true" } ] } }
```

</div>
