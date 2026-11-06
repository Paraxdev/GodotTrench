---
source: 6e5c9bc3787f689328791dca9df024446698b311
---

# Brushes und CSG { #brushes-and-csg }

Aus Brushes baust du den größten Teil eines Levels. Sie sind schnell gezeichnet, rasten sauber am Raster ein und werden in Godot zu Meshes mit passender Kollision. Alles hier nutzt das Werkzeug **Auswählen** (Q), solange ein Abschnitt nicht ein anderes nennt.

## Zeichnen und verschieben { #draw-and-move }

Zieh irgendwo neben der Auswahl, in einer beliebigen Ansicht, und eine Box folgt der Maus. Die 2D-Ansichten schreiben ihre Größe daneben. In der 3D-Ansicht beginnt sie auf der Fläche unter dem Cursor und ist einen Rasterschritt dick. In einer 2D-Ansicht übernimmt sie die Tiefe des zuletzt gezeichneten Brushes, beim allerersten sind es 128 Einheiten.

![Die vier Ansichten, während in 3D eine Box gezeichnet wird, die neue Box ist 144 mal 144 Einheiten groß und einen Rasterschritt, 16 Einheiten, dick](../assets/brushes/draw-box-3d.png)

Ist ein Brush markiert, kannst du ihn herumschieben:

| Eingabe | Ergebnis |
| --- | --- |
| Die Auswahl ziehen | Verschiebt sie, Shift sperrt auf die Hauptachse |
| Ctrl+Ziehen der Auswahl | Verschiebt eine Kopie, das Original bleibt |
| Alt+Ziehen der Auswahl (3D) | Verschiebt senkrecht. Beginnt es neben der Auswahl, umkreist Alt+Ziehen die Kamera |
| Pfeiltasten, Page Up, Page Down | Um einen Rasterschritt verschieben |
| `[` und `]` | Das Raster halbieren und verdoppeln |

In 3D bekommt die Auswahl außerdem ein Gizmo zum Verschieben, Drehen und Skalieren. Die Drehung rastet auf 15° ein, mit Shift auf 1°. Wenn es dir im Weg ist, schalte es unter *Ansicht > Transform-Gizmo* aus.

## Genaue Position und Größe { #exact-position-and-size }

Manchmal willst du genaue Zahlen, etwa einen Türschneider, der genau so tief ist wie die Wand. Markier den Brush und tipp sie in den **Inspektor** rechts.

![Der Inspektor mit einem markierten Brush, der seine Felder für Position und Größe zeigt](../assets/brushes/inspector-position-size.png)

Die Position ist die unterste Ecke, das kleinste X, Y und Z, also die Ecke, die meistens auf dem Raster sitzt. Eine neue Größe lässt die Auswahl von dieser Ecke weg wachsen, die Position bleibt also stehen. Ein Feld zu ziehen springt in Rasterschritten, solange das Einrasten an ist, ein getippter Wert bleibt genau erhalten. Dieselben Felder erscheinen bei Meshes, bei einer Brush-Entität und bei einer Auswahl aus mehreren Objekten.

## Umformen { #reshape }

Einen markierten Brush kannst du umformen, ohne das Werkzeug Auswählen zu verlassen:

| Eingabe | Ergebnis |
| --- | --- |
| Shift+Ziehen einer markierten Fläche (3D) | Diese Fläche drücken oder ziehen |
| Ctrl+Shift+Ziehen einer markierten Fläche (3D) | Einen neuen Brush aus ihr herausziehen |
| Eine Kante der Auswahl ziehen (2D) | Die Größe ändern. Die Kante leuchtet gelb, und der Cursor ändert sich dort, wo ein Ziehen die Größe ändert |

![Shift+Ziehen der Seitenfläche einer markierten Box in der 3D-Ansicht, die Box dehnt sich in Rasterschritten](../assets/brushes/push-face.gif)

Für Schrägen, Rampen und schräge Wände drückst du C für das Werkzeug **Clip**:

1. Klick zwei oder drei Punkte entlang des Schnitts an. Sie rasten am Raster ein und landen in 3D auf der Fläche, die du anklickst.
2. Drück Tab, um zu wählen, was bleibt, die vordere, die hintere oder beide Hälften. Der Text unten in der Ansicht sagt dir, welche gerade aktiv ist.
3. Drück Enter, um zu schneiden. Backspace entfernt den letzten Punkt, falls du dich verklickt hast.

![Das Clip-Werkzeug mit zwei Punkten in der Vorderansicht über der oberen Ecke einer Box, die Schnittlinie ist auch in der 3D-Ansicht zu sehen](../assets/brushes/clip-points.png)

Um einzelne Ecken zu ziehen, drückst du V für das **Vertex**-Werkzeug, oder Tab bei markierten Brushes. Es zeigt Griffe an den Ecken und kleine blaue Punkte an den Kantenmitten und Flächenmittelpunkten. Klick einen Punkt an, um ihn zu markieren, oder zieh ihn vom Brush weg, um dort eine Ecke hinzuzufügen. Delete entfernt die markierten Vertices.

![Das Vertex-Werkzeug an einer abgeschnittenen Box, gelbe Eckgriffe und blaue Punkte auf den Kanten und Flächen](../assets/brushes/vertex-handles.png)

> [!NOTE]
> Ein Brush darf keine Dellen haben, deshalb wird eine Verschiebung abgelehnt, die eine machen würde. Du siehst einen roten Geist dort, wo er landen würde, und eine Zeile in der Statusleiste. Zieh den Punkt stattdessen nach außen, oder wandle den Brush in ein [Mesh](meshes.md) um, wenn du Dellen und Überhänge brauchst.

## Drehen, skalieren und messen { #rotate-scale-and-measure }

