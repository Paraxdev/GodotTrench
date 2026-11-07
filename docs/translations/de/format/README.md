---
source: d3f0019b345d8a4b85cb01b22ccb9fd1cdee21d4
---

# Das .tdm-Kartenformat { #the-tdm-map-format }

Diese Seiten sind für dich, wenn du ein Tool schreibst, das `.tdm`-Dateien liest oder schreibt, etwa einen Importer, einen Konverter oder eine CI-Prüfung. Zum Bauen von Levels brauchst du davon nichts.

Eine `.tdm`-Datei enthält drei Dinge: die worldspawn-Eigenschaften, ein bisschen Editor-Zustand und einen Baum aus Nodes mit dem eigentlichen Inhalt.

Der Editor speichert sie als binären Container aus zstd-komprimierten Chunks, deshalb bleiben die Dateien klein, und eine beschädigte Datei lässt sich trotzdem öffnen. Ältere Editoren haben stattdessen ein einzelnes UTF-8-JSON-Dokument gespeichert, und jeder Leser lädt beide Formen. Beide enthalten denselben Baum, deshalb zeigen diese Seiten ihn als das JSON, das `trenchdot --dump` ausgibt.

| Seite | Lies sie, wenn du |
| --- | --- |
| [Container-Aufbau](container.md) | Die binäre Datei liest oder schreibst. Sie behandelt den Dateikopf, die Chunks und wie jeder Wert als Bytes kodiert wird |
| [Beschädigte Dateien](recovery.md) | Wissen willst, was sich aus einer kaputten Datei noch laden lässt und wo jeder Leser meldet, was verloren ging |
| [Nodes](nodes.md) | Den Baum durchläufst. Sie erklärt die Form, die jeder Node gemeinsam hat, wie IDs funktionieren und wie Ebenen-, Gruppen-, Entitäts- und Instanz-Nodes aussehen |
| [Geometrie](geometry.md) | Brushes und Meshes liest oder baust, einschließlich ihrer Flächen, der Texturprojektion und der Displacements |
| [Terrain und Scatter](terrain-scatter.md) | Höhenkarten-Terrains und Sets verstreuter Modelle verarbeitest |
| [JSON-Aufbau](json.md) | Mit der lesbaren Form arbeitest, zwischen beiden Formen konvertierst oder lesen willst, was der Editor in die Zwischenablage legt |

