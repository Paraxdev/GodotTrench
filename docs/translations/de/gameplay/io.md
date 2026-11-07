---
source: 757483e9586cbd895b01c35597f84b4f54089ad0
---

# So funktioniert Entity-I/O { #how-entity-io-works }

Mit Entity-I/O lässt eine Karte Dinge passieren, ohne dass du Code schreibst, verdrahtet so, wie es Hammer für Source macht.

Jede Entität hat **Outputs**, Ereignisse, die sie meldet, etwa ein gedrückter Knopf, und **Inputs**, Dinge, die du ihr auftragen kannst, etwa eine Tür, die sich öffnet. Eine Verbindung sagt: Wenn dieser Output auslöst, rufe jenen Input an einem Ziel auf. Die Verdrahtung wird in der Karte gespeichert, du verbindest also im Editor einen Knopf mit einer Tür, und im Spiel öffnet sich die Tür.

## Eine Verbindung { #a-connection }

Ein Output kann beliebig viele Verbindungen haben. Jede hat diese Felder:

| Feld | Was es tut |
| --- | --- |
| Output | Das Ereignis dieser Entität, das die Verbindung startet, zum Beispiel `pressed` bei einem Knopf |
| Ziel | Welche Entitäten den Aufruf bekommen, meistens ein targetname, siehe [Ziele](parameters.md#targets) |
| Input | Was die Ziele tun sollen, zum Beispiel `open` bei einer Tür |
| Parameter | Ein optionaler Wert, der an den Input übergeben wird, etwa eine Schadenshöhe, siehe [Parameter](parameters.md#parameters) |
| Verzögerung | Sekunden, die vor dem Aufruf des Inputs gewartet wird, 0 ruft ihn sofort auf. Damit staffelst du Ereignisse, etwa Lichter, die nacheinander angehen |
| Mal | Wie oft die Verbindung auslösen darf. -1 heißt jedes Mal, 1 macht ein einmaliges Ereignis daraus |

Du bearbeitest sie im Inspektor unter **Ausgänge** der gewählten Entität oder als Leitungen im [Logikgraph](logic-graph.md). Beides ändert dieselben Outputs. (Der Inspektor hat für *Mal* nur ein Kästchen **einmal auslösen**, für jede andere Anzahl nutzt du das Verbindungsfeld im Logikgraph.)

![Der Abschnitt Ausgänge im Inspektor eines Knopfs: Output pressed, Ziel front_door, Input open, ein leerer Parameter, Verzögerung 0.00 s und das Kästchen einmal auslösen](../assets/io/inspector-outputs.png)

In Godot ist ein Output ein Signal und ein Input eine Methode: `func_button` hat `signal pressed(activator)` und `func_door` hat `func open()`. Deine eigenen Skripte machen genauso mit, siehe [Eigene Entitäten](custom-entities.md).

## Wenn ein Output auslöst { #when-an-output-fires }

Nach *Verzögerung* Sekunden wird der Input an jedem Node aufgerufen, den das Ziel findet. Findet ein Ziel nichts, passiert nichts. Bei einem targetname oder Node-Pfad ist das fast immer ein Tippfehler, deshalb gibt es beim ersten Auslösen eine Warnung aus. Ein verzögerter Aufruf wird verworfen, wenn seine Entität inzwischen den Baum verlassen hat, zum Beispiel weil sie getötet wurde.

Ohne Verzögerung läuft die ganze Kette innerhalb des `emit` des Signals, bevor der Code, der es ausgesendet hat, weitermacht. Das ist wichtig, wenn dein Skript direkt nach dem Aussenden einen Zustand liest, denn die Türen und Zähler weiter unten haben dann schon reagiert.

## Der Aktivator { #the-activator }

Der Aktivator ist, wer die Kette gestartet hat, meistens der Spieler, der den Knopf gedrückt hat oder in den Trigger gelaufen ist. Weiter unten in der Kette bedeutet `!activator` diesen Node, sodass ein Trigger genau den verletzen oder teleportieren kann, der ihn berührt hat.

Er stammt aus dem ersten `Node`-Argument des Signals, das ausgelöst hat, und überlebt deshalb nur, solange jedes Signal der Kette eins weiterreicht. Die eingebauten Signale verhalten sich so:

| Signale | Was mit dem Aktivator passiert |
| --- | --- |
| `triggered`, `pressed`, `entered`, `exited`, `hurt`, `teleported`, `pushed` | Unverändert weitergegeben, diese Signale tragen den Node, der sie verursacht hat |
| `arrived(corner)`, `spawned(node)`, `reached(train)` | Ersetzt durch den Node in Klammern, weiter unten ist `!activator` also dieser Pfadpunkt, gespawnte Node oder Zug |
| `opened`, `timer`, `step_N`, `on_true`, `on_case_N`, `result`, `broken`, `switched`, `map_spawn` | Verworfen. Diese tragen keinen Node, ab hier findet `!activator` nichts |

> [!TIP]
> Brauchst du den Spieler mehrere Schritte weiter unten in einer Kette, die den Aktivator verloren hat? Ziele stattdessen auf `!player`, das hängt nicht von der Kette ab.
