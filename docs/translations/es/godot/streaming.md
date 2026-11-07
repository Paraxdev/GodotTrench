---
source: 0215c0eddc11970f765e58b4ce44056eebf2815e
---

# Streaming de mapas grandes { #streaming-big-maps }

Los mapas grandes de exterior pueden ocultar lo que queda lejos, así que el mapa cuesta lo que el jugador ve en lugar de todo lo que contiene. La construcción reparte las mallas del mapa en chunks sobre una cuadrícula, y solo se dibujan los chunks cercanos a la cámara.

## Activarlo { #turning-it-on }

1. No selecciones nada

    Pulsa Escape para que no haya nada seleccionado. El Inspector de la derecha muestra entonces *Mapa (worldspawn)*.

2. Añade `chunk_streaming`

    Escribe `chunk_streaming` en el campo *key* de abajo, `1` en *value*, y haz clic en **Añadir**. (Si ya aparece en la lista, solo actívala ahí.)

    ![El Inspector de worldspawn con chunk_streaming escrito en el campo key, 1 en el campo value y el botón Añadir resaltado](../assets/streaming/worldspawn-add-key.png)

    Deberías verla en la lista con las demás claves de worldspawn:

    ![El Inspector de worldspawn con chunk_streaming marcada](../assets/streaming/worldspawn-chunk-streaming.png)

3. Guarda el mapa

    La siguiente construcción añade un `TrenchDotStreamer` al mapa.

(Opcional) añade `chunk_size` y `load_radius` de la misma forma:

| Clave | Por defecto | Significado |
| --- | --- | --- |
| `chunk_size` | 2048 | Tamaño del chunk en unidades de mapa, 64 m. Los chunks más pequeños ocultan con más precisión pero cuestan más de comprobar |
| `load_radius` | 8192 | A qué distancia de la cámara siguen visibles los chunks, en unidades de mapa, 256 m |

El streaming solo oculta mallas. La colisión y los scripts siguen funcionando en todas partes, así que ahorra coste de renderizado, no de físicas ni de lógica. Los overlays, el contenido generado en tiempo de ejecución y todo lo que vaya en una puerta o un tren en movimiento nunca se ocultan.

El streamer sigue la cámara actual del viewport, o un `info_player_start` mientras no haya cámara. Para fijarlo a una cámara, pon su `camera_path` desde código después de la construcción, ya que una reconstrucción reemplaza al streamer. Solo funciona en el juego, así que el editor de Godot siempre muestra el mapa entero.

Según van y vienen los chunks, el streamer emite `area_loaded(key, bounds)` y `area_unloaded(key, bounds)`, para que puedas hacer streaming de tu propio contenido a su lado:

```gdscript
streamer.area_loaded.connect(func(key, bounds): spawn_wildlife(bounds))
```

## Secciones en streaming { #streaming-sections }

¿Quieres jugar varios mapas como un solo mundo sin pantallas de carga? Divídelo en secciones y únelas con salas conectoras. Solo hay una sección en el árbol de escena a la vez. La siguiente se carga en segundo plano mientras el jugador explora, y entra en su lugar mientras el jugador cruza un conector, alineada para que la sala a su alrededor siga exactamente igual. Esto necesita el [paquete de entidades de gameplay](../gameplay/entities/README.md#installing-and-customizing).

1. Construye una sala conectora

    Un pasillo con una curva a cada lado, para que ninguno de los dos extremos se vea desde su centro. Pon un `info_landmark` en el medio, y dos volúmenes `func_section_stream`, uno que cubra cada mitad, que se encuentren en el medio. Añádela a la [biblioteca de prefabs](../editor/prefab-library.md) para que todas las copias sean iguales.

2. Pégala donde se encuentran dos mapas

    En cada mapa que limita con otro, pega el conector donde se encuentran los dos y nombra su landmark, por ejemplo `lm_hall_cellar`. Pon el `section` de cada mitad a la sección de ese lado y `landmark` al nombre del landmark. Los dos mapas reciben la misma copia con los mismos nombres, girada igual.

3. Precalcula la iluminación en cada mapa de sección

    SDFGI solo nota la geometría que trae un cambio cuando sus cascadas se desplazan, así que una sección cambiada quedaría sin luz. La iluminación precalculada viaja con la sección.

4. Haz una escena por sección

    Una escena por sección con su mapa construido, de la forma habitual.

5. Añade el streamer

    Añade un nodo `TDSectionStreamer` a la escena de tu mundo y pon su `first_section` y su jugador, mira [TDSectionStreamer](../gameplay/entities/sections.md#tdsectionstreamer).

> [!TIP]
> Mantén la posición de un landmark en un múltiplo de 480 unidades de mapa, o de cualquier tamaño en que se repitan todas las texturas del conector, para que las texturas encajen igual en las dos copias.
