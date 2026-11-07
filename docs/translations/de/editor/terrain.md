---
source: 5b022e12202fe97edce0105dd2916cbd30e66e36
---

# Terrain { #terrain }

Ein Terrain ist ein großes Raster aus Höhen, das du wie Ton modellierst, mit bis zu vier darübergemalten Texturen. Nimm es für offenes Gelände wie Hügel, Strände und Täler, wo es ewig dauern würde, Brushes von Hand zu formen.

Diese Seite zeigt, wie du eins anlegst und mit dem Modellier-Werkzeug (G) formst. Das Texturieren steht auf [Terrainebenen](terrain-layers.md) und [Terrain malen](terrain-painting.md).

## Ein Terrain anlegen { #creating-a-terrain }

1. Den Dialog öffnen

    Geh in der Menüleiste auf *Terrain > Terrain erstellen…*.

    ![Das Menü Terrain geöffnet, Terrain erstellen… mit dem Mauszeiger darauf](../assets/terrain/menu-create-terrain.png)

2. Eine Größe wählen

    **Auflösung** und **Zellgröße** bestimmen, wie groß es wird, und die Zeile **Größe** darunter zeigt das Ergebnis in Einheiten und Metern. Die Vorgabe von 129 x 129 Vertices mit Zellen von 64 Einheiten ergibt 8192 Einheiten, 256 m. Siehe [Die Zellgröße wählen](#picking-the-cell-size) weiter unten.

    ![Der Dialog Terrain erstellen mit seinen Vorgaben: 129 x 129, Zellen mit 64 u, Hügel und die vier Ebenenfelder](../assets/terrain/create-terrain-dialog.png)

3. (Optional) Eine Startform wählen

    **Form** ist der Untergrund, mit dem du anfängst. Vorgabe ist Hügel, die anderen sind Flach, Berg, Insel, Tal und Grate. **PNG importieren…** liest stattdessen eine Graustufen-Höhenkarte ein, bei der hellere Pixel höher liegen. Beides geht nur hier, beim Anlegen.

4. (Optional) Die Ebenen wählen

    **Basisebene**, **Ebene für Hänge**, **Ebene für Gipfel** und **Ebene für Senken** sind die vier Texturen des Terrains, und **Aktuelles nehmen** setzt das Material ein, auf das du zuletzt geklickt hast. Ist **Ebenen nach Neigung und Höhe bemalen** angehakt, werden sie sofort bemalt, siehe [Automatisch malen](terrain-painting.md#auto-paint).

5. Unten auf **Erstellen** klicken

    Das Terrain wird am 3D-Cursor zentriert, oder an der **Mitte**, die du eintippst, wenn **3D-Cursor** nicht angehakt ist, und diese Höhe wird zur Null des Terrains. Du solltest das neue Terrain ausgewählt sehen, das Modellier-Werkzeug ist schon aktiv.

## Die Zellgröße wählen { #picking-the-cell-size }

| Einstellung | Bedeutung |
| --- | --- |
| Auflösung | Wie viele Vertices an jeder Seite entlanglaufen, von 33 bis 1025. Mehr Vertices halten mehr Details, kosten aber mehr beim Zeichnen und Modellieren |
| Zellgröße | Der Abstand in Karteneinheiten zwischen zwei benachbarten Vertices |

257 Vertices mit Zellen von 64 Einheiten ergeben zum Beispiel 16384 Einheiten, 512 m.

Zwischen zwei Vertices liegt nichts, die Zellgröße ist also das feinste Detail, das das Terrain halten kann. Ein Gebirge braucht große Zellen, ein Garten kleine.

Falsch gewählt? Wähl das Terrain aus und öffne **Neu abtasten** im Inspektor. Es baut das Raster mit 65, 129, 257 oder 513 Vertices an der längeren Seite neu auf und behält Höhen, Bemalung und Löcher. Das Terrain behält seine Größe, mehr Vertices bedeuten also kleinere Zellen.

![Der Inspektor des Terrains mit offenem Menü Neu abtasten und dem Mauszeiger auf 257 Vertices](../assets/terrain/inspector-resample.png)

**Chunk-Zellen** (standardmäßig 32) teilt das Terrain zum Zeichnen und für die Kollision in quadratische Stücke. Kleinere Chunks bauen sich beim Modellieren schneller neu auf, größere bedeuten weniger Nodes in Godot.

## Modellieren { #sculpting }

Drück G, wähl links oben in der Optionsleiste des Werkzeugs einen Modus und zieh über das Terrain. Es wirkt auf die ausgewählten Terrains, oder auf alle, wenn nichts ausgewählt ist.

![Das Modusmenü des Modellier-Werkzeugs geöffnet in der Optionsleiste, neben den Feldern für Radius und Stärke](../assets/terrain/sculpt-modes.png)

Halt die Maustaste gedrückt, dann steigt der Boden unter dem Brush weiter an.

![Mit gehaltenem Anheben und einem Brush von 900 Einheiten wächst aus der Insel ein Hügel](../assets/terrain/sculpt-raise.gif)

| Modus | Was er tut |
| --- | --- |
| Anheben, Absenken | Baut Höhe auf oder gräbt sie ab, solange du die Taste hältst. Ein Klick macht eine kleine Beule |
| Glätten | Entspannt jeden Vertex zu seinen Nachbarn hin, um Klumpen und scharfe Grate weicher zu machen |
| Einebnen | Zieht den Boden auf die Höhe, in der der Strich begann, für Plattformen, Straßen und Plateaus |
| Rauschen | Fügt kleine zufällige Beulen hinzu, damit ebener Boden weniger künstlich aussieht |
| Terrassieren | Rastet Höhen auf Stufen von **Schritt** Einheiten ein, für Reisfelder oder abgestufte Klippen. Das Feld erscheint neben dem Modus |
| Alpha malen, Alpha löschen, Ebene malen | Malen statt formen. Die Alpha-Modi sind für Displacements gedacht, Ebene malen malt die Terrainebene, die neben dem Modus eingestellt ist. Auf einem Terrain macht das das [Misch-Werkzeug](terrain-painting.md) mit mehr Kontrolle |
| Loch, Loch füllen | Schneidet ganze Zellen heraus oder füllt sie wieder, Kollision eingeschlossen. Nimm es für Höhlen- oder Bunkereingänge |

Ctrl+Mausrad ändert die Brush-Größe, Shift tauscht Anheben und Absenken, und gehaltenes Ctrl glättet, in welchem Modus auch immer.

**Stärke** legt fest, wie schnell der Brush arbeitet. Anheben und Absenken bewegen die Mitte des Brushs pro Punkt Stärke 8 Einheiten in der Sekunde, egal bei welchem Radius, die Vorgabe 4 baut also 32 Einheiten in der Sekunde auf. Für Hügel und Inseln dreh lieber die Stärke hoch, statt eine Minute lang zu halten.

Halt den Radius über der Zellgröße des Terrains, ein kleinerer Brush stupst nur den einen Vertex unter sich an (die Statusleiste sagt dir, wenn das passiert).

Um neu anzufangen, setzt *Terrain > Modellieren > Einebnen* oder **Einebnen** im Inspektor das ganze ausgewählte Terrain auf seine Nullhöhe zurück.

> [!TIP]
> Forme mit einem großen Brush und glätte nach jedem groben Durchgang. Kleine Brushes hinterlassen Klumpen.

## In Godot { #in-godot }

Jedes Terrain wird zu einem `TrenchDotTerrain`-Node, einem `StaticBody3D` (Godots Node für festen Boden, der sich nie bewegt), dessen Kollision dem entspricht, was er zeichnet. Für Gameplay-Code liefert `height_at(x, z)` die Bodenhöhe an einer lokalen Position, praktisch, um zur Laufzeit Dinge auf den Boden zu setzen.

<div class="td-mcp" markdown>

## MCP { #mcp }

Die Operation `sculpt` des Tools `terrain_edit` wendet einen einzelnen Tupfer an, und seine Stärke ist, wie weit dieser Tupfer die Mitte bewegt, wieder unabhängig vom Radius. Einen Strich eine Sekunde lang bei Stärke 4 zu halten entspricht einem Tupfer der Stärke 32.

</div>
