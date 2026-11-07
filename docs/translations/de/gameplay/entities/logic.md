---
source: 5168ac40e48ba63850a1be1795dae420bfe6bf5f
---

# Logik { #logic }

> [!NOTE]
> Diese Entitäten gehören zum Gameplay-Entitätenpaket, siehe
> [Installieren und anpassen](README.md#installing-and-customizing).

Unsichtbare Punkt-Entitäten, die entscheiden, wann etwas passiert. Sie sitzen zwischen einem Trigger oder Knopf und dem, was er beeinflusst, und formen das Signal auf dem Weg dorthin. Entitäten, die mit Zahlen arbeiten, stehen unter [Mathe und Werte](math.md), und die, die Code oder Animationen ausführen, unter [Skripte und Aufrufe](scripting.md).

## logic_relay { #logic_relay }

Gibt `trigger` als `triggered` weiter, solange es aktiviert ist. Verdrahte mehrere Dinge mit einem Relay und schalte es dann mit `disable` ab, um die ganze Kette auf einmal auszuschalten.

* **Inputs:** `trigger(activator)`, `enable`, `disable`, `toggle`
* **Outputs:** `triggered(activator)`
* **Schlüssel:** `start_disabled` 0

## logic_branch { #logic_branch }

Merkt sich wahr oder falsch und löst bei `test` entweder `on_true` oder `on_false` aus, für Prüfungen wie "die Tür öffnet sich nur, wenn der Strom an ist". Der Generator setzt den Wert auf wahr, der Türknopf prüft ihn.

* **Inputs:** `set_true`, `set_false`, `toggle`, `test`, `set_and_test(value)`
* **Outputs:** `on_true`, `on_false`
* **Schlüssel:** `start_value` 0

## logic_gate { #logic_gate }

Hält zwei Wahr-oder-falsch-Werte, `a` und `b`, und verknüpft sie mit einem Gatter, für Prüfungen wie "die Tür öffnet sich, wenn der Strom an **und** der Hebel gezogen ist". Jede Änderung an einem Input, die das Ergebnis umkehrt, löst `changed(on)` und dann `on_true` oder `on_false` aus, niemand muss also nachfragen. `test` löst das aktuelle Ergebnis noch einmal aus, falls etwas anderes es gerade wissen muss.

* **Inputs:** `set_a(value)`, `set_b(value)`, `toggle_a`, `toggle_b`, `test`
* **Outputs:** `on_true`, `on_false`, `changed(on)`
* **Schlüssel:** `gate` and, `start_a` 0, `start_b` 0

| `gate` | Wahr, wenn |
| --- | --- |
| `and` | beide wahr sind |
| `or` | mindestens einer wahr ist |
| `xor` | genau einer wahr ist |
| `nand` | nicht beide wahr sind |
| `nor` | keiner wahr ist |
| `xnor` | beide gleich sind |
| `not` | `a` falsch ist, `b` wird ignoriert |

`set_a` und `set_b` ohne Wert setzen auf wahr, ein Output `entered` eines Triggers, der mit `set_a` verdrahtet ist, schaltet `a` also ein. Eine Zahl ist wahr, außer sie ist 0, und Text ist nur bei `true`, `yes` oder `1` wahr. Die Startwerte lösen nichts aus, und eine Änderung, die das Ergebnis unverändert lässt, ebenfalls nicht.

## logic_flipflop { #logic_flipflop }

Wechselt bei jedem Auslösen, sodass ein Knopf etwas ein- und wieder ausschalten kann. `on_a` löst aus, wenn es einschaltet, und `on_b`, wenn es ausschaltet. `reset` setzt es auf `start_on` zurück, ohne eines von beiden auszulösen, und löst nur `changed` aus, wenn sich dadurch etwas geändert hat.

* **Inputs:** `trigger`, `reset`
* **Outputs:** `on_a`, `on_b`, `changed(on)`
* **Schlüssel:** `start_on` 0 (an, sodass das erste Auslösen `on_b` auslöst)

## logic_case { #logic_case }

Vergleicht den empfangenen Wert mit bis zu acht Fällen und löst den Output des ersten passenden aus, oder `on_default`, wenn keiner passt. So wird eine Zahl, etwa der Index eines `logic_random` oder der Punktestand eines `math_value`, zu unterschiedlichen Ereignissen.

* **Inputs:** `in_value(value)`
* **Outputs:** `on_case_1` bis `on_case_8`, `on_default`
* **Schlüssel:** `case_1` bis `case_8`, leer

Zahlen werden nach Wert verglichen, der Fall `2` passt also auf den Wert 2 und auf `2.0`. Text muss genau übereinstimmen, `Red` ist nicht `red`, und Leerzeichen um einen Fall herum werden ignoriert. Leere Fälle werden übersprungen, der erste Treffer gewinnt, und ein fehlender Wert geht an `on_default`.

## logic_random { #logic_random }

Wählt zufällig einen von bis zu acht Outputs oder würfelt eine Wahrscheinlichkeit. Verdrahte `out_1` bis `out_3` mit drei Lichtern, dann schaltet `pick` eines davon ein. `roll` entscheidet stattdessen nach Wahrscheinlichkeit, zum Beispiel ob eine Truhe Beute enthält.

* **Inputs:** `pick`, `roll`
* **Outputs:** `out_1` bis `out_8`, `picked(index)`, `on_success`, `on_fail`
* **Schlüssel:** `count` 3 (1 bis 8), `no_repeat` 0, `chance` 50 (Prozent)

`pick` löst `picked` mit der Nummer des gewählten Outputs aus und dann diesen Output, `picked` kann also einen [`logic_case`](#logic_case) speisen. Mit `no_repeat` wählt es nie den Output, den es beim letzten Mal gewählt hat, außer `count` ist 1. `roll` löst `on_success` mit einer Wahrscheinlichkeit von `chance` Prozent aus und sonst `on_fail`, bei 0 gelingt es also nie und bei 100 immer.

## logic_counter { #logic_counter }

Zählt zwischen `min` und `max`, für Rätsel wie "drücke alle drei Knöpfe": Jeder Knopf sendet `add`, und `hit_max` öffnet die Tür. Der Wert wird begrenzt, und nichts löst aus, wenn er sich nicht ändert. `hit_max` und `hit_min` lösen aus, wenn eine Änderung auf einer Grenze landet, `reset` löst nur `changed` aus.

* **Inputs:** `add(amount)` (standardmäßig 1), `subtract(amount)`, `set_value(value)`, `reset`
* **Outputs:** `hit_max`, `hit_min`, `changed(value)`
* **Schlüssel:** `min` 0, `max` 3, `start_value` 0

## logic_timer { #logic_timer }

Löst `timer` immer wieder aus, für alles, was sich wiederholt, etwa ein flackerndes Licht oder Gegnerwellen. Es wartet zwischen den Auslösungen `interval` Sekunden und läuft ab dem Laden der Karte, außer `start_on` ist 0. Ist `random_max` größer als `random_min`, ist jede Wartezeit zufällig in diesem Bereich, damit die Wiederholungen weniger mechanisch wirken. `once` stoppt es nach der ersten Auslösung, wodurch es zu einer einfachen Verzögerung wird.

* **Inputs:** `start`, `stop`, `toggle`, `fire` (einmal, sofort)
* **Outputs:** `timer`
* **Schlüssel:** `interval` 1.0, `random_min` 0, `random_max` 0, `start_on` 1, `once` 0

## logic_auto { #logic_auto }

Löst `map_spawn` einmal aus, nachdem alle Entitäten der Karte bereit sind. Verdrahte hier alles, was von Anfang an laufen soll, etwa Musik oder einen Introtext.

* **Outputs:** `map_spawn`

## logic_sequence { #logic_sequence }

Eine Zeitleiste für Zwischensequenzen. Sie löst bis zu acht nummerierte Schritte der Reihe nach aus und wartet vor jedem, auch vor dem ersten. Verdrahte jeden Schritt mit dem, was in diesem Moment passiert, einer Tür, die sich öffnet, einer Textzeile, einer Figur, die losläuft.

* **Inputs:** `start`, `stop`, `reset`
* **Outputs:** `step_1` bis `step_8`, `step(index)`, `finished`
* **Schlüssel:** `steps` 3 (1 bis 8), `interval` 1.0, `times` leer, `start_active` 0 (läuft ab dem Laden der Karte), `loop` 0

`times` listet eine Wartezeit pro Schritt auf, etwa `0 1 0.5`, und Schritte ohne Angabe nutzen `interval`. Eine Sequenz mit Schleife löst nie `finished` aus. `start` wird ignoriert, solange sie läuft, und nach `stop` beginnt sie wieder bei Schritt 1. Siehe das [Zwischensequenz-Tutorial](../../tutorials/cutscene.md).
