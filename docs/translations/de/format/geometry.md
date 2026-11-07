---
source: d8695a266e0b8802237d74d9393aab6b51f516c4
---

# Geometrie { #geometry }

## Brush { #brush }

Ein konvexer Körper, gespeichert als seine exakten Ecken plus eine Liste von Flächen, die in diese Ecken indizieren, du bekommst die Ecken also, ohne Ebenen zu schneiden. Die Flächenebenen werden beim Laden neu berechnet.

| Key | Typ | Bedeutung |
| --- | --- | --- |
| `vertices` | Array von `[x, y, z]` | Ecken in Karteneinheiten |
| `faces` | Array von Flächen | Ein Polygon pro Fläche |

Jede Fläche:

| Key | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `indices` | Array von Ganzzahlen | erforderlich | Indizes in `vertices`, mindestens drei, von außen gesehen gegen den Uhrzeigersinn |
| `material` | String | `""` | Wird immer geschrieben. Texturname, zum Beispiel `base/wall` |
| `uv` | Objekt | paraxial | Wird immer geschrieben. Wie die Textur auf der Fläche liegt, siehe [Texturprojektion](#texture-projection) |
| `props` | Objekt von String auf String | weggelassen | Freie Flächenattribute, etwa die Blend-Keys weiter unten |
| `disp` | Objekt | weggelassen | Macht die Fläche zu einem [Displacement](#displacement) |
| `colors` | Array von `[r, g, b, a]` | weggelassen | [Vertexfarbe](../editor/texturing.md#vertex-paint), eine Farbe pro Index |

Der Godot-Bau überspringt einen Brush mit weniger als vier Vertices oder Flächen, denn kein geschlossener Körper hat weniger.

Eine Fläche kann auch zwischen zwei Materialien überblenden. Die Flächen-Props `blend_material`, `blend_detile`, `blend_uv_scale` und `blend_detile_sharpen` richten das ein, dann blendet das Alpha der Vertexfarbe über die Fläche von `material` zu `blend_material` über.

## Texturprojektion { #texture-projection }

`uv` ist eine Projektion im Stil von Valve 220 mit expliziten Achsen: Zwei Weltrichtungen geben an, in welche Richtung die Textur über die Fläche läuft, und ein Versatz und eine Skalierung platzieren sie.

| Key | Typ | Bedeutung |
| --- | --- | --- |
| `u_axis`, `v_axis` | `[x, y, z]` | Weltrichtung von u und v der Textur |
| `offset` | `[u, v]` | Verschiebung in Texeln |
| `scale` | `[u, v]` | Karteneinheiten pro Texel |
| `rotation` | Zahl | Optional und nur informativ, die Achsen enthalten die Rotation schon |

Ein Punkt `p` landet auf dem Texel `dot(p, u_axis) / scale.x + offset.x`, und für v genauso. Für die endgültige UV teilst du das durch die Texturgröße. Das ist die `texture_size` des Materials, wenn es eine hat, siehe [Materialien](../godot/materials.md), sonst die Pixelgröße des Bildes.

Ein fehlendes `uv` bedeutet `u_axis = [1, 0, 0]` und `v_axis = [0, 0, 1]` bei Skalierung 1. Ist `uv` vorhanden, ist jeder Key außer `rotation` erforderlich.

## Textureinstellungen { #texture-settings }

Das `textures`-Objekt der obersten Ebene enthält Einstellungen, die sich jede Fläche mit einem Material teilt, nach Materialname geordnet. Geschrieben werden nur Einstellungen, die vom Standard abweichen, und ein Material ohne solche entfällt.

| Key | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `bake` | bool | `true` | Flächen bekommen einen Platz in der Lightmap. Ausgeschaltet gibt Godot ihnen das gemittelte gebackene Licht |
| `casts` | bool | `true` | Flächen blockieren beim Bake Licht |
| `texel_scale` | Zahl | `1` | Multipliziert die Texelgröße der Lightmap auf diesen Flächen, 0,125 bis 16 |
| `size` | `[w, h]` | keiner | Karteneinheiten, die eine Wiederholung abdeckt, statt der Texturgröße von oben |
| `projection` | String | `"face"` | `"world"` ignoriert das `uv` jeder Brush-Fläche und nutzt die zurückgesetzte Projektion ihrer Normalen, Versatz 0 und Skalierung 1 |

`world` gilt nur für Brush-Flächen, Mesh-Flächen behalten ihre Projektion.

## Displacement { #displacement }

Eine Viereckfläche kann ein Displacement im Hammer-Stil tragen, ein Raster aus Vertices, die jeweils entlang der Flächennormalen herausgeschoben werden.

| Key | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `power` | Ganzzahl | erforderlich | 1 bis 4. Das Raster hat `2^power + 1` Vertices pro Seite |
| `heights` | Array von Zahlen | erforderlich | Wie weit jeder Rasterpunkt entlang der Normalen geschoben wird |
| `alphas` | Array von Zahlen | weggelassen | Blendgewicht pro Rasterpunkt, 0 zeigt das Flächenmaterial |

Beide Arrays sind zeilenweise mit `(2^power + 1)^2` Einträgen. Die Zeilen laufen von der ersten Ecke der Fläche zu ihrer vierten, die Spalten von der ersten Ecke zur zweiten.

## Mesh { #mesh }

Ein bearbeitbares Polygon-Mesh. Anders als ein Brush kann es konkav oder offen sein oder Flächen haben, die nicht eben sind.

| Key | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `vertices` | Array von `[x, y, z]` | erforderlich | Ecken in Karteneinheiten |
| `faces` | Array von Flächen | erforderlich | Dieselben Keys wie bei Brush-Flächen, plus `uvs` |
| `smooth_angle` | Zahl | `0`, weggelassen | Flächen, die in weniger als so vielen Grad aufeinandertreffen, teilen sich Normalen und werden weich schattiert. 0 ist flache Schattierung |
| `decal` | bool | `false`, weggelassen | Markiert das Mesh als [Decal](../editor/decals.md) |

Das Glätten vergleicht jede Flächenecke mit jeder Fläche um ihren Vertex. Braucht ein Mesh mehr als 64 Vergleiche pro Ecke oder insgesamt mehr als 262.144, wird es stattdessen flach schattiert. Dahin kommt nur ein Vertex, der von Hunderten Flächen geteilt wird, etwa die Spitze eines Kegels mit mehr als etwa 650 Seiten.

Das `uvs` einer Mesh-Fläche ist ein Array von `[u, v]`, eines pro Index, in Texturbreiten. Ist es vorhanden, ersetzt es `uv`, und eine Anzahl, die nicht zu `indices` passt, lässt den Editor die Karte ablehnen.
