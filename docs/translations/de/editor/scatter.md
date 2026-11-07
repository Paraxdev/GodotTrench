---
source: ba7a98139fdce301c2c6a3f48c26f62837b5d180
---

# Scatter { #scatter }

Das Werkzeug **Scatter** (B) malt Bäume, Felsen und Gras auf Flächen deiner Wahl, sodass du einen Hang in Sekunden bedeckst, statt jedes Modell von Hand zu platzieren. Das [Tutorial zur Meeresinsel](../tutorials/sea-island-scatter.md) führt dich durch ein vollständiges Setup.

![Mit dem Scatter-Werkzeug wird ein Waldstreifen quer über eine Insel gemalt](../assets/scatter/scatter-paint.gif)

## Scatter-Sets { #scatter-sets }

Alles, was du streust, liegt in einem **Scatter-Set** auf einer eigenen Ebene, zum Beispiel ein Set für Bäume und eins für Gras. Drück B, dann öffnet sich rechts das Scatter-Panel mit einer Karte für jedes Modell im aktiven Set.

![Das Scatter-Panel mit aktivem Set Büsche: das Set-Menü, +, Voreinstellung, Umbenennen und eine Karte pro Modell](../assets/scatter/scatter-panel.png)

| Wenn du | Dann |
| --- | --- |
| Ein leeres Set anfangen willst | **+** neben dem Set-Menü |
| Mit einer Voreinstellung anfangen willst | **Voreinstellung**, dann eins der zehn eingebauten Sets wählen |
| Modelle hinzufügen willst | Zieh sie aus dem Panel Modelle herein, oder **Modelle hinzufügen…**. **Gewählte Props hinzufügen** nimmt die Modelle der ausgewählten Entitäten |
| Ein Modell nicht mehr malen willst | Hak **malen** auf seiner Karte ab. Was es schon platziert hat, bleibt |
| Ein Modell abstimmen willst | Klick auf den Pfeil rechts an seiner Karte |

![Das Menü Voreinstellung geöffnet mit Wald, Kiefern, Mischwald, Detailwald, Low-Poly-Bäume, Büsche, Unterholz, Felsen, Findlinge und Gras, der Mauszeiger auf Felsen](../assets/scatter/preset-menu.png)

Die Voreinstellungen Felsen, Findlinge, Gras, Unterholz und Low-Poly-Bäume nutzen Blockbench-Modelle, die in den Editor eingebaut sind. Wählst du eine, landen die Modelle, die sie braucht, offline in `res://trenchdot/nature`. Die Voreinstellungen Wald, Kiefern, Mischwald, Detailwald und Büsche nutzen die glTF-Bäume und -Büsche des Natur-Pakets, einen Download von etwa 23 MB. In einem Projekt ohne das Paket öffnet die Wahl einer davon *Godot > Inhalt zum Projekt hinzufügen…* bei den Natur-Modellen, statt ein Set anzulegen, dessen Modelle fehlen. *Terrain > Scatter > Natur-Modelle installieren…* oder **Natur-Modelle installieren…** unten im Panel öffnet es ebenfalls. Dateien, die schon im Projekt liegen, werden nie überschrieben.

### Modellkarten { #model-cards }

Klapp eine Karte über ihren Pfeil auf, um dieses eine Modell abzustimmen.

![Eine aufgeklappte Karte für bush_round mit Skalieren, Abstand, Ausrichten, Neigung, Einsinken, Zufällige Drehung und Material](../assets/scatter/model-card.png)

| Einstellung der Karte | Was sie tut |
| --- | --- |
| Gewicht | Wie oft dieses Modell im Vergleich zu den anderen im Set gewählt wird |
| Skalieren | Eine zufällige Größe zwischen den beiden Werten |
| Abstand | Der kleinste Abstand in Karteneinheiten zu jeder anderen Instanz |
| Ausrichten | 0 stellt das Modell aufrecht, 1 neigt es so, dass es der Oberfläche folgt |
| Neigung | Die größte zufällige Schräglage in Grad |
| Einsinken | Wie weit das Modell in den Boden gedrückt wird, um das untere Ende eines Stamms oder Felsens zu verstecken |
| Zufällige Drehung | Dreht jede Instanz in eine zufällige Richtung |
| Material | Wird anstelle der eigenen Materialien des Modells gezeichnet. Bleibt es leer, gilt das Material des Sets, falls es eins hat |

