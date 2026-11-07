---
source: 5b0744accf923762bf8a9e744e70cb250be2483b
---

# Línea de comandos { #command-line }

```sh
trenchdot [options] [map]
```

Sin opciones el editor simplemente se abre, con el mapa si das uno. `trenchdot --help` (o `-h`) lista las opciones y termina.

| Opción | Qué hace |
| --- | --- |
| `--project <dir>` | Carga el proyecto de Godot de `<dir>` o de una carpeta superior y confía en él durante la sesión, mira [Confianza en el proyecto](godot/running-godot.md#project-trust). Sin ella el editor carga el último proyecto |
| `--default-prefs` | Arranca con las preferencias por defecto y nunca las guarda. Sirve para scripts y pruebas, y para comprobar si un problema viene de tus propios ajustes |
| `--offscreen` | Abre la ventana fuera de todas las pantallas, sin foco ni botón en la barra de tareas. Sigue renderizando, así que los scripts y las pruebas pueden hacer capturas mientras tú sigues trabajando |
| `--live-link-port=<port>` | Habla con el editor de Godot por este puerto en lugar del puerto *Live Link de Godot* de Preferencias, solo en esta ejecución. Los scripts y las pruebas pasan un puerto libre, así que sus guardados nunca reconstruyen mapas en el Godot que tienes abierto |
| `--hot-reload-port=<port>` | Hace lo mismo con el puerto *Recarga en caliente del juego en marcha*, por el que escucha un juego en ejecución para recargar los mapas guardados |
| `--dump <map>` | Imprime un mapa como JSON y termina |
| `--to-json <map> <out>` | Escribe un mapa como JSON y termina |
| `--to-tdm <map> <out>` | Escribe un mapa como `.tdm` binario y termina |
| `--export-glb <map> <out.glb>` | Escribe la geometría y los materiales del mapa como glTF binario para Blender y otras herramientas 3D, y termina |
| `--export-obj <map> <out.obj>` | Escribe lo mismo como OBJ, con un `.mtl` y una carpeta de texturas al lado, y termina |

Una opción desconocida, o una a la que le falta su valor, imprime un error y termina con el código 2, sin abrir ninguna ventana. Una conversión o exportación que falla imprime el motivo y termina con el código 1.

Las conversiones y exportaciones nunca abren una ventana, así que son seguras en scripts y en hooks de git, mira [Archivos de mapa en git](development.md#map-files-in-git).

Las exportaciones encuentran el proyecto de Godot del mapa a partir de su carpeta y usan los valores por defecto de *Archivo > Exportar*: el mapa entero, sin objetos ocultos ni instancias de scatter. La lógica de entidades, los triggers y los scripts se dejan fuera, mira [Exportar a Blender y otras herramientas 3D](editor/exporting.md).

<div class="td-mcp" markdown>

## MCP { #mcp }

Estas opciones preparan el [servidor MCP](mcp.md), para que scripts y agentes de IA puedan controlar el editor.

| Opción | Qué hace |
| --- | --- |
| `--mcp` | Sirve MCP por stdin y stdout |
| `--mcp-http[=<port>]` | Sirve MCP por HTTP en 127.0.0.1, puerto 7841 por defecto |
| `--mcp-token=<token>` | Obliga a los clientes HTTP a enviar este token, diga lo que diga Preferencias. Tiene prioridad sobre `TRENCHDOT_MCP_TOKEN`, mira [El token HTTP](mcp.md#the-http-token) |

`--default-prefs` va bien con cualquiera de las dos, para que una sesión con script arranque siempre con los mismos ajustes. Con `TRENCHDOT_DISABLE_MCP=1` definido, ninguna de las dos opciones arranca un servidor, mira [Apagar MCP](mcp.md#turning-mcp-off).

</div>
