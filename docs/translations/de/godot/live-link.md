---
source: c998bca32c1c4bd85f0c95739d78a13d4ede986a
---

# Live-Link-Protokoll { #live-link-protocol }

Der Live Link ist der Weg, auf dem TrenchDot und der Godot-Editor miteinander reden. Das Addon startet einen kleinen Server im Godot-Editor, und TrenchDot verbindet sich damit, um beim Speichern einen Neubau auszulösen, Änderungen im [Live-Modus](live.md) zu streamen und anzuzeigen, ob Godot läuft.

Diese Seite brauchst du selten, der Link funktioniert einfach, sobald das Addon aktiviert ist. Sie ist für den Fall da, dass du ein eigenes Werkzeug schreibst, das Godot genauso steuert, oder herausfinden willst, warum die Verbindung scheitert. Wenn ein Speichern keinen Neubau auslöst, fang lieber bei [Neu bauen beim Speichern](live.md#rebuild-on-save) an, die Statusleiste nennt meistens den Grund.

## Transport { #transport }

Newline-delimited JSON über TCP auf `127.0.0.1:7842`. Jede Nachricht hat ein `event` und optional ein `seq` und bekommt genau eine Antwortzeile, die das `seq` zurückgibt und `ok` enthält. Fehlschläge antworten mit `ok: false` und einem `error`. Der Editor hält eine Verbindung offen und sendet jede Sekunde einen `status`-Heartbeat.

Willst du einen anderen Port, ändere ihn auf beiden Seiten: `trenchdot/live_link_port` in den Projekteinstellungen von Godot und *Godot Live Link* unter *Datei > Einstellungen* in TrenchDot.

Das [Hot Reload](live.md#hot-reload-in-a-running-game)-Autoload in einem laufenden Spiel lauscht auf `127.0.0.1:7843` und beantwortet nur `map_saved`.

## Events { #events }

Ein Client sendet diese Events. Die Spalte Antwort nennt die Felder, mit denen Godot außer `ok` und `seq` antwortet, oder was es tut.

| Event | Felder | Antwort |
| --- | --- | --- |
| `hello` | | `project` und `godot` (Version), ein schneller Check, ob der Link antwortet |
| `status` | | `project`, `godot` (Version), `pid`, `scene` (der Szenen-Tab, den Godot zeigt) und `maps: [{path, epoch}]` dafür |
| `map_saved` | `path` | `rebuilt`, die Anzahl der aus der Datei neu gebauten Karten-Nodes, `scene`, `waiting` und die `missing_classes`, `settings` und `warnings` des Baus |
| `build` | `path`, `text` | Wie `map_saved`, gebaut aus `text` statt aus der Datei |
| `focus` | | Holt das Godot-Fenster nach vorn |
| `export_game_config` | | Schreibt `trenchdot_game.json` |
| `scan` | | Startet einen Scan der Projektdateien. Danach prüft jede Live-Sitzung ihre Entity-Definitionen |
| `play` | | Speichert die offenen Szenen und spielt die Hauptszene, oder `ok: false` mit einem `error`, wenn das Projekt keine hat |
| `live_begin` | `path`, `text`, `content` | `epoch` und `unresolved` |
| `live_resync` | `path`, `text` | `epoch` und `unresolved` |
| `live_delta` | `path`, `ops` | `epoch` und `unresolved`, oder `ok: false, resync: true`, wenn die Ops nicht passen |
| `live_end` | `path`, `revert` | Mit `revert` wird aus der gespeicherten Datei neu gebaut |
| `inspect` | `path`, `ids` | `nodes` mit Name, Klasse, Anzahl der Kinder und Position je Karten-Node-ID, dazu `live` und `pending` |
| `capture` | `path`, `camera`, `width`, `height`, `text` | `png` (base64), `width`, `height` und `warnings`, die beim Bauen geloggten Fehler und Warnungen |
| `walkable` | `path`, `agent`, `text` | `vertices` (flache Liste aus x, y, z), `polygons`, `islands` (Polygonindizes, größte zuerst), `units_per_meter`, `agent` und `msec` |

`capture` rendert die Welt des ersten Karten-Nodes, der `path` nutzt. `camera` enthält `position` und `forward` in Karteneinheiten und -achsen sowie `fov`, das horizontale Sichtfeld in Grad. Mit `text` baut Godot diese Karte zuerst, sonst wartet es, bis eine Live-Sitzung aufgeholt hat. Die Antwort kommt nach etwa einem Dutzend Frames, andere Nachrichten werden in der Zwischenzeit beantwortet.

`walkable` bäckt das Navigationsmesh der ganzen bearbeiteten Szene mit `TrenchDotNav` für `agent` (`radius`, `height`, `max_climb`, `max_slope` in Metern und Grad) und liefert es in den Einheiten und Achsen der Karte zurück. Mit `text` baut Godot diese Karte zuerst, außer sie wurde zuletzt aus demselben Text gebaut.

Pfade sind absolut und haben Schrägstriche. Nur `FuncGodotMap`-Nodes in der bearbeiteten Szene, also dem Szenen-Tab, den Godot zeigt, mit aktivem *Auto Rebuild On Save* nehmen teil. `scene` ist ihr `res://`-Pfad, bei geschlossener Szene leer.

`waiting` listet die anderen offenen Szenen-Tabs, die die Karte nutzen. Nach einem `map_saved` bauen sie die Datei, sobald ihr Tab angezeigt wird, nach einem `build` nicht. Ein `map_saved` für ein Prefab baut außerdem die Karten der bearbeiteten Szene neu, die es instanziieren, direkt oder über andere Prefabs, und zählt sie in `rebuilt` mit.

Was die Builds eines `map_saved` oder `build` gemeldet haben, kommt in drei Listen, jede nur, wenn sie nicht leer ist: `missing_classes` enthält die sortierten Classnames, die keine Definition beschrieb, `settings` die Pfade der Karteneinstellungen, die die Builds benutzt haben, und `warnings` bis zu 20 einzeilige Warnungen aus dem Build-Bericht. `unresolved` in einer Live-Antwort listet die sortierten Classnames, die dieser Schritt ohne Definition gebaut hat, ebenfalls nur, wenn es welche gibt.

`scan` passt zu Dateien, die außerhalb von Godot geschrieben wurden, etwa ein installiertes Entity-Pack. Eine Live-Sitzung, deren Szene mit anderen Entity-Definitionen gebaut wurde, als ihre Karteneinstellungen nach dem Scan enthalten, baut ihre Karte neu. Ein Addon ohne `scan` oder `play` beantwortet sie als unbekanntes Event.

`text` ist die ganze Karte als JSON, mit der gebackenen Beleuchtung unter `lightmap`, wenn die Karte einen Bake hat, damit ein Live-Build sie behält. Das macht etwa 11 Bytes pro Atlas-Texel aus, rund 11 MB bei einem Atlas von 1024 mal 1024, deshalb sendet der Editor die ganze Karte nur erneut, wenn eine Sitzung startet, nach einem neuen Bake oder nach einer großen Änderung.

`content` ist die Content-ID im END-Chunk der gespeicherten Datei, damit eine Szene, die aus einer unveränderten Datei gebaut wurde, beim Start einer Sitzung nicht noch einmal gebaut wird.

Ist der Port belegt, meistens durch einen zweiten Godot-Editor mit demselben Projekt, warnt das Addon einmal im Output-Panel und versucht es alle drei Sekunden erneut, sodass es übernimmt, sobald der andere Editor schließt.

## Live-Ops { #live-ops }

| Op | Felder | Bedeutung |
| --- | --- | --- |
| `set` | `id`, `parent`, `index`, `node` | Einen Node einfügen oder ersetzen, in der JSON-Form des `.tdm`-Formats, ohne Kinder |
| `remove` | `id` | Einen Node samt Teilbaum entfernen, gesendet wird nur die oberste entfernte ID |
| `translate` | `ids`, `offset` | Nodes verschieben, beim Ziehen statt der Geometrie gesendet |
| `properties` | `properties` | Die worldspawn-Eigenschaften ersetzen |

Die `epoch` einer Karte steigt, wann immer sie außerhalb einer Live-Sitzung gebaut wird oder sich die `FuncGodotMap`-Nodes ändern, die sie nutzen. Zeigt der nächste `status` eine neue Epoch, fang mit `live_begin` von vorn an.

Erzeugte Nodes tragen ihre Karten-Node-ID in den Metadaten `_td_id`, Ebenen- und Gruppen-Nodes in `_td_group`.

## Ohne Fenster testen { #testing-without-a-window }

Der Link funktioniert auch mit einem Godot-Editor im Headless-Modus, was sich für automatisierte Tests anbietet. Öffne eine Szene mit einer `FuncGodotMap` und sende dann `inspect` an Port 7842, um zu sehen, was Godot gebaut hat:

```sh
godot --headless --editor --path godot res://path/scene.tscn
```

(Hinweis: Führe bei einem frischen Checkout zuerst zweimal `godot --headless --path godot --import` aus. Ein Durchgang registriert nur die Klassen des Addons.)

Ein Godot im Headless-Modus zeichnet nichts, deshalb antwortet `capture` dort mit einem Fehler. Teste Captures mit einem Editor mit Fenster.

Die Addon-Tests tun das in [`godot/tests/editor/live_link_tabs.gd`](https://github.com/Paraxdev/TrenchDot/blob/main/godot/tests/editor/live_link_tabs.gd), das in einem Headless-Editor Szenen-Tabs öffnet, schließt, neu lädt und wechselt und prüft, was der Link meldet.

<div class="td-mcp" markdown>

## MCP { #mcp }

Der [MCP-Server](../mcp.md) lässt Agenten und Skripte den Editor steuern, und einige seiner Tools laufen über den Live Link. Das Tool `screenshot` nutzt `capture`, wenn es ein Bild aus Godot liefern soll.

Für Headless-Tests, die auch die Karte bearbeiten, starte TrenchDot mit seinem MCP-Server auf demselben Projekt wie den Headless-Godot-Editor und bearbeite die Karte über MCP:

```sh
trenchdot --mcp-http --project godot
```

</div>
