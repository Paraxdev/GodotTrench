---
source: 2bf109094dcc42ebb11cf6c736e210c0e4e79b02
---

# Skripte und Aufrufe { #scripts-and-calls }

> [!NOTE]
> Diese Entitäten gehören zum Gameplay-Entitätenpaket, siehe
> [Installieren und anpassen](README.md#installing-and-customizing).

Punkt-Entitäten für alles, was einfache Verdrahtung nicht erreicht: Spielzustand, dein eigener Spielcode, Animationen und die Frage, warum eine Kette nicht auslöst.

## logic_script { #logic_script }

Führt beim Auslösen ein GDScript-Snippet aus, für ein bisschen Logik, die keine eingebaute Entität bietet, etwa die Gesundheit des Spielers zu lesen, ohne eine eigene Entität zu schreiben.

* **Inputs:** `run(activator)`, `run_with(parameter)`
* **Outputs:** `ran(result)`, mit dem Rückgabewert des Snippets
* **Schlüssel:** `source` leer, `expression` leer (ein Einzeiler, wird genutzt, wenn `source` leer ist)

Das Snippet sieht `this` (die Entität selbst), `activator` (wer die Kette gestartet hat), `parameter` (den eingehenden Wert), `io` (die Helfer von `TrenchDotIO`, zum Beispiel um Entitäten nach Namen zu finden) und `tree` (den `SceneTree`). `run_with` hat immer einen leeren `activator`. Ein Kompilierfehler wird beim Laden der Karte als Warnung protokolliert. Dieses Snippet fügt dem Spieler Schaden zu:

```gdscript
var p = io.find_targets(this, "!player", null)
if not p.is_empty():
	p[0].take_damage(25, this)
```

> [!WARNING]
> Rücke verschachtelte Zeilen mit Tabs ein. Mit Leerzeichen lässt sich das Snippet nicht kompilieren, es gibt einen Fehler wegen gemischter Einrückung.

> [!WARNING]
> Das Snippet ist Code, der in der Karte mitkommt. Er läuft, sobald das Spiel läuft, mit denselben Rechten wie die eigenen Skripte des Projekts. Behandle eine heruntergeladene Karte mit Code also wie jedes Programm, das du herunterlädst: Sie kann alles auf deinem Computer tun. Das Panel Probleme warnt vor jedem `logic_script` mit Code, vor jedem Output, der einem welchen Code übergeben kann, und vor jedem Prefab mit einem von beiden. Lies das, bevor du die Karte spielst.

## logic_call { #logic_call }

Ruft eine Methode an einem beliebigen Ziel mit festen Argumenten auf, um deinen Spielcode zu erreichen, etwa um Punkte auf einem Autoload `Game` zu addieren. Anders als ein einfacher Output warnt es, wenn das Ziel oder die Methode nicht existiert, sodass Tippfehler im Log auftauchen. C#-Methoden funktionieren unter ihren PascalCase-Namen.

* **Inputs:** `trigger(activator)`, `call_with(parameter)`
* **Outputs:** `called(result)`
* **Schlüssel:** `call_target` `/root/Game`, `method` leer, `arguments` `[]`

`call_target` ist ein Node-Pfad, eine `@group`, ein targetname, `!activator` oder `!self`. `arguments` ist ein JSON-Array wie `["key_red", "$activator"]` und übernimmt die Platzhalter aus [Parameter](../parameters.md), aufgelöst am `logic_call` selbst. `call_with` setzt seinen Parameter dort ein, wo `"$parameter"` steht, oder übergibt ihn als einziges Argument.

## logic_animate { #logic_animate }

Spielt Animationen am ersten `AnimationPlayer` bei oder unter `target` ab, für Figuren und Props, die ihre eigenen Animationen mitbringen. Setze `target` auf `!activator`, um den zu animieren, der die Kette gestartet hat.

* **Inputs:** `play(name)`, `stop`, `queue(name)`, `seek(time)`
* **Outputs:** `finished(anim)`
* **Schlüssel:** `target` `!self`, `animation` leer (wird abgespielt, wenn `play` keinen Namen hat), `speed` 1.0

## logic_debug { #logic_debug }

Gibt bei `write` eine Zeile aus und zeigt seine letzten Zeilen in Debug-Builds als schwebendes Label an, damit du im Spiel siehst, ob eine Kette es erreicht und mit welchem Parameter. Mit eingeschaltetem `trace_all` protokolliert es jeden Output im Spiel, ein schneller Weg, eine ganze Kette zu beobachten. Siehe [Debugging](../debugging.md).

* **Inputs:** `write(parameter)`, `toggle` (das Label), `show`, `hide`
* **Outputs:** `printed(text)`
* **Schlüssel:** `message` `{name} received {parameter} from {activator}` (`{time}` geht auch), `show_label` 1,
  `trace_all` 0
