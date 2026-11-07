---
source: 8142a4423329f307647582ed665e04b916677fd2
---

# Container-Aufbau { #container-layout }

Eine binäre `.tdm` besteht aus einem 12 Byte großen Dateikopf und danach Chunks, einer direkt hinter dem anderen, bis zum Ende der Datei. Jede Zahl ist Little Endian und hat feste Breite.

| Offset | Größe | Bedeutung |
| --- | --- | --- |
| 0 | 8 | Magic `89 47 54 4D 0D 0A 1A 0A`, also `\x89GTM\r\n\x1a\n` |
| 8 | 4 | Container-Version, u32, derzeit 1 |

Ein Leser lehnt eine Container-Version ab, die neuer ist als seine eigene.

Das `\r\n` im Magic fängt eine Datei ab, die durch eine Zeilenende-Umwandlung gelaufen ist, etwa in Git. Ein Leser meldet so eine Datei als verstümmelt, statt sie zu lesen.

## Chunks { #chunks }

Jeder Chunk besteht aus einem 24 Byte großen Header und danach seiner Nutzlast. Der Sync-Marker und die Header-Prüfsumme unterscheiden einen echten Chunk-Header von zufälligen Bytes, so findet ein Leser in einer [beschädigten Datei](recovery.md) den nächsten guten Chunk.

| Offset | Größe | Bedeutung |
| --- | --- | --- |
| 0 | 4 | Sync-Marker `89 47 54 43`, also `\x89GTC` |
| 4 | 4 | Tag, vier druckbare ASCII-Zeichen wie `NODE` oder `END ` |
| 8 | 4 | Flags, u32. Bit 0 bedeutet, dass die Nutzlast ein zstd-Frame ist |
| 12 | 4 | Rohgröße, u32, die Größe der Nutzlast nach dem Dekomprimieren |
| 16 | 4 | Gespeicherte Größe, u32, die Anzahl der Nutzlast-Bytes, die dem Header folgen |
| 20 | 4 | Header-Prüfsumme, u32: der Tag als u32 gelesen, XOR Flags, XOR Rohgröße, XOR gespeicherte Größe, XOR `0x47544D43` |

