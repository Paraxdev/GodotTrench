---
source: 5b0744accf923762bf8a9e744e70cb250be2483b
---

# Kommandozeile { #command-line }

```sh
trenchdot [options] [map]
```

Ohne Optionen öffnet sich einfach der Editor, mit der Karte, wenn du eine angibst. `trenchdot --help` (oder `-h`) listet die Optionen auf und beendet sich.

| Option | Wirkung |
| --- | --- |
| `--project <dir>` | Lädt das Godot-Projekt in `<dir>` oder in einem Ordner darüber und vertraut ihm für die Sitzung, siehe [Projektvertrauen](godot/running-godot.md#project-trust). Ohne sie lädt der Editor das letzte Projekt |
| `--default-prefs` | Startet mit den Standardeinstellungen und speichert sie nie. Nützlich für Skripte und Tests und um zu prüfen, ob ein Problem von deinen eigenen Einstellungen kommt |
| `--offscreen` | Öffnet das Fenster außerhalb aller Bildschirme, ohne Fokus und ohne Taskleistenknopf. Es rendert weiterhin, Skripte und Tests können also Screenshots machen, während du weiterarbeitest |
| `--live-link-port=<port>` | Spricht den Godot-Editor über diesen Port an, statt über den Port *Godot Live Link* in den Einstellungen, nur für diesen Lauf. Skripte und Tests übergeben einen freien Port, ihre Speicherungen bauen also nie Karten in dem Godot um, das du offen hast |
| `--hot-reload-port=<port>` | Macht dasselbe für den Port *Hot Reload im laufenden Spiel*, auf dem ein laufendes Spiel lauscht, um gespeicherte Karten neu zu laden |
| `--dump <map>` | Gibt eine Karte als JSON aus und beendet sich |
| `--to-json <map> <out>` | Schreibt eine Karte als JSON und beendet sich |
| `--to-tdm <map> <out>` | Schreibt eine Karte als binäre `.tdm` und beendet sich |
| `--export-glb <map> <out.glb>` | Schreibt Geometrie und Materialien der Karte als binäres glTF für Blender und andere 3D-Tools und beendet sich |
| `--export-obj <map> <out.obj>` | Schreibt dasselbe als OBJ, mit einer `.mtl` und einem Ordner mit Texturen daneben, und beendet sich |

Eine unbekannte Option oder eine, der ihr Wert fehlt, gibt einen Fehler aus und beendet sich mit Code 2, es öffnet sich kein Fenster. Eine Konvertierung oder ein Export, der fehlschlägt, gibt den Grund aus und beendet sich mit Code 1.

Die Konvertierungen und Exporte öffnen nie ein Fenster, sie sind also sicher in Skripten und Git-Hooks, siehe [Kartendateien in Git](development.md#map-files-in-git).

Die Exporte finden das Godot-Projekt der Karte über ihren Ordner und nutzen die Standardwerte von *Datei > Exportieren*: die ganze Karte, ohne ausgeblendete Objekte und Scatter-Instanzen. Entitätslogik, Trigger und Skripte bleiben draußen, siehe [Export nach Blender und in andere 3D-Tools](editor/exporting.md).

<div class="td-mcp" markdown>

## MCP { #mcp }

Diese Optionen richten den [MCP-Server](mcp.md) ein, damit Skripte und KI-Agenten den Editor steuern können.

| Option | Wirkung |
| --- | --- |
| `--mcp` | Bedient MCP über stdin und stdout |
| `--mcp-http[=<port>]` | Bedient MCP über HTTP auf 127.0.0.1, standardmäßig Port 7841 |
| `--mcp-token=<token>` | Lässt HTTP-Clients dieses Token senden, was auch immer in den Einstellungen steht. Es schlägt `TRENCHDOT_MCP_TOKEN`, siehe [Das HTTP-Token](mcp.md#the-http-token) |

`--default-prefs` passt gut zu beiden, damit eine Skript-Sitzung jedes Mal mit denselben Einstellungen beginnt. Ist `TRENCHDOT_DISABLE_MCP=1` gesetzt, startet keine der beiden Optionen einen Server, siehe [MCP ausschalten](mcp.md#turning-mcp-off).

</div>
