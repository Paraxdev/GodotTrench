---
source: d21533e024f6fb1ea2fed11362601fee5c4efd4d
---

# Nodes { #nodes }

Der Inhalt einer Karte ist ein Baum aus Nodes. Jeder Node hat dieselbe äußere Form, mit seinen Keys in dieser Reihenfolge:

| Key | Typ | Geschrieben | Bedeutung |
| --- | --- | --- | --- |
| `id` | Ganzzahl | immer | Eindeutig in der Karte |
| `type` | String | immer | `layer`, `group`, `entity`, `brush`, `mesh`, `terrain`, `scatter` oder `instance` |
| Typfelder | | | Die Keys dieses Node-Typs, siehe unten, [Geometrie](geometry.md) und [Terrain und Scatter](terrain-scatter.md) |
| `label` | String | wenn gesetzt | Name, den du mit *Umbenennen* vergeben hast, siehe unten |
| `graph` | Array aus 2 Zahlen | wenn gesetzt | Wo der Node einer Entität im [Logic-Graphen](../gameplay/logic-graph.md) sitzt |
| `hidden` | bool | wenn `true` | Im Editor ausgeblendet |
| `locked` | bool | wenn `true` | Lässt sich im Editor nicht auswählen oder bearbeiten |
| `children` | Array von Nodes | wenn nicht leer | Kind-Nodes, der Reihe nach |

`hidden` und `locked` sind nur Editor-Zustand, Godot baut einen ausgeblendeten Node also trotzdem. Wenn etwas nicht in den Bau soll, leg es in eine Ebene mit `omit_from_export`.

`label` ist der Name, den du im Editor einem Brush, Mesh, einer Entität, Instanz oder einem Terrain gegeben hast. Er wird statt eines Standardnamens wie `brush12` angezeigt. Ebenen, Gruppen und Scatter-Sets haben nie einen, sie führen ihren Namen in `name`.

Godot nutzt `label` nur bei Terrains, der Node heißt dann `terrain_<id>_<label>`. Brushes und Meshes gehen in ihre Entität über, und eine Entität bekommt ihren Node-Namen aus `targetname`. Editoren, die älter sind als der Key, lesen Dateien mit ihm weiterhin, verwerfen ihn aber beim Speichern.

`graph` ist ebenfalls Editor-Zustand. Es ist die Stelle, an der der Node einer Entität im Logic-Graphen sitzt, in Graph-Punkten, und nur Nodes, die jemand verschoben hat, haben es. Godot ignoriert es, und ein Wert, der nicht aus zwei Zahlen besteht, wird beim Laden verworfen.

## IDs { #ids }

IDs sind positive Ganzzahlen, die über das Speichern hinweg gleich bleiben, damit andere Daten auf einen Node zeigen können. Der Live Link schickt Änderungen per ID, Scatter-Sets nennen ihre Oberflächen per ID, und Godot legt die ID jedes erzeugten Nodes in dessen `_td_id`-Metadaten ab. Eine ID `0` oder ein Duplikat, das es nur in einer von Hand bearbeiteten Datei gibt, bekommt beim Laden eine frische ID.

