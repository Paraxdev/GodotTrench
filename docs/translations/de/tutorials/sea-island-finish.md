---
source: 5d131fe08a12db6615a97a4cbd455746685c5f02
---

# Seeinsel 5: Steg und Godot { #sea-island-5-pier-and-godot }

## Der Steg { #the-pier }

1. Ihm eine eigene Ebene geben

    Klicke oben im Outliner auf **Ebene hinzufügen**. Die neue Ebene wird zur aktuellen, der Steg landet also darauf, und du kannst ihn getrennt vom Terrain ausblenden oder sperren.

    ![Der Outliner mit markiertem Ebene hinzufügen und seinem Tooltip: neue Ebene, sie wird zur aktuellen Ebene, in die neue Objekte kommen](../assets/sea-island/outliner-add-layer.png)

2. Bauen

    Drücke **Q** für das Werkzeug Auswählen, denn ein Ziehen mit dem Scatter-Werkzeug malt weiter Modelle. Zeichne dann einfache Brushes, die von der Mitte des Strandes aufs Meer hinausführen, in den Maßen der Tabelle unten.

3. Props hinzufügen

    Gehe zu *Datei > Importieren > Modell-Prop (.bbmodel, .glb)…* und wähle Modelle aus dem Ordner `models/polyhaven` des Demo-Projekts: zwei `containers/wooden_crate_01` und ein `containers/wine_barrel_01` auf dem Deck, ein `nautical/lifebuoy` an einem Pfosten und eine `nautical/ocean_buoy` im Wasser. Jedes landet am 3D-Cursor, schiebe es von dort an seinen Platz.

    ![Das Datei-Menü mit geöffnetem Importieren und markiertem Modell-Prop (.bbmodel, .glb)…](../assets/sea-island/menu-import-model-prop.png)

| Teil | Größe | Material |
| --- | --- | --- |
| Deck | 128 breit, 1100 lang, 8 dick, Oberseite 48 über dem Wasser | `showcase/planks` |
| Pfosten | 16 Einheiten im Quadrat, zwei Reihen alle 200 Einheiten, von 320 unter dem Wasser bis zum Deck | `showcase/beam` |
| Geländer | Zwei kurze Pfosten am äußeren Ende, 40 hoch | `showcase/beam` |

Ein Prop verweist über einen `res://`-Pfad auf sein Modell, die Datei muss also im geöffneten Godot-Projekt liegen. Wählst du eine von woanders, bietet der Editor an, sie zuerst nach `res://models` zu kopieren.

![Der Steg vom Meer aus](../assets/sea-island/pier.jpg)

## In Godot ansehen { #see-it-in-godot }

Speichere die Karte in deinem Godot-Projekt und baue sie mit einer `FuncGodotMap`, siehe [Karten bauen](../godot/building.md). Jeder Teil der Karte wird zu einer anderen Art von Node:

| Kartenteil | Was du in Godot bekommst |
| --- | --- |
| Terrain | Ein `TrenchDotTerrain`, in Chunks geteilt, jeder ein Mesh, das die vier Ebenen mischt und eigene Kollision hat |
| Meer | Der `func_illusionary`-Brush, sichtbar ohne Kollision |
| Baum-, Busch- und Findling-Sets | MultiMeshes, dazu pro Modell ein statischer Körper mit einer Kollisionsform für jede Instanz, alle teilen sich eine Form |
| Gras-Sets | MultiMeshes ohne Kollision, die mit der Entfernung ausblenden |

![Die fertige Insel in Godot](../assets/sea-island/godot-overview.jpg)

![Der Steg in Godot](../assets/sea-island/godot-pier.jpg)
