---
source: d3f0019b345d8a4b85cb01b22ccb9fd1cdee21d4
---

# El formato de mapa .tdm { #the-tdm-map-format }

Estas páginas son para ti si escribes una herramienta que lee o escribe archivos `.tdm`, como un importador, un conversor o una comprobación de CI. No necesitas nada de esto para construir niveles.

Un archivo `.tdm` contiene tres cosas: las propiedades de worldspawn, un poco de estado del editor y un árbol de nodos con el contenido real.

El editor lo guarda como un contenedor binario de bloques comprimidos con zstd, así que los archivos son pequeños y un archivo dañado todavía se abre. Los editores antiguos guardaban en su lugar un único documento JSON en UTF-8, y todos los lectores cargan ambos. Las dos formas contienen el mismo árbol, así que estas páginas lo muestran como el JSON que imprime `trenchdot --dump`.

| Página | Léela cuando |
| --- | --- |
| [Estructura del contenedor](container.md) | Analizas o escribes el archivo binario. Cubre la cabecera del archivo, los bloques y cómo se codifica cada valor en bytes |
| [Archivos dañados](recovery.md) | Quieres saber qué se sigue cargando de un archivo corrupto, y dónde informa cada lector de lo que se perdió |
| [Nodos](nodes.md) | Recorres el árbol. Explica la forma que comparten todos los nodos, cómo funcionan los ids y los nodos de capa, grupo, entidad e instancia |
| [Geometría](geometry.md) | Lees o construyes brushes y mallas, con sus caras, la proyección de textura y los displacements |
| [Terreno y scatter](terrain-scatter.md) | Manejas terrenos de mapa de alturas y conjuntos de modelos esparcidos |
| [Estructura JSON](json.md) | Trabajas con la forma legible, conviertes entre las dos formas o lees lo que el editor pone en el portapapeles |

