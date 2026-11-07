---
source: be4d0912557053b84612829a7948190773c48b4d
---

# Seeinsel 1: Terrain { #sea-island-1-terrain }

Eine große Insel mit zwei Hügeln, Klippen, einem Strand, Pfaden, Wald und einem Steg, gebaut in fünf Teilen. Die Zahlen sind die, die für die Bilder verwendet wurden, in Karteneinheiten bei 32 pro Meter.

![Die fertige Insel von Süden](../assets/sea-island/island-finished.jpg)

## Das Terrain erstellen { #create-the-terrain }

Gehe in der Menüleiste zu *Terrain > Terrain erstellen…*.

![Das Terrain-Menü mit markiertem Terrain erstellen…](../assets/sea-island/menu-create-terrain.png)

Fülle den Dialog so aus:

| Einstellung | Wert und warum |
| --- | --- |
| Auflösung, Zellgröße | 257 x 257 Punkte, 64 Einheiten Abstand. Die Zeile Größe darunter zeigt, was du bekommst, 16384 Einheiten (512 m) |
| Form | Insel, Höhe 1600, Merkmalsgröße 3000, Erosion 20, Seed 1. Insel steigt in der Mitte an und fällt zu den Rändern ab. Die Merkmalsgröße ist die Breite der größten Buckel, und Erosion lässt steile Hänge abrutschen, damit sie verwittert aussehen |
| Basisebene, Ebene für Hänge, Ebene für Gipfel, Ebene für Senken | `showcase/grass`, `showcase/cliff`, `showcase/dirt`, `showcase/sand`. Kachel 256 für die ersten beiden, 192 für die anderen. Die Basis deckt alles ab, was du nicht übermalst, die anderen sind für steiles, hohes und tiefes Gelände gedacht |
| Mitte | Hake **3D-Cursor** ab, dann `0 -576 0` |
| Ebenen nach Neigung und Höhe bemalen | Nicht angehakt, Teil 3 malt von Hand |

(Der Kachelwert gibt an, wie viele Einheiten eine Wiederholung der Textur abdeckt.)

![Der ausgefüllte Dialog Terrain erstellen: 257 x 257, Zellgröße 64, Insel, Höhe 1600, Merkmalsgröße 3000, Erosion 20, Seed 1, die vier showcase-Ebenen, Mitte 0 -576 0 und Ebenen nach Neigung und Höhe bemalen nicht angehakt](../assets/sea-island/create-terrain-dialog.png)

Klicke unten auf **Erstellen**. Du solltest eine grüne Insel sehen, die schon gewählt ist, und der Editor wechselt für dich zum Werkzeug Modellieren.

Das Meer liegt auf Höhe 0, wenn du das Terrain also um 576 absenkst, wird sein äußerer Ring geflutet und die Mitte bleibt trocken. Spätere Teile nennen die Ebenen mit ihrer Slotnummer: 0 Gras, 1 Klippe, 2 Erde, 3 Sand.

## Das Meer hinzufügen { #add-the-sea }

Das Meer ist ein flacher Brush, in den der Spieler hineinlaufen kann.

1. Eine riesige Box zeichnen

    Drücke **Q** für das Werkzeug Auswählen und **Escape**, damit das Terrain nicht mehr gewählt ist, sonst modelliert oder verschiebt dein Ziehen es. Ziehe dann in der Ansicht Oben eine Box auf, die um ein Vielfaches größer ist als das Terrain, sonst zeigt sich ihr Rand am Horizont.

2. Flach machen

    Mache sie in der Ansicht Vorne 32 Einheiten dick, mit der Oberseite auf Höhe 0.

3. Zu Wasser machen

    Klicke bei gewählter Box im Tab Materialien auf `showcase/water`. Klicke dann mit Shift auf das Wasser in der 3D-Ansicht, um nur die Oberseite zu wählen, und setze *Skalierung* im Abschnitt *Ausrichtung* des Inspektors auf 8 und 8. So wiederholt sich die Wassertextur nicht alle paar Meter.

    ![Der Inspektor mit 1 gewählten Flächen, Material showcase/water und Ausrichtung Skalierung auf 8.00 und 8.00](../assets/sea-island/inspector-alignment-scale.png)

4. Den Spieler hineinwaten lassen

    Klicke einmal auf das Wasser, weg von der Insel, damit wieder die ganze Box gewählt ist statt nur ihrer Oberseite. Klicke sie dann mit der rechten Maustaste an und wähle *Brush-Entität erstellen > func_illusionary*. Sie wird ohne Kollision gerendert, der Spieler kann also ins Wasser laufen.

Jetzt solltest du eine Insel im Meer haben, bereit zum Modellieren:

![Die erzeugte Insel im Meer, vor dem Modellieren](../assets/sea-island/fresh-island.jpg)

Weiter: [Seeinsel 2: Modellieren](sea-island-sculpt.md).

<div class="td-mcp" markdown>

## MCP { #mcp }

Der [MCP-Server](../mcp.md) lässt Agenten und Skripte den Editor steuern.
[examples/mcp/sea_island.json](https://github.com/Paraxdev/TrenchDot/blob/main/examples/mcp/sea_island.json) ist ein
MCP-Skript, das dieselbe Insel baut, du kannst es also abspielen, um einen Wert zu prüfen.

</div>
