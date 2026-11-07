---
source: 59a13276848e83b80ec8fffb5e9295c0444e1a52
---

# Terreno y scatter { #terrain-and-scatter }

## Terreno { #terrain }

Un mapa de alturas sobre una cuadrícula regular, una altura por vértice de la cuadrícula, pintado con hasta cuatro capas de textura. Los datos voluminosos son texto base64 en JSON y bytes sin procesar en el [contenedor binario](container.md#value-encoding).

| Clave | Tipo | Por defecto | Significado |
| --- | --- | --- | --- |
| `origin` | `[x, y, z]` | obligatoria | Posición en el mundo del primer vértice |
| `resolution` | `[nx, nz]` | obligatoria | Número de vértices a lo largo de X y Z, al menos 2 cada uno |
| `cell_size` | number | obligatoria | Unidades de mapa entre vértices vecinos |
| `heights` | base64 | obligatoria | Floats de 32 bits little endian, una altura por vértice |
| `layers` | array | obligatoria | Hasta cuatro capas de textura, mira más abajo |
| `splat` | base64 | se omite | Cuatro bytes de peso por vértice, uno por capa, que dicen cuánto se ve cada capa allí |
| `holes` | base64 | se omite | Un byte por celda, un valor distinto de cero recorta la celda del terreno |
| `chunk_cells` | integer | `32` | Siempre se escribe. Celdas por lado de chunk. El terreno se divide en chunks de este tamaño para el renderizado y la colisión |

El vértice `(i, j)` está en `origin + (i * cell_size, heights[j * nx + i], j * cell_size)`. `splat` usa el mismo orden, sus pesos se normalizan al leer, y una entrada toda a cero significa la capa 0. `holes` cubre las `(nx - 1) * (nz - 1)` celdas en el mismo orden.

Cada capa:

| Clave | Tipo | Por defecto | Significado |
| --- | --- | --- | --- |
| `material` | string | obligatoria | Material que pinta esta capa |
| `tile` | number | `256` | Unidades de mapa por repetición de la textura |
| `detile` | number | `0` | De 0 a 1, con cuánta fuerza se rompe la repetición para que deje de notarse el patrón de la cuadrícula |
| `detile_sharpen` | number | `0.5` | De 0 a 1, cuánta nitidez conserva el resultado tras la desrepetición |

Mira [Capas de terreno](../editor/terrain-layers.md) para ver cómo se ven estos ajustes en la práctica.

Los terrenos se mantienen alineados con los ejes. Una instancia mueve el centro de un terreno con toda su transformación, rotación incluida, pero nunca gira la propia cuadrícula.

## Scatter { #scatter }

Un conjunto de scatter guarda en un solo nodo muchas copias de unos pocos modelos, como árboles, rocas o hierba pintados sobre superficies. Mira [Scatter](../editor/scatter.md) para la parte del editor. Todas las claves menos `targets` y `material` se escriben siempre.

| Clave | Tipo | Por defecto | Significado |
| --- | --- | --- | --- |
| `name` | string | obligatoria | Nombre que se muestra en el editor |
| `kind` | string | `props` | Lo que construye Godot. `props` conserva scripts y colisión, `foliage` es un MultiMesh sin colisión |
| `targets` | array de ids de nodo | se omite | Las superficies sobre las que se pinta el conjunto |
| `items` | array | obligatoria | Los modelos que esparce, mira más abajo |
| `collision` | string | `none` para foliage, si no `convex` | Forma de colisión por instancia: `none`, `box`, `convex`, `decomposition` o `trimesh` |
| `cast_shadows` | bool | `true` | Si las instancias proyectan sombras |
| `visibility_range` | number | `0` | Oculta las instancias más allá de esta distancia, 0 es nunca |
| `chunk_size` | number | `0` | Divide el conjunto en celdas de este tamaño para que Godot pueda descartar lo que está fuera de pantalla, 0 es una sola celda. Los conjuntos nuevos usan 2048 |
| `static_props_multimesh` | bool | `false` | Dibuja también como MultiMesh las escenas de props que llevan scripts. Más barato para muchos props, pero se pierden sus scripts |
| `material` | string | se omite | Material que sustituye al de todo el conjunto |
| `instances` | array | vacío | Las instancias colocadas, mira más abajo |

Si copias un conjunto junto con una superficie a la que apunta, la copia apunta a la superficie copiada.

Cada elemento es un modelo que el conjunto puede colocar:

| Clave | Tipo | Por defecto | Significado |
| --- | --- | --- | --- |
| `source` | string | obligatoria | Modelo o escena `res://` |
| `weight` | number | `1` | Probabilidad relativa de que se elija este elemento |
| `scale` | `[min, max]` | `[0.8, 1.2]` | Rango de una escala uniforme aleatoria |
| `spacing` | number | `48` | Distancia mínima a cualquier otra instancia |
| `align` | number | `0` | 0 se coloca erguido, 1 sigue la normal de la superficie, los valores intermedios se inclinan a medias |
| `random_yaw` | bool | `true` | Gira cada instancia hacia un rumbo aleatorio |
| `tilt` | number | `0` | Mayor inclinación aleatoria en grados |
| `sink` | number | `0` | Unidades de mapa que se hunden en la superficie, para que las raíces y las bases de las rocas no floten |
| `material` | string | se omite | Material que sustituye al de este elemento. Gana sobre el del conjunto |
| `enabled` | bool | `true`, se omite | Si el pincel pinta este elemento. Las instancias ya colocadas se quedan |

Cada instancia es un array plano `[item, x, y, z, pitch, yaw, roll, scale]`, donde `item` indexa `items` y los ángulos están en grados en orden YXZ. Al guardar, la posición se redondea a 0,01, los ángulos a 0,1 y la escala a 0,001, lo que permite al contenedor guardarlos como [columnas de enteros](container.md#value-encoding).
