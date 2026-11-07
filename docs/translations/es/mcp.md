---
source: d5a662c31724dba93b77bd318ab3a570c43e95b6
---

# Servidor MCP { #mcp-server }

El editor lleva incorporado un servidor [MCP](https://modelcontextprotocol.io). Un agente como Claude Code, o un simple script de Python, puede usarlo para inspeccionar y editar mapas, hacer capturas de pantalla y enviar entrada a la interfaz.

Úsalo para que un asistente de IA construya o revise un nivel, para automatizar ediciones repetitivas o para probar el editor automáticamente. El servidor está apagado hasta que lo inicias, y solo escucha en tu propio equipo.

## Cómo iniciarlo { #starting-it }

| Transporte | Cómo iniciarlo |
| --- | --- |
| HTTP | Lanza el editor con `--mcp-http` para el puerto 7841, o con `--mcp-http=PORT` para otro puerto. Para que arranque siempre, ve a *Archivo > Preferencias*, abre *Avanzado*, marca *activar al iniciar* junto a *Servidor MCP (HTTP, localhost)*, pon el puerto a su lado y reinicia |
| stdio | Lanza con `--mcp`. El cliente inicia el propio editor en cada sesión y habla con él por la entrada y la salida estándar |

Elige HTTP cuando mantienes el editor abierto y trabajas en él tú mismo mientras un agente ayuda. Elige stdio cuando el cliente debe tener su propio editor, o cuando otras personas usan el mismo equipo. Para conectar Claude Code a cualquiera de los dos:

```sh
claude mcp add --transport http trenchdot http://127.0.0.1:7841/mcp   # HTTP
claude mcp add trenchdot -- /path/to/trenchdot --mcp                 # stdio
```

El `.mcp.json` del repositorio ya apunta Claude Code al servidor HTTP. *Archivo > Preferencias* muestra el comando del servidor que está en marcha, y *Copiar comando* lo pone en el portapapeles.

¿Iniciaste el editor en otro puerto? Define `TRENCHDOT_MCP_URL` y
[tools/mcp_client.py](https://github.com/Paraxdev/TrenchDot/blob/main/tools/mcp_client.py) y
[tools/mcp_script.py](https://github.com/Paraxdev/TrenchDot/blob/main/tools/mcp_script.py) se conectan ahí en su lugar.

### El token HTTP { #the-http-token }

Cualquier programa de tu equipo puede llegar al servidor HTTP, y también otras personas con sesión iniciada en él. Para dejarlas fuera, marca *Exigir un token* en *Avanzado* de *Archivo > Preferencias*. Desde la siguiente petición, el servidor solo responde a las que envían su token como `Authorization: Bearer <token>`. El comando de Preferencias incluye entonces el token, úsalo en lugar de `.mcp.json`. stdio no necesita token, solo el cliente que inició el editor puede hablar con él.

El editor crea un token aleatorio la primera vez que lo pides y lo guarda en `mcp-token`, un archivo que solo tú puedes leer, junto a sus ajustes: `%APPDATA%\TrenchDot\data` en Windows, `~/.local/share/trenchdot` en Linux y `~/Library/Application Support/TrenchDot` en macOS. Una ejecución con `--default-prefs` guarda el suyo en `trenchdot-default-prefs.mcp-token` en la carpeta temporal.

Para elegir tú mismo el token en una ejecución, define `TRENCHDOT_MCP_TOKEN` o lanza con `--mcp-token=<token>`, que tiene prioridad sobre la variable. Cualquiera de las dos activa el token diga lo que diga Preferencias, y *Exigir un token* aparece marcado y en gris. `tools/mcp_client.py` y `tools/mcp_script.py` envían un token solo cuando se lo das, con `--token` o `TRENCHDOT_MCP_TOKEN`.

## Apagar MCP { #turning-mcp-off }

Para mantener a agentes y scripts fuera del editor por completo, desmarca *MCP* en la parte superior del apartado MCP de *Archivo > Preferencias*. El servidor HTTP se cierra al instante y corta sus conexiones abiertas, un cliente en stdio recibe un error en cada petición y los demás ajustes de MCP se ponen en gris. El ajuste se guarda al momento, así que todos los editores que inicies sin `--default-prefs` mantienen MCP apagado hasta que lo marques de nuevo, lo que devuelve el servidor HTTP a su puerto.

![El apartado MCP de Preferencias: MCP marcado, la fila del servidor HTTP con su puerto y Exigir un token](assets/mcp/prefs-mcp.png)

Para apagar MCP en una sola ejecución diga lo que diga Preferencias, define `TRENCHDOT_DISABLE_MCP` con cualquier valor que no sea vacío ni `0`. *MCP* aparece entonces desmarcado y en gris. Mientras MCP está apagado, `--mcp` y `--mcp-http` no inician ningún servidor. El editor se abre igualmente y su línea de estado dice que MCP está apagado, y un cliente en stdio recibe un error que lo indica en lugar de esperar.

Un agente también puede apagar MCP, con `set_editor` y `"mcp": false`. Solo tú puedes volver a encenderlo.

## Herramientas { #tools }

| Área | Para qué sirve | Herramientas |
| --- | --- | --- |
| Inspeccionar | Leer el mapa, el estado del editor, las definiciones de entidades y cómo usar una entidad desde el código del juego, sin cambiar nada | `get_state`, `summarize_map`, `list_nodes`, `get_node`, `changes_since`, `validate_map`, `get_game_config`, `code_reference` |
| Crear | Añadir brushes, mallas, terrenos, entidades y modelos importados, y recortar a lo largo de los bordes de las caras | `create_brush`, `create_mesh`, `create_terrain`, `create_entity`, `import_model`, `trim` |
| Editar | Cambiar lo que existe: claves de entidades, selección, colocación, caras, texturas, terreno, capas y grupos, y worldspawn | `update_entity`, `select`, `transform`, `duplicate`, `set_face`, `mesh_edit`, `texture`, `terrain_edit`, `hierarchy`, `set_map_properties`, `bake_lighting` |
| Pintura y gameplay | Esparcir modelos, mezclar materiales, pegar piezas de la biblioteca de prefabs y convertir brushes en puertas, plataformas y botones | `scatter`, `blend`, `prefab_library`, `random_fill`, `gameplay` |
| Editor | Ejecutar acciones de menú, abrir y guardar archivos, cambiar ajustes del editor, mover la cámara, hacer capturas, enviar entrada y comprobar por dónde puede caminar un jugador | `run_action`, `map_file`, `open_project`, `set_editor`, `set_camera`, `screenshot`, `simulate_input`, `walkability` |
| Proyecto | Añadir el paquete de entidades de gameplay, los modelos de naturaleza o la demo al proyecto de Godot abierto | `project_content` |
| Scripts | Reproducir una lista guardada de llamadas a herramientas, mira [Scripts MCP](mcp-scripts.md) | `run_script` |

La descripción de cada herramienta lista sus operaciones y argumentos, así que un agente los aprende directamente del servidor.
[tools/mcp_client.py](https://github.com/Paraxdev/TrenchDot/blob/main/tools/mcp_client.py) los imprime con `list`
y llama a una herramienta desde la consola:

```sh
python tools/mcp_client.py call screenshot '{"target":"3d","width":1280,"height":720}' --out shot.png
```

## Capturas de pantalla { #screenshots }

`screenshot` devuelve un PNG de toda la ventana del editor, o de una vista cuando `target` es `3d`, `top`, `front` o `side`. Para una vista, los demás argumentos deciden lo que obtienes:

| Argumentos | Resultado |
| --- | --- |
| Ninguno | Toda la ventana del editor |
| Solo `target` | Esa vista acoplada tal como está, con las superposiciones |
| `width` y `height` | Renderizada fuera de pantalla hasta 4096 píxeles, sin cuadrícula, cajas de entidades, gizmos ni selección |
| `width`, `height` y `"overlays": true` | Fuera de pantalla, con las superposiciones del editor |
| `"overlays": false` | Una captura limpia al tamaño acoplado |

### Lo que renderiza Godot { #what-godot-renders }

`"source": "godot"` pide al editor de Godot que renderice el mapa con sus luces, entorno, niebla y posprocesado reales. Recibes ese PNG, más los errores o avisos que Godot registró al construirlo. Viene bien para juzgar la iluminación y los materiales sin lanzar el juego.

```sh
python tools/mcp_client.py call screenshot '{"source":"godot","position":[0,256,512],"look_at":[0,64,0]}' --out godot.png
```

1. Guarda el mapa dentro del proyecto que está abierto en TrenchDot.
2. Abre ese proyecto en el editor de Godot, con una escena cuyo `FuncGodotMap` construya el mapa.
3. Llama a `screenshot` con `"source": "godot"`. Sin `position` y `look_at` usa la cámara de la vista 3D.

Sin el [modo en vivo](godot/live.md) Godot primero construye el mapa tal como se ve en TrenchDot, como *Construir en Godot*. `"build": false` captura la escena tal como la tiene Godot. `width` y `height` valen por defecto 1280 por 720, y `fov` el de la vista 3D.

> [!NOTE]
> Godot necesita una ventana para renderizar, un Godot sin ventana devuelve un error en lugar de una imagen. La ventana puede estar en segundo plano o minimizada.

## Leer y revisar un mapa { #reading-and-reviewing-a-map }

| Paso | Herramienta |
| --- | --- |
| Obtener una visión general | `summarize_map` lista las capas, las clases de entidad, los enlaces de E/S y los materiales, y los recursos que faltan |
| Encontrar nodos | `list_nodes` filtra por `layer`, `material`, `property` (`{"model": "*crate*"}`) o una `box` del mapa |
| Mirar uno | `get_node` devuelve un nodo. `compact: true` omite vértices y caras |
| Mostrar lo que cambiaste | `changes_since` compara con el último guardado, un número de `undo_steps` atrás, o un `file` de mapa |
| Deshacerlo | Cada llamada y cada `run_script` es un paso de deshacer llamado `MCP: ...`. `run_action undo` con `steps` deshace varios |

Mira [Revisar cambios](editor/reviewing.md) para mantener los mapas en git.

## Comportamiento que conviene conocer { #behavior-worth-knowing }

| Tema | Comportamiento |
| --- | --- |
| Navegadores | El servidor HTTP rechaza cualquier petición con una cabecera `Origin` y cualquier POST que no sea `application/json`, así que las páginas web no pueden llegar a él, ni siquiera las servidas desde localhost |
| Errores | Una herramienta que no pudo hacer su trabajo devuelve `isError`, incluido un `run_action` fallido |
| Cuadros de archivo | Las acciones que abren uno se rechazan, usa `map_file` con una ruta. `run_action save` necesita un mapa que ya tenga archivo |
| Pestañas sin guardar | `run_action close_tab` rechaza los cambios sin guardar salvo que `args.discard` sea true. `args.close_others` cierra todas las pestañas menos la activa |
| Rutas de archivo | `res://` y las rutas relativas de `map_file`, `run_script`, `import_model`, `changes_since` y `validate_map` parten del proyecto abierto, o del directorio de trabajo del editor si no hay ninguno. `map_file open` y `open_tab` convierten la ruta en absoluta antes de abrir, así un `save` posterior no puede acabar en otro sitio |
| Ajustes del editor | `transform` y `run_action rotate` y `flip` giran sobre el centro de la selección, ajustado a la cuadrícula solo mientras el ajuste está activo, y mantienen las texturas bloqueadas mientras el bloqueo UV está activo. `transform` acepta `snap` y `uv_lock` para anularlo en una llamada, `duplicate` acepta `uv_lock`. `run_action sculpt` y `blend` usan los ajustes de herramienta que una persona puede cambiar, `terrain_edit` usa valores fijos por defecto |
| Selección | `select` rechaza los ids desconocidos y salta los nodos ocultos o bloqueados, listándolos en `skipped` |
| CSG | `csg_subtract`, `csg_merge`, `csg_intersect`, `csg_hollow` y `clip_apply` devuelven `replaced`, los ids antiguos asignados a los nuevos |
| Tiempos de espera | Las llamadas caducan a los 120 s, salvo `run_script`, `project_content` y `bake_lighting` |
| Pregunta de contenido | La pregunta que recibe un proyecto nuevo sobre el contenido ya hecho se cierra con la primera llamada que no sea `get_state`, `screenshot` o `simulate_input`, y no vuelve a abrirse en esa sesión |
| Variables de script | `$name/rest` (sin llaves) es un error cuando `name` es una variable conocida, ya que solo `$name` solo o `${name}/rest` la expanden. Escribe `${project}/maps` |

## Notas de herramientas { #tool-notes }

| Herramienta | Nota |
| --- | --- |
| `run_action` `copy`, `cut`, `paste` | Copy y cut devuelven el texto del portapapeles, paste lo recibe como `args.text` con un `offset` u `origin` opcional |
| `run_action` `clip_apply` | Recibe un plano (`point` y `normal`, o tres `points`) y `keep`: `front`, `back` o `both` |
| `run_action` `create_decal` | Coloca un [decal](editor/decals.md), úsalo en lugar de brushes finos |
| `create_brush` | `openings` abre puertas y ventanas, `{min, max, count, step, rows, row_step}` repite una como una cuadrícula |
| `create_mesh` `rotate` | `[x, y, z]` grados en el orden YXZ de Godot, así que `[0, 0, 90]` tumba un cilindro a lo largo de X |
| `transform` | Aplica translate, rotate, flip y scale_to en ese orden, como un solo paso de deshacer. `rotate` gira 90 grados sobre Y salvo que se diga otra cosa |
| `create_terrain` | `origin` es la esquina mínima del terreno, a diferencia de otras herramientas. `center` lo coloca alrededor de un punto en su lugar |
| `create_brush`, `create_mesh` | Una cumbrera a dos aguas corre a lo largo de Z en los brushes y de X en las mallas. Ambas herramientas aceptan `ridge_x` y `ridge_z` |
| `map_file` exports | `path` es el archivo escrito, conservado para llamadores antiguos, `exported` también lo nombra y `map_path` es el mapa abierto |
| `terrain_edit` | `probe [x, z]` sin `op` solo lee la altura. `clear_layer` necesita `layer` |
| `validate_map` | Acepta como entrada cualquier método o propiedad de la clase o del script de Godot del destino. `coplanar_faces` marca las caras que parpadearán. Cada problema tiene `bounds`. `path` comprueba un mapa guardado sin abrirlo, `project: true` comprueba todos los mapas del proyecto |
| `run_action` `place_entities` | Recibe `args.classnames`, y `at` y `normal` para apoyar las entidades en una superficie como hace un arrastre desde la pestaña Entidades. Sin classnames es un error |
| `simulate_input` drags | Un arrastre con `"release": false` mantiene el botón y sus modificadores pulsados después de que la llamada vuelva, así una captura muestra el arrastre en curso, por ejemplo una tarjeta junto al cursor. Un evento `release` en una llamada posterior lo suelta donde está el puntero, o en su `x` e `y`, o en `world`, y un release con solo una de `x` e `y` es un error. El arrastre y sus modificadores siguen pulsados, también sobre el teclado real, hasta ese release, una acción `new_map` o un `close_tab` correcto, o hasta que MCP se apague, así que nunca olvides el release. Los clics y arrastres se rechazan hasta entonces |
| `run_action` `reload_materials` | Vuelve a explorar la carpeta de texturas ahora, aunque los archivos nuevos se encuentran solos de todos modos |
| `set_editor` `live_link_port` | Habla con un editor de Godot cuyo proyecto define otro `trenchdot/live_link_port` |
| `scatter` `preset` | Añade los modelos de Blockbench incorporados que necesita un preajuste. Un preajuste glTF en un proyecto sin el paquete de naturaleza es un error que nombra `project_content` |
| `bake_lighting` | Precalcula como [Precalcular la iluminación](editor/light-baking.md) y espera a que termine. Las opciones omitidas conservan con lo que se precalculó el mapa por última vez. `bake: false` solo cambia la `view`, `remove: true` descarta el precalculado. `out_of_date` indica si el mapa cambió desde entonces |
| `project_content` | `install: "nature"` o `"demo"` descarga de la release y responde cuando los archivos ya están. `install: "gameplay"` copia el paquete de entidades de gameplay del addon del proyecto, también dentro de scripts. Sin argumentos dice lo que tiene el proyecto |
