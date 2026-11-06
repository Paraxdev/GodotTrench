---
source: 2bf892f3f08e81fde13de0d18bb8d2c0a06c8753
---

# Decals { #decals }

Decals bringen Schmutz, Flecken, Risse, Poster und Schilder auf eine Oberfläche. Sie werden einen Hauch näher an der Kamera gezeichnet als die Wand dahinter und darübergeblendet, sodass sie weder an der Wand noch untereinander flackern.

## Ein Decal platzieren { #placing-a-decal }

1. Das Material finden

    Such es im Panel Materialien, das Suchfeld oben links hilft dabei.

2. Alt halten und es auf eine Fläche ziehen

    Die Karte neben dem Cursor erinnert dich daran, dass Alt ein Decal ablegt.

    ![Night/shop_sign wird über eine Backsteinwand gezogen, die Karte neben dem Cursor sagt: Auf eine Fläche ablegen, Shift deckt den ganzen Brush ab, Alt legt ein Decal ab](../assets/decals/decal-alt-drop.png)

3. Loslassen

    Das Decal landet dort, wo du es abgelegt hast, als ein Meter großes Quadrat von 32 mal 32 Einheiten, das aus der Fläche herausschaut.

    ![Das Decal mit dem Ladenschild auf der Backsteinwand](../assets/decals/decal-on-wall.png)

Ein Decal ist ein normales Mesh, du verschiebst, drehst und skalierst es also wie eins. Ist es markiert, zeigt der Inspektor **Decal (überblendet, doppelseitig)** angehakt, dazu die Felder für Position und Größe, um es genau zu platzieren.

![Der Inspektor bei markiertem Decal, Decal (überblendet, doppelseitig) angehakt und seine Größe von 32 mal 32](../assets/decals/decal-inspector.png)

Dasselbe Kontrollkästchen macht aus jedem dünnen Mesh ein Decal, etwa für eines mit eigener Kontur.

Weiche Kanten bleiben weich. Für einen harten Schnitt nimmst du ein Material im Alpha-Scissor-Modus, siehe [Transparenz](../godot/materials.md#transparency).

## Warum kein dünner Brush { #why-not-a-thin-brush }

> [!WARNING]
> Fake keine Decals mit dünnen Brushes oder Meshes, die ein paar Millimeter vor einer Wand stehen. Sie flackern.

Aus der Nähe sieht ein dünner Brush gut aus. Tritt zurück, und er flackert in Streifen durch die Wand, die beim Bewegen wandern. Das nennt man Z-Fighting.

Die GPU findet pro Pixel heraus, welche Fläche am nächsten liegt, indem sie deren Tiefen vergleicht, und sie speichert die Tiefe in einer begrenzten Zahl von Stufen. Die Stufen werden gröber, je weiter eine Fläche entfernt ist und je flacher du an ihr entlangschaust. Ein paar Meter weiter weg landen Brush und Wand auf derselben Stufe, sodass jedes Pixel fast zufällig eine der beiden wählt, und die Wahl ändert sich bei jeder Kamerabewegung. Den Brush weiter von der Wand wegzuschieben schiebt das Flackern nur weiter weg, und aus der Nähe schwebt er dann sichtbar vor der Wand.

Ein Decal hat dieses Problem nicht. Wenn Godot die Map baut, zieht es jedes Decal um 0,05 Prozent seiner Entfernung zur Kamera hin, einen halben Millimeter bei 1 Meter und 5 Zentimeter bei 100 Metern, sodass es in jeder Entfernung vor der Wand bleibt. Der Zug verläuft entlang deiner Blickrichtung, das Decal verschiebt sich auf dem Bildschirm also nicht. Außerdem schreibt es keine eigene Tiefe, sodass sich überlappende Decals ineinander blenden, statt zu kämpfen.

Ein dünnes Mesh ist in Ordnung, sobald du daran **Decal (überblendet, doppelseitig)** anhakst, denn damit wird es ein echtes Decal.

## Projizierte Decals { #projected-decals }

Ein normales Decal ist flach. Für einen Fleck, der über eine Stufe, ein Rohr oder holprigen Boden reicht, nimmst du stattdessen ein projiziertes Decal. Das ist eine Entität, die ein Material auf alles in ihrer Box leuchtet, Props eingeschlossen. Es kostet mehr beim Zeichnen, bleib also auf ebenen Flächen bei normalen Decals.

Es kommt als `infodecal` mit dem Gameplay-Entitätenpaket. [Projizierte Decals](projected-decals.md) zeigt dir, wie du es einrichtest, platzierst und worauf du achten musst.

<div class="td-mcp" markdown>

## MCP { #mcp }

Über den [MCP-Server](../mcp.md) platzierst du ein Decal mit `run_action` `create_decal` und übergibst `material`, `at`, `normal` und `size` als `[width, height]`.

</div>
