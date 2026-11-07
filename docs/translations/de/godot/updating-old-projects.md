---
source: 4c71c99f5fabec89f69f0c99554b4f3a60fb4148
---

# Ein GodotTrench-Projekt aktualisieren { #updating-a-godottrench-project }

TrenchDot hieß früher GodotTrench, und Beta-Builds hießen kurz LevelBench. Hast du damals ein Projekt eingerichtet oder Karten gespeichert, funktionieren sie weiter. Du aktualisierst das Addon nur einmal.

## Schritte { #steps }

1. Starte den neuen Editor. Beim ersten Start kopiert er deine Einstellungen, vertrauenswürdigen Projekte und zuletzt geöffneten Dateien aus dem alten Einstellungsordner. Der alte Ordner bleibt, wo er ist, der alte Editor funktioniert also auch weiter.
2. Öffne dein Godot-Projekt. Das Einrichtungsfenster öffnet sich und sagt, dass das Addon im Projekt älter ist als der Editor, auch wenn die Versionsnummern übereinstimmen. Schließ zuerst Godot, und drück dann **Addon auf 1.2.3 aktualisieren**, wobei 1.2.3 für die Version steht.
3. Öffne das Projekt einmal in Godot. Das Addon verschiebt seine Projekteinstellungen von `godottrench/` nach `trenchdot/` und schreibt `trenchdot_game.json` anstelle von `godottrench_game.json`.

## Was übernommen wird { #what-carries-over }

| Du hattest | Was passiert |
| --- | --- |
| `.gtm`-Karten | Sie öffnen sich unverändert. Die Statuszeile sagt "room.gtm aus dem Format GodotTrench aktualisiert, speichere, um es zu behalten". Speichern behält den Namen `.gtm`, deine Szenen finden die Karte also weiter. Neue Karten werden als `.tdm` gespeichert. |
| `GODOTTRENCH_`-Umgebungsvariablen | Sie funktionieren weiter. Sind beide Namen gesetzt, gewinnt der mit `TRENCHDOT_`. |
| Das Gameplay-Paket in `res://godottrench/entities` | Es funktioniert weiter, und deine eigenen Skripte, die seine Klassen erweitern, kompilieren weiter. Installierst du auch das neue Paket, gewinnt das neue, wo beide eine Entität definieren. Lass den alten Ordner an seinem Platz, solange deine Szenen ihn nutzen. |
| Natur-Pakete und Modelle in `res://godottrench/` | Karten, die sie platzieren, funktionieren weiter. Die Scatter-Presets suchen in `res://trenchdot/nature`, füg das Natur-Paket also erneut hinzu, um neuen Scatter zu malen. |
| Vom alten Addon gebaute Szenen | Sie laden, weil Godot die verschobenen Addon-Skripte über ihre UID findet. Die Karte erneut zu bauen frischt sie auf. |

<div class="td-mcp" markdown>

## MCP-Clients { #mcp-clients }

Der MCP-Server heißt jetzt `trenchdot`. Benenn ihn in der Konfiguration deines Clients um. Die Token-Variable ist `TRENCHDOT_MCP_TOKEN`, und das alte `GODOTTRENCH_MCP_TOKEN` funktioniert weiter.

</div>