Eine komprimierte Nutzlast ist ein zstd-Frame mit Inhaltsprüfsumme. Eine Rohgröße über 1 GiB bedeutet, dass der Header beschädigt ist, und auch eine ganze Datei hat [Grenzen](#limits).

Der Editor schreibt die Chunks in dieser Reihenfolge:

1. Ein `HEAD`-Chunk.
2. Die `NODE`-Chunks.
3. Alle unbekannten Chunks, die er aus der geladenen Datei behalten hat.
4. Ein `LMAP`-Chunk, wenn die Karte gebackene Beleuchtung hat.
5. Der `END`-Chunk, immer zuletzt.

### HEAD { #head }

Das [Objekt der obersten Ebene](README.md#top-level) ohne `layers`.

### NODE { #node }

Ein Stapel ganzer Nodes:

| Key | Typ | Bedeutung |
| --- | --- | --- |
| `parents` | int64-Array | Die Parent-ID jedes Nodes, 0 für eine Ebene |
| `nodes` | Array von Node-Objekten | Die Nodes, jeweils ohne `children` |

Die Nodes kommen in Baumreihenfolge, ein Parent steht also immer vor seinen Kindern, im selben Chunk oder in einem früheren. Der Editor beginnt einen neuen Chunk, sobald ein Stapel vor der Kompression 64 KiB überschreitet, und teilt einen Node nie auf zwei Chunks auf. So nimmt ein beschädigter Chunk nur seine eigenen Nodes mit.

Um den Baum wieder aufzubauen, hängst du jeden Node an die `children` des Nodes an, den seine Parent-ID nennt.

### END { #end }

Der `END`-Chunk lässt einen Leser prüfen, dass nichts fehlt:

| Key | Typ | Bedeutung |
| --- | --- | --- |
| `version` | Ganzzahl | Die Kartenversion, damit sich die Nodes auch bei beschädigtem `HEAD` laden lassen |
| `nodes` | Ganzzahl | Wie viele Nodes die `NODE`-Chunks enthalten |
| `chunks` | Ganzzahl | Wie viele Chunks vor `END` stehen |
| `content` | Ganzzahl | Inhalts-ID, siehe unten |

Die Inhalts-ID ist ein 64-Bit-FNV-1a-Hash der rohen Nutzlasten von `HEAD` und `NODE` in Dateireihenfolge, auf seine unteren 52 Bit gekürzt, damit er als JSON-Zahl überlebt. Godot hebt sie zusammen mit der gebauten Szene auf, damit eine [Live-Sitzung](../godot/live-link.md) erkennen kann, dass sich eine Karte nicht geändert hat, und den Neubau überspringt.

### LMAP { #lmap }

Die [gebackene Beleuchtung](../editor/light-baking.md), nur vorhanden, wenn die Karte einen Bake hat. Es ist ein einziges Dictionary mit 18 Keys in einem eigenen Chunk, damit der Rest der Karte schnell lädt und ein älterer Editor ihn unangetastet behält. Weil es groß ist, komprimiert der Editor es mit einer schnelleren zstd-Stufe als die anderen Chunks. Positionen und Abstände sind in Karteneinheiten.

| Key | Typ | Bedeutung |
| --- | --- | --- |
| `version` | Ganzzahl | Layout-Version, derzeit 1. Ein Leser verwirft einen Bake mit einer neueren |
| `width`, `height` | Ganzzahl | Größe der Lightmap in Texeln |
| `scene` | Ganzzahl | Fingerabdruck der Geometrie, Lichter und des Himmels, aus denen der Bake entstand, um zu erkennen, wann er veraltet ist |
| `texel_size` | Float | Karteneinheiten pro Texel |
| `light` | Bytes | Lineares Licht, drei Little-Endian-Half-Floats (Rot, Grün, Blau) pro Texel, Zeile für Zeile |
| `shadow` | Bytes | Wie viel von der Sonne jeden Texel erreicht, 0 bis 255 |
| `ao` | Bytes | Ambient Occlusion pro Texel, 255 dort, wo nichts in der Nähe ist |
| `chart_keys` | int64-Array | Eine Node-ID und ein Flächenindex pro Chart, bei einem Terrain ist die Fläche 0 |
| `chart_rows` | float32-Array | Acht Floats pro Chart, zwei Zeilen `a b c d`, die die Lightmap-Koordinate 0 bis 1 des Charts als `a*x + b*y + c*z + d` angeben |
| `nodes` | int64-Array | Paare aus einer Node-ID und dem Fingerabdruck ihrer Geometrie zum Zeitpunkt des Bakes |
| `stale` | int64-Array | Nodes, deren Geometrie sich seitdem geändert hat oder die weg sind. Leser überspringen ihre Charts |
| `fallback` | float32-Array | Das durchschnittliche Licht als Rot, Grün und Blau, für Flächen ohne Chart |
| `probe_points` | float32-Array | Positionen der Lichtsonden, je drei Floats |
| `probe_sh` | float32-Array | Neun Rot-, Grün- und Blau-Koeffizienten der sphärischen Harmonischen pro Sonde, wie Godot sie speichert |
| `probe_tetrahedra` | int32-Array | Vier Sondenindizes pro Tetraeder |
| `probe_bsp_planes` | float32-Array | Eine Ebene pro BSP-Node, Normale und Abstand |
| `probe_bsp_children` | int32-Array | Das Kind über und unter jeder Ebene: ein weiterer Node, wenn positiv, Tetraeder `t` als `-t - 1`, nichts als der kleinste int32 |

Die Sonden-Keys folgen dem Layout von Godots `LightmapGIData`, und ein Leser lässt die Sonden weg, wenn ihre Größen nicht zusammenpassen. Ein Chunk, der sich nicht laden lässt, wird mit der Warnung verworfen, neu zu backen.

In einer JSON-Karte liegt dasselbe Dictionary unter einem `lightmap`-Key auf oberster Ebene, mit `light`, `shadow` und `ao` als Base64-Text.

### Andere Chunks { #other-chunks }

Leser überspringen einen Chunk mit einem Tag, den sie nicht kennen. Der Editor behält ihn Byte für Byte und schreibt ihn beim Speichern hinter seinen `NODE`-Chunks zurück.

Ein neuer Chunk ist damit der Weg, Daten hinzuzufügen, die ältere Editoren mitschleppen, ohne sie zu verstehen. Bedenke, dass ein älterer Editor Nodes gelöscht haben kann, auf die dein Chunk verweist, ein neuer Chunk-Typ muss also gültig bleiben, wenn diese Nodes weg sind.

## Wertkodierung { #value-encoding }

Jede Nutzlast ist ein Wert in Godots binärer Variant-Serialisierung, dem Format von `var_to_bytes`, und nutzt nur die Typen unten. Jeder Wert beginnt mit einem u32-Header, dessen niedriges Byte der Typ ist. Bit 16 des Headers markiert eine 64-Bit-Ganzzahl oder einen 64-Bit-Float. Bit 16 bei einem anderen Typ oder irgendein anderes Bit macht den Chunk unlesbar. Das tut auch ein Wert, der in mehr als 128 verschachtelten Arrays und Dictionaries steckt.

| Typ | Header | Körper |
| --- | --- | --- |
| null | 0 | Nichts |
| bool | 1 | u32, 0 oder 1 |
| Ganzzahl | 2 | i32, oder i64 mit gesetztem Bit 16 |
| Float | 3 | f32, oder f64 mit gesetztem Bit 16. Der Editor schreibt immer f64 |
| String | 4 | u32-Bytelänge, dann die UTF-8-Bytes, dann Nullen als Auffüllung auf ein Vielfaches von 4 |
| Dictionary | 27 | u32-Anzahl, dann jeder Key (ein String) gefolgt von seinem Wert |
| Array | 28 | u32-Anzahl, dann die Werte |
| Bytes | 29 | u32-Länge, dann die Bytes, dann Nullen als Auffüllung auf ein Vielfaches von 4 |
| int32-Array | 30 | u32-Anzahl, dann je ein i32 |
| int64-Array | 31 | u32-Anzahl, dann je ein i64 |
| float32-Array | 32 | u32-Anzahl, dann je ein f32 |
| float64-Array | 33 | u32-Anzahl, dann je ein f64 |

Leser ignorieren Bit 31 der Anzahl bei einem Dictionary oder Array, das Godot für geteilte Container setzt, aber nicht bei der Anzahl eines gepackten Arrays. JSON-Ganzzahlen werden als Ganzzahlen und JSON-Floats als Floats geschrieben, die Umwandlung in Binär und zurück ergibt also dasselbe JSON.

Um Dateien klein zu halten, bekommen drei Arten von Werten eine kompaktere Form. Ein Leser muss sowohl die kompakte als auch die einfache annehmen:

| JSON-Wert | Gespeichert als | Wann |
| --- | --- | --- |
| Array aus 32 oder mehr Zahlen | Ein int32-Array, wenn alle Ganzzahlen sind, die hineinpassen. Ein float32-Array, wenn alle Floats sind, die ein 32-Bit-Float exakt hält. Ein float64-Array für andere Floats | Immer |
| `heights`, `splat` und `holes` eines Terrains (Base64-Text) | Bytes, die dekodierten Daten | Wenn das Base64 in kanonischer Form ist |
| Scatter-`instances` | Ein int32-Array aus Spalten | Wenn jeder Wert ein Float und ein ganzzahliges Vielfaches seiner Schrittweite ist |

Die Scatter-Spalten enthalten zuerst jeden Item-Index, dann jedes x und so weiter durch die acht Felder einer Instanz. Jeder Wert wird als ganzzahlige Anzahl seiner Schrittweite gespeichert: 1 beim Item, 100 pro Einheit bei der Position, 10 pro Grad bei den Winkeln und 1000 bei der Skalierung. Das Teilen ergibt den exakten Float zurück, zum Beispiel ist `1234 / 100.0` gleich `12.34`.

> [!NOTE]
> `tdm_file.gd` macht aus Scatter-Spalten wieder Instanz-Arrays, lässt Terrain-Bytes aber als `PackedByteArray` und lange Zahlen-Arrays als gepackte Arrays. GDScript, das eine Karte liest, muss diese neben gewöhnlichen Arrays akzeptieren.

## Grenzen { #limits }

Ein paar Kilobyte zstd können sich zu Gigabytes entpacken, deshalb führt ein Leser für jede Datei ein Budget. Er nimmt die Chunks `HEAD`, `NODE`, `END` und `LMAP` in Dateireihenfolge und liest keinen, der nicht in das passt, was übrig ist:

1. Vor dem Entpacken wird die Rohgröße des Chunks von einem Budget von 1 GiB abgezogen.
2. Der entpackte Chunk bleibt im Speicher, während seine Werte gebaut werden, seine Rohgröße und der Speicher seiner Werte müssen also zusammen in das passen, was von einem Budget von 1200 MiB übrig ist, neben dem, was die Nodes eines `NODE`-Chunks brauchen, sobald die Karte daraus gebaut ist. Davon abgezogen wird nur der Speicher der Werte und der Nodes. Ein Chunk, der nicht passt, nimmt nichts, die Chunks danach lassen sich also noch lesen.

Chunks mit anderen Tags werden nie entpackt und nehmen nichts. Der Speicher wird so gezählt, ohne einen Wert zu bauen:

| Wert | Speicher |
| --- | --- |
| Jeder Wert, Dictionary-Keys und die Elemente gepackter Arrays eingeschlossen | 80 Bytes |
| Ein String, zusätzlich | 6 Bytes pro Byte, da JSON ein Byte als `\u0000` schreiben kann |
| Bytes, zusätzlich | `4 * ceil(n / 3) + n` für `n` Bytes, ihre Größe als Base64 und die Kopie, in die ein Terrain oder Bake sie dekodiert |
| Ein Node eines `NODE`-Chunks, zusätzlich zu seinen Werten | 5 KiB für eine Entität, 12 KiB für ein Scatter-Set, 1 KiB für jeden anderen Node |

Die Node-Kosten sind eine obere Grenze dafür, was die kleinsten Nodes jedes Typs im Editor brauchen, sobald er die Karte öffnet (eine Gruppe oder ein Brush kommt auf etwa 650 Bytes und wird mit 1 KiB berechnet), eine kleine Datei aus vielen winzigen Nodes kann also nicht mehr Speicher nehmen als das Budget. Nodes, die in einem `NODE`-Chunk unter `children` verschachtelt sind, werden verworfen, und der Leser meldet, wie viele der Nodes des Chunks welche enthielten, da der Editor sie dort nie schreibt. Scatter-Instanzen werden nur nach ihren Werten gezählt, die auf mehr kommen als die etwa 400 Bytes, die der Editor für eine gebaute Instanz hält, da sich jede Instanz eines Sets ihr Modell auf der GPU teilt.

Die größte Showcase-Karte braucht 79 MiB, eine Scatter-Instanz 640 bis 720 Bytes, und Licht, Schatten und Okklusion eines Bakes bei der größten Atlasgröße 299 MiB. Der Editor weigert sich, eine Karte zu speichern, die eine der beiden Grenzen überschreitet, jede Datei, die er schreibt, lässt sich also vollständig lesen.
