---
source: cb5ad8fa1eb282094d0cdec538d72382b939f070
---

# Beleuchtung backen { #baking-lighting }

Beim Backen wird das Licht von Sonne, Himmel, deinen Lampen und leuchtenden Materialien einmal ausgerechnet, Abpraller eingeschlossen, und als Lightmap in der Karte gespeichert. Godot zeichnet dann dieses Licht, statt es in jedem Frame zu berechnen. Du bekommst weiches Abprall-Licht, farbiges Streulicht von Lampen und dunklere Ecken fast ohne Kosten zur Laufzeit. Lichter, die du schalten oder bewegen willst, bleiben neben dem Backen in Echtzeit.

![Links: ein Raum in der Beleuchteten Vorschau, flaches Grau mit hartkantigen Sonnenflecken unter den Fenstern. Rechts: derselbe Raum nach dem Backen, warmes Lampenlicht, dunklere Ecken und weiche Sonnenflecken](../assets/light-baking/lit-preview-and-baked.png)

Das Backen deckt statische Geometrie ab: Welt-Brushes, Meshes, Terrains, Displacements und Brush-Entitäten, die sich nicht bewegen. Bewegte Brush-Entitäten wie Türen und Plattformen, Trigger-Volumen, Decals, Werkzeugtexturen und vom Export ausgenommene Ebenen werden weder vom Backen beleuchtet noch werfen sie gebackene Schatten.

## Eine Karte backen { #baking-a-map }

1. Das Fenster Beleuchtung backen öffnen

    Wähl *Godot > Beleuchtung backen…*.

    ![Das Menü Godot mit dem Mauszeiger auf Beleuchtung backen und seinem Tooltip](../assets/light-baking/menu-bake-lighting.png)

2. Prüfen, was hineingeht

    Die oberste Zeile fasst es zusammen, zum Beispiel "Bäckt die Sonne, 1 Licht, den Himmel und leuchtende Materialien in 102 Flächen", und sagt, welche Lichter in Echtzeit bleiben.

    ![Das Fenster Beleuchtung backen mit Qualität auf Mittel, Texelgröße 16,0 u, Schattenweichheit 0,30 und Läuft auf CPU](../assets/light-baking/bake-dialog.png)

