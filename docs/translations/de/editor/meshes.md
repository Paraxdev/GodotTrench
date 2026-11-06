---
source: d32761c1e423392e984f1e0d2875aadc77d595ce
---

# Meshes { #meshes }

Anders als Brushes können Meshes Dellen, Löcher und gebogene Flächen haben. Nimm sie also für Felsen, Rohre, runde Bögen und andere Formen, die Brushes nicht hinbekommen.

## Ein Mesh bearbeiten { #editing-a-mesh }

Markier die Brushes, die du umformen willst, und wähl *Mesh > Mesh bearbeiten*. Der Editor wandelt sie in ein Mesh um und beginnt sofort mit dem Bearbeiten.

![Das Menü Mesh mit überfahrenem Eintrag Mesh bearbeiten, darunter Brushes in Mesh umwandeln (Ctrl+Shift+E), Meshes in Brushes umwandeln, Meshes verbinden (Ctrl+Shift+J) und die Untermenüs Topologie, Komponenten, Normalen und Schattierung und Transformation](../assets/meshes/menu-edit-mesh.png)

Bei einem markierten Mesh macht **Tab** dasselbe, und noch einmal Tab bringt dich zurück zu den Objekten. Tab bei Brushes öffnet stattdessen das Vertex-Werkzeug, damit ein versehentlicher Tastendruck nie einen Brush in ein Mesh verwandelt. Die Schaltfläche Mesh-Werkzeug in der Werkzeugleiste beginnt das Bearbeiten, ohne etwas umzuwandeln. Ctrl+Shift+E wandelt nur um, Ctrl+Shift+J verbindet Meshes zu einem.

Beim Bearbeiten schaltet die Optionsleiste des Werkzeugs oben zwischen **Vertex**, **Kante** und **Fläche** um und hat Schaltflächen für die häufigen Operationen.

![Die Optionsleiste des Mesh-Werkzeugs mit Vertex, Kante und Fläche, dann Unterteilen, Verschmelzen, Füllen, Löschen, Glätten und Flach](../assets/meshes/mesh-tool-options.png)

## Tasten { #keys }

Die Tasten folgen Blender. Drück zum Beispiel 3, klick die obere Fläche eines Quaders an und drück E, beweg dann die Maus nach oben und klick, um sie zu extrudieren:

![Ein Mesh-Quader im Flächenmodus, dessen obere Fläche in Rasterschritten nach oben extrudiert ist](../assets/meshes/extrude-face.gif)

| Taste | Aktion |
| --- | --- |
| 1, 2, 3 | Auf Vertices, Kanten oder Flächen arbeiten |
| A, Alt+A | Alles auswählen, nichts auswählen |
| Alt+Klick, Ctrl+Alt+Klick | Den Edge-Loop durch die angeklickte Kante auswählen, oder den Ring quer dazu |
| Ctrl+L | Alles auswählen, was mit der Auswahl zusammenhängt |
| G, R, S | Verschieben, drehen, skalieren |
| E | Neue Geometrie aus der Auswahl herausziehen (Extrude) |
| I | Inset, eine kleinere Fläche innerhalb jeder markierten Fläche |
| Ctrl+B, Ctrl+Shift+B | Kanten oder Vertices abfasen und so die Ecke abschneiden (Bevel) |
| Ctrl+R | Loop Cut, ein neuer Kantenring quer durch das Mesh. Mit dem Mausrad oder + und - stellst du die Anzahl der Schnitte ein |
| K | Messer, klick zwei Punkte an, um entlang dieser Linie zu schneiden |
| M | Die markierten Vertices an ihrem Mittelpunkt verschmelzen |
| F | Füllen, eine neue Fläche über die markierten Vertices |
| X oder Delete | Löschen |
| Shift+D | Duplizieren und verschieben |
| P | Die Auswahl in ein neues Mesh abtrennen |
| Alt+F | Normalen umkehren, für Flächen, die von der falschen Seite sichtbar sind |

Beim Verschieben, Drehen oder Skalieren sperrst du mit X, Y oder Z auf diese Achse, oder mit Shift und dem Buchstaben sperrst du sie aus. Tipp eine Zahl für einen genauen Wert. Klick oder Enter bestätigt, Rechtsklick oder Escape bricht ab. Die Hinweiszeile unten in der Ansicht listet das während der Arbeit auf.

## Gizmo { #gizmo }

Doppelklick auf ein Element legt ihm ein Gizmo zum Verschieben, Drehen und Skalieren an. So schiebst du einen Vertex sauber entlang einer Achse, und es funktioniert auch in den 2D-Ansichten. Escape blendet es aus und behält die Auswahl.

Das Menü *Mesh* enthält den Rest, etwa Unterteilen, Triangulieren, Verdicken, Spiegeln und glatte Schattierung.
