---
source: c1ad5f7b4051d96e108a3133de3b683670efbf64
---

# Tastenkürzel { #shortcuts }

Das sind die Standardtasten, die Voreinstellung TrenchBroom.

Willst du andere Tasten? *Hilfe > Tastenkürzel…* listet jede Belegung auf. Tipp in den Filter, um eine zu finden, drück **Setzen** und dann das neue Kürzel. Über **Voreinstellung** oben in diesem Fenster oder über *Datei > Einstellungen > Tastenbelegung* wechselst du zu Hammer oder Blender, siehe [Weitere Voreinstellungen](#other-presets).

![Das Fenster Tastenkürzel, gefiltert nach csg, mit dem Dropdown Voreinstellung, Überschreibungen zurücksetzen und je einer Schaltfläche Setzen und Löschen in jeder Zeile](../assets/shortcuts/keyboard-shortcuts-window.png)

Kürzel warten, während du in ein Feld tippst oder ein Menü offen ist. Solange der Zeiger über einem schwebenden Fenster steht, etwa dem Formgenerator, funktionieren nur Speichern, Rückgängig und Wiederherstellen, damit eine Taste, die für das Fenster gedacht war, nie deine Karte löscht oder verändert.

## Werkzeuge { #tools }

Drück die Taste eines Werkzeugs noch einmal, um zu Auswählen zurückzukehren. Esc bricht ein Ziehen oder einen Pinselstrich ab, sonst geht es zurück zu Auswählen, und in Auswählen hebt es die Auswahl auf.

| Taste | Werkzeug |
| --- | --- |
| Q | Auswählen, Brushes zeichnen und Dinge verschieben |
| C | Clip, um Brushes entlang einer Ebene zu schneiden |
| V | Vertex, um die Ecken von Brushes zu ziehen |
| R | Drehen |
| T | Skalieren |
| Tab | Die Auswahl dort bearbeiten, wo sie steht: Meshes im Mesh-Werkzeug, Brushes im Vertex-Werkzeug. Es wandelt nie einen Brush um, das macht *Mesh > Mesh bearbeiten* |
| Shift+T | Textur, um Texturen auf Flächen auszurichten |
| P | Malen, um Vertexfarben auf Flächen zu malen |
| G | Terrain und Displacements modellieren |
| Shift+G | Terrainebenen mischen |
| B | Scatter |
| Shift+E | Volumen, um ein Trigger-Volumen aufzuziehen |
| Shift+P | Pfad, um eine Kette von `path_corner`-Entitäten zu platzieren |
| M | Abstände messen |

## Maus in den Ansichten { #mouse-in-the-views }

| Eingabe | Ergebnis |
| --- | --- |
| Neben der Auswahl ziehen | Einen Brush zeichnen |
| Die Auswahl ziehen | Verschieben. Ctrl kopiert, Shift sperrt die Achse, Alt verschiebt in 3D vertikal |
| Klick, Doppelklick | Auswählen, die ganze Gruppe auswählen |
| Kurz darauf noch einmal an derselben Stelle klicken, oder Alt+Klick (2D) | Das nächste Objekt unter dem Cursor auswählen, etwa eine Wand unter ihrer Decke in der Ansicht Oben |
| Ctrl+Klick | Zur Auswahl hinzufügen oder daraus entfernen, in den Ansichten und im Outliner |
| Shift+Klick im Outliner | Die Zeilen ab der zuletzt angeklickten auswählen |
| Shift+Klick, Ctrl+Shift+Klick | Eine Fläche auswählen, eine Fläche hinzufügen oder entfernen |
| Shift+Ziehen einer gewählten Fläche (3D) | Sie schieben oder ziehen |
| Ctrl+Shift+Ziehen einer gewählten Fläche (3D) | Einen neuen Brush extrudieren |
| Rechts ziehen (3D) | Sich umsehen. Halte es gedrückt und flieg mit WASD, Q und E für runter und hoch |
| Mittel ziehen | Verschieben |
| Alt+links ziehen (3D) | Orbit. Im Werkzeug Auswählen verschiebt ein Ziehen, das auf der Auswahl oder ihrem Gizmo beginnt, sie stattdessen vertikal |
| Ctrl+Mausrad (Modellieren, Mischen, Malen, Scatter) | Die Pinselgröße ändern |

## Bearbeiten { #editing }

| Taste | Aktion |
| --- | --- |
| Ctrl+Z, Ctrl+Shift+Z oder Ctrl+Y | Rückgängig, Wiederherstellen |
| Ctrl+X, Ctrl+C, Ctrl+V | Ausschneiden, Kopieren, Einfügen unter der Maus |
| Ctrl+D, Ctrl+Shift+D | Duplizieren, verknüpft duplizieren |
| F2 | Das gewählte Objekt im Outliner umbenennen |
| Ctrl+R | Das letzte Drehen, Spiegeln, Verschieben oder Duplizieren wiederholen. Beim Mesh-Bearbeiten macht es einen Loop Cut |
| Delete oder Backspace | Löschen |
| Ctrl+A, Esc, Ctrl+I | Alles, nichts, Auswahl umkehren |
| Ctrl+T, Ctrl+Shift+T | Die gewählten Brushes als Schablone nutzen und auswählen, was sie berühren, oder was in ihnen liegt. Die Schablonen-Brushes werden gelöscht |
| Ctrl+G, Ctrl+Shift+G | Gruppieren, Gruppierung aufheben |
| Ctrl+H, Ctrl+J, Ctrl+Shift+H | Die Auswahl verbergen, sie isolieren, indem alles andere verborgen wird, alles anzeigen |
| Pfeiltasten, Page Up, Page Down | Um einen Rasterschritt verschieben |

## Brushes, Meshes und Texturen { #brushes-meshes-and-textures }

| Taste | Aktion |
| --- | --- |
| Ctrl+K, Ctrl+M, Ctrl+L | CSG subtrahieren (die Auswahl aus dem herausschneiden, was sie berührt), konvex verschmelzen, Schnittmenge |
| Ctrl+Shift+K | Aushöhlen, einen massiven Brush in einen Raum mit Wänden verwandeln |
| Ctrl+Shift+B | Formgenerator, für Bögen, Zylinder, Treppen und andere fertige Formen |
| Ctrl+Shift+E, Ctrl+Shift+J | Brushes in Mesh umwandeln, Meshes verbinden |
| Ctrl+Shift+W | Brushes in die Welt verschieben, aus ihrer Entität nehmen, sodass sie schlichte Level-Geometrie werden |
| Ctrl+Shift+U | UV-Sperre umschalten, die Texturen an einem Brush festhält, während er sich bewegt |
| Alt+H | Hotspot-Anpassung, die gewählte Flächen in das am besten passende Rechteck eines Trim Sheets einpasst |

## Ansicht und Dateien { #view-and-files }

| Taste | Aktion |
| --- | --- |
| F oder Ctrl+U | Die Auswahl oder die ganze Karte einrahmen |
| `[` und `]`, oder `-` und `=` | Das Raster halbieren oder verdoppeln |
| F4, F3 | Durch die Schattierung schalten (texturiert, flach, beleuchtet, Drahtgitter), Beleuchtete Vorschau umschalten |
| Shift+Space | Die Ansicht unter dem Zeiger maximieren |
| Ctrl+Shift+1 bis 9, Ctrl+1 bis 9 | Kamera-Lesezeichen speichern, abrufen |
| Ctrl+=, Ctrl+-, Ctrl+0 | Oberflächenskalierung größer, kleiner, zurücksetzen |
| F1 | Das [Handbuch](interface.md#help-and-the-manual) beim Werkzeug, Panel-Tab, Menüeintrag oder Inspektor-Abschnitt unter dem Zeiger öffnen, überall sonst die Befehlspalette |
| Ctrl+Klick auf ein Werkzeug in der Werkzeugleiste, Cmd+Klick auf macOS | Seine Seite im Handbuch öffnen |
| Ctrl+Shift+P | Befehlspalette |
| Ctrl+N, Ctrl+O | Neue Karte, Karte öffnen |
| Ctrl+S, Ctrl+Shift+S | Speichern, Speichern unter |
| Ctrl+Shift+N, Ctrl+Tab, Ctrl+W | Neuer Tab, nächster Tab, Tab schließen |
| F5 | Das Godot-Projekt starten |

> [!NOTE]
> F2 hat früher die Schattierung durchgeschaltet. Jetzt benennt es um, wie in Godot, Blender und den meisten Dateimanagern, und die Schattierung ist auf F4 gewandert. Eine Belegung, die du selbst im Fenster Tastenkürzel gesetzt hast, bleibt, wie sie war.

## Weitere Voreinstellungen { #other-presets }

Eine Voreinstellung ändert nur die Tasten unten, alles andere bleibt wie oben. Deine eigenen Belegungen liegen über jeder Voreinstellung, **Überschreibungen zurücksetzen** im Fenster Tastenkürzel löscht sie.

| Taste | Voreinstellung Hammer |
| --- | --- |
| Shift+S, Shift+X, Shift+V, Shift+A | Auswählen, Clip, Vertex, Textur-Werkzeug |
| H, Ctrl+H, U | Verbergen, isolieren, alles anzeigen |
| Ctrl+W | Brushes in die Welt verschieben |
| Ctrl+M | Formgenerator |
| Ctrl+L | Um 90° um die vertikale Achse drehen |
| Ctrl+Shift+M, Ctrl+Shift+L | CSG konvex verschmelzen, Schnittmenge |
| Ctrl+F4 | Tab schließen |

| Taste | Voreinstellung Blender |
| --- | --- |
| A, Alt+A | Alles, nichts auswählen |
| X | Löschen |
| Shift+D, Alt+D | Duplizieren, verknüpft duplizieren |
| S | Skalieren-Werkzeug |
| H, Alt+H, / | Verbergen, alles anzeigen, isolieren |
| Ctrl+J | Meshes verbinden |
| Z | Durch die Schattierung schalten |
| Shift+A | Formgenerator |
| F3 | Befehlspalette |
| F4 | Beleuchtete Vorschau |
| Alt+Shift+H | Hotspot-Anpassung |