Du willst funktionierenden Code sehen? Die Referenzimplementierung ist
[`format.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/format.rs), mit dem Container in
[`binary.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/binary.rs) und
[`variant.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/variant.rs). Auf der Godot-Seite liest
[`tdm_file.gd`](https://github.com/Paraxdev/trenchdot_func/blob/main/src/trenchdot/tdm_file.gd) die Datei, und
[`tdm_parser.gd`](https://github.com/Paraxdev/trenchdot_func/blob/main/src/trenchdot/tdm_parser.gd) baut sie.

## Oberste Ebene { #top-level }

| Key | Typ | Geschrieben | Bedeutung |
| --- | --- | --- | --- |
| `format` | String | immer | Immer `"trenchdot-map"`. Daran erkennt man eine Karte unter anderem JSON |
| `version` | Ganzzahl | immer | Kartenversion, derzeit 1 |
| `properties` | Objekt von String auf String | wenn nicht leer | worldspawn-Keys, nach Namen sortiert |
| `editor` | Objekt | wenn nicht alles Standard ist | Ansichtszustand des Editors. Er wird nie nach Godot exportiert |
| `textures` | Objekt | wenn eine Textur Einstellungen hat | [Textureinstellungen](geometry.md#texture-settings) nach Materialname |
| `layers` | Array von Nodes | immer | Die [Ebenen-Nodes](nodes.md#layer) der Karte, der Reihe nach. Alles andere hängt darunter |
| `lightmap` | Objekt | JSON-Karten mit einem Bake | Die gebackene Beleuchtung, die eine binäre Karte in ihrem [LMAP-Chunk](container.md#lmap) hält |

Aus `layers` werden nur `layer`-Nodes gelesen, und eine Karte ohne solche bekommt beim Laden eine Ebene `Default`.

Worldspawn-Werte sind immer Strings, wie in einer `.map`-Datei. Wenn Godot die Karte baut, macht `TrenchDotEnvironment` aus diesen Keys einen Himmel, Nebel und eine Sonne:

| Keys | Was sie steuern |
| --- | --- |
| `sun_angles`, `sun_color`, `sun_energy` | Das gerichtete Sonnenlicht: wohin es zeigt, seine Farbe und wie hell es ist |
| `ambient_color`, `ambient_energy` | Licht, das Flächen erreicht, auf die die Sonne nicht fällt, damit Schatten nicht pechschwarz sind |
| `sky_top_color`, `sky_horizon_color`, `sky_ground_color`, `sky_energy` | Der prozedurale Himmelsverlauf und seine Helligkeit |
| `sky_panorama` | Ein `res://`-Bild, das in Godot den prozeduralen Himmel ersetzt. Die Editor-Vorschau und ihr Bake nutzen weiter die Himmelsfarben |
| `fog_color`, `fog_density` | Distanznebel |
| `glow_intensity`, `ssr` | Bloom auf hellen Flächen und Bildschirmraum-Reflexionen, wenn `ssr` auf `1` steht |

Wenn dein Spiel ein eigenes `WorldEnvironment` mitbringt, setze `environment` auf `sun_only`, dann wird nur die Sonne gebaut. `0` oder `none` baut nichts, siehe [Umgebung und Sonne](../godot/building.md#environment-and-sun). `sky_source` merkt sich, welche Textur die Himmelsflächen einer importierten Karte hatten.

Das Objekt `editor` ist Ansichtszustand, der nur im Editor eine Rolle spielt:

| Key | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `cameras` | Objekt, Keys `"1"` bis `"9"` | leer | Gespeicherte Kamerapositionen, jede als `{"position": [x, y, z], "yaw": r, "pitch": r}` mit Winkeln in Radiant |
| `cordon` | `{"min": [x, y, z], "max": [x, y, z]}` | keiner | Die Cordon-Box in Karteneinheiten, mit der du an einem Teil einer großen Karte arbeitest |
| `cordon_enabled` | bool | `false` | Blendet Objekte außerhalb des Cordons aus und lässt sie aus Exporten mit Cordon weg |
| `bake` | `{"quality", "texel_size", "softness", "backend"}` | keiner | Die Optionen, mit denen die Karte zuletzt [gebacken](../editor/light-baking.md) wurde, damit der nächste Bake von ihnen ausgeht |
| `graph_frames` | Array | leer | Rahmen des [Logic-Graphen](../gameplay/logic-graph.md#frames), jeder als `{"title", "rect": [x, y, width, height], "color"}` in Graph-Punkten. Ein beschädigter wird beim Laden verworfen |

## Versionen und Kompatibilität { #versions-and-compatibility }

`format` und `version` beschreiben die Daten. Die Container-Version im Dateikopf ist davon getrennt, sie ändert sich nur, wenn sich der Chunk-Aufbau ändert.

Das macht jeder Leser mit einer Datei, die er nicht ganz versteht:

| Situation | Editor | Godot-Addon |
| --- | --- | --- |
| Anderes `format` | Lehnt die Datei ab | Baufehler |
| Neuere `version` | Lehnt die Datei ab | Baufehler mit der Bitte, das Addon zu aktualisieren |
| Fehlende `version` | Liest sie als älteste Version | Genauso |
| Unbekannter Key | Ignoriert ihn, und nach dem nächsten Speichern ist er weg | Ignoriert ihn |
| Unbekannter Chunk | Behält ihn und schreibt ihn beim Speichern zurück | Überspringt ihn |
| Flächenindex außerhalb des Bereichs | Lehnt die Karte ab und nennt den Node | Überspringt diesen Brush |
| Displacement-`heights` in falscher Größe | Lehnt die Karte ab und nennt den Node | Baut die Fläche flach |

Der Editor verwirft beim Speichern unbekannte Keys. Wenn du also Daten hinzufügst, die ein älterer Editor nicht verlieren darf, brauchst du entweder einen Sprung der Kartenversion, den ältere Editoren ablehnen, oder einen eigenen Chunk, den sie unangetastet behalten. Siehe [Andere Chunks](container.md#other-chunks).

## Koordinaten und Einheiten { #coordinates-and-units }

Positionen nutzen die Achsen von Godot: Y nach oben, X nach rechts, Z zum Betrachter hin. Der Bau teilt durch den `inverse_scale_factor` der Karteneinstellungen, standardmäßig 32, also ergeben 32 Karteneinheiten einen Meter.

| Größe | Einheit |
| --- | --- |
| Positionen, Größen, Höhen, Abstände, Zellengröße | Karteneinheiten |
| Winkel von Entitäten, Instanzen und Scatter | Grad: Pitch um X, Yaw um Y, Roll um Z, in der Reihenfolge YXZ angewendet wie Godots `rotation_degrees` |
| Yaw und Pitch eines Kamera-Lesezeichens | Radiant |
| UV-Versatz | Texel |
| UV-Skalierung | Karteneinheiten pro Texel |
| Mesh-`uvs` | Texturbreiten, 1 ist also eine volle Wiederholung der Textur |
