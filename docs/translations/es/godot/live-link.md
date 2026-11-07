---
source: c998bca32c1c4bd85f0c95739d78a13d4ede986a
---

# Protocolo del Live Link { #live-link-protocol }

El Live Link es la forma en que TrenchDot y el editor de Godot hablan entre sí. El addon ejecuta un pequeño servidor dentro del editor de Godot, y TrenchDot se conecta a él para lanzar reconstrucciones cuando guardas, enviar las ediciones del [modo en vivo](live.md) y mostrar si Godot se está ejecutando.

Rara vez necesitarás esta página, el enlace simplemente funciona en cuanto el addon está activado. Está aquí por si escribes tu propia herramienta que maneje Godot del mismo modo, o quieres averiguar por qué falla la conexión. Si un guardado no reconstruye, empieza mejor por [Reconstruir al guardar](live.md#rebuild-on-save), la barra de estado suele decir por qué.

## Transporte { #transport }

JSON delimitado por saltos de línea sobre TCP en `127.0.0.1:7842`. Cada mensaje tiene un `event` y un `seq` opcional, y recibe exactamente una línea de respuesta que repite el `seq` y lleva `ok`. Los fallos responden `ok: false` con un `error`. El editor mantiene una conexión abierta y envía un latido `status` cada segundo.

Para usar otro puerto, cámbialo en los dos lados: `trenchdot/live_link_port` en los ajustes del proyecto de Godot y *Live Link de Godot* en *Archivo > Preferencias* en TrenchDot.

El autoload de [recarga en caliente](live.md#hot-reload-in-a-running-game) de un juego en ejecución escucha en `127.0.0.1:7843` y solo responde a `map_saved`.

## Eventos { #events }

Un cliente envía estos eventos. La columna Respuesta lista los campos con los que Godot responde además de `ok` y `seq`, o lo que hace.

| Evento | Campos | Respuesta |
| --- | --- | --- |
| `hello` | | `project` y `godot` (versión), una comprobación rápida de que el enlace responde |
| `status` | | `project`, `godot` (versión), `pid`, `scene` (la pestaña de escena que Godot muestra) y `maps: [{path, epoch}]` para ella |
| `map_saved` | `path` | `rebuilt`, el número de nodos de mapa reconstruidos a partir del archivo, `scene`, `waiting`, y los `missing_classes`, `settings` y `warnings` de la construcción |
| `build` | `path`, `text` | Como `map_saved`, construido a partir de `text` en lugar del archivo |
| `focus` | | Trae la ventana de Godot al frente |
| `export_game_config` | | Escribe `trenchdot_game.json` |
| `scan` | | Inicia un análisis de los archivos del proyecto. Cuando termina, cada sesión en vivo comprueba sus definiciones de entidad |
| `play` | | Guarda las escenas abiertas y ejecuta la escena principal, o `ok: false` con un `error` cuando el proyecto no tiene ninguna |
| `live_begin` | `path`, `text`, `content` | `epoch` y `unresolved` |
| `live_resync` | `path`, `text` | `epoch` y `unresolved` |
| `live_delta` | `path`, `ops` | `epoch` y `unresolved`, o `ok: false, resync: true` cuando las ops no encajan |
| `live_end` | `path`, `revert` | Con `revert`, reconstruye a partir del archivo guardado |
| `inspect` | `path`, `ids` | `nodes` con nombre, clase, número de hijos y posición por cada id de nodo de mapa, más `live` y `pending` |
| `capture` | `path`, `camera`, `width`, `height`, `text` | `png` (base64), `width`, `height` y `warnings`, los errores y advertencias registrados durante la construcción |
| `walkable` | `path`, `agent`, `text` | `vertices` (lista plana de x, y, z), `polygons`, `islands` (índices de polígonos, la mayor primero), `units_per_meter`, `agent` y `msec` |

`capture` renderiza el mundo del primer nodo de mapa que use `path`. `camera` contiene `position` y `forward` en unidades y ejes del mapa y `fov`, el campo de visión horizontal en grados. Con `text`, Godot construye antes ese mapa, y si no, espera a que una sesión en vivo se ponga al día. La respuesta llega tras una docena de fotogramas más o menos, y mientras tanto se responden los demás mensajes.

`walkable` precalcula la malla de navegación de toda la escena editada con `TrenchDotNav` para `agent` (`radius`, `height`, `max_climb`, `max_slope` en metros y grados) y la devuelve en las unidades y ejes del mapa. Con `text`, Godot construye antes ese mapa, salvo que ya se hubiera construido por última vez a partir del mismo texto.

Las rutas son absolutas y con barras normales. Solo participan los nodos `FuncGodotMap` de la escena editada, la pestaña de escena que Godot muestra, que tengan *Auto Rebuild On Save* activado. `scene` es su ruta `res://`, o vacía si no hay ninguna escena abierta.

`waiting` lista las otras pestañas de escena abiertas que usan el mapa. Tras un `map_saved` construyen el archivo cuando se muestra su pestaña, tras un `build` no. Un `map_saved` de un prefab también reconstruye los mapas de la escena editada que lo instancian, directamente o a través de otros prefabs, y los cuenta en `rebuilt`.

Lo que informaron las construcciones de un `map_saved` o `build` llega en tres listas, cada una solo cuando no está vacía: `missing_classes` contiene los nombres de clase ordenados que ninguna definición describía, `settings` las rutas de los ajustes del mapa que usaron las construcciones, y `warnings` hasta 20 advertencias de una línea del informe de construcción. `unresolved` en una respuesta en vivo lista los nombres de clase ordenados que ese paso construyó sin definición, también solo cuando los hay.

`scan` sirve para archivos escritos fuera de Godot, como un pack de entidades instalado. Una sesión en vivo cuya escena se construyó con otras definiciones de entidad que las que tienen sus ajustes del mapa tras el análisis vuelve a construir su mapa. Un addon sin `scan` o `play` los responde como un evento desconocido.

`text` es el mapa completo como JSON, con la iluminación precalculada bajo `lightmap` cuando el mapa tiene un bake, para que una construcción en vivo la conserve. Eso añade unos 11 bytes por texel del atlas, alrededor de 11 MB para un atlas de 1024 por 1024, así que el editor vuelve a enviar el mapa completo solo cuando empieza una sesión, tras un bake nuevo o tras un cambio grande.

`content` es el id de contenido del fragmento END del archivo guardado, así que una escena construida a partir de un archivo sin cambios no se vuelve a construir cuando empieza una sesión.

Cuando el puerto está ocupado, normalmente por un segundo editor de Godot con el mismo proyecto, el addon avisa una vez en el panel Output y vuelve a probar el puerto cada tres segundos, así que toma el relevo cuando el otro editor se cierra.

## Ops en vivo { #live-ops }

| Op | Campos | Significado |
| --- | --- | --- |
| `set` | `id`, `parent`, `index`, `node` | Inserta o reemplaza un nodo, con la forma JSON del formato `.tdm`, sin hijos |
| `remove` | `id` | Quita un nodo con su subárbol, solo se envía el id quitado más alto |
| `translate` | `ids`, `offset` | Mueve nodos, se envía mientras arrastras en lugar de la geometría |
| `properties` | `properties` | Reemplaza las propiedades del worldspawn |

El `epoch` de un mapa sube cada vez que se construye fuera de una sesión en vivo o cambian los nodos `FuncGodotMap` que lo usan. Cuando el siguiente `status` muestre un epoch nuevo, empieza de nuevo con `live_begin`.

Los nodos generados llevan su id de nodo de mapa en el metadato `_td_id`, y los nodos de capa y de grupo en `_td_group`.

## Probar sin ventana { #testing-without-a-window }

El enlace también funciona con un editor de Godot sin cabeza (headless), lo que viene bien para pruebas automáticas. Abre una escena que tenga un `FuncGodotMap`, y envía `inspect` al puerto 7842 para ver lo que construyó Godot:

```sh
godot --headless --editor --path godot res://path/scene.tscn
```

(Nota: en un checkout nuevo, ejecuta antes `godot --headless --path godot --import` dos veces. Una sola pasada solo registra las clases del addon.)

Un Godot headless no dibuja nada, así que `capture` responde ahí con un error. Prueba las capturas con un editor con ventana.

Las pruebas del addon hacen esto en
[`godot/tests/editor/live_link_tabs.gd`](https://github.com/Paraxdev/TrenchDot/blob/main/godot/tests/editor/live_link_tabs.gd),
que abre, cierra, recarga y cambia pestañas de escena en un editor headless y comprueba lo que informa el enlace.

<div class="td-mcp" markdown>

## MCP { #mcp }

El [servidor MCP](../mcp.md) permite a agentes y scripts manejar el editor, y algunas de sus herramientas pasan por el Live Link. La herramienta `screenshot` usa `capture` cuando se le pide una imagen desde Godot.

Para pruebas headless que también editen el mapa, inicia TrenchDot con su servidor MCP sobre el mismo proyecto que el editor de Godot headless y edita el mapa por MCP:

```sh
trenchdot --mcp-http --project godot
```

</div>
