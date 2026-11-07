---
source: ac40f8ef73c25c546d11381f2a654f03d847b7aa
---

# MCP-Skripte { #mcp-scripts }

Ein MCP-Skript ist eine JSON-Datei mit einer Liste von [MCP](mcp.md)-Tool-Aufrufen, und `run_script` spielt sie der Reihe nach ab. Es hält fest, wie eine Karte gebaut wurde, als lesbare Schritte, sodass du die Karte von Grund auf neu bauen und sie in einem Pull Request als Text prüfen kannst, und auch die Tests können es abspielen.

Die Showcase-Karten in [examples/mcp](https://github.com/Paraxdev/TrenchDot/tree/main/examples/mcp) sind auf diese Art gebaut, und sie dienen zugleich als Beispiele für jedes Tool.

| Skript | Baut |
| --- | --- |
| `mountain_house.json` | Eine Hütte, die an einer Klippe hängt, mit einem Klippenlift und Wäldern |
| `church_school.json` | Eine Kirche und eine Schule, in der drei Knöpfe das Hoftor öffnen |
| `lighthouse_forest.json` | Ein Leuchtturm mit rotierendem Leuchtfeuer, ein Boot auf einem `func_train` und erscheinende Irrlichter |
| `withered_city.json` | Eine verfallene Neonstadt bei Nacht mit Blitzen, Glow und `ssr` |
| `night_district.json` | Eine Industriestraße bei Nacht mit einem Rolltor und einem Viaduktzug |
| `sea_island.json` | Das [Tutorial zur Seeinsel](tutorials/sea-island.md) |
| `scripted_scene.json` | Das [Tutorial zur Zwischensequenz](tutorials/cutscene.md) |
| `logic_playground.json` | Eine Halle für die [Mathe- und Logik-Nodes](gameplay/entities/logic.md): Münzen, ein Punktestand, ein Gate an der Tresortür und eine Lichtlotterie |

## Ausführen { #running-them }

Starte den Editor mit dem Demoprojekt und seinem HTTP-Server, siehe [Starten](mcp.md#starting-it), und spiele dann ein Skript mit `tools/mcp_script.py` ab. Fragt der Editor nach einem [Token](mcp.md#the-http-token), übergib es mit `--token` oder `TRENCHDOT_MCP_TOKEN`. Sein Befehl `shot` speichert einen Screenshot von der Kameraposition, die du ihm gibst, und `--shade` wählt die Schattierung:

```sh
trenchdot --mcp-http --project godot                        # the editor with the demo project
python tools/mcp_script.py run examples/mcp/sea_island.json   # replay a script
python tools/mcp_script.py shot view.png --pos 4300,900,3300 --look 2200,700,1200 --shade lit
```

Die Skripte speichern in `godot/demo/maps/showcase`, außer `scripted_scene.json` und `logic_playground.json`, die in `godot/demo/maps` speichern.

Ein Lauf hält beim ersten fehlgeschlagenen Schritt an, außer du übergibst `--continue-on-error`, und `--url` richtet den Runner auf einen Editor an einem anderen Port. Jeder Lauf ist in jeder Karte, die er ändert, ein einziger Undo-Schritt, benannt nach dem `label` des Skripts oder seiner Datei.

## Skriptformat { #script-format }

```json
{
  "format": "trenchdot-mcp-script",
  "label": "Hollow room",
  "vars": { "save_dir": "${project}/maps" },
  "steps": [
    { "note": "A hollow room." },
    { "tool": "map_file", "args": { "op": "new" } },
    { "tool": "create_brush", "args": { "min": [0, 0, 0], "max": [256, 128, 256] }, "save": "room" },
    { "tool": "select", "args": { "ids": "$room.ids" } },
    { "tool": "run_action", "args": { "action": "csg_hollow" } },
    { "tool": "map_file", "args": { "op": "save", "path": "${save_dir}/room.tdm" } }
  ]
}
```

`vars` setzt Standardwerte für deine eigenen Variablen. Ein `vars`-Objekt, das du an `run_script` übergibst, überschreibt sie, ein Skript kann also an verschiedene Orte speichern, und die Ordner, die du so übergibst, sind solche, aus denen das Skript lesen und in die es schreiben darf.

Ein Schritt ohne `tool` ist ein Kommentar. Ein Schritt mit `save` legt sein Ergebnis unter diesem Namen ab, damit spätere Schritte die IDs nutzen können, die er angelegt hat.

Skripte werden wie Karten weitergegeben, deshalb bearbeiten und speichern ihre Schritte nur die Karte. Ein Schritt kann kein anderes Projekt und keine seiner Karten öffnen, Godot nicht starten, nicht darin bauen, den Live-Modus nicht einschalten, MCP nicht ausschalten, nicht in die Zwischenablage kopieren oder ausschneiden, keine ungespeicherten Änderungen verwerfen und keine anderen Tabs schließen. Undo und Redo erreichen nur, was das Skript selbst getan hat, die Arbeit, die du nicht gespeichert hast, bleibt also in jedem Tab erhalten. `screenshot`, `simulate_input`, `walkability` und `run_script` lassen sich ebenfalls nicht in einem Skript ausführen, und Installationen über `project_content` außer `gameplay` auch nicht.

Jede Datei, die ein Schritt liest oder schreibt, muss im offenen Projekt, im eigenen Ordner des Skripts oder in einem Ordner liegen, den du in `vars` übergibst, auch wenn ein Link darin woandershin zeigt. Das gilt für die Karten, die es öffnet, prüft, vergleicht, speichert und exportiert, für die Karten, Modelle, Prefabs und Texturen, die es mit den Dateien importiert, die sie nennen, und für alles, was Konvertierungen, Importe, Exporte und die Prefab-Bibliothek schreiben. Prefabs, Modelldateien, Texturen und Hammer-Instanzen, die eine Karte oder ein Modell anderswo nennt, bleiben außen vor, Netzwerkpfade werden abgelehnt, bevor der Editor sie ansieht, und ein Speichern ohne Pfad schreibt nur in die Karte zurück, die beim Start des Skripts offen war.

Die Regel gilt auch für Materialdateien, ihre Bilder und Hotspot-Dateien, die Prefab-Bibliothek und Natur-Presets. Eine Textur, deren Dateien außerhalb dieser Ordner liegen, gilt in einem Skript als fehlend, und Anpassen oder Ausrichten fällt dann auf die Standard-Texturgröße zurück. Die Zusammenfassung listet die Dateien, die sie nicht lesen konnte, dann in `unread_textures` auf.

Ein abgelehnter Schritt erscheint mit dem Grund in den `errors` der Zusammenfassung. Die Zusammenfassung zählt Tool-Aufrufe als `steps` und Kommentare als `notes`.

Strings in `args` können sich auf gespeicherte Ergebnisse und ein paar eingebaute Variablen beziehen:

| Schreibe | Bedeutet |
| --- | --- |
| `"$name.path"` | Der gespeicherte JSON-Wert. Zahlen indizieren Arrays, wie in `$room.ids.0` |
| `"${name.path}"` | Sein Text innerhalb eines längeren Strings |
| `"$a.ids + $b.ids"` | Beide zu einem Array verbunden. `["$a.ids", "$b.ids"]` geht auch |
| `$last` | Das Ergebnis des vorigen Schritts |
| `$project` | Der Ordner des offenen Godot-Projekts, ein Fehler, wenn keines offen ist |
| `$script_dir` | Der Ordner der Skriptdatei |
| `$$` | Ein wörtliches `$`, ein Node-Pfad ist also `"$$Player"` |

Ein unbekannter Name ist ein Fehler. Ein `$`, das keinen Namen beginnen kann, wie in `"$5"`, bleibt, wie es ist. IDs, die als Text angegeben sind, etwa `"42"`, funktionieren überall dort, wo eine Node-ID erwartet wird.

`$name/rest` ohne geschweifte Klammern wird nicht expandiert, ein `$project/maps` ohne Klammern ist also ein Fehler und kein Pfad, den niemand so meinte. Nimm `"${project}/maps"`.

## IDs nach CSG { #ids-after-csg }

CSG- und Clip-Schritte ersetzen die Brushes, die sie schneiden, durch neue mit neuen IDs. Der Runner schreibt IDs, die frühere Schritte gespeichert haben, passend um, `"$wall.ids"` meint also weiterhin die ganze Wand, nachdem du ein Fenster hineingeschnitten hast, und ein entfernter Brush fällt heraus.

Umgeschrieben werden nur Werte unter Keys, die Node-IDs tragen: `id`, `ids`, `selection`, `selected`, `nodes`, `letters`, `copies`, `entity`, `group`, `parent`, `insert_parent`, `current_layer`, `open_groups`, `targets` und Keys, die auf `_id` oder `_ids` enden. Anzahlen wie `moved` von `hierarchy reparent` bleiben, wie sie sind, nimm stattdessen dessen `ids`.

> [!TIP]
> Schreib in Aufrufen von `scatter` und `blend` jede Brush-Einstellung aus. Alles, was du weglässt, kommt aus den aktuellen Einstellungen des Scatter-Panels oder Blend-Werkzeugs des Editors.
