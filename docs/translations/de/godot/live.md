---
source: 15ac2f37b7d278fa849fb066b91734f671f35658
---

# Speichern, Live-Modus und Hot Reload { #saving-live-mode-and-hot-reload }

## Neu bauen beim Speichern { #rebuild-on-save }

Lass die Szene mit deiner `FuncGodotMap` im Godot-Editor offen und speichere in TrenchDot einfach mit Ctrl+S. Godot baut diese Szene sofort neu, solange ihr Tab der ist, den Godot zeigt, und der Karten-Node *Auto Rebuild On Save* an hat. Eine Szene in einem anderen Tab, die die Karte nutzt, wird neu gebaut, wenn du zu ihrem Tab wechselst. Einen Tab zu schließen und wieder zu öffnen oder eine Szene neu zu laden, stört das nicht. Wenn du ein [Prefab](../editor/prefab-library.md) speicherst, werden die Karten dieser Szene neu gebaut, die es nutzen, direkt oder über ein anderes Prefab.

Die Statusleiste unten links sagt dir, was Godot getan hat. Hat es geklappt, sollte dort stehen:

![Die Statusleiste mit "Live-Link: Godot hat 1 Karten-Node(s) neu gebaut"](../assets/live/status-rebuilt.png)

Speichere die Szene in Godot, um das Ergebnis zu behalten. Steht in der Statusleiste etwas anderes, sagt dir die Meldung, warum:

![Die Statusleiste mit der Meldung, dass room.tscn room.tdm nutzt, aber nicht der aktuelle Szenen-Tab in Godot ist und beim Wechsel dorthin neu gebaut wird](../assets/live/status-not-current-tab.png)

| Die Statusleiste sagt | Tu das |
| --- | --- |
| Die Szene ist nicht der aktuelle Szenen-Tab in Godot | Wechsle zu diesem Tab, dann wird sie neu gebaut |
| Der aktuelle Szenen-Tab hat keine `FuncGodotMap` für die Karte | Öffne die Szene, die die Karte baut, oder schalte *Auto Rebuild On Save* an ihrem Karten-Node ein |
| Godot hat keine Szene offen | Öffne die Szene mit der Karte in Godot, und ist sie offen, lies weiter |
| Godot läuft nicht mit dem TrenchDot-Addon | Öffne das Projekt im Godot-Editor mit aktiviertem Addon |
| Godot hat die Karte neu gebaut, aber die Karteneinstellungen haben für manche Klassen keine Definition, sie werden also als einfache Nodes gebaut | Füg die Definitionen hinzu, siehe [Entitäten, die nichts tun](entities-troubleshooting.md) |
| Godot hat die Karte mit Warnungen neu gebaut, siehe Bereich Probleme | Lies sie im Bereich Probleme, wo sie mit "Godot:" beginnen |

Nur ein Godot-Editor kann den Port des Live Links halten. Ein zweiter Editor mit demselben Projekt warnt in seinem Bereich Output, dass der Live Link nicht lauschen konnte, und TrenchDot redet weiter mit dem ersten, der vielleicht keine Szene offen hat. Schließ den Editor, den du nicht brauchst, und der andere übernimmt den Link innerhalb weniger Sekunden. (Hinweis: *Godot > Projekt im Godot-Editor öffnen* holt einen Editor, der das Projekt schon hat, nach vorn, statt einen zweiten zu starten.)

## Live-Modus { #live-mode }

Der Live-Modus schiebt deine Änderungen in die im Godot-Editor offene Szene, bevor du speicherst, damit du Beleuchtung und Maßstab im echten Renderer beurteilen kannst.

1. Die Szene in Godot öffnen

    Die Karte muss einmal im Projekt gespeichert sein, und die in Godot offene Szene braucht eine `FuncGodotMap`, die sie nutzt.

2. Einschalten

    Klick auf den Link-Knopf am rechten Ende der Werkzeugleiste neben dem Godot-Roboter, oder nutze *Godot > Live-Modus*. Der Knopf leuchtet auf, und sein Tooltip sagt, worauf er noch wartet, falls etwas fehlt.

    ![Die Werkzeugleiste mit eingeschaltetem Live-Modus-Knopf und seinem Tooltip, der sagt, dass Änderungen Godot erreichen, bevor du speicherst](../assets/live/toolbar-live-mode-on.png)

3. Wie gewohnt bearbeiten

    Jede Änderung erscheint sofort in Godot.

    ![Änderungen in TrenchDot, die im Godot-Editor erscheinen, bevor gespeichert wird](../screenshots/TrenchDot-demo-live-view.gif)

Der Live-Modus braucht den Godot Live Link, der standardmäßig an ist. Beide stehen unter *Datei > Einstellungen*, wo *Godot Live-Modus* derselbe Schalter ist wie der Knopf in der Werkzeugleiste.

