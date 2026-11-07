---
source: 721efa7c5ececa362093af607d2e2c2faa92ad9b
---

# Secciones en streaming { #streamed-sections }

> [!NOTE]
> Estas entidades vienen con el paquete de entidades de gameplay, mira
> [Instalar y personalizar](README.md#installing-and-customizing).

Entidades para jugar varios mapas como un solo mundo sin pantalla de carga. Cada mapa es una sección, y las secciones se encuentran en una sala conectora de la que ambos mapas tienen una copia idéntica. [Secciones en streaming](../../godot/streaming.md#streaming-sections) explica paso a paso cómo configurarlo.

## info_landmark { #info_landmark }

El punto compartido de una sala conectora. Pon una en medio de la sala, con el mismo nombre en ambas copias. Cuando entra la siguiente sección, se coloca de modo que su punto de referencia caiga exactamente donde estaba el anterior.

* **Claves:** `targetname`

## func_section_stream { #func_section_stream }

Una mitad de una sala conectora, un volumen invisible. Mientras el jugador está en él, la sección que nombra es la que está cargada. Un conector necesita dos, uno por lado, y la línea donde se encuentran es donde las secciones se intercambian, así que ponla donde no se pueda ver ningún extremo de la sala, por ejemplo en medio de un pasillo en forma de Z.

* **Claves:** `section` (la sección de este lado: la ruta de su escena, como `res://levels/pools_2.tscn`, o un nombre que se encuentre en el `sections_dir` del streamer), `landmark` (el `targetname` del `info_landmark` de la sala)

## TDSectionStreamer { #tdsectionstreamer }

No es una entidad sino el nodo que ejecuta el streaming. Añádelo a la escena de tu mundo y establece:

| Propiedad | Qué es |
| --- | --- |
| `first_section` | La sección en la que empieza el juego, una ruta de escena o un nombre |
| `sections_dir` | Carpeta en la que se encuentran los nombres cortos, como `name.tscn` o `name.scn` |
| `player_scene` | El jugador que se genera en el `info_player_start` de la primera sección |
| `player_path` | O un jugador que ya está en la escena. Sin ninguno de los dos, se conserva el primer nodo del grupo `player` que trae una sección |
| `recenter_distance` | Metros que un bucle de secciones puede alejarse del origen antes de que todo el mundo vuelva atrás, 8000 |

Emite `section_changed(from, to)` tras cada intercambio.
