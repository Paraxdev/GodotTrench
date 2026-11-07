---
source: 59a13276848e83b80ec8fffb5e9295c0444e1a52
---

# Terrain und Scatter { #terrain-and-scatter }

## Terrain { #terrain }

Eine Höhenkarte auf einem regelmäßigen Raster, eine Höhe pro Rasterpunkt, bemalt mit bis zu vier Texturebenen. Die Massendaten sind Base64-Text im JSON und rohe Bytes im [binären Container](container.md#value-encoding).

| Key | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `origin` | `[x, y, z]` | erforderlich | Weltposition des ersten Punkts |
| `resolution` | `[nx, nz]` | erforderlich | Anzahl der Punkte entlang X und Z, jeweils mindestens 2 |
| `cell_size` | Zahl | erforderlich | Karteneinheiten zwischen benachbarten Punkten |
| `heights` | Base64 | erforderlich | Little-Endian-32-Bit-Floats, eine Höhe pro Punkt |
| `layers` | Array | erforderlich | Bis zu vier Texturebenen, siehe unten |
| `splat` | Base64 | weggelassen | Vier Gewichtsbytes pro Punkt, eines je Ebene, die angeben, wie stark jede Ebene dort zu sehen ist |
| `holes` | Base64 | weggelassen | Ein Byte pro Zelle, ungleich null schneidet die Zelle aus dem Terrain heraus |
| `chunk_cells` | Ganzzahl | `32` | Wird immer geschrieben. Zellen pro Chunk-Seite. Das Terrain wird für Rendering und Kollision in Chunks dieser Größe geteilt |

Der Punkt `(i, j)` liegt bei `origin + (i * cell_size, heights[j * nx + i], j * cell_size)`. `splat` nutzt dieselbe Reihenfolge, seine Gewichte werden beim Lesen normalisiert, und ein Eintrag aus lauter Nullen bedeutet Ebene 0. `holes` deckt die `(nx - 1) * (nz - 1)` Zellen in derselben Reihenfolge ab.

Jede Ebene:

| Key | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `material` | String | erforderlich | Material, das diese Ebene malt |
| `tile` | Zahl | `256` | Karteneinheiten pro Texturwiederholung |
| `detile` | Zahl | `0` | 0 bis 1, wie stark die Wiederholung aufgebrochen wird, damit das Rastermuster nicht mehr auffällt |
| `detile_sharpen` | Zahl | `0.5` | 0 bis 1, wie scharf das Ergebnis nach dem Enttiling bleibt |

Wie sich diese Einstellungen in der Praxis anfühlen, steht unter [Terrainebenen](../editor/terrain-layers.md).

Terrains bleiben achsenparallel. Eine Instanz verschiebt den Mittelpunkt eines Terrains um ihre ganze Transformation, Rotation eingeschlossen, dreht aber nie das Raster selbst.

## Scatter { #scatter }

Ein Scatter-Set hält in einem Node viele Kopien von wenigen Modellen, etwa Bäume, Felsen oder Gras, die auf Oberflächen gemalt werden. Die Editor-Seite steht unter [Scatter](../editor/scatter.md). Jeder Key außer `targets` und `material` wird immer geschrieben.

| Key | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `name` | String | erforderlich | Name, der im Editor angezeigt wird |
| `kind` | String | `props` | Was Godot baut. `props` behält Skripte und Kollision, `foliage` ist ein MultiMesh ohne Kollision |
| `targets` | Array von Node-IDs | weggelassen | Die Oberflächen, auf die das Set gemalt wird |
| `items` | Array | erforderlich | Die Modelle, die es verstreut, siehe unten |
| `collision` | String | `none` bei Foliage, sonst `convex` | Kollisionsform pro Instanz: `none`, `box`, `convex`, `decomposition` oder `trimesh` |
| `cast_shadows` | bool | `true` | Ob die Instanzen Schatten werfen |
| `visibility_range` | Zahl | `0` | Blendet Instanzen jenseits dieser Entfernung aus, 0 heißt nie |
| `chunk_size` | Zahl | `0` | Teilt das Set in Zellen dieser Größe, damit Godot weglassen kann, was nicht im Bild ist, 0 ist eine Zelle. Neue Sets nutzen 2048 |
| `static_props_multimesh` | bool | `false` | Zeichnet Prop-Szenen, die Skripte tragen, ebenfalls als MultiMesh. Billiger bei vielen Props, aber ihre Skripte fallen weg |
| `material` | String | weggelassen | Material-Override für das ganze Set |
| `instances` | Array | leer | Die platzierten Instanzen, siehe unten |

Kopierst du ein Set zusammen mit einer Oberfläche, auf die es zielt, zielt die Kopie auf die kopierte Oberfläche.

Jedes Item ist ein Modell, das das Set platzieren kann:

| Key | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `source` | String | erforderlich | `res://`-Modell oder -Szene |
| `weight` | Zahl | `1` | Relative Wahrscheinlichkeit, dass dieses Item gewählt wird |
| `scale` | `[min, max]` | `[0.8, 1.2]` | Bereich für eine zufällige gleichmäßige Skalierung |
| `spacing` | Zahl | `48` | Mindestabstand zu jeder anderen Instanz |
| `align` | Zahl | `0` | 0 steht aufrecht, 1 folgt der Flächennormalen, Werte dazwischen neigen sich ein Stück |
| `random_yaw` | bool | `true` | Dreht jede Instanz in eine zufällige Richtung |
| `tilt` | Zahl | `0` | Größte zufällige Neigung in Grad |
| `sink` | Zahl | `0` | Karteneinheiten, die in die Oberfläche gedrückt werden, damit Wurzeln und Felsfüße nicht schweben |
| `material` | String | weggelassen | Material-Override für dieses Item. Er geht dem des Sets vor |
| `enabled` | bool | `true`, weggelassen | Ob der Pinsel dieses Item malt. Bereits platzierte Instanzen bleiben |

Jede Instanz ist ein flaches Array `[item, x, y, z, pitch, yaw, roll, scale]`, wobei `item` in `items` indiziert und die Winkel Grad in der Reihenfolge YXZ sind. Beim Speichern wird die Position auf 0,01, die Winkel auf 0,1 und die Skalierung auf 0,001 gerundet, sodass der Container sie als [Integer-Spalten](container.md#value-encoding) ablegen kann.
