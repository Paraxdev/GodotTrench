---
source: 721efa7c5ececa362093af607d2e2c2faa92ad9b
---

# Gestreamte Abschnitte { #streamed-sections }

> [!NOTE]
> Diese Entitäten gehören zum Gameplay-Entitätenpaket, siehe
> [Installieren und anpassen](README.md#installing-and-customizing).

Entitäten, um mehrere Karten ohne Ladebildschirm als eine Welt zu spielen. Jede Karte ist ein Abschnitt, und Abschnitte treffen sich in einem Verbindungsraum, von dem beide Karten eine identische Kopie enthalten. [Abschnitte streamen](../../godot/streaming.md#streaming-sections) führt dich durch die Einrichtung.

## info_landmark { #info_landmark }

Der gemeinsame Punkt eines Verbindungsraums. Setze einen in die Mitte des Raums, mit demselben Namen in beiden Kopien. Wenn der nächste Abschnitt hereinkommt, wird er so platziert, dass seine Landmarke genau dort landet, wo die alte war.

* **Schlüssel:** `targetname`

## func_section_stream { #func_section_stream }

Eine Hälfte eines Verbindungsraums, ein unsichtbares Volumen. Solange der Spieler darin steht, ist der Abschnitt geladen, den es benennt. Ein Verbindungsraum braucht zwei, eines pro Seite, und die Linie, an der sie sich treffen, ist die Stelle, an der die Abschnitte wechseln. Lege sie dorthin, wo keines der beiden Enden des Raums zu sehen ist, zum Beispiel in die Mitte eines Z-förmigen Gangs.

* **Schlüssel:** `section` (der Abschnitt auf dieser Seite: sein Szenenpfad wie `res://levels/pools_2.tscn` oder ein Name, der sich in `sections_dir` des Streamers findet), `landmark` (der `targetname` des `info_landmark` des Raums)

## TDSectionStreamer { #tdsectionstreamer }

Keine Entität, sondern der Node, der das Streaming steuert. Füge ihn deiner Weltszene hinzu und setze:

| Eigenschaft | Was sie ist |
| --- | --- |
| `first_section` | Der Abschnitt, in dem das Spiel startet, ein Szenenpfad oder ein Name |
| `sections_dir` | Ordner, in dem Kurznamen gefunden werden, als `name.tscn` oder `name.scn` |
| `player_scene` | Der Spieler, der am `info_player_start` des ersten Abschnitts gespawnt wird |
| `player_path` | Oder ein Spieler, der schon in der Szene ist. Ohne beides bleibt der erste Node in der Gruppe `player`, den ein Abschnitt mitbringt |
| `recenter_distance` | Meter, die eine Schleife aus Abschnitten vom Ursprung abdriften darf, bevor die ganze Welt zurückgesetzt wird, 8000 |

Er sendet nach jedem Wechsel `section_changed(from, to)`.
