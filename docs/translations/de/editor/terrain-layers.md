---
source: 7732f7c6b2b4601df9d9b8913237f6914fb6ca69
---

# Terrainebenen { #terrain-layers }

Ein Terrain hat eine bis vier Ebenen, jede ist ein Material plus ein paar Einstellungen dazu, wie es sich wiederholt. Wähl das Terrain aus, dann findest du sie im Inspektor. Die Ebenen teilen sich den Boden, wenn du also Sand auf Gras malst, nimmst du dem Gras Gewicht weg.

![Der Inspektor eines Terrains mit vier Ebenen, jede mit Material, Kachel, Enttiling und Schärfe](../assets/terrain-layers/layers-inspector.png)

| Wenn du | Dann |
| --- | --- |
| Eine Ebene hinzufügen willst | **+ Ebene (aktuelles Material)** unter den Ebenen, das das Material nimmt, auf das du zuletzt im Panel Materialien geklickt hast. Es erscheint, solange das Terrain weniger als vier hat |
| Die letzte Ebene entfernen willst | **- letzte Ebene** |
| Das Material einer Ebene wechseln willst | Tipp seinen Namen in das Feld der Ebene, oder zieh ein Material aus dem Panel Materialien auf das Terrain. Die Zeile unten im Inspektor sagt, in welche Ebene ein Ablegen geht, und bei aktivem Misch-Werkzeug ist es die Ebene, die das Werkzeug malt |

**Ebene 0 ist der Basisboden.** Unbemalter Boden zeigt Ebene 0, und Radieren malt zu ihr zurück. Leg also deinen häufigsten Boden dorthin, meistens Gras.

Ein neues Terrain bekommt die Ebenen, die im Dialog *Terrain erstellen* benannt sind, standardmäßig vier. Legst du ein Material für eine Ebene ab, die das Terrain noch nicht hat, wird sie als nächste Ebene hinzugefügt.

## Kachel und Enttiling { #tile-and-de-tiling }

Jede Textur, die sich über eine große Fläche wiederholt, zeigt aus der Ferne ein Rastermuster. Diese Einstellungen tauschen Schärfe aus der Nähe gegen das Verstecken dieses Musters.

| Einstellung | Bedeutung |
| --- | --- |
| Kachel | Wie viele Karteneinheiten eine Wiederholung der Textur abdeckt. 128 bis 384 funktioniert gut für Boden, auf dem man läuft |
| Enttiling | 0 bis 1. Dreht und verschiebt jede Wiederholung zufällig, sodass das Raster nicht mehr auffällt |
| Schärfe | 0 bis 1, erscheint, sobald Enttiling über 0 liegt. 0 blendet die verschobenen Wiederholungen gleichmäßig, das sieht weich aus. 1 blendet sie nur entlang ihrer Nähte, und die Textur bleibt knackig |

Eine kleine Kachel ist aus der Nähe scharf, wiederholt sich aber von weitem sichtbar, eine große verschwimmt dir zu Füßen. Enttiling versteckt die Wiederholung ohne größere Kachel, kostet aber zusätzliche Texturzugriffe, lass es also auf 0, wo die Wiederholung nicht auffällt.

![Kachel 256 mit Enttiling 0 links und 0,7 rechts](../assets/terrain-painting/detile-compare.jpg)

> [!NOTE]
> Das `metadata/texture_size` des Materials gilt nicht für Terrain, nur **Kachel** tut das.

## So sieht es in Godot aus { #how-it-looks-in-godot }

Jede Ebene wird von oben und von den Seiten projiziert, sodass Klippen Fels zeigen statt eines Schmierflecks, und es entspricht dem, was du im Editor siehst. Jede Ebene nutzt die Farbtextur, die Normal-Map, die Rauheit und die Emission ihres Materials. Ambient Occlusion wird verwendet, wenn sie in der Rauheitstextur steckt, wie in einer ORM-Textur, oder wenn die Ebene keine Rauheitstextur hat.

![Die bemalten Klippen der Meeresinsel in Godot](../assets/terrain-painting/godot-cliffs.jpg)
