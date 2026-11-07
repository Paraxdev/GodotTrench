---
source: 92464837c8b6e8506e85d0270bb18896476cd41b
---

# Estructura JSON { #json-layout }

La forma JSON es lo que guardaban los editores antiguos. También es lo que usan `--dump`, un guardado `.json`, el portapapeles y el Live Link.

Los lectores tratan como JSON cualquier archivo que no empiece por `\x89GTM`, con o sin marca de orden de bytes UTF-8. Un mapa JSON antiguo se carga tal cual y pasa a binario en su siguiente guardado como `.tdm`.

[`json_fmt.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/json_fmt.rs) lo imprime para que los diffs sigan siendo legibles. Indenta con dos espacios, mantiene las claves en el orden en que las listan estas páginas y pone los arrays y objetos cortos en una sola línea. Cada cara es una línea, y también lo es un array `vertices` de hasta 64 entradas, así que mover un brush solo cambia las líneas de ese brush.

Tu propia herramienta no tiene que copiar esta disposición, cualquier JSON válido con el mismo contenido se carga igual.

## Conversión { #converting }

| Comando | Resultado |
| --- | --- |
| `trenchdot --dump map.tdm` | Imprime el mapa como JSON |
| `trenchdot --to-json map.tdm map.json` | Escribe el mapa como JSON |
| `trenchdot --to-tdm map.json map.tdm` | Escribe un mapa binario, y rechaza un archivo que el editor no podría abrir |

Las conversiones copian el contenido del archivo sin cargarlo en el editor, así que conservan las claves que el editor no conoce.

El editor también abre mapas `.json`, y *Guardar como* con un nombre `.json` escribe JSON. Para ver diffs JSON de mapas binarios en git, mira [Archivos de mapa en git](../development.md#map-files-in-git).

## Portapapeles { #clipboard }

Copiar y pegar usan la misma forma de nodo en otro envoltorio:

```json
{
  "format": "trenchdot-clipboard",
  "version": 1,
  "nodes": [ ... ]
}
```

`nodes` contiene los subárboles copiados sin sus padres. Pegar ejecuta las mismas comprobaciones que cargar, da a cada nodo un id nuevo y conserva `label`, `hidden` y `locked`. Una capa copiada pega sus hijos.

El Live Link envía nodos sueltos con la misma forma, sin `children`.

## Ejemplo { #example }

Un extracto recortado de `godot/demo/maps/scripted_scene.tdm` tal como lo imprime `trenchdot --dump`, con el brush del suelo y un barril explosivo:

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

El suelo es una losa de 640 por 640 unidades y 8 unidades de grosor con su parte superior en Y 8. Su primera cara, `[0,1,2,3]`, es el lado +X, con el orden de vértices antihorario visto desde +X.

<div class="td-mcp" markdown>

## MCP { #mcp }

Las herramientas del [servidor MCP](../mcp.md) también leen y devuelven nodos en esta forma JSON.

</div>
