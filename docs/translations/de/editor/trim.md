---
source: 3cc68efb1c3e922c9ae3811f442a5d5e642414b6
---

# Trim { #trim }

*Brush > Trim hinzufügen…* legt Streifen aus Brushes an die Kanten von Flächen, die von ihnen abstehen. Am unteren Rand von Wänden sind das Fußleisten, oben Stuckleisten, um einen Boden herum eine Einfassung, um ein Becken herum sein Beckenrand. Hier siehst du einen schlichten Raum vor und nach einem einzigen Klick:

![Ein verputzter Raum mit einer Türöffnung und markierten Wänden, noch ohne Trim](../assets/trim/room-before.png)

![Derselbe Raum mit dunklen Fußleisten am unteren Rand der Wände, die bis zur Türöffnung laufen](../assets/trim/room-after.png)

1. (Optional) Das Trim-Material wählen

    Drück Escape, damit nichts markiert ist, und klick dann im Panel Materialien auf das Material. Es wird zum aktuellen Material, ohne dass sich die Map ändert.

2. Die Wände markieren

    Jede senkrechte Fläche der markierten Brushes bekommt Trim. Für einzelne Flächen klickst du sie stattdessen mit Shift an, mit Ctrl+Shift+Klick fügst du weitere hinzu oder nimmst sie wieder heraus.

3. Den Dialog öffnen

    Wähl *Brush > Trim hinzufügen…*. Er startet mit dem aktuellen Material, mit **Aktuelles nehmen** wechselst du später wieder darauf. Unten steht, wie viele Flächen er gefunden hat.

4. Den Streifen einstellen

    Wähl, welche **Kanten** ihn bekommen: unten, oben, die Seiten oder alle. **Höhe** ist, wie weit der Streifen über die Fläche läuft, **Tiefe**, wie weit er absteht, und mit **Einzug** bleibt zwischen Kante und Streifen ein Abstand.

    ![Der Dialog Trim hinzufügen mit Kanten auf unten, Höhe 6 u, Tiefe 2 u, Einzug 0 u, showcase/planks_dark als Material, angehaktem Nur wo ein Boden unter der Kante liegt, ausgeschalteter Kollision und 20 markierten Flächen](../assets/trim/add-trim-dialog.png)

5. Klick auf **Trim hinzufügen**

    Die Statusleiste sagt, wie viele Streifen entstanden sind.

Solange **Kollision** aus ist, der Standard, landen die Streifen in einer `func_detail_illusionary`, damit Spieler nicht an ihnen hängen bleiben. Hak die Option an oder nimm ein Spiel ohne diese Entität, dann landen sie stattdessen in einer einfachen Gruppe.

Wo zwei Trim-Flächen aufeinandertreffen, läuft ein Streifen bis in die Ecke und der andere hört an ihm auf, sodass sich Ecken schließen, ohne dass sich zwei Streifen überlappen. Am Ende einer frei stehenden Wand oder einer Türöffnung läuft der Streifen um die Ecke herum. Ein Streifen, der in einem anderen Brush landen würde, etwa an den Enden eines Sturzes zwischen zwei Wandstücken, wird weggelassen.

**Nur wo ein Boden unter der Kante liegt** beschränkt Trim unten auf Kanten, die auf etwas stehen, damit die Unterseite eines Sturzes über einer Türöffnung keine Fußleiste bekommt.

> [!TIP]
> Trim besteht aus ganz normalen Brushes. Du kannst einzelne Streifen danach wie jeden anderen Brush neu texturieren, löschen oder umformen.

<div class="td-mcp" markdown>

## MCP { #mcp }

Das Tool `trim` nimmt `faces` als `[[brush, face], ...]`, oder Brush-`ids`, oder die Auswahl, dazu `edges`, `height`, `depth`, `inset`, `material`, `on_floor` und `collision`. Es liefert die neue Gruppe oder Entität zurück und wie viele Streifen es angelegt hat.

</div>