¿Quieres ver código que funciona? La implementación de referencia es
[`format.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/format.rs), con el contenedor en
[`binary.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/binary.rs) y
[`variant.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/variant.rs). Del lado de Godot,
[`tdm_file.gd`](https://github.com/Paraxdev/trenchdot_func/blob/main/src/trenchdot/tdm_file.gd) lee el archivo y
[`tdm_parser.gd`](https://github.com/Paraxdev/trenchdot_func/blob/main/src/trenchdot/tdm_parser.gd) lo construye.

## Nivel superior { #top-level }

| Clave | Tipo | Se escribe | Significado |
| --- | --- | --- | --- |
| `format` | string | siempre | Siempre `"trenchdot-map"`. Distingue un mapa de otro JSON |
| `version` | integer | siempre | Versión del mapa, actualmente 1 |
| `properties` | object de string a string | cuando no está vacío | Claves de worldspawn, ordenadas por nombre |
| `editor` | object | cuando no todo es el valor por defecto | Estado de la vista del editor. Nunca se exporta a Godot |
| `textures` | object | cuando alguna textura tiene ajustes | [Ajustes de textura](geometry.md#texture-settings) por nombre de material |
| `layers` | array de nodos | siempre | Los [nodos de capa](nodes.md#layer) del mapa, en orden. Todo lo demás cuelga de ellos |
| `lightmap` | object | mapas JSON con un precalculado | La iluminación precalculada, que un mapa binario guarda en su [bloque LMAP](container.md#lmap) |

De `layers` solo se leen los nodos `layer`, y un mapa que no tenga ninguno recibe una capa `Default` al cargarse.

Los valores de worldspawn son siempre strings, como en un archivo `.map`. Cuando Godot construye el mapa, `TrenchDotEnvironment` convierte estas claves en un cielo, niebla y un sol:

| Claves | Qué controlan |
| --- | --- |
| `sun_angles`, `sun_color`, `sun_energy` | La luz direccional del sol: hacia dónde apunta, su color y cuánta fuerza tiene |
| `ambient_color`, `ambient_energy` | La luz que llega a las superficies a las que el sol no llega, para que las sombras no queden negras del todo |
| `sky_top_color`, `sky_horizon_color`, `sky_ground_color`, `sky_energy` | El degradado del cielo procedural y su brillo |
| `sky_panorama` | Una imagen `res://` que sustituye al cielo procedural en Godot. La vista previa del editor y su precalculado siguen usando los colores del cielo |
| `fog_color`, `fog_density` | Niebla de distancia |
| `glow_intensity`, `ssr` | Bloom en las superficies brillantes, y reflejos en espacio de pantalla cuando `ssr` vale `1` |

Si tu juego trae su propio `WorldEnvironment`, pon `environment` en `sun_only` y solo se construye el sol. `0` o `none` no construye nada, mira [entorno y sol](../godot/building.md#environment-and-sun). `sky_source` recuerda qué textura tenían las caras de cielo de un mapa importado.

El objeto `editor` es estado de la vista que solo importa dentro del editor:

| Clave | Tipo | Por defecto | Significado |
| --- | --- | --- | --- |
| `cameras` | object, claves `"1"` a `"9"` | vacío | Posiciones de cámara guardadas, cada una `{"position": [x, y, z], "yaw": r, "pitch": r}` con ángulos en radianes |
| `cordon` | `{"min": [x, y, z], "max": [x, y, z]}` | ninguno | La caja del cordon en unidades de mapa, que sirve para trabajar en una parte de un mapa grande |
| `cordon_enabled` | bool | `false` | Oculta los objetos fuera del cordon y los deja fuera de las exportaciones con cordon |
| `bake` | `{"quality", "texel_size", "softness", "backend"}` | ninguno | Las opciones con las que se [precalculó](../editor/light-baking.md) el mapa por última vez, para que el siguiente precalculado parta de ellas |
| `graph_frames` | array | vacío | Marcos del [grafo Logic](../gameplay/logic-graph.md#frames), cada uno `{"title", "rect": [x, y, width, height], "color"}` en puntos del grafo. Uno dañado se descarta al cargar |

## Versiones y compatibilidad { #versions-and-compatibility }

`format` y `version` describen los datos. La versión del contenedor en la cabecera del archivo es aparte, solo cambia cuando cambia la disposición de los bloques.

Esto es lo que hace cada lector con un archivo que no entiende del todo:

| Situación | Editor | Addon de Godot |
| --- | --- | --- |
| Otro `format` | Rechaza el archivo | Error de construcción |
| `version` más nueva | Rechaza el archivo | Error de construcción que pide actualizar el addon |
| Falta `version` | La lee como la versión más antigua | Igual |
| Clave desconocida | La ignora, y desaparece tras el siguiente guardado | La ignora |
| Bloque desconocido | Lo conserva y lo vuelve a escribir al guardar | Lo omite |
| Índice de cara fuera de rango | Rechaza el mapa, nombrando el nodo | Omite ese brush |
| `heights` de un displacement con tamaño incorrecto | Rechaza el mapa, nombrando el nodo | Construye la cara plana |

El editor descarta las claves desconocidas al guardar. Así que si añades datos que un editor antiguo no debe perder, necesitas o bien subir la versión del mapa, que los editores antiguos se niegan a abrir, o bien un bloque propio, que conservan intacto. Mira [Otros bloques](container.md#other-chunks).

## Coordenadas y unidades { #coordinates-and-units }

Las posiciones usan los ejes de Godot: Y hacia arriba, X a la derecha, Z hacia el observador. La construcción divide entre el `inverse_scale_factor` de los ajustes del mapa, 32 por defecto, así que 32 unidades de mapa hacen un metro.

| Magnitud | Unidad |
| --- | --- |
| Posiciones, tamaños, alturas, espaciado, tamaño de celda | Unidades de mapa |
| Ángulos de entidades, instancias y scatter | Grados: pitch alrededor de X, yaw alrededor de Y, roll alrededor de Z, aplicados en orden YXZ como el `rotation_degrees` de Godot |
| Yaw y pitch de los marcadores de cámara | Radianes |
| Desplazamiento UV | Texels |
| Escala UV | Unidades de mapa por texel |
| `uvs` de la malla | Anchos de textura, así que 1 es una repetición completa de la textura |
