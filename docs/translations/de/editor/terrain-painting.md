---
source: 7757565706801cf3511cdef72eaa79c5eb0dc3d0
---

# Terrain malen { #terrain-painting }

Das Werkzeug **Mischen** (Shift+G) malt die [Ebenen](terrain-layers.md) eines Terrains, zum Beispiel Sand an einem Strand oder Fels an einer Klippe. Wie Modellieren wirkt es auf die ausgewählten Terrains, oder auf alle, wenn nichts ausgewählt ist.

## Das Misch-Werkzeug { #the-blend-tool }

1. Das Werkzeug wählen

    Drück Shift+G, oder nimm *Terrain > Mischen > Misch-Werkzeug*. Seine Optionen erscheinen links oben in der Optionsleiste des Werkzeugs.

    ![Die Optionen des Misch-Werkzeugs im Modus Neigung: Modus, Abfall, Ebene, Radius, Stärke und der Neigungsbereich](../assets/terrain-painting/blend-toolbar.png)

2. Einen Modus wählen

    Das erste Menü in der Leiste bestimmt, was der Brush malt, die Modi stehen unten in der Liste.

    ![Das Modusmenü des Misch-Werkzeugs geöffnet, der Mauszeiger auf Neigung](../assets/terrain-painting/blend-modes.png)

3. Die Ebene wählen und malen

    Stell **Ebene** (0 bis 3) auf die Ebene, die du willst, und zieh dann über das Terrain. Mehrere leichte Durchgänge geben weichere Ränder als ein starker.

    ![Ein Dreckweg wird mit Ebene 2 im Modus Malen die Insel hinauf gemalt](../assets/terrain-painting/blend-paint-path.gif)

| Modus | Was er malt |
| --- | --- |
| malen | Fügt die gewählte Ebene unter dem Brush hinzu |
| löschen | Malt zurück zu Ebene 0, dem Basisboden |
| glätten | Macht die Grenzen zwischen Ebenen weicher |
| Schärfe | Lässt die stärkste Ebene gewinnen, für klare Grenzen |
| Rauschen | Nur in wolkenförmigen Flecken der angegebenen **Größe**, für Moos, trockenes Gras oder Kies |
| Neigung | Nur dort, wo der Boden zwischen zwei Winkeln liegt. `30` bis `90` legt Fels auf alles, was steiler als 30° ist |
| Höhe | Nur zwischen zwei Welthöhen, für Sand an einer Wasserlinie oder Schnee oberhalb der Baumgrenze |

Neigung und Höhe machen riesige Brushes ungefährlich, ein Brush in Hügelgröße verwandelt im Modus Neigung nur die Klippen in Fels.

Das zweite Menü in der Leiste ist der Abfall, also wie die Stärke von der Mitte des Brushs zu seinem Rand hin ausblendet.

| Abfall | Form |
| --- | --- |
| glätten | Ein sanftes Ausblenden, die übliche Wahl |
| linear | Ein gleichmäßiges Ausblenden von der Mitte zum Rand |
| konstant | Die ganze Scheibe mit derselben Stärke, für harte Ränder |
| sprühen | Zufällige Sprenkel, für Kieselsteine und Blumen |

Shift macht aus malen, Rauschen, Neigung und Höhe ein Löschen, und Ctrl glättet. Ctrl+Mausrad ändert die Brush-Größe.

(Hinweis: **Mischmaterial setzen**, **Wiederholung**, **Enttiling**, **Schärfe** und **Kachelung anwenden** weiter rechts in der Leiste sind zum Mischen von Brush-Flächen da, ein Terrain übernimmt diese Einstellungen von seinen Ebenen.)

## Automatisch malen { #auto-paint }

Automatisch malen bemalt ein Terrain nach seiner Form neu, ein schneller erster Durchgang, bevor du von Hand malst.

1. Das Terrain auswählen

    Automatisch malen fasst nur die ausgewählten Terrains an.

2. Es ausführen

    Geh auf *Terrain > Mischen > Ebenen automatisch bemalen*, oder klick im Inspektor auf **Automatisch malen**.

    ![Das Menü Terrain > Mischen mit dem Mauszeiger auf Ebenen automatisch bemalen und angehaktem Bänder ab Meereshöhe bei y 0](../assets/terrain-painting/menu-auto-paint.png)

| Ebene | Kommt auf |
| --- | --- |
| 1 | Hänge, die steiler als etwa 50° sind |
| 2 | Die oberen 20 % des Höhenbereichs |
| 3 | Die unteren 8 % des Höhenbereichs |

Ebene 0 deckt alles andere ab. Ein Terrain mit weniger Ebenen bekommt die fehlenden Bänder in seiner letzten Ebene.

Der Bereich beginnt am tiefsten Punkt, bei einer Insel ist das der Meeresboden. Hak **Bänder ab Meereshöhe** an, um stattdessen von einer Höhe aus zu messen, die du angibst, etwa der Wasserlinie bei y 0 oben.

![Die Meeresinsel vor Automatisch malen, nur Gras](../assets/terrain-painting/auto-paint-before.jpg)

![Dieselbe Insel nach Automatisch malen: Fels an den Klippen, Erde auf dem Gipfel und Sand entlang der Strände](../assets/terrain-painting/auto-paint-after.jpg)

> [!WARNING]
> Automatisch malen ersetzt jede Bemalung, modelliere also zuerst, male dann automatisch, und male zuletzt von Hand.

## Wie schmal du malen kannst { #how-narrow-you-can-paint }

Die Farbe sitzt auf den Vertices, ein Strich braucht also einen Radius von etwa zwei Zellen für ein sauberes Band. Unter einer Zelle wird daraus eine Treppe. Für einen schmalen Pfad nimm kleinere Zellen, oder leg ein Mesh oder [Decal](decals.md) darüber.

![Striche mit Radius 40, 96 und 160 auf Zellen von 64 Einheiten, daneben das Raster der Vertices, das die Farbe hält](../assets/terrain-painting/brush-width-grid.jpg)