![Die Zeilen Godot Live Link, Godot Live-Modus und Hot Reload im laufenden Spiel in den Einstellungen](../assets/live/prefs-live-link.png)

Beim Ziehen bewegen sich Nodes mit, und die meisten Änderungen bauen nur neu, was sie berührt haben, damit die Vorschau mithält. *Omit From Export* an einer Ebene umzuschalten und Prefab-Instanzen zu bearbeiten baut stattdessen die ganze Karte neu, sobald du pausierst.

Die Vorschau kommt einem vollen Build nah, ist aber nicht immer identisch. Speichern macht immer einen vollen Build, und beim Beenden ohne Speichern schickt TrenchDot Godot zurück zur gespeicherten Datei, nichts, was du im Live-Modus ausprobiert hast, bleibt also, wenn du es nicht speicherst.

### Wann der Live-Modus die ganze Karte neu baut { #when-live-mode-rebuilds-the-whole-map }

Godot baut die ganze Karte von selbst neu, wenn die Szene nicht mehr dem entspricht, was ein voller Build ergäbe. Startet der Live-Modus auf einer Szene, geschieht das sofort, sonst, sobald du das Bearbeiten pausierst:

| Wann | Warum |
| --- | --- |
| Der Live-Modus startet auf einer Szene, die mit anderen Entitätsdefinitionen gebaut wurde | Die Szene wurde gebaut, bevor du das Gameplay-Entitätenpaket installiert oder eine Definition hinzugefügt hast, manche Entitäten sind also einfache Nodes |
| Der Live-Modus startet auf einer Szene, die frühere Live-Änderungen verändert haben | Die Szene enthält noch Live-Änderungen einer früheren Sitzung, die du nie gespeichert hast |
| Die Entitätsdefinitionen ändern sich, während der Live-Modus läuft | Du hast eine FGD oder eine Definition im Inspector von Godot gespeichert, eine Skriptklasse hat sich geändert, oder neue Dateien sind angekommen, etwa ein installiertes Paket |
| Du speicherst ein Prefab, das die Karte nutzt | Die Szene zeigt das Prefab noch so, wie es war |

Offene Szenen außerhalb des Live-Modus behalten ihre Entitäten, wie sie gebaut wurden, und der Bereich Output von Godot warnt, wenn eine davon mit anderen Entitätsdefinitionen gebaut wurde. Drück darin **Build Map** und speichere.

### Entitäten ohne Definition { #entities-without-a-definition }

Baut der Live-Modus eine Entität, deren Klasse Godot nicht definiert hat, nennt die Statusleiste die Klasse und die FGD, in die sie gehört:

![Die Statusleiste mit der Meldung, dass Godot keine Definition für crypt_lamp hat, einen einfachen Node zeigt und die Klasse in die FGD von res://crypt_settings.tres gehört](../assets/live/status-no-definition.png)

Der Tooltip des Live-Modus-Knopfes behält denselben Hinweis, solange die Karte diese Klasse nutzt:

![Der eingeschaltete Live-Modus-Knopf, sein Tooltip sagt, dass Änderungen Godot erreichen, bevor du speicherst, und dass Godot keine Definition für crypt_lamp hat](../assets/live/toolbar-live-mode-no-definition.png)

Füg die Definition hinzu und speichere sie, dann baut Godot die Karte von selbst neu. Der Hinweis verschwindet mit deinem nächsten Speichern. Wohin eine Definition gehört, steht unter [Entitäten, die nichts tun](entities-troubleshooting.md).

## Hot Reload in einem laufenden Spiel { #hot-reload-in-a-running-game }

So siehst du deine Speicherstände in einem Spiel, das schon läuft:

1. Das Autoload hinzufügen

    Geh in Godot zu *Project > Project Settings > Globals > Autoload* und füg dieses Skript hinzu:

    ```
    res://addons/func_godot/src/trenchdot/runtime/trenchdot_hot_reload.gd
    ```

2. Das Spiel aus dem Godot-Editor starten

    Das Autoload lauscht nur in Spielen, die aus dem Godot-Editor gestartet wurden, außer du setzt sein `enabled_in_exported_builds` auf true.

3. In TrenchDot speichern

    Das Spiel baut die Karten neu, die die gespeicherte Datei nutzen, und sendet `map_reloaded(map)`. Nutze dieses Signal, um den Spieler an einen Spawnpunkt zurückzusetzen oder den Spielzustand zurückzusetzen. Die Statusleiste sollte mit "laufendes Spiel hat 1 Karte(n) neu geladen" enden.

Es lauscht auf Port 7843, der zu *Hot Reload im laufenden Spiel* in den Einstellungen des Editors passen muss. TrenchDot schickt ihm nur dann Speicherstände, wenn der Live Link an ist.