| Werkzeug | Eingabe | Ergebnis |
| --- | --- | --- |
| **Drehen** (R) | Einen Ring um die Auswahl ziehen | Dreht in 15°-Schritten, mit Shift in 1°-Schritten. Eine 2D-Ansicht hat einen Ring, um die Achse, entlang der sie schaut |
| **Skalieren** (T) | Einen Griff an den Grenzen der Auswahl ziehen | Ändert die Größe auf dem Raster, mit Alt auf beiden Seiten gleichzeitig, sodass die Mitte stehen bleibt. Die 3D-Ansicht hat an jeder Seite einen Griff |
| **Messen** (M) | Ziehen, oder zwei Punkte anklicken | Zeigt den Abstand in Einheiten und Metern, und unten in der Ansicht seine X-, Y-, Z- und horizontalen Anteile |

![Das Messen-Werkzeug in der 3D-Ansicht, eine Linie mit der Beschriftung 230.76 u (7.21 m) und unten die Anteile X, Y und Z](../assets/brushes/measure.png)

## Formgenerator { #shape-generator }

Für Zylinder, Bögen, Treppen, Dächer und andere Formen, die von Hand gezeichnet viele Brushes brauchen würden, nimmst du den Formgenerator.

1. Zeichne eine Box dorthin, wo die Form hin soll, und lass sie markiert.
2. Öffne *Brush > Formgenerator…* (Ctrl+Shift+B).
3. Wähl die **Form** und ihre Optionen. Der Dialog nennt die Grenzen, die er füllt, und dass er die Auswahl ersetzt.
4. Klick auf **Erstellen**.

![Der Dialog Formgenerator mit Zylinder, 16 Seiten und der Y-Achse, der eine Auswahl von 128 x 192 x 128 füllt](../assets/brushes/shape-generator.png)

![Links die markierte Box, rechts der Zylinder mit 16 Seiten, der sie ersetzt hat](../assets/brushes/shape-generator-box-to-cylinder.png)

Zylinder und Kegel stehen standardmäßig aufrecht. Wähl unter **Achse** **X** oder **Z**, um einen innerhalb derselben Box hinzulegen, für Rohre, Baumstämme oder einen Balken, statt ihn danach zu drehen. Torus, Bogenwand, Satteldach, Turmspitze und Gitter werden immer als [Mesh](meshes.md) erzeugt. Zylinder, Kegel und Kugeln können mit **Als bearbeitbares Mesh erstellen** auch eins sein, und **Glatte Schattierung** rundet sie dann ab. Die Form **Text** baut Blockbuchstaben aus Brushes, für Schilder und Bodenbeschriftung.

## CSG { #csg }

Die CSG-Operationen findest du unter *Brush > CSG*, das auch ihre beiden Einstellungen enthält.

![Das Menü Brush mit offenem Untermenü CSG, das Subtrahieren, Konvex vereinen, Schnittmenge und Aushöhlen mit ihren Kurzbefehlen auflistet, dazu das Kontrollkästchen Ausgeschnittene Flächen behalten das Material des Ziels und das Feld Wandstärke](../assets/brushes/menu-brush-csg.png)

Alle vier haben auch eine Schaltfläche in der Werkzeugleiste, rechts neben den Schaltflächen für Einrasten und UV-Sperre.

![Die vier CSG-Schaltflächen in der Werkzeugleiste, Konvex vereinen überfahren mit seinem Tooltip](../assets/brushes/toolbar-csg.png)

| Operation | Taste | Ergebnis |
| --- | --- | --- |
| Subtrahieren | Ctrl+K | Schneidet die Auswahl aus jedem Brush heraus, den sie berührt, und löscht dann die Auswahl. Eine Auswahl, die keinen anderen Brush berührt, bleibt erhalten, und die Statusleiste sagt das |
| Aushöhlen | Ctrl+Shift+K | Macht aus einem Brush Wände um sein altes Volumen, 16 Einheiten dick, solange du *Wandstärke* im selben Menü nicht änderst. Ein Brush, der für zwei Wände zu dünn ist, bleibt unberührt |
| Konvex vereinen | Ctrl+M | Verschmilzt die Auswahl zu einem konvexen Brush, der sie umhüllt |
| Schnittmenge | Ctrl+L | Behält nur, wo sich die markierten Brushes überlappen |

Um eine Türöffnung oder ein Fenster auszustanzen, setz einen Brush dorthin, wo das Loch hin soll, und subtrahiere ihn, [Erste Schritte](../getting-started.md) führt dich durch ein Beispiel. Die Flächen, die ein Schnitt öffnet, bekommen das Material des Schneiders, außer du hakst im selben Menü *Ausgeschnittene Flächen behalten das Material des Ziels* an.

## Displacements { #displacements }

Für unebenen Boden in einem Brush-Level, etwa einen Schutthang, kannst du eine Fläche in ein Displacement verwandeln und es modellieren.

1. Klick mit Shift eine vierseitige Fläche an, oder markier den Brush, um seine obere Fläche zu nehmen.
2. Öffne *Brush > Displacement* und wähl einen der Einträge **Erstellen**. Eine höhere Stufe gibt ein feineres Raster.
3. Das Werkzeug **Modellieren** (G) schaltet sich von selbst ein, zieh über die Fläche, um sie zu formen.

![Das Menü Brush mit offenem Untermenü Displacement, Erstellen, Stufe 2 (5x5), Erstellen, Stufe 3 (9x9) und Erstellen, Stufe 4 (17x17), dann Displacements vernähen und Displacement entfernen](../assets/brushes/menu-brush-displacement.png)

Für größere Außenbereiche ist ein [Terrain](terrain.md) besser geeignet.
