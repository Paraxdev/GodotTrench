---
source: 6da371c3430389f428469b408b32808d1d2b0af6
---

# Seeinsel 3: Malen { #sea-island-3-painting }

Wechsle mit **Shift+G** zum Misch-Werkzeug oder klicke in der Werkzeugleiste auf die Schaltfläche Mischen direkt neben Modellieren. Wie Modellieren arbeitet es am gewählten Terrain oder an allem, wenn nichts gewählt ist. Seine Einstellungen liegen in der Werkzeugoptionsleiste oben links.

![Die Werkzeugleiste mit markierter Schaltfläche Misch-Werkzeug und ihrem Tooltip: ziehen, um Mischungen auf Terrainebenen, Displacements und Flächen mit Mischmaterial zu malen](../assets/sea-island/toolbar-blend.png)

Die Modi **Neigung** und **Höhe** malen nur dort, wo der Boden innerhalb eines Bereichs von Steilheit oder Höhe liegt. Ein Radius von 12000 deckt die ganze Insel ab, ein Klick in diesen Modi malt also jedes Ufer und jede Klippe auf einmal.

## Sand und Fels { #sand-and-rock }

| Einstellung | Sand | Fels |
| --- | --- | --- |
| Modus | Höhe, von `-3000` bis `40` | Neigung, von `32` bis `90` |
| Abfall | konstant, volle Stärke über den ganzen Pinsel | konstant |
| Ebene | 3 | 1 |
| Radius, Stärke | 12000, 1 | 12000, 1 |

Stelle das ein und klicke dann für beide je einmal in die Mitte der Insel. Du solltest sehen, wie alles unter 40 zu Sand wird und jeder steile Hang zu Fels:

![Modus Höhe: alles unter 40 Einheiten wird zu Sand](../assets/sea-island/sand-height.jpg)

![Modus Neigung: alles steiler als 32° wird zu Fels](../assets/sea-island/rock-slope.jpg)

Um den Strand zu verbreitern, male Ebene 3 am Südufer im Modus **malen**, Abfall weich, Radius 700, Stärke 0.7. Der weiche Abfall blendet zum Rand des Pinsels hin aus, der Rand des Sandes bleibt also weich.

## Pfade { #paths }

Modus **malen**, Abfall weich, Ebene 2, Radius 140, Stärke 0.8. Ziehe:

1. Von der Oberkante des Strandes in langen Kurven hinauf zum Haupthügel.
2. Von der Spitze des Haupthügels am Grat entlang zur Klippenkante.
3. Vom Ostende des Strandes zum zweiten Hügel.

![Der Pfad hinauf zum Haupthügel wird gemalt](../assets/sea-island/paths.jpg)

![Der Pfad aus der Nähe](../assets/sea-island/path-close.jpg)

## Die Wiederholung aufbrechen { #break-up-the-repeat }

Aus der Ferne zeigt sich eine sich wiederholende Textur als Raster. Wähle das Terrain im Outliner (ein Klick in eine Ansicht würde mit dem Misch-Werkzeug malen), und stelle dann **Enttiling** und **Schärfe** in jeder Zeile Ebene des Inspektors ein. Schärfe hält die enttilte Textur knackig statt weich.

| Ebene | Enttiling | Schärfe |
| --- | --- | --- |
| 0 Gras | 0.5 | 0.5 |
| 1 Klippe | 0.7 | 0.4 |
| 2 Erde | 0.5 | 0.5 |
| 3 Sand | 0.4 | 0.5 |

![Der Terrain-Inspektor mit den vier Zeilen Ebene, jede mit Material, Kachel, Enttiling und Schärfe](../assets/sea-island/inspector-terrain-layers.png)

Weiter: [Seeinsel 4: Scatter](sea-island-scatter.md).
