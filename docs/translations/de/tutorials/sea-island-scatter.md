---
source: 4d58791b780474701e360cec324c41c44739f30d
---

# Seeinsel 4: Scatter { #sea-island-4-scatter }

Beim Scattern setzt du viele Bäume, Büsche, Steine oder Gräser mit einem Pinsel, statt sie einzeln zu platzieren. Jede Art von Bepflanzung unten ist ein eigenes Scatter-Set, und ein Set wächst nur auf seinen Zielen, den Flächen, die du dafür wählst.

## Ein Set einrichten { #set-up-a-set }

Jedes Set auf dieser Seite beginnt gleich.

1. Das Scatter-Werkzeug wählen

    Drücke **B** oder klicke in der Werkzeugleiste auf die Schaltfläche Scatter. Rechts neben dem Inspektor öffnet sich der Tab Scatter. (Schließt du ihn, holt ihn **Scatter-Panel** in der Werkzeugoptionsleiste zurück.)

    ![Die Werkzeugleiste mit markierter Schaltfläche Scatter-Werkzeug und ihrem Tooltip: ziehen, um die Modelle des aktiven Scatter-Sets zu malen](../assets/sea-island/toolbar-scatter.png)

2. Mit einer Voreinstellung beginnen

    Klicke oben im Scatter-Panel auf **Voreinstellung** und wähle die Voreinstellung, die der Schritt nennt.

    ![Das Scatter-Panel mit geöffnetem Menü Voreinstellung und markiertem Mischwald](../assets/sea-island/scatter-preset-menu.png)

3. Das Terrain als Ziel setzen

    Klicke auf die Pipette neben **Ziele**, dann auf das Terrain in der 3D-Ansicht. Du solltest es unter Ziele aufgelistet sehen, als `terrain #2` oder ähnlich.

    ![Die Zeile Ziele des Scatter-Panels mit markierter Pipette und ihrem Tooltip](../assets/sea-island/scatter-eyedropper.png)

Lass **Cursor folgen** nicht angehakt, denn es malt auf alles unter dem Pinsel und ignoriert die Ziele. Lass **Abstand zu anderen Sets halten** angehakt, damit die Sets Abstand zueinander halten und nichts durch einen Stamm wächst.

## Wald { #woodland }

1. **Voreinstellung** > *Mischwald*, dann klicke oben auf **Umbenennen** und nenne es `woodland`. Setze das Terrain als Ziel.
2. Stelle im Abschnitt Brush **Dichte** 0.16 und **Gefälle** 0 bis 30 ein und hake **Höhe** an, 120 bis 2000. Der Gefällebereich hält Bäume von den Klippen fern, der Höhenbereich vom Strand.

    ![Das untere Scatter-Panel: terrain #2 als Ziel, Cursor folgen nicht angehakt, Dichte 0.16, Gefälle 0 bis 30, Höhe angehakt von 120 bis 2000, Abstand zu anderen Sets halten angehakt und die Schaltfläche Ziele füllen markiert](../assets/sea-island/scatter-woodland-settings.png)

3. Klicke unten im Panel auf **Ziele füllen**, um das ganze Terrain auf einmal zu bepflanzen.

Die Dichte zählt Versuche pro 64 x 64 Einheiten, 0.16 ist also etwa ein Baum alle 5 m.

![Der Wald nach Ziele füllen](../assets/sea-island/scatter-woodland.jpg)

Halte **Shift** gedrückt und ziehe mit Radius 260 an jedem Pfad entlang, um die Bäume dort zu löschen. Lösche eine Lichtung auf der Spitze des Haupthügels (Radius etwa 520) und eine Wiese hinter dem Strand (etwa 700).

![Bäume vom Pfad löschen](../assets/sea-island/scatter-erase-paths.jpg)

## Hero-Bäume, Büsche und Findlinge { #hero-trees-bushes-and-boulders }

Jede Zeile ist ein neues Set aus der genannten Voreinstellung, wie beim Wald auf das Terrain gerichtet. Hero-Bäume sind einige große, detaillierte Bäume, die dort stehen, wo der Spieler genau hinsieht.

| Set | Voreinstellung | Wie du es platzierst | Radius | Dichte | Gefälle | Höhe |
| --- | --- | --- | --- | --- | --- | --- |
| Hero-Bäume | Detailwald | Einzelne Klicks dort, wo die Pfade den Strand verlassen, und rund um die Lichtung auf der Spitze | 260 | 0.08 | 0 bis 30 | aus |
| Büsche | Büsche | **Ziele füllen**, dann mit Radius 190 an den Pfaden löschen | | 0.35 | 0 bis 32 | 45 bis 200 |
| Findlinge | Findlinge | Ein Strich entlang der Oberkante der Klippen | 1100 | 0.035 | 12 bis 50 | 20 bis 2000 |

![Hero-Bäume am Fuß des Pfades](../assets/sea-island/scatter-hero-trees.jpg)

![Büsche in einem Band über dem Sand](../assets/sea-island/scatter-bushes.jpg)

![Findlinge an den Klippen](../assets/sea-island/scatter-boulders.jpg)

## Gras { #grass }

1. Beginne ein Set aus der Voreinstellung *Gras*. Gras ist Laub, gezeichnet ohne Kollision und Schatten, es kann also dicht sein, ohne das Spiel auszubremsen.
2. Male die offenen Wiesen mit Radius 650, Dichte 2.5, Gefälle 0 bis 28, Höhe 45 bis 2000. Lösche es an den Pfaden mit Radius 170.

    ![Wiesengras hinter dem Strand](../assets/sea-island/scatter-meadow.jpg)

3. Für Gras auf den Findlingen beginne noch ein Set *Gras*. Klicke mit der Pipette auf einen Findling statt auf das Terrain, damit das Findling-Set sein Ziel wird. Du solltest es unter Ziele mit `(set)` hinter dem Namen sehen. Dann **Ziele füllen** mit Dichte 6 und Gefälle 0 bis 45.

    ![Die Zeile Ziele mit cliff boulders (set) als Ziel](../assets/sea-island/scatter-target-boulders.png)

> [!NOTE]
> Füllen setzt Gras auf die Findlinge, wie sie in diesem Moment sind, schließe also zuerst die Findlinge ab.

![Gras wächst auf einem Findling](../assets/sea-island/scatter-boulder-grass.jpg)

Weiter: [Seeinsel 5: Steg und Godot](sea-island-finish.md).