## Ziele { #targets }

Ein Set landet nur auf seinen **Zielen**, ein auf ein Terrain gemalter Wald bedeckt also nie das Haus darauf. Alt+Klick in einer Ansicht, oder ein Klick auf die Pipette rechts bei *Ziele* und dann ein Klick in einer Ansicht, fügt ein Ziel hinzu oder entfernt es. Ein Set ohne Ziele nimmt, was sein erster Strich trifft. Auch ein anderes Set kann ein Ziel sein, so wächst Gras auf Findlingen.

![Der Abschnitt Ziele des Scatter-Panels mit dem Terrain als Ziel, die Pipette oben rechts](../assets/scatter/targets.png)

Hakst du **Cursor folgen** an, ignoriert das die Ziele und malt auf jede Fläche oder jedes gestreute Prop unter dem Brush.

## Malen { #painting }

Die schnellen Einstellungen sitzen links oben in der Optionsleiste des Werkzeugs: das Set, **Malen** oder **Löschen**, Radius, Dichte, max. Neigung, **Cursor folgen**, die Pipette und **Füllen**. **Scatter-Panel** holt das Panel zurück, falls du es geschlossen hast.

![Die Optionsleiste des Scatter-Werkzeugs mit dem Set Büsche, Malen, Radius 512 u, Dichte 1,00 und max. Neigung 40°](../assets/scatter/scatter-toolbar.png)

| Eingabe | Aktion |
| --- | --- |
| Links ziehen | Malen |
| Shift+Links ziehen | Löschen, nur aus dem aktiven Set |
| Ctrl+Mausrad | Die Brush-Größe ändern |
| **Ziele füllen** | Jedes Ziel auf einmal bedecken, innerhalb der Grenzen für Gefälle und Höhe |

Der Abschnitt *Brush* des Panels enthält den Rest.

![Der Abschnitt Brush des Scatter-Panels: Radius, Dichte, Gefälle, Höhe, Abfall, Nur Freiliegendes, Löschmenge, Seed, Ausgabe und die Kontrollkästchen darunter](../assets/scatter/brush-settings.png)

**Dichte** ist, wie viele Platzierungen der Brush pro 64 mal 64 Einheiten versucht, und der Abstand jedes Modells begrenzt, wie viele wirklich passen. **Nur Freiliegendes** überspringt Punkte, über denen Geometrie liegt, das hält Gras aus dem Schatten von Balkonen und Dächern fern. **Abstand zu anderen Sets halten**, standardmäßig an, verhindert, dass ein Busch durch den Stamm eines anderen Sets wächst.

## In Godot { #in-godot }

Die **Art** eines Sets entscheidet, was Godot baut.

| Art | Gebaut als |
| --- | --- |
| Props | MultiMeshes mit Kollision und Schatten. Modelle, deren Szenen Skripte tragen, werden einzeln instanziiert, damit die Skripte laufen |
| Foliage | MultiMeshes ohne Kollision und Schatten, standardmäßig ab 2400 Einheiten ausgeblendet. Gedacht für Gras und kleine Pflanzen |

Wähl das Set im Outliner aus, und seine Zeile **Kollision** im Inspektor bestimmt die Form, mit der jede Instanz kollidiert. Neue Props-Sets starten auf konvex, Foliage-Sets auf keine.

| Kollision | Form pro Instanz |
| --- | --- |
| keine | Keine Kollision |
| Box | Eine Box um die Instanz |
| konvex | Eine konvexe Hülle um die Instanz |
| Zerlegung | Mehrere konvexe Hüllen, die konkaven Modellen wie einem Bogen folgen, langsamer zu bauen |
| Trimesh | Jede Fläche des Modells, exakt, aber am teuersten |

Wenn du einzelne Props bearbeiten willst, ersetzt **In Entitäten** unten im Scatter-Panel, oder *Terrain > Scatter > Scatter-Sets in Entitäten* für die ausgewählten Sets, ein Set durch eine `prop_model`-Entität pro Instanz.

![Der untere Teil des Scatter-Panels mit Art, Malen, Löschen, Ziele füllen, Instanzen löschen, In Entitäten und Natur-Modelle installieren…](../assets/scatter/panel-buttons.png)