3. Eine Qualität und eine Texelgröße wählen

    Siehe [die Optionen](#dialog-options) unten. Die Vorgaben reichen für einen ersten Blick.

4. Auf **Backen** klicken

    Ein Fortschrittsfenster zeigt die Phase, und du kannst währenddessen weiterarbeiten. Änderungen während des Backens sind nicht darin.

    **Abbrechen** stoppt es, und die Karte behält die Beleuchtung, die sie hatte.

5. Das Ergebnis ansehen

    Ist es fertig, schaltet die 3D-Ansicht auf die Schattierung **Gebackene Beleuchtung**, und die Statusleiste sagt, wie lange es gedauert hat und wie groß die Lightmap ist.

Das Gebackene wird in der `.tdm` gespeichert, und das Backen ist ein einziger Undo-Schritt. **Backen entfernen** im selben Fenster wirft es weg, Godot beleuchtet die Karte dann wieder in Echtzeit.

## Optionen des Dialogs { #dialog-options }

| Option | Was sie tut |
| --- | --- |
| Qualität | Wie viele Strahlen und Abpraller der Tracer aufwendet, siehe die Tabelle unten |
| Texelgröße | Karteneinheiten, die ein Lightmap-Texel abdeckt, 2 bis 128, standardmäßig 16 (50 cm). Kleiner ist schärfer und langsamer |
| Schattenweichheit | 0 bis 1. Verbreitert Lichter, die kein eigenes `light_size` haben, damit Schatten weiche Ränder bekommen. 0 hält sie hart |
| Läuft auf | CPU, GPU oder CPU + GPU, siehe unten |

| Qualität | Strahlen pro Texel | Abpraller | Schattenstrahlen pro Licht | Wofür |
| --- | --- | --- | --- | --- |
| Vorschau | 32 | 1 | 4 | Lichter platzieren, verrauscht, aber schnell |
| Mittel | 96 | 2 | 8 | Alltagsarbeit, die Vorgabe |
| Hoch | 192 | 3 | 12 | Ein sauberes Ergebnis |
| Final | 384 | 4 | 16 | Der Build, den du auslieferst, auf großen Karten langsam |

Die Lightmap ist an einer Seite höchstens 4096 Texel groß. Passt eine Karte mit der gewünschten Texelgröße nicht hinein, nimmt das Backen eine gröbere und sagt das in der Statusleiste. Die Karte merkt sich die Optionen, mit denen sie zuletzt gebacken wurde.

### CPU, GPU oder beides { #cpu-gpu-or-both }

| Läuft auf | Nimm es, wenn |
| --- | --- |
| CPU | Funktioniert immer, auf jedem Kern. Die sichere Vorgabe |
| GPU | Der Rechner eine dedizierte Grafikkarte hat, die meistens viel schneller ist |
| CPU + GPU | Beide stark sind. Die Karte und jeder Kern holen sich Arbeit aus einer Warteschlange, jeder so viel, wie er schafft |

Lässt sich die GPU nicht nutzen, etwa weil es keine gibt, sie ein Software-Renderer ist oder die Karte zu groß für ihre Puffer ist, backt die CPU die Karte, und die Statusleiste sagt, warum. Fällt eine GPU mittendrin aus, übergibt sie den Rest an die CPU.

## Das Gebackene ansehen { #looking-at-the-bake }

*Ansicht > Schattierung > Gebackene Beleuchtung* zeigt das Gebackene, und sobald die Karte welches hat, nimmt F4 es mit, wenn es die Schattierung durchschaltet. *Ansicht > Schattierung > Gebackene Ansicht* wählt, was es zeigt.

![Ansicht > Schattierung > Gebackene Ansicht geöffnet, Texturen mit gebackenem Licht angehakt und Nur Lightmap mit dem Mauszeiger](../assets/light-baking/menu-baked-view.png)

| Gebackene Ansicht | Zeigt |
| --- | --- |
| Texturen mit gebackenem Licht | Die Texturen, von der Lightmap beleuchtet, nah an dem, was Godot zeigt |
| Nur Lightmap | Nur das Licht, ohne Texturen, um Abprall und Rauschen zu beurteilen |
| Sonnenschatten-Maske | Wie viel von der Sonne an jede Stelle kommt, weiß bei voller Sonne |
| Umgebungsverdeckung | Abdunkelung in Ecken und unter Objekten, nur zum Prüfen, Godot nutzt sie nicht |

![Derselbe Raum in den vier gebackenen Ansichten: Texturen mit gebackenem Licht oben links, die Lightmap allein oben rechts, die Sonnenschatten-Maske unten links und die Umgebungsverdeckung unten rechts](../assets/light-baking/baked-views.png)

## Licht-Schlüssel { #light-keys }

Jedes `light` und `light_spot` hat einen Schlüssel `bake_mode`, und worldspawn hat `sun_bake_mode` für die Sonne:

| Wert | Im Backen | In Godot |
| --- | --- | --- |
| `auto` | Wie `baked`, nur dass ein Licht mit einem `targetname` oder `start_on` 0 in Echtzeit bleibt, da I/O es schalten kann. Die Sonne wird immer gebacken | Statisch oder deaktiviert |
| `baked` | Direktes und abgepralltes Licht samt Schatten | Statisch, die Lightmap trägt es |
| `bounce` | Nur das abgeprallte Licht | Dynamisch, Godot zeichnet das direkte Licht und seine Schatten in Echtzeit |
| `realtime` | Ausgelassen | Deaktiviert, ganz in Echtzeit |

Nimm `bounce` für eine Lampe, deren Schatten bewegten Objekten folgen sollen, während ihr Abprall den Raum trotzdem füllt. Auch Godots eigene Namen `static`, `dynamic` und `disabled` funktionieren. Ein Echtzeitlicht beleuchtet bewegte Objekte außerdem über die [Lichtsonden](../godot/baked-lighting.md#light-probes).

`light_size` ist der Radius der Lampe in Metern. Eine größere Lampe wirft weichere Schatten, und 0 nimmt stattdessen die Schattenweichheit des Fensters. `light_indirect_energy` skaliert das abgeprallte Licht, und `omni_attenuation`, `spot_attenuation` und `spot_angle_attenuation` formen den Abfall. Das Godot-Licht bekommt sie alle als Eigenschaften gleichen Namens, sodass das Echtzeitlicht zum Gebackenen passt.

## Wenn ein Backen veraltet { #when-a-bake-goes-out-of-date }

Das Gebackene merkt sich die Geometrie und Lichter, aus denen es entstand. Statische Geometrie zu verschieben, umzuformen, hinzuzufügen oder zu löschen, oder ein Licht, eine Lichtsonde, die Schlüssel für Sonne und Himmel in worldspawn, die verwendeten Texturen oder ihre Einstellungen zu ändern, macht es veraltet. Das Fenster Beleuchtung backen warnt dann, dass sich die Karte seit dem letzten Backen geändert hat.

![Das Fenster Beleuchtung backen mit der gelben Zeile "Die Karte hat sich seit dem letzten Backen geändert. Backe erneut, um die Beleuchtung zu aktualisieren." und den Schaltflächen Backen und Backen entfernen](../assets/light-baking/bake-dialog-out-of-date.png)

Bis du erneut backst, zeigen geänderte Flächen im Editor die Beleuchtete Vorschau und in Godot das durchschnittliche Licht der Karte, während der Rest sein gebackenes Licht behält. Hier wurde die Säule nach dem Backen verschoben: Sie zeigt die Beleuchtete Vorschau, und ihr alter Schatten liegt noch auf dem Boden. Back erneut, bevor du auslieferst.

![Der gebackene Raum nach dem Verschieben der Säule, die Säule ist gleichmäßig beleuchtet und ihr alter gebackener Schatten bleibt, wo sie stand](../assets/light-baking/bake-out-of-date.png)

## Textureinstellungen { #texture-settings }

Wähl eine Fläche aus und öffne **Textureinstellungen** im Inspektor, um festzulegen, wie jede Fläche mit dieser Textur behandelt wird, in der ganzen Karte. Die Einstellungen werden mit der Karte gespeichert.

![Der Inspektor mit einer ausgewählten Fläche und geöffneten Textureinstellungen: Licht darauf backen, Wirft gebackene Schatten, Lightmap-Detail, Wiederholen alle und Projektion](../assets/light-baking/texture-settings.png)

| Einstellung | Was sie tut | Wofür |
| --- | --- | --- |
| Licht darauf backen | Aus lässt die Flächen aus der Lightmap heraus, Godot gibt ihnen das durchschnittliche Licht der Karte | Himmel, Wasser, Flächen, die niemand aus der Nähe sieht |
| Wirft gebackene Schatten | Aus lässt beim Backen Licht durch die Flächen | Glas, Gitter, dünne Decals |
| Lightmap-Detail | Multipliziert die Texelgröße auf diesen Flächen, unter 1 ist schärfer, über 1 spart Platz | Scharfe Schatten auf einem Boden, unscharfe an einer fernen Decke |
| Wiederholen alle | Karteneinheiten, die eine Wiederholung der Textur abdeckt, statt ihrer Pixelgröße | Ein Maßstab für eine Textur überall |
| Projektion | Welt ignoriert Versatz, Skalierung und Drehung jeder einzelnen Fläche, sodass die Textur nahtlos durchläuft | Tapete, Fliesen, Beton |

> [!TIP]
> Für eine Karte im Backrooms-Stil setz die Tapete einmal auf die Projektion Welt und eine Wiederholungsgröße, dann richtet sich jede Wand, die du damit zeichnest, aus, ohne dass du das Textur-Werkzeug anfasst.

Eine Fläche mit beiden Backen-Optionen aus bleibt ganz aus dem Backen heraus. Wiederholungsgröße und Projektion gelten auch in Godot.

<div class="td-mcp" markdown>

## MCP { #mcp }

Das Tool `bake_lighting` backt mit denselben Optionen wie das Fenster, `quality`, `texel_size`, `softness` und `backend` (`cpu`, `gpu` oder `hybrid`), und `view` schaltet die Gebackene Ansicht um. Es wartet, bis das Backen fertig ist, und liefert die Größe der Lightmap, die Zahlen, die genutzte GPU oder warum keine genutzt wurde, und `out_of_date`. Die Operation `settings` des Tools `texture` liest und setzt Textureinstellungen, mit `material`, `bake`, `casts`, `texel_scale`, `size` (`null` löscht es) und `projection` (`face` oder `world`). Siehe [MCP-Server](../mcp.md).

</div>
