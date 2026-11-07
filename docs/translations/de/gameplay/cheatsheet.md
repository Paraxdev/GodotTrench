---
source: 414dc5a3711ba57511bd4ecc18dc19f159af44d7
---

# I/O-Spickzettel { #io-cheatsheet }

Eine einseitige Zusammenfassung von [So funktioniert Entity-I/O](io.md) und [Ziele, Parameter und Inputs](parameters.md), für den Fall, dass du die Idee kennst und nur die Details brauchst.

## Wenn ein Output auslöst { #when-an-output-fires }

Das Signal löst aus, zählt gegen *Mal*, wartet *Verzögerung* und ruft dann den Input an jedem Node auf, den das Ziel findet. Findet es keinen, passiert nichts, und ein targetname oder Node-Pfad, der nichts findet, warnt einmal.

## Ziele { #targets }

| Schreibe | Erreicht |
| --- | --- |
| `main_door` | Jede Entität mit diesem targetname, in derselben Karte |
| `door_*` | Jeden targetname, der mit `door_` beginnt |
| `@enemies` | Jeden Node in dieser Godot-Gruppe, irgendwo im Baum |
| `!player` | Den ersten Node in der Gruppe `player` |
| `!activator` | Wer die Kette gestartet hat, `!caller` bedeutet dasselbe |
| `!self` | Die Entität, der der Output gehört |
| `/root/Game` | Den Node unter diesem absoluten Pfad |

## Parameter { #parameters }

Ein einfacher Wert wird als int, float, bool und dann `x y z` als `Vector3` versucht, sonst bleibt er ein String. Ein JSON-`[a, b]` wird auf mehrere Argumente verteilt. `$activator`, `$self`, `$position` und `$caller_name` werden an der **empfangenden** Entität eingesetzt.

Ohne Parameter geht der Aktivator in das erste Objekt-Argument, und andere Werte, die der Output mitgebracht hat, füllen den Rest der Reihe nach.

## Wer den Aktivator behält { #who-keeps-the-activator }

| Signale | Aktivator |
| --- | --- |
| `triggered`, `pressed`, `entered`, `exited`, `hurt`, `teleported`, `pushed` | Unverändert weitergegeben |
| `arrived(corner)`, `spawned(node)`, `reached(train)` | Ersetzt durch den Node in Klammern |
| `opened`, `timer`, `step_N`, `on_true`, `on_case_N`, `result`, `broken`, `switched`, `map_spawn` | Verworfen |

Hat die Kette ihn verloren und du brauchst den Spieler, ziele auf `!player`.

## Inputs, die jeder Node hat { #inputs-every-node-has }

`kill`, `show`, `hide`, `enable`, `disable` und `toggle` funktionieren an jedem Node, solange die Entität keine Methode dieses Namens hat. Ein Input, der zu einer Eigenschaft passt, setzt sie auf den Parameter.

## Geltungsbereich von logic_script { #logic_script-scope }

Ein [`logic_script`](entities/scripting.md) führt beim Auslösen ein GDScript-Snippet aus. Darin ist `this` das logic_script selbst, `activator` der Node, der die Kette gestartet hat, `parameter` der übergebene Wert, `io` die Helfer von `TrenchDotIO` und `tree` der `SceneTree`. Rücke mit Tabs ein.

```gdscript
var p = io.find_targets(this, "!player", null)
if not p.is_empty():
	p[0].take_damage(25, this)
```

## Nichts passiert? { #nothing-happened }

Prüfe das Panel Probleme, dann den Logikgraph, dann F3 im Spiel, dann logge `TrenchDotIO.events()`. Siehe [Verdrahtung debuggen](debugging.md).
