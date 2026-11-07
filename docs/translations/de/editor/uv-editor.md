---
source: 1cc5b4057e3a84e4d2bcf666b68e70eb94fb265b
---

# UV-Editor { #uv-editor }

Der UV-Editor zeichnet die ausgewählten Brush- und Mesh-Flächen flach über ihre gekachelte Textur, damit du siehst, welchen Teil der Textur jede Fläche zeigt. Nimm ihn für alles, was das [Textur-Werkzeug](texturing.md#the-texture-tool) in der 3D-Ansicht nicht kann, etwa Mesh-UVs auf das Pixel genau zu setzen.

## Öffnen { #opening-it }

Der UV-Editor ist ein Tab im rechten Panel neben dem Inspektor, und auch *Textur > UV-Editor* holt ihn nach vorn. Wähl dann Flächen aus: Shift+Klick auf eine in einer Ansicht, oder nimm das Textur-Werkzeug. Die oberste Zeile nennt das Material und wie viele Flächen du ausgewählt hast.

![Der UV-Editor mit einer ausgewählten Wandfläche, als oranges Rechteck über der gekachelten Textur showcase/plaster gezeichnet](../assets/uv-editor/uv-editor-face.png)

> [!TIP]
> Solange der UV-Editor der sichtbare Tab seines Panels ist, wählt Ctrl+Klick auf einen Brush oder ein Mesh alle seine Flächen aus, sodass du jede Seite auf einmal texturieren kannst. Brushes und Meshes, die schon ausgewählt waren, werden ebenfalls zu ihren Flächen. Enthält die Auswahl etwas anderes, zum Beispiel ein Terrain oder eine Entität, fügt Ctrl+Klick Objekte wie gewohnt hinzu und entfernt sie.

Eine Brush-Fläche bekommt ihre Textur normalerweise aus einem einzigen Versatz, einer Skalierung und einer Drehung. Eine Fläche mit **expliziten UVs** speichert stattdessen für jede Ecke eine Texturposition, und das brauchen Meshes. Der UV-Editor zeichnet diese Flächen blau, und du kannst ihre Ecken einzeln anwählen. Orange Flächen haben keine expliziten UVs und bewegen sich als Ganzes.

## Steuerung { #controls }

Das **?** am Ende der obersten Zeile zeigt dir diese Übersicht auch im Editor.

| Eingabe | Ergebnis |
| --- | --- |
| Leeren Platz ziehen | Die Textur verschieben, oder die ganzen Flächen, wenn sie explizite UVs haben |
| Mausrad, Rechtsziehen | Skalieren, drehen |
| Auf eine Ecke klicken | Sie samt den Ecken benachbarter Flächen auswählen, die an sie genäht sind |
| Shift+Klick, Ctrl+Klick | Hinzufügen, entfernen |
| Alt+Klick | Nur die Ecke dieser einen Fläche auswählen, um eine Naht zu trennen |
| Shift oder Ctrl+Ziehen auf leerem Platz | Rahmenauswahl, hinzufügend oder entfernend |
| Eine ausgewählte Ecke ziehen | Die ganze Auswahl verschieben |
| Doppelklick auf eine Ecke | Gizmo, das sich nur entlang U oder V bewegt, Escape blendet es aus |
| Mittel-Ziehen, Ctrl+Mausrad | Die Zeichenfläche verschieben, zoomen |

Sind Ecken ausgewählt, skalieren und drehen Mausrad und Rechtsziehen sie um ihre Mitte. Die Felder U und V unter der Zeichenfläche setzen die Auswahl pixelgenau.

Die kleinen Schaltflächen hinter den Menüs schalten Pixel-Snap um, zeigen die Hotspots der Textur, machen aus einem einfachen Ziehen eine Rahmenauswahl, zeigen das Gizmo, schalten die UV-Sperre um und rahmen die Flächen wieder ein. Zeig auf eine, dann siehst du, welche welche ist.

## Mesh-UV-Projektionen { #mesh-uv-projections }

Sind Mesh-Flächen ausgewählt, legt das Menü **Projektion** explizite UVs für dich an. Es nutzt die Texel-Dichte, die eine Brush-Fläche mit derselben Textur bekäme, sodass Meshes zu den Brushes um sie herum passen. Zeig auf einen Eintrag, für eine Erklärung in einer Zeile.

![Das Menü Projektion des UV-Editors geöffnet, Zylinder Y mit Mauszeiger und seinem Tooltip, Wickelt sich um diese Weltachse durch die Mitte der Flächen, u läuft darum herum](../assets/uv-editor/projection-menu.png)

| Befehl | Ergebnis |
| --- | --- |
| *Planar*, *Planar (X)*, *Planar (Y)*, *Planar (Z)* | Projiziert entlang der Richtung, in die die Flächen überwiegend zeigen, oder entlang einer Weltachse, und hält sie dabei in einem Stück |
| *Box* | Projiziert jede Fläche von ihrer nächsten Weltachse aus, für kantige Formen |
| *Zylinder X*, *Zylinder Y*, *Zylinder Z*, *Kugel* | Wickeln die Textur um die Flächen, für Rohre, Säulen und Kuppeln |
| *Ansicht* | Projiziert geradewegs entlang der 3D-Kamera |
| *Abwickeln* | Legt die Flächen Kante für Kante flach aus, damit Streifen und Raster durchgehend bleiben |
| *Auf Welt zurücksetzen* | Die Projektion einer frisch zurückgesetzten Brush-Fläche |
| *Planar backen* | Macht aus der planaren Textur bearbeitbare Eck-UVs, wird angezeigt, solange manche Flächen noch keine haben |
| *UVs löschen* | Wirft die expliziten UVs wieder weg |
| *Normalisieren* | Skaliert die UVs einmal so, dass sie in die Textur passen |
| *Packen* | Packt die Inseln mit einer einheitlichen Texel-Dichte in die Textur |

Hier hat eine Säule mit 12 Seiten *Zylinder Y* bekommen, ihre Seiten laufen jetzt nebeneinander als ein Streifen um die Textur:

![Der UV-Editor nach Zylinder Y auf einer Meshsäule mit 12 Seiten, ihre Seitenflächen als ein durchgehender Streifen ausgelegt](../assets/uv-editor/uv-editor-cylinder.png)

Das Menü **UV** spiegelt, dreht, passt ein, richtet aus, begradigt und rastet auf Pixel ein. Das Menü **Hotspot** passt auf ein Hotspot-Rechteck ein, siehe [Hotspots](texturing.md#hotspots).
