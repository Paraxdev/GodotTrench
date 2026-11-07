---
source: 92464837c8b6e8506e85d0270bb18896476cd41b
---

# JSON-Aufbau { #json-layout }

Die JSON-Form haben ältere Editoren gespeichert. Sie ist auch das, was `--dump`, eine `.json`-Speicherung, die Zwischenablage und der Live Link benutzen.

Leser behandeln jede Datei, die nicht mit `\x89GTM` beginnt, als JSON, mit oder ohne UTF-8-Byte-Order-Mark. Eine alte JSON-Karte lädt sich so, wie sie ist, und wird beim nächsten Speichern als `.tdm` binär.

[`json_fmt.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/json_fmt.rs) gibt sie so aus, dass Diffs lesbar bleiben. Es rückt um zwei Leerzeichen ein, hält die Keys in der Reihenfolge, in der diese Seiten sie aufführen, und setzt kurze Arrays und Objekte in eine Zeile. Jede Fläche ist eine Zeile, ebenso ein `vertices`-Array mit bis zu 64 Einträgen, wenn du also einen Brush verschiebst, ändern sich nur die Zeilen dieses Brushes.

Dein eigenes Tool muss dieses Layout nicht übernehmen, jedes gültige JSON mit demselben Inhalt lädt genauso.

## Konvertieren { #converting }

| Befehl | Ergebnis |
| --- | --- |
| `trenchdot --dump map.tdm` | Gibt die Karte als JSON aus |
| `trenchdot --to-json map.tdm map.json` | Schreibt die Karte als JSON |
| `trenchdot --to-tdm map.json map.tdm` | Schreibt eine binäre Karte und lehnt eine Datei ab, die der Editor nicht öffnen könnte |

Die Konvertierungen kopieren den Dateiinhalt, ohne ihn in den Editor zu laden, deshalb behalten sie Keys, die der Editor nicht kennt.

Der Editor öffnet auch `.json`-Karten, und *Speichern unter* mit einem `.json`-Namen schreibt JSON. JSON-Diffs von binären Karten in Git findest du unter [Kartendateien in Git](../development.md#map-files-in-git).

## Zwischenablage { #clipboard }

Kopieren und Einfügen benutzen dieselbe Node-Form in einer anderen Hülle:

```json
{
  "format": "trenchdot-clipboard",
  "version": 1,
  "nodes": [ ... ]
}
```

`nodes` enthält die kopierten Teilbäume ohne ihre Parents. Beim Einfügen laufen dieselben Prüfungen wie beim Laden, jeder Node bekommt eine frische ID, und `label`, `hidden` und `locked` bleiben erhalten. Eine kopierte Ebene fügt ihre Kinder ein.

Der Live Link schickt einzelne Nodes in derselben Form, ohne `children`.

## Beispiel { #example }

Ein gekürzter Auszug aus `godot/demo/maps/scripted_scene.tdm`, so wie `trenchdot --dump` ihn ausgibt, mit dem Boden-Brush und einem explosiven Fass:

```json
{
  "format": "trenchdot-map",
  "version": 1,
  "properties": {
    "ambient_color": "60 66 82",
    "classname": "worldspawn",
    "message": "Scripted Scene",
    "sky_ground_color": "60 62 70",
    "sky_horizon_color": "120 130 150",
    "sky_top_color": "40 54 92",
    "sun_angles": "-40 -55",
    "sun_energy": "0.6"
  },
  "layers": [
    {
      "id": 1,
      "type": "layer",
      "name": "Default",
      "color": "#7e67e6ff",
      "omit_from_export": false,
      "children": [
        {
          "id": 2,
          "type": "brush",
          "vertices": [[320.0,8.0,-320.0],[320.0,8.0,320.0],[320.0,0.0,320.0],[320.0,0.0,-320.0],[-320.0,8.0,320.0],[-320.0,8.0,-320.0],[-320.0,0.0,-320.0],[-320.0,0.0,320.0]],
          "faces": [
            {"indices":[0,1,2,3],"material":"showcase/cobble","uv":{"u_axis":[0.0,0.0,-1.0],"v_axis":[0.0,-1.0,0.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}},
            {"indices":[4,5,6,7],"material":"showcase/cobble","uv":{"u_axis":[0.0,0.0,1.0],"v_axis":[0.0,-1.0,0.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}},
            {"indices":[1,0,5,4],"material":"showcase/cobble","uv":{"u_axis":[1.0,0.0,0.0],"v_axis":[0.0,0.0,1.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}},
            {"indices":[3,2,7,6],"material":"showcase/cobble","uv":{"u_axis":[1.0,0.0,0.0],"v_axis":[0.0,0.0,1.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}},
            {"indices":[2,1,4,7],"material":"showcase/cobble","uv":{"u_axis":[1.0,0.0,0.0],"v_axis":[0.0,-1.0,0.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}},
            {"indices":[6,5,0,3],"material":"showcase/cobble","uv":{"u_axis":[-1.0,0.0,0.0],"v_axis":[0.0,-1.0,0.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}}
          ]
        },
        {
          "id": 17,
          "type": "entity",
          "classname": "prop_physics",
          "origin": [0.0,24.0,176.0],
          "angles": [0.0,0.0,0.0],
          "properties": {
            "explosion_damage": "25",
            "explosion_radius": "256",
            "explosive": "1",
            "health": "5",
            "model": "res://demo/scenes/barrel.tscn",
            "size": "16 24 16",
            "targetname": "barrel"
          },
          "outputs": [{"output":"broken","target":"hp_readout","input":"run"}]
        }
      ]
    }
  ]
}
```

Der Boden ist eine Platte von 640 mal 640 Einheiten und 8 Einheiten Dicke, deren Oberseite bei Y 8 liegt. Ihre erste Fläche, `[0,1,2,3]`, ist die +X-Seite, von +X aus gesehen gegen den Uhrzeigersinn gewickelt.

<div class="td-mcp" markdown>

## MCP { #mcp }

Die Tools des [MCP-Servers](../mcp.md) lesen und liefern Nodes ebenfalls in dieser JSON-Form.

</div>
