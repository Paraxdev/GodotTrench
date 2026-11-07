---
source: 7bb83afeb1e5ebd8423b507ac79851c690337899
---

# Zufällig füllen { #random-fill }

*Brush > Zufällig füllen…* bedeckt ein Raster aus Zellen mit Teilen aus der [Prefab-Bibliothek](prefab-library.md) und wählt das Teil jeder Zelle nach Gewicht. Bau dir einen kleinen Bausatz aus Teilen, etwa einem Büroraum, einem Flur, einem Säulenraster und einer offenen Fläche, und die Füllung legt daraus einen ganzen Grundriss aus. Jeder Seed gibt einen anderen Plan, und derselbe Seed gibt immer denselben.

1. Deine Teile bauen

    Bau jedes Teil eine Zelle breit (standardmäßig 128 Einheiten), zentriert auf dem Ursprung. Halt Wände vom Rand der Zelle fern, damit Teile nebeneinander immer verbunden sind.

    Füg es dann der Bibliothek hinzu: Wähl es aus, öffne **Auswahl hinzufügen** oben im Tab Prefabs, hak **Position behalten** an, gib einen Namen ein und klick auf **Hinzufügen** (siehe [Eigene Teile](prefab-library.md#your-own-pieces)).

    ![Der Tab Prefabs mit geöffnetem Auswahl hinzufügen und angehaktem Position behalten, sein Tooltip sagt, dass der Kartenursprung als Ursprung des Teils behalten wird, wie bei einem Bausatz für Zufällig füllen](../assets/random-fill/prefabs-keep-position.png)

2. Den Dialog öffnen

    Geh auf *Brush > Zufällig füllen…*.

    ![Das Menü Brush mit dem Mauszeiger auf Zufällig füllen…](../assets/random-fill/menu-random-fill.png)

3. Die Teile hinzufügen

    Wähl jedes Teil über **Teil hinzufügen…** und gib ihm ein **Gewicht**: Ein Teil mit Gewicht 6 füllt etwa doppelt so viele Zellen wie eins mit Gewicht 3. **drehen** lässt ein Teil sich um ein zufälliges Vielfaches von 90 Grad drehen.

    ![Der Dialog Zufällig füllen mit drei Teilen, ihren Gewichten, Gewicht leer 4, einem Raster von 12 mal 8 und seiner farbigen Vorschau](../assets/random-fill/random-fill-dialog.png)

4. (Optional) Manche Zellen offen lassen

    **Gewicht leer** wirkt wie ein Teil, das nichts platziert.

5. Das Raster platzieren

    Wähl den Boden aus und klick auf **An Auswahl anpassen**, oder stell die Größe der **Zelle**, die Anzahl der **Zellen** und die **Ecke** selbst ein.

6. (Optional) Die Teile gruppieren

    **Gruppierung** sammelt jedes Teil in Flecken von etwa **Fleckengröße** Zellen, sodass ein Flügel mit Büros neben einer Halle mit Säulen liegt, statt dass alles gleichmäßig gemischt ist.

7. Füllen

    Das farbige Raster zeigt das Layout vorab, eine Farbe pro Teil. Klick auf **Füllen**, und **Neu würfeln** ersetzt die letzte Füllung durch den nächsten Seed.

    ![Ein Boden, gefüllt mit eckigen und runden Säulen und Kistenstapeln, einige Zellen bleiben leer](../assets/random-fill/random-fill-result.jpg)

Die Füllung ist eine Gruppe aus schlichten Kopien, sie lässt sich also backen und bauen wie alles, was du von Hand platzierst, und du kannst danach jede Zelle bearbeiten. Die Entitätsnamen jeder Kopie bekommen ein eigenes Präfix, `rf<seed>_<n>-`, sodass ein Teil mit verdrahteten Entitäten, etwa eine Lampe, die per Timer flackert, in jeder Zelle für sich funktioniert.

> [!TIP]
> Füll verschiedene Arten von Details in getrennten Durchgängen über dasselbe Raster, zum Beispiel eine Füllung für die Wände und eine für die Deckenlampen, jeweils mit eigenen Gewichten, etwa ein paar kaputte oder flackernde Lampen unter funktionierenden.

<div class="td-mcp" markdown>

## MCP { #mcp }

Das Tool `random_fill` nimmt `pieces` als `[{name, weight, rotate}]`, `empty`, `min`, `cell`, `cells`, `seed`, `cluster` und `cluster_size`. `preview: true` gibt das Layout nur als Skizze zurück, `replace` entfernt zuerst eine frühere Füllung.

</div>
