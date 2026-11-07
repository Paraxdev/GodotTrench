---
source: be5056a246dacf54d3a4aecafff90bff3f0d1aad
---

# Prefab-Bibliothek { #prefab-library }

Der Tab **Prefabs** unten, neben Materialien und Modellen, enthält fertige Teile zum Einfügen in eine Karte: Treppen, Türöffnungen, Säulen, kleine Räume und Details. Stell dir die Prefab-Sammlungen vor, aus denen TrenchBroom-Mapper kopieren, nur direkt in den Editor eingebaut.

![Der Tab Prefabs mit Prefabs suchen, der Kategorieliste, dem Regler für die Kartengröße und einer Karte für jedes eingebaute Teil, die Switchback-Treppe mit Mauszeiger und ihrem Tooltip: Stairs/Switchback stair, 192 × 128 × 352 Einheiten, Eingebaut](../assets/prefab-library/prefabs-panel.png)

Jede Karte zeichnet ihr Teil von der Seite, die am meisten zeigt: von oben bei Grundrissen wie einer Treppe, von vorn bei Wänden wie einer Türöffnung. Heller heißt näher.

## Ein Teil einfügen { #pasting-a-piece }

1. Auf die Stelle zeigen

    Zeig in einer Ansicht auf die Stelle, an die das Teil soll, das setzt den 3D-Cursor.

2. Auf die Karte klicken

    Tipp in *Prefabs suchen* oder wähl eine Kategorie, um es zu finden, und klick dann darauf. Das Teil wird um diese Stelle zentriert eingefügt und am Raster eingerastet, wie bei Ctrl+V.

    ![Die Switchback-Treppe in einen kleinen Raum eingefügt, ihre Stufen in den grauen Dev-Materialien](../assets/prefab-library/pasted-switchback-stair.png)

Nur die Stelle im Grundriss folgt dem Cursor. Die Höhe bleibt so, wie das Teil gespeichert wurde, in einem oberen Stockwerk hebst du es also danach an.

Das Einfügen ist eine schlichte Kopie ohne Verbindung zurück zur Bibliothek, du kannst sie also frei umformen oder neu texturieren. Die eingebauten Teile nutzen die `dev/`-Materialien, die jedes Projekt hat, texturier sie im Tab Materialien neu.

Ein Rechtsklick auf eine Karte gibt *Einfügen* und *Kopieren*. Kopieren legt das Teil in die Zwischenablage, damit du es in eine andere Karte einfügen oder als Text teilen kannst.

Um eine ganze Fläche mit zufällig gewählten Teilen zu bedecken, nimm [Zufällig füllen](random-fill.md).

> [!TIP]
> Ein Teil ist nach dem Einfügen ganz normaler Karteninhalt. Für etwas, das überall synchron bleiben soll, wo es benutzt wird, etwa ein Fensterrahmen, der sich über ein ganzes Gebäude wiederholt, mach stattdessen eine [Prefab-Instanz](organizing.md#prefabs).

## Eigene Teile { #your-own-pieces }

1. Die Objekte auswählen

    Wähl aus, was das Teil werden soll, bei geöffnetem Godot-Projekt.

2. Die Auswahl hinzufügen

    Öffne **Auswahl hinzufügen** oben im Tab Prefabs, gib dem Teil einen Namen und eine Kategorie und klick auf **Hinzufügen**.

    ![Auswahl hinzufügen geöffnet, mit Grand stair als Name, Stairs als Kategorie, Position behalten nicht angehakt und dem Tooltip von Hinzufügen: Speichert die Auswahl als .tdm in res://prefab_library, ihre untere Mitte auf dem Ursprung](../assets/prefab-library/add-the-selection.png)

3. Die neue Karte finden

    Sie erscheint zuerst, mit blauem Rahmen. Ein Rechtsklick darauf gibt *Zum Bearbeiten öffnen*, das das Teil als Karte öffnet, und *Im Dateimanager zeigen*. Eingebaute Teile lassen sich nicht bearbeiten.

    ![Rechtsklickmenü auf der Karte Grand stair mit Einfügen, Kopieren, Zum Bearbeiten öffnen mit Mauszeiger und Im Dateimanager zeigen](../assets/prefab-library/card-menu.png)

Das Teil wird als `.tdm` in `res://prefab_library/<category>/` gespeichert, verschoben, sodass die untere Mitte der Auswahl auf dem Ursprung liegt. Hak **Position behalten** an, damit es stattdessen bleibt, wo es ist, für ein Teil, das du mit Absicht um den Ursprung der Karte gebaut hast, etwa die Teile eines Bausatzes für [Zufällig füllen](random-fill.md).

Jede `.tdm` in diesem Ordner ist ein Teil. Speichere Karten selbst dort mit *Datei > Speichern unter…*, und sie erscheinen sofort als Karten. Für Dateien, die du von Hand hineinkopiert hast, oder einen Ordner mit Teilen, der zwischen Projekten geteilt wird, drück die Schaltfläche ⟳.

## Aus TrenchBroom einfügen { #pasting-from-trenchbroom }

TrenchDot fügt auch ein, was TrenchBroom in die Zwischenablage legt. Kopier Brushes in TrenchBroom oder aus einer TrenchBroom-Prefab-Sammlung im Web und drück Ctrl+V in einer Karte. Gruppen kommen als Gruppen an, Entitäten als Entitäten, und die Achsen werden genauso gedreht wie beim [Importieren einer .map](importing.md). Texturnamen bleiben erhalten, die Werkzeugtexturen anderer Spiele werden durch die des Projekts ersetzt.

<div class="td-mcp" markdown>

## MCP { #mcp }

Das Tool `prefab_library` listet Teile auf (`op: list`), fügt eines über sein `category/name` ein (`op: paste`, mit `origin` oder `offset`), gibt seinen Zwischenablagetext zurück (`op: copy`) und fügt der Bibliothek des Projekts Objekte hinzu (`op: add`, die Auswahl, sofern keine `ids` angegeben sind). `run_action` mit `paste` akzeptiert auch TrenchBroom-Zwischenablagetext, nicht nur den von TrenchDot.

</div>
