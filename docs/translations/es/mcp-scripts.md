---
source: ac40f8ef73c25c546d11381f2a654f03d847b7aa
---

# Scripts MCP { #mcp-scripts }

Un script MCP es un archivo JSON con una lista de llamadas a herramientas [MCP](mcp.md), y `run_script` las reproduce en orden. Guarda cómo se construyó un mapa como pasos legibles, así que puedes reconstruir el mapa desde cero y revisarlo como texto en una pull request, y las pruebas también pueden reproducirlo.

Los mapas de muestra de [examples/mcp](https://github.com/Paraxdev/TrenchDot/tree/main/examples/mcp) se construyen así, y sirven de ejemplo para todas las herramientas.

| Script | Construye |
| --- | --- |
| `mountain_house.json` | Una cabaña colgada de un acantilado, con un ascensor de acantilado y bosques |
| `church_school.json` | Una iglesia y una escuela donde tres botones abren la puerta del patio |
| `lighthouse_forest.json` | Un faro con una baliza giratoria, un bote en un `func_train` y wisps que aparecen |
| `withered_city.json` | Una ciudad de neón en ruinas de noche con relámpagos, brillo y `ssr` |
| `night_district.json` | Una calle industrial de noche con una puerta enrollable y un tren sobre un viaducto |
| `sea_island.json` | El [tutorial de la isla marina](tutorials/sea-island.md) |
| `scripted_scene.json` | El [tutorial de la escena animada](tutorials/cutscene.md) |
| `logic_playground.json` | Una sala para los [nodos de matemáticas y lógica](gameplay/entities/logic.md): monedas, una puntuación, una compuerta en la puerta de la cámara acorazada y una lotería de luces |

## Ejecutarlos { #running-them }

Inicia el editor con el proyecto de demostración y su servidor HTTP, mira [Iniciarlo](mcp.md#starting-it), y después reproduce un script con `tools/mcp_script.py`. Si el editor pide un [token](mcp.md#the-http-token), pásalo con `--token` o `TRENCHDOT_MCP_TOKEN`. Su comando `shot` guarda una captura desde la posición de cámara que le des, y `--shade` elige el sombreado:

```sh
trenchdot --mcp-http --project godot                        # the editor with the demo project
python tools/mcp_script.py run examples/mcp/sea_island.json   # replay a script
python tools/mcp_script.py shot view.png --pos 4300,900,3300 --look 2200,700,1200 --shade lit
```

Los scripts guardan en `godot/demo/maps/showcase`, salvo `scripted_scene.json` y `logic_playground.json`, que guardan en `godot/demo/maps`.

Una ejecución se detiene en el primer paso que falla salvo que pases `--continue-on-error`, y `--url` apunta el ejecutor a un editor en otro puerto. Cada ejecución es un solo paso de deshacer en cada mapa que cambia, con el nombre del `label` del script o de su archivo.

## Formato del script { #script-format }

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

`vars` define valores por defecto para tus propias variables. Un objeto `vars` pasado a `run_script` los sustituye, así que un mismo script puede guardar en sitios distintos, y las carpetas que pasas de esta forma son aquellas de las que el script puede leer y en las que puede escribir.

Un paso sin `tool` es un comentario. Un paso con `save` guarda su resultado con ese nombre, para que los pasos siguientes puedan usar los ids que creó.

Los scripts se comparten como los mapas, así que sus pasos solo editan y guardan el mapa. Un paso no puede abrir otro proyecto ni uno de sus mapas, iniciar Godot, construir en él, activar el modo en vivo, desactivar MCP, copiar o cortar al portapapeles, descartar cambios sin guardar ni cerrar otras pestañas. Deshacer y rehacer solo alcanzan lo que hizo el propio script, así que el trabajo que no has guardado se queda en cada pestaña. `screenshot`, `simulate_input`, `walkability` y `run_script` tampoco pueden ejecutarse dentro de un script, ni las instalaciones de `project_content` distintas de `gameplay`.

Todo archivo que un paso lee o escribe tiene que estar en el proyecto abierto, en la carpeta del propio script o en una carpeta que pases en `vars`, también cuando un enlace dentro de ellos apunta a otro sitio. Eso cubre los mapas que abre, comprueba, compara, guarda y exporta, los mapas, modelos, prefabs y texturas que importa con los archivos que nombran, y todo lo que escriben las conversiones, las importaciones, las exportaciones y la biblioteca de prefabs. Los prefabs, archivos de modelo, texturas e instancias de Hammer que un mapa o un modelo nombra en otro lugar se dejan fuera, las rutas de red se rechazan antes de que el editor las mire, y un guardado sin ruta escribe de vuelta solo en el mapa que estaba abierto cuando empezó el script.

La regla también vale para los archivos de material, sus imágenes y los archivos de hotspots, la biblioteca de prefabs y los presets de naturaleza. Una textura cuyos archivos están fuera de esas carpetas se lee como ausente dentro de un script, así que ajustarla o alinearla recurre al tamaño de textura por defecto. El resumen lista entonces los archivos que no pudo leer en `unread_textures`.

Un paso rechazado aparece en los `errors` del resumen con el motivo. El resumen cuenta las llamadas a herramientas como `steps` y los comentarios como `notes`.

Los strings de `args` pueden referirse a resultados guardados y a unas pocas variables integradas:

| Escribe | Significa |
| --- | --- |
| `"$name.path"` | El valor JSON guardado. Los números indexan arrays, como en `$room.ids.0` |
| `"${name.path}"` | Su texto dentro de un string más largo |
| `"$a.ids + $b.ids"` | Ambos unidos en un solo array. `["$a.ids", "$b.ids"]` también funciona |
| `$last` | El resultado del paso anterior |
| `$project` | La carpeta del proyecto de Godot abierto, un error si no hay ninguno abierto |
| `$script_dir` | La carpeta del archivo del script |
| `$$` | Un `$` literal, así que la ruta de un nodo es `"$$Player"` |

Un nombre desconocido es un error. Un `$` que no puede empezar un nombre, como `"$5"`, se queda como está. Los ids dados como texto, como `"42"`, funcionan donde se espera un id de nodo.

`$name/rest` sin llaves no se expande, así que un `$project/maps` sin llaves es un error en lugar de una ruta que nadie quiso escribir. Usa `"${project}/maps"`.

## Ids tras CSG { #ids-after-csg }

Los pasos de CSG y de corte reemplazan los brushes que cortan por otros nuevos con ids nuevos. El ejecutor reescribe los ids guardados por pasos anteriores para que coincidan, así que `"$wall.ids"` sigue significando la pared entera después de cortarle una ventana, y un brush eliminado desaparece de la lista.

Solo se reescriben los valores bajo claves que llevan ids de nodo: `id`, `ids`, `selection`, `selected`, `nodes`, `letters`, `copies`, `entity`, `group`, `parent`, `insert_parent`, `current_layer`, `open_groups`, `targets` y las claves que terminan en `_id` o `_ids`. Los recuentos como `moved` de `hierarchy reparent` se quedan como están, usa en su lugar su `ids`.

> [!TIP]
> Escribe explícitamente cada ajuste de brush en las llamadas a `scatter` y `blend`. Todo lo que se omita sale de los ajustes actuales del panel Scatter o de la herramienta Blend del editor.
