---
source: 12dfe8079d66dd10bb5ca7191aa90e4b3c01fe5b
---

# Geskriptete Zwischensequenz { #scripted-cutscene }

Der Spieler betritt einen Raum, ein Schild blendet sich ein, ein Führer läuft los, ein Fass explodiert, eine Lampe geht an und der Ausgang öffnet sich. Nichts davon braucht eigenen Code: Eine Timer-Entität löst pro Takt einen Output aus, und jeder Output ist mit der Entität verdrahtet, die handeln soll. Du findest sie als Karte `scripted_scene` im Demo-Projekt.

![Die Karte scripted_scene im Logik-Panel: start_zone löst cutscene.start aus, die step-Outputs von cutscene laufen zum Führer, zum Fass und zum Hinweis, und der broken-Output des Fasses läuft zu hp_readout. Die Lampe und die Ausgangstür liegen weiter unten, außerhalb des Bildes](../assets/cutscene/logic-graph.png)

## Der Zeitablauf { #the-timeline }

Alle Entitäten hier kommen mit dem Gameplay-Entitätenpaket. Installiere es zuerst, falls dein Projekt es noch nicht hat, mit *Gameplay > Gameplay-Entitäten installieren*.

1. Den Eingang abdecken

    Zeichne einen Brush über den Eingang und mache ihn zu einem `trigger_once` namens `start_zone` (Rechtsklick, *Brush-Entität erstellen > trigger_once*). Er löst `triggered` aus, wenn der Spieler zum ersten Mal hineingeht, und schaltet sich dann selbst ab, die Szene läuft also nur einmal.

2. Den Zeitablauf hinzufügen

    Wähle `logic_sequence` aus *Gameplay > Logik*, sie landet am 3D-Cursor. Nenne sie `cutscene`, setze `steps` auf 5 und `interval` auf 1.5. Nach dem Start löst sie `step_1`, `step_2` und so weiter aus, jeweils 1,5 Sekunden auseinander.

    ![Das Gameplay-Menü mit geöffnetem Logik und markiertem logic_sequence unter Am Cursor platzieren](../assets/cutscene/menu-logic-sequence.png)

3. Vom Trigger aus starten

    Verbinde den Output `triggered` von `start_zone` mit `cutscene.start`, mit *Gameplay > Logik > Zwei gewählte Entitäten verknüpfen…* wie in [Knopf öffnet eine Tür](button-door.md).

4. Jeden Takt verdrahten

    Verbinde jeden step-Output von `cutscene` mit der Entität, die in diesem Takt handelt, ein Paar nach dem anderen. Auch das Ziehen von einem Output-Pin zu einem Input-Pin im [Logik-Panel](../gameplay/logic-graph.md) funktioniert.

| Output | Ziel und Input | Was passiert |
| --- | --- | --- |
| `step_1` | `hint.show` | Ein `game_text` zeigt "Folge dem Führer" |
| `step_2` | `guide.start` | Ein `npc_walker` geht an seiner `path_corner`-Kette los, einer Reihe von Markern, die jeweils den nächsten benennen |
| `step_3` | `barrel.ignite` | Ein explosives `prop_physics` brennt seine Lunte ab, explodiert dann und zerbricht |
| `step_4` | `lamp.turn_on` | Die Lampe geht an |
| `step_5` | `exit_door.open` | Die Ausgangstür fährt nach oben |

Wähle `cutscene`, dann siehst du alle fünf im Inspektor unter *Ausgänge*.

![Der Inspektor der logic_sequence namens cutscene: steps 5, interval 1.5 und Ausgänge (I/O, 5), beginnend mit step_1 zu hint.show](../assets/cutscene/inspector-sequence.png)

Die Sequenz wartet vor jedem Schritt, auch vor dem ersten, Schritt 1 löst also bei 1,5 s aus und Schritt 5 bei 7,5 s. Für ungleichmäßige Abstände setze `times` auf eine Wartezeit pro Schritt, etwa `0.5 2 1 1 3`.

## Die Gesundheit des Spielers lesen { #reading-the-players-health }

Manche Reaktionen brauchen ein bisschen Logik, die kein eingebauter Input abdeckt. Ein `logic_script` führt beim Auslösen ein kurzes GDScript-Stück aus. Verbinde den Output `broken` des Fasses mit `hp_readout.run`, einem `logic_script`, dessen `source` die Gesundheit des Spielers in einen HUD-`game_text` namens `hpmsg` schreibt:

```gdscript
var players = io.find_targets(this, "!player", null)
if players.is_empty():
	return
var hp = int(players[0].health)
for label in io.find_targets(this, "hpmsg", null):
	label.set_text("HP " + str(hp))
	label.show()
```

Nimm hier `!player`, nicht `!activator`. `!activator` ist, wer die Kette angestoßen hat, und die step-Outputs und `broken` tragen niemanden mit. `!player` findet immer den Knoten in der Gruppe `player`. `broken` löst aus, nachdem die Explosion ihren Schaden angerichtet hat, die Gesundheit ist also schon aktuell.

## Abspielen { #playing-it }

Das Szenenskript
[godot/demo/scenes/scripted_scene.gd](https://github.com/Paraxdev/TrenchDot/blob/main/godot/demo/scenes/scripted_scene.gd)
baut nur die Karte, setzt den Spieler an den `info_player_start` und fügt ein `TrenchDotDebugOverlay` hinzu. Drücke im Spiel **F3**, um zuzusehen, wie jeder Output auslöst. So erkennst du am schnellsten einen Schritt, der auf den falschen Namen verdrahtet ist.

<div class="td-mcp" markdown>

## MCP { #mcp }

Die Demokarte entsteht, indem der Editor über den [MCP-Server](../mcp.md) gesteuert wird. Die Aufrufe stehen in
[examples/mcp/scripted_scene.json](https://github.com/Paraxdev/TrenchDot/blob/main/examples/mcp/scripted_scene.json).

</div>
