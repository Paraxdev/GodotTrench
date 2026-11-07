---
source: 425ace012cd6d8410cbf64a89bb7dc31275f8980
---

# Verdrahtung debuggen { #debugging-wiring }

Die meisten Verdrahtungsfehler scheitern still: Die Tür bleibt einfach zu. Prüfe in dieser Reihenfolge, von der schnellsten bis zur gründlichsten Prüfung.

> [!TIP]
> Eine Tür oder ein Trigger, der den Spieler nicht einmal blockiert, wurde ohne seine Definition gebaut, das ist kein Verdrahtungsfehler. Siehe [Entitäten, die nichts tun](../godot/entities-troubleshooting.md).

## 1. Das Panel Probleme { #1-the-issues-panel }

Öffne den Tab **Probleme**, links neben Outliner und Verlauf. Er markiert Outputs, die auf einen targetname zielen, den keine Entität hat, und Inputs, die die Klasse des Ziels nicht hat. Ein Tippfehler in einem targetname ist der häufigste Grund, warum eine Tür zubleibt.

![Das Panel Probleme mit einem Fehler: Output triggered zielt auf unbekannte Entität 'hall_lihgt' an trigger_once, mit einer Schaltfläche Beheben](../assets/debugging/issues-panel.png)

Klicke ein Problem an, um seine Entität zu wählen und in den Ansichten zu zentrieren, und korrigiere dann den Namen im Inspektor.

> [!WARNING]
> **Beheben** bei einem fehlenden Ziel rät nicht den richtigen Namen, es löscht die Verbindung. Bei einem Tippfehler korrigierst du den Namen lieber von Hand.

## 2. Der Logikgraph { #2-the-logic-graph }

Im [Logikgraph](logic-graph.md) erscheint ein Ziel, das niemand hat, als roter Node mit einer gestrichelten roten Leitung, die sagt, dass keine Entität diesen Namen hat.

![Der Logikgraph: der Output triggered von trigger_once läuft über eine gestrichelte rote Leitung zu einem roten Node hall_lihgt, keine Entität hat diesen Namen](../assets/debugging/graph-missing-target.png)

**Simulieren** löst einen Output auf dem Papier aus und hebt die ganze Kette hervor, siehe [Simulieren](logic-graph.md#simulate).

## 3. F3 im Spiel { #3-f3-in-game }

Füge deiner Szene in Godot einen `TrenchDotDebugOverlay`-Node hinzu oder als Autoload. Starte das Spiel und drücke F3: Es listet Outputs, während sie auslösen, und zeichnet jedes `Area3D`-Volumen mit seinem targetname, sodass du siehst, ob ein Trigger nie ausgelöst hat oder ins Leere.

Schalte es nach dem Spawnen von Triggern noch einmal um, Volumen werden beim Einschalten gesammelt. Die Taste ist die Eigenschaft `toggle_key` des Overlays.

## 4. Logging { #4-logging }

Platziere eine `logic_debug`-Entität und hake im Inspektor ihr Kästchen `trace_all` an. Jeder Output, der auslöst, wird dann in die Godot-Ausgabe geschrieben.

Verbinde dich im Code mit `TrenchDotIO.events().fired`, das für jeden Output ausgesendet wird, auch für solche, deren Ziel auf nichts aufgelöst wurde.

```gdscript
TrenchDotIO.events().fired.connect(func(source, output, target, input, parameter):
	print(source.name, ".", output, " -> ", target, ".", input))
```

## Wie jeder Fehler aussieht { #what-each-mistake-looks-like }

| Fehler | Symptom | Lösung |
| --- | --- | --- |
| Das Ziel löst auf nichts auf | `[TD I/O] <node>.<output>: target '<name>' matches no node` beim ersten Auslösen | Lass das Ziel genau zum targetname der Entität passen, Groß- und Kleinschreibung eingeschlossen |
| Das Ziel hat keinen solchen Input | `[TD I/O] <node> has no input '<name>'` in der Godot-Ausgabe | Wähle einen Input, den die Klasse des Ziels hat, siehe die [Entitätsreferenz](entities/README.md) |
| Output, den die Entität nie aussendet | `[TD I/O] <node> has no signal '<output>'` einmal, wenn die Karte gebaut wird | Wähle einen Output, den die Klasse hat, oder löse ihn aus dem Code mit `TrenchDotIO.fire_output()` aus |
| Zur Laufzeit gespawnte oder umbenannte Entität | Wird über ihren targetname nicht gefunden | Rufe `TrenchDotIO.invalidate()` auf, siehe [Änderungen zur Laufzeit](parameters.md#runtime-changes) |