Ein Prefab ist eine eigene `.tdm`-Datei mit eigenen IDs, zwei Kopien davon würden also kollidieren. Um das zu vermeiden, addiert der Godot-Bau zur ID jedes Nodes, den eine [Instanz](#instance) mitbringt, eine Million mal die Nummer des Vorkommens. Zwei Kopien desselben Prefabs teilen sich nie IDs, egal wie tief sie verschachtelt sind.

## Wohin Nodes gehören { #where-nodes-go }

| Node | Erlaubter Parent |
| --- | --- |
| Ebene | Nur oberste Ebene |
| Gruppe | Ebene oder Gruppe |
| Brush, Mesh, Terrain, Scatter-Set, Punkt-Entität, Instanz | Ebene oder Gruppe |
| Brush oder Mesh einer Brush-Entität | Diese Entität |

Eine Entität mit Kindern ist eine Brush-Entität, zum Beispiel eine Tür aus Brushes. Brushes und Meshes außerhalb jeder Entität gehören zu worldspawn.

## Ebene { #layer }

Ein Teil der Karte auf oberster Ebene, im Outliner des Editors zu sehen.

| Key | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `name` | String | erforderlich | Name, der im Outliner angezeigt wird |
| `color` | String | erforderlich | `#rrggbbaa`. Beim Lesen wird `#rrggbb` akzeptiert |
| `omit_from_export` | bool | `false` | Wird immer geschrieben. Bei true überspringt der Bau die ganze Ebene, für Referenzgeometrie, die nie nach Godot gelangen soll |

## Gruppe { #group }

Nodes, die zusammengefasst sind, damit du sie gemeinsam auswählst und bewegst.

| Key | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `name` | String | erforderlich | Name, der im Outliner angezeigt wird |
| `link_id` | Ganzzahl | weggelassen | Gruppen mit derselben Link-ID sind verknüpfte Kopien, angelegt mit *Verknüpft duplizieren* |
| `transform` | 16 Zahlen | Identität, weggelassen | Die Platzierung dieser Kopie relativ zu den anderen, eine 4x4-Matrix in spaltenweiser Reihenfolge |

Der Inhalt einer Gruppe ist im Weltraum gespeichert, schon an Ort und Stelle. Eine [verknüpfte Kopie](../editor/organizing.md#groups) wiederholt eine Änderung, die an einer Kopie gemacht wurde, in allen anderen, und `transform` nutzt der Editor nur, um die Änderung von einer Kopie zur nächsten zu tragen. Der Godot-Bau ignoriert `transform` und `link_id`.

## Entität { #entity }

| Key | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `classname` | String | erforderlich | Entitätsklasse aus der Spielkonfiguration, zum Beispiel `prop_physics` |
| `origin` | `[x, y, z]` | `[0, 0, 0]` | Wird immer geschrieben. Position einer Punkt-Entität |
| `angles` | `[pitch, yaw, roll]` | `[0, 0, 0]` | Wird immer geschrieben. Grad, Reihenfolge YXZ wie bei `rotation_degrees` |
| `properties` | Objekt von String auf String | weggelassen | Entitäts-Keys, nach Namen sortiert |
| `outputs` | Array | weggelassen | I/O-Verbindungen, siehe unten |

Eigenschaftswerte sind immer Strings, egal welchen FGD-Typ sie haben, zum Beispiel `"travel": "0 168 0"`.

Eine Eigenschaft `angles`, `angle` oder `mangle` hat Vorrang vor den `angles` des Nodes und wird auf die Quake-Art gelesen: Pitch und Roll negiert, Yaw um 180 Grad gedreht. Bei Klassen, die mit `light` beginnen, ist `mangle` stattdessen Yaw, Pitch und Roll. Ein `angle` von -1 oder -2 zeigt gerade nach oben oder unten. Der Editor zeichnet die Entität in dieselbe Richtung, und wenn du sie dort drehst, wandert die Eigenschaft in die `angles` des Nodes.

Der Bau ignoriert `origin` und `angles` einer Brush-Entität, denn ihre Brushes sitzen schon dort, wo sie hingehören.

Jeder Output ist eine [I/O-Verbindung](../gameplay/io.md): Wenn diese Entität ein Signal auslöst, ruft sie eine Methode an einem Ziel auf.

| Key | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `output` | String | erforderlich | Signal dieser Entität, das die Verbindung auslöst |
| `target` | String | erforderlich | Wer den Aufruf erhält, siehe [Ziele](../gameplay/parameters.md#targets) |
| `input` | String | erforderlich | Methode, die am Ziel aufgerufen wird |
| `parameter` | String | `""`, weggelassen | Wert, der an den Input übergeben wird |
| `delay` | Zahl | `0`, weggelassen | Sekunden, die vor dem Aufruf gewartet wird |
| `times` | Ganzzahl | `-1`, weggelassen | Wie oft sie feuern darf, `-1` für jedes Mal |

## Instanz { #instance }

Ein Verweis auf eine andere `.tdm`-Datei, mit Position und Rotation platziert, wie Hammers `func_instance`. So werden Prefabs platziert: Der Inhalt liegt einmal in seiner eigenen Datei, und jede Instanz zeigt ihn.

| Key | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `path` | String | erforderlich | Die referenzierte Karte, relativ zur referenzierenden Karte, absolut oder `res://` |
| `origin` | `[x, y, z]` | erforderlich | Wo die Instanz platziert ist |
| `angles` | `[pitch, yaw, roll]` | erforderlich | Grad, Reihenfolge YXZ |
| `fixup` | String | `""`, weggelassen | Namenspräfix für den Inhalt der Instanz, siehe unten |

Der Bau nimmt die Kinder jeder Ebene der referenzierten Karte, die nicht vom Export ausgenommen ist, und verschiebt sie um `origin` und `angles`. Texturprojektionen wandern mit der Geometrie mit. Das worldspawn der referenzierten Karte wird nicht benutzt. Instanzen lassen sich bis zu acht Stufen tief verschachteln.

Ein Godot-Bau liest jede referenzierte Karte einmal, und alle Instanzen einer Karte zusammen dürfen 2.097.152 Werte hinzufügen, wobei ein Wert ein Array-Element oder Dictionary-Eintrag der referenzierten Karte ist und je 8 Zeichen Text als ein weiterer zählen. Darüber lässt der Bau die übrigen Instanzen weg und meldet einen Fehler. Gewöhnliche Karten bleiben weit darunter, nur Instanzen von Instanzen, vielfach vervielfacht, erreichen das.

Mit einem Fixup von `p1` wird aus `door` in `targetname`, `target`, `destination`, `call_target`, jeder Eigenschaft, die die FGD als `target_source` oder `target_destination` deklariert, und in jedem Output-`target` der Wert `p1-door`. Werte, die mit `!`, `@` oder `/` beginnen, bleiben unberührt. Verschachtelte Fixups addieren sich von außen nach innen, `b` in `a` ergibt also `a-b-door`, und *Instanz auflösen* im Editor ergibt dieselben Namen.
