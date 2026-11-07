---
source: d5a662c31724dba93b77bd318ab3a570c43e95b6
---

# MCP-Server { #mcp-server }

Im Editor steckt ein [MCP](https://modelcontextprotocol.io)-Server. Ein Agent wie Claude Code oder ein schlichtes Python-Skript kann damit Karten untersuchen und bearbeiten, Screenshots machen und Eingaben in die Oberfläche spielen.

Nutze ihn, damit ein KI-Assistent einen Level baut oder prüft, um wiederkehrende Änderungen zu skripten oder um den Editor automatisch zu testen. Der Server ist aus, bis du ihn startest, und er lauscht nur auf deinem eigenen Rechner.

## Starten { #starting-it }

| Transport | So startest du ihn |
| --- | --- |
| HTTP | Starte den Editor mit `--mcp-http` für Port 7841 oder mit `--mcp-http=PORT` für einen anderen Port. Soll er bei jedem Start laufen, gehe zu *Datei > Einstellungen*, öffne *Erweitert*, hake *beim Start aktivieren* neben *MCP-Server (HTTP, localhost)* an, stelle daneben den Port ein und starte neu |
| stdio | Starte mit `--mcp`. Der Client startet den Editor für jede Sitzung selbst und spricht über Standardeingabe und Standardausgabe mit ihm |

Nimm HTTP, wenn du den Editor offen lässt und selbst darin arbeitest, während ein Agent hilft. Nimm stdio, wenn der Client seinen eigenen Editor besitzen soll oder wenn andere Leute denselben Computer nutzen. So verbindest du Claude Code mit einem von beiden:

```sh
claude mcp add --transport http trenchdot http://127.0.0.1:7841/mcp   # HTTP
claude mcp add trenchdot -- /path/to/trenchdot --mcp                 # stdio
```

Die `.mcp.json` des Repositorys verweist Claude Code schon auf den HTTP-Server. *Datei > Einstellungen* zeigt den Befehl für den Server, der läuft, und *Befehl kopieren* legt ihn in die Zwischenablage.

Hast du den Editor auf einem anderen Port gestartet? Setze `TRENCHDOT_MCP_URL`, dann verbinden sich
[tools/mcp_client.py](https://github.com/Paraxdev/TrenchDot/blob/main/tools/mcp_client.py) und
[tools/mcp_script.py](https://github.com/Paraxdev/TrenchDot/blob/main/tools/mcp_script.py) stattdessen dorthin.

### Das HTTP-Token { #the-http-token }

Jedes Programm auf deinem Computer erreicht den HTTP-Server, und ebenso andere Leute, die daran angemeldet sind. Um sie auszusperren, hake unter *Erweitert* in *Datei > Einstellungen* *Token verlangen* an. Ab der nächsten Anfrage antwortet der Server nur noch auf Anfragen, die sein Token als `Authorization: Bearer <token>` senden. Der Befehl in den Einstellungen enthält dann das Token, nimm ihn statt der `.mcp.json`. stdio braucht kein Token, nur der Client, der den Editor gestartet hat, kann mit ihm sprechen.

Der Editor erzeugt ein zufälliges Token, wenn du zum ersten Mal eins brauchst, und legt es in `mcp-token` ab, einer Datei, die nur du lesen kannst, neben seinen Einstellungen: `%APPDATA%\TrenchDot\data` unter Windows, `~/.local/share/trenchdot` unter Linux und `~/Library/Application Support/TrenchDot` unter macOS. Ein Lauf mit `--default-prefs` behält sein eigenes in `trenchdot-default-prefs.mcp-token` im Temp-Ordner.

Um das Token für einen Lauf selbst zu wählen, setze `TRENCHDOT_MCP_TOKEN` oder starte mit `--mcp-token=<token>`, das die Variable schlägt. Beides schaltet das Token ein, was auch immer in den Einstellungen steht, und *Token verlangen* erscheint angehakt und ausgegraut. `tools/mcp_client.py` und `tools/mcp_script.py` senden ein Token nur, wenn du eins mitgibst, mit `--token` oder `TRENCHDOT_MCP_TOKEN`.

## MCP ausschalten { #turning-mcp-off }

Um Agenten und Skripte ganz aus dem Editor herauszuhalten, nimm im MCP-Teil von *Datei > Einstellungen* oben den Haken bei *MCP* weg. Der HTTP-Server schließt sofort und kappt seine offenen Verbindungen, ein Client über stdio bekommt bei jeder Anfrage einen Fehler, und die übrigen MCP-Einstellungen werden ausgegraut. Die Einstellung wird sofort gespeichert, jeder Editor, den du ohne `--default-prefs` startest, lässt MCP also aus, bis du den Haken wieder setzt, was den HTTP-Server auf seinem Port zurückbringt.

![Der MCP-Teil der Einstellungen: MCP angehakt, die Zeile des HTTP-Servers mit ihrem Port und Token verlangen](assets/mcp/prefs-mcp.png)

Um MCP für einen Lauf auszuschalten, was auch immer in den Einstellungen steht, setze `TRENCHDOT_DISABLE_MCP` auf alles außer leer oder `0`. *MCP* erscheint dann nicht angehakt und ausgegraut. Solange MCP aus ist, starten `--mcp` und `--mcp-http` keinen Server. Der Editor öffnet sich trotzdem und seine Statuszeile sagt, dass MCP aus ist, und ein Client über stdio bekommt einen Fehler, der das sagt, statt zu warten.

Auch ein Agent kann MCP ausschalten, mit `set_editor` und `"mcp": false`. Nur du kannst es wieder einschalten.

## Werkzeuge { #tools }

| Bereich | Wofür er da ist | Werkzeuge |
| --- | --- | --- |
| Untersuchen | Die Karte, den Editorzustand, die Entitätsdefinitionen und die Nutzung einer Entität aus Spielcode lesen, ohne etwas zu ändern | `get_state`, `summarize_map`, `list_nodes`, `get_node`, `changes_since`, `validate_map`, `get_game_config`, `code_reference` |
| Erstellen | Brushes, Meshes, Terrains, Entitäten und importierte Modelle hinzufügen und an Flächenkanten entlang trimmen | `create_brush`, `create_mesh`, `create_terrain`, `create_entity`, `import_model`, `trim` |
| Bearbeiten | Ändern, was existiert: Entitätsschlüssel, Auswahl, Platzierung, Flächen, Texturen, Terrain, Ebenen und Gruppen sowie worldspawn | `update_entity`, `select`, `transform`, `duplicate`, `set_face`, `mesh_edit`, `texture`, `terrain_edit`, `hierarchy`, `set_map_properties`, `bake_lighting` |
| Malen und Gameplay | Modelle scattern, Materialien mischen, Teile aus der Prefab-Bibliothek einfügen und Brushes zu Türen, Plattformen und Knöpfen machen | `scatter`, `blend`, `prefab_library`, `random_fill`, `gameplay` |
| Editor | Menüaktionen ausführen, Dateien öffnen und speichern, Editoreinstellungen ändern, die Kamera bewegen, Screenshots machen, Eingaben senden und prüfen, wo ein Spieler gehen kann | `run_action`, `map_file`, `open_project`, `set_editor`, `set_camera`, `screenshot`, `simulate_input`, `walkability` |
| Projekt | Das Gameplay-Entitätenpaket, die Natur-Modelle oder die Demo zum geöffneten Godot-Projekt hinzufügen | `project_content` |
| Skripte | Eine gespeicherte Liste von Werkzeugaufrufen abspielen, siehe [MCP-Skripte](mcp-scripts.md) | `run_script` |

Die Beschreibung jedes Werkzeugs listet seine Operationen und Argumente auf, ein Agent lernt sie also direkt vom Server.
[tools/mcp_client.py](https://github.com/Paraxdev/TrenchDot/blob/main/tools/mcp_client.py) gibt sie mit `list` aus
und ruft ein Werkzeug von der Shell aus auf:

```sh
python tools/mcp_client.py call screenshot '{"target":"3d","width":1280,"height":720}' --out shot.png
```

## Screenshots { #screenshots }

`screenshot` liefert ein PNG des ganzen Editorfensters oder einer Ansicht, wenn `target` `3d`, `top`, `front` oder `side` ist. Bei einer Ansicht entscheiden die anderen Argumente, was du bekommst:

| Argumente | Ergebnis |
| --- | --- |
| Keine | Das ganze Editorfenster |
| Nur `target` | Diese angedockte Ansicht, wie sie ist, mit Overlays |
| `width` und `height` | Offscreen gerendert bis 4096 Pixel, ohne Raster, Entitätsboxen, Gizmos oder Auswahl |
| `width`, `height` und `"overlays": true` | Offscreen, mit den Editor-Overlays |
| `"overlays": false` | Ein sauberes Bild in der angedockten Größe |

### Was Godot rendert { #what-godot-renders }

`"source": "godot"` bittet den Godot-Editor, die Karte mit seinen echten Lichtern, seiner Umgebung, seinem Nebel und seiner Nachbearbeitung zu rendern. Du bekommst dieses PNG zurück, dazu alle Fehler oder Warnungen, die Godot beim Bauen protokolliert hat. Praktisch, um Licht und Materialien zu beurteilen, ohne das Spiel zu starten.

```sh
python tools/mcp_client.py call screenshot '{"source":"godot","position":[0,256,512],"look_at":[0,64,0]}' --out godot.png
```

1. Speichere die Karte in dem Projekt, das in TrenchDot geöffnet ist.
2. Öffne dieses Projekt im Godot-Editor, mit einer Szene, deren `FuncGodotMap` die Karte baut.
3. Rufe `screenshot` mit `"source": "godot"` auf. Ohne `position` und `look_at` nimmt es die Kamera der 3D-Ansicht.

Ohne [Live-Modus](godot/live.md) baut Godot die Karte zuerst so, wie sie in TrenchDot zu sehen ist, wie *In Godot bauen*. `"build": false` nimmt die Szene auf, wie Godot sie hat. `width` und `height` sind standardmäßig 1280 mal 720 und `fov` der der 3D-Ansicht.

> [!NOTE]
> Godot braucht ein Fenster zum Rendern, ein Godot ohne Fenster gibt einen Fehler statt eines Bildes zurück. Das Fenster darf im Hintergrund oder minimiert sein.

## Eine Karte lesen und prüfen { #reading-and-reviewing-a-map }

| Schritt | Werkzeug |
| --- | --- |
| Einen Überblick bekommen | `summarize_map` listet die Ebenen, Entitätsklassen, I/O-Verbindungen und Materialien auf, dazu fehlende Assets |
| Nodes finden | `list_nodes` filtert nach `layer`, `material`, `property` (`{"model": "*crate*"}`) oder einer `box` in der Karte |
| Einen ansehen | `get_node` liefert einen Node. `compact: true` lässt Vertices und Flächen weg |
| Zeigen, was du geändert hast | `changes_since` vergleicht mit der letzten Speicherung, einer Anzahl `undo_steps` zurück oder einer Karten-`file` |
| Zurücknehmen | Jeder Aufruf und jedes `run_script` ist ein Rückgängig-Schritt namens `MCP: ...`. `run_action undo` mit `steps` macht mehrere rückgängig |

Siehe [Änderungen prüfen](editor/reviewing.md), um Karten in Git zu halten.

## Gut zu wissen { #behavior-worth-knowing }

| Thema | Verhalten |
| --- | --- |
| Browser | Der HTTP-Server weist jede Anfrage mit einem `Origin`-Header und jedes POST ab, das nicht `application/json` ist, Webseiten erreichen ihn also nicht, auch nicht solche, die von localhost kommen |
| Fehler | Ein Werkzeug, das seine Arbeit nicht tun konnte, liefert `isError`, auch ein fehlgeschlagenes `run_action` |
| Dateidialoge | Aktionen, die einen öffnen, werden abgewiesen, nimm `map_file` mit einem Pfad. `run_action save` braucht eine Karte, die schon eine Datei hat |
| Ungespeicherte Tabs | `run_action close_tab` weist ungespeicherte Änderungen ab, außer `args.discard` ist true. `args.close_others` schließt stattdessen jeden Tab außer dem aktiven |
| Dateipfade | `res://` und relative Pfade in `map_file`, `run_script`, `import_model`, `changes_since` und `validate_map` beginnen beim geöffneten Projekt oder, ohne eines, im Arbeitsverzeichnis des Editors. `map_file open` und `open_tab` lösen den Pfad zu einem absoluten auf, bevor sie öffnen, ein späteres `save` kann also nicht woanders landen |
| Editoreinstellungen | `transform` und `run_action rotate` und `flip` drehen um die Mitte der Auswahl, nur am Raster eingerastet, solange Einrasten an ist, und halten Texturen gesperrt, solange UV-Sperre an ist. `transform` nimmt `snap` und `uv_lock`, um das für einen Aufruf zu überstimmen, `duplicate` nimmt `uv_lock`. `run_action sculpt` und `blend` nutzen die Werkzeugeinstellungen, die ein Mensch ändern kann, `terrain_edit` nutzt feste Standardwerte |
| Auswahl | `select` weist unbekannte IDs ab und überspringt ausgeblendete oder gesperrte Nodes, die es in `skipped` aufführt |
| CSG | `csg_subtract`, `csg_merge`, `csg_intersect`, `csg_hollow` und `clip_apply` liefern `replaced`, alte IDs auf neue abgebildet |
| Zeitlimits | Aufrufe laufen nach 120 s ab, außer `run_script`, `project_content` und `bake_lighting` |
| Inhaltsfrage | Die Frage, die ein neues Projekt zu fertigen Inhalten bekommt, schließt sich beim ersten Aufruf außer `get_state`, `screenshot` und `simulate_input` und öffnet sich in dieser Sitzung nicht wieder |
| Skriptvariablen | `$name/rest` (ohne Klammern) ist ein Fehler, wenn `name` eine bekannte Variable ist, denn nur `$name` allein oder `${name}/rest` lösen sie auf. Schreibe `${project}/maps` |

## Hinweise zu Werkzeugen { #tool-notes }

| Werkzeug | Hinweis |
| --- | --- |
| `run_action` `copy`, `cut`, `paste` | Copy und cut liefern den Text der Zwischenablage, paste nimmt ihn als `args.text` mit optionalem `offset` oder `origin` |
| `run_action` `clip_apply` | Nimmt eine Ebene (`point` und `normal` oder drei `points`) und `keep`: `front`, `back` oder `both` |
| `run_action` `create_decal` | Platziert ein [Decal](editor/decals.md), nimm es statt dünner Brushes |
| `create_brush` | `openings` schneidet Türen und Fenster aus, `{min, max, count, step, rows, row_step}` wiederholt eine als Raster |
| `create_mesh` `rotate` | `[x, y, z]` Grad in der YXZ-Reihenfolge von Godot, `[0, 0, 90]` legt einen Zylinder also entlang X |
| `transform` | Wendet translate, rotate, flip und scale_to in dieser Reihenfolge an, als ein Rückgängig-Schritt. `rotate` dreht um 90 Grad um Y, sofern nichts anderes gesagt wird |
| `create_terrain` | `origin` ist die minimale Ecke des Terrains, anders als bei anderen Werkzeugen. `center` platziert es stattdessen um einen Punkt |
| `create_brush`, `create_mesh` | Ein Giebelfirst verläuft bei Brushes entlang Z und bei Meshes entlang X. Beide Werkzeuge nehmen `ridge_x` und `ridge_z` |
| `map_file` exports | `path` ist die geschriebene Datei, für ältere Aufrufer beibehalten, `exported` nennt sie auch und `map_path` ist die geöffnete Karte |
| `terrain_edit` | `probe [x, z]` ohne `op` liest nur die Höhe. `clear_layer` braucht `layer` |
| `validate_map` | Akzeptiert jede Methode oder Eigenschaft der Godot-Klasse oder des Skripts des Ziels als Input. `coplanar_faces` markiert Flächen, die flackern werden. Jedes Problem hat `bounds`. `path` prüft eine gespeicherte Karte, ohne sie zu öffnen, `project: true` prüft jede Karte im Projekt |
| `run_action` `place_entities` | Nimmt `args.classnames` sowie `at` und `normal`, um die Entitäten auf eine Fläche zu setzen, so wie ein Ziehen aus dem Tab Entitäten es tut. Keine classnames ist ein Fehler |
| `simulate_input` drags | Ein Ziehen mit `"release": false` hält die Taste und ihre Modifikatoren gedrückt, nachdem der Aufruf zurückkehrt, ein Screenshot zeigt das laufende Ziehen also, zum Beispiel eine Karte neben dem Cursor. Ein `release`-Ereignis in einem späteren Aufruf lässt es dort fallen, wo der Zeiger ist, oder bei seinem `x` und `y` oder `world`, und ein Release mit nur einem von `x` und `y` ist ein Fehler. Das Ziehen und seine Modifikatoren bleiben gedrückt, auch über die echte Tastatur, bis zu diesem Release, einer Aktion `new_map` oder einem erfolgreichen `close_tab` oder MCP ausgeht, vergiss das Release also nie. Klicks und Ziehen werden bis dahin abgewiesen |
| `run_action` `reload_materials` | Durchsucht den Texturordner jetzt neu, auch wenn neue Dateien ohnehin von selbst gefunden werden |
| `set_editor` `live_link_port` | Spricht einen Godot-Editor an, dessen Projekt einen anderen `trenchdot/live_link_port` setzt |
| `scatter` `preset` | Fügt die eingebetteten Blockbench-Modelle hinzu, die eine Voreinstellung braucht. Eine glTF-Voreinstellung in einem Projekt ohne das Natur-Paket ist ein Fehler, der `project_content` nennt |
| `bake_lighting` | Backt wie [Beleuchtung backen](editor/light-baking.md) und wartet darauf. Weggelassene Optionen behalten, womit die Karte zuletzt gebacken wurde. `bake: false` schaltet nur die `view` um, `remove: true` entfernt das Backen. `out_of_date` sagt, ob sich die Karte seitdem geändert hat |
| `project_content` | `install: "nature"` oder `"demo"` lädt aus dem Release herunter und antwortet, sobald die Dateien da sind. `install: "gameplay"` kopiert das Gameplay-Entitätenpaket aus dem Addon des Projekts, auch in Skripten. Ohne Argumente sagt es, was das Projekt hat |
