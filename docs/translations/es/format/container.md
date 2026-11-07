---
source: 8142a4423329f307647582ed665e04b916677fd2
---

# Estructura del contenedor { #container-layout }

Un `.tdm` binario es una cabecera de archivo de 12 bytes y después bloques uno detrás de otro hasta el final del archivo. Todos los números son little endian y de ancho fijo.

| Desplazamiento | Tamaño | Significado |
| --- | --- | --- |
| 0 | 8 | Número mágico `89 47 54 4D 0D 0A 1A 0A`, es decir `\x89GTM\r\n\x1a\n` |
| 8 | 4 | Versión del contenedor, u32, actualmente 1 |

Un lector rechaza una versión del contenedor más nueva que la suya.

El `\r\n` del número mágico detecta un archivo que pasó por una conversión de finales de línea, en git por ejemplo. Un lector lo señala como archivo estropeado en lugar de leerlo.

## Bloques { #chunks }

Cada bloque es una cabecera de 24 bytes y después su carga. El marcador de sincronización y la comprobación de la cabecera distinguen una cabecera de bloque real de bytes al azar, así es como un lector encuentra el siguiente bloque bueno en un [archivo dañado](recovery.md).

| Desplazamiento | Tamaño | Significado |
| --- | --- | --- |
| 0 | 4 | Marcador de sincronización `89 47 54 43`, es decir `\x89GTC` |
| 4 | 4 | Etiqueta, cuatro caracteres ASCII imprimibles como `NODE` o `END ` |
| 8 | 4 | Flags, u32. El bit 0 indica que la carga es un frame zstd |
| 12 | 4 | Tamaño sin comprimir, u32, el tamaño de la carga tras descomprimir |
| 16 | 4 | Tamaño almacenado, u32, el número de bytes de carga que siguen a la cabecera |
| 20 | 4 | Comprobación de la cabecera, u32: la etiqueta leída como u32, XOR flags, XOR tamaño sin comprimir, XOR tamaño almacenado, XOR `0x47544D43` |

Una carga comprimida es un frame zstd con checksum del contenido. Un tamaño sin comprimir por encima de 1 GiB significa que la cabecera está dañada, y un archivo entero también tiene [límites](#limits).

El editor escribe los bloques en este orden:

1. Un bloque `HEAD`.
2. Los bloques `NODE`.
3. Los bloques desconocidos que conservó del archivo que cargó.
4. Un bloque `LMAP`, cuando el mapa tiene iluminación precalculada.
5. El bloque `END`, siempre el último.

### HEAD { #head }

El [objeto de nivel superior](README.md#top-level) sin `layers`.

### NODE { #node }

Un lote de nodos completos:

| Clave | Tipo | Significado |
| --- | --- | --- |
| `parents` | array de int64 | El id del padre de cada nodo, 0 para una capa |
| `nodes` | array de objetos de nodo | Los nodos, cada uno sin `children` |

Los nodos vienen en orden de árbol, así que un padre siempre va antes que sus hijos, en el mismo bloque o en uno anterior. El editor empieza un bloque nuevo cuando un lote pasa de 64 KiB antes de comprimir, y nunca parte un nodo entre dos bloques. Así un bloque dañado solo se lleva consigo sus propios nodos.

Para reconstruir el árbol, añade cada nodo a los `children` del nodo que nombra su id de padre.

### END { #end }

El bloque `END` deja que un lector compruebe que no falta nada:

| Clave | Tipo | Significado |
| --- | --- | --- |
| `version` | integer | La versión del mapa, para que los nodos se carguen aunque `HEAD` esté dañado |
| `nodes` | integer | Cuántos nodos contienen los bloques `NODE` |
| `chunks` | integer | Cuántos bloques vienen antes de `END` |
| `content` | integer | Id de contenido, mira más abajo |

El id de contenido es un hash FNV-1a de 64 bits de las cargas `HEAD` y `NODE` sin comprimir en el orden del archivo, recortado a sus 52 bits bajos para que sobreviva como número JSON. Godot lo guarda con la escena construida, así que una [sesión en vivo](../godot/live-link.md) puede saber que un mapa no ha cambiado y omitir la reconstrucción.

### LMAP { #lmap }

La [iluminación precalculada](../editor/light-baking.md), que solo está cuando el mapa tiene un precalculado. Es un único diccionario con 18 claves en un bloque propio, así que el resto del mapa carga rápido y un editor antiguo lo conserva intacto. Como es grande, el editor lo comprime con un nivel de zstd más rápido que el de los demás bloques. Las posiciones y las distancias están en unidades de mapa.

| Clave | Tipo | Significado |
| --- | --- | --- |
| `version` | integer | Versión de la estructura, actualmente 1. Un lector descarta un precalculado con una más nueva |
| `width`, `height` | integer | Tamaño del lightmap en texels |
| `scene` | integer | Huella de la geometría, las luces y el cielo con los que se hizo el precalculado, para saber cuándo está desactualizado |
| `texel_size` | float | Unidades de mapa por texel |
| `light` | bytes | Luz lineal, tres half floats little endian (rojo, verde, azul) por texel, fila a fila |
| `shadow` | bytes | Cuánto del sol llega a cada texel, de 0 a 255 |
| `ao` | bytes | Oclusión ambiental por texel, 255 donde no hay nada cerca |
| `chart_keys` | array de int64 | Un id de nodo y un índice de cara por chart, la cara es 0 para un terreno |
| `chart_rows` | array de float32 | Ocho floats por chart, dos filas `a b c d` que dan la coordenada de 0 a 1 del chart en el lightmap como `a*x + b*y + c*z + d` |
| `nodes` | array de int64 | Pares de un id de nodo y la huella de su geometría cuando se precalculó |
| `stale` | array de int64 | Nodos cuya geometría cambió desde entonces, o que ya no existen. Los lectores omiten sus charts |
| `fallback` | array de float32 | La luz media como rojo, verde y azul, para las caras sin chart |
| `probe_points` | array de float32 | Posiciones de las sondas de luz, tres floats cada una |
| `probe_sh` | array de float32 | Nueve coeficientes de armónicos esféricos de rojo, verde y azul por sonda, como los guarda Godot |
| `probe_tetrahedra` | array de int32 | Cuatro índices de sonda por tetraedro |
| `probe_bsp_planes` | array de float32 | Un plano por nodo BSP, normal y distancia |
| `probe_bsp_children` | array de int32 | El hijo por encima y por debajo de cada plano: otro nodo si es positivo, el tetraedro `t` como `-t - 1`, nada como el int32 más pequeño |

Las claves de las sondas siguen la estructura del `LightmapGIData` de Godot, y un lector deja fuera las sondas cuando sus tamaños no cuadran. Un bloque que falla al cargar se descarta con un aviso para que vuelvas a precalcular.

En un mapa JSON el mismo diccionario está bajo una clave `lightmap` de nivel superior, con `light`, `shadow` y `ao` como texto base64.

### Otros bloques { #other-chunks }

Los lectores omiten un bloque con una etiqueta que no conocen. El editor lo conserva byte por byte y lo vuelve a escribir tras sus bloques `NODE` al guardar.

Eso convierte un bloque nuevo en la forma de añadir datos que los editores antiguos se llevan consigo sin entenderlos. Ten en cuenta que un editor antiguo puede haber borrado nodos a los que se refiere tu bloque, así que un tipo de bloque nuevo tiene que seguir siendo válido cuando esos nodos ya no están.

## Codificación de valores { #value-encoding }

Cada carga es un valor en la serialización binaria Variant de Godot, el formato de `var_to_bytes`, usando solo los tipos de abajo. Cada valor empieza con una cabecera u32 cuyo byte bajo es el tipo. El bit 16 de la cabecera marca un entero o float de 64 bits. El bit 16 en cualquier otro tipo, o cualquier otro bit, deja el bloque ilegible. También lo hace un valor dentro de más de 128 arrays y diccionarios anidados.

| Tipo | Cabecera | Cuerpo |
| --- | --- | --- |
| null | 0 | Nada |
| bool | 1 | u32, 0 o 1 |
| integer | 2 | i32, o i64 con el bit 16 activado |
| float | 3 | f32, o f64 con el bit 16 activado. El editor siempre escribe f64 |
| string | 4 | Longitud en bytes u32, después los bytes UTF-8, después relleno con ceros hasta un múltiplo de 4 |
| dictionary | 27 | Cantidad u32, después cada clave (un string) seguida de su valor |
| array | 28 | Cantidad u32, después los valores |
| bytes | 29 | Longitud u32, después los bytes, después relleno con ceros hasta un múltiplo de 4 |
| array de int32 | 30 | Cantidad u32, después un i32 cada uno |
| array de int64 | 31 | Cantidad u32, después un i64 cada uno |
| array de float32 | 32 | Cantidad u32, después un f32 cada uno |
| array de float64 | 33 | Cantidad u32, después un f64 cada uno |

Los lectores ignoran el bit 31 de la cantidad de un diccionario o array, que Godot activa en los contenedores compartidos, pero no el de la cantidad de un array empaquetado. Los enteros JSON se escriben como enteros y los floats JSON como floats, así que convertir a binario y de vuelta da el mismo JSON.

Para mantener los archivos pequeños, tres tipos de valor tienen una forma más compacta. Un lector tiene que aceptar tanto la compacta como la normal:

| Valor JSON | Se guarda como | Cuándo |
| --- | --- | --- |
| Array de 32 o más números | Un array de int32 cuando todos son enteros que caben. Un array de float32 cuando todos son floats que un float de 32 bits representa exactamente. Un array de float64 para otros floats | Siempre |
| `heights`, `splat` y `holes` de un terreno (texto base64) | bytes, los datos decodificados | Cuando el base64 está en forma canónica |
| `instances` de un scatter | Un array de int32 de columnas | Cuando todos los valores son floats y múltiplos enteros de su paso |

Las columnas del scatter contienen primero el índice de cada elemento, después cada x, y así hasta los ocho campos de una instancia. Cada valor se guarda como un número entero de pasos: 1 para el elemento, 100 por unidad para la posición, 10 por grado para los ángulos y 1000 para la escala. Al dividir se recupera el float exacto, por ejemplo `1234 / 100.0` es `12.34`.

> [!NOTE]
> `tdm_file.gd` convierte las columnas del scatter de vuelta en arrays de instancias, pero deja los bytes de un terreno como `PackedByteArray` y los arrays largos de números como arrays empaquetados. El GDScript que lee un mapa tiene que aceptarlos junto a los arrays normales.

## Límites { #limits }

Unos pocos kilobytes de zstd pueden descomprimirse a gigabytes, así que un lector reparte un presupuesto por archivo. Toma los bloques `HEAD`, `NODE`, `END` y `LMAP` en el orden del archivo y no lee uno que no quepa en lo que queda:

1. Antes de descomprimir, el tamaño sin comprimir del bloque se resta de un presupuesto de 1 GiB.
2. El bloque descomprimido se queda en memoria mientras se construyen sus valores, así que su tamaño sin comprimir y la memoria de sus valores tienen que caber juntos en lo que queda de un presupuesto de 1200 MiB, además de lo que ocupan los nodos de un bloque `NODE` una vez que el mapa se construye a partir de ellos. De él solo se resta la memoria de los valores y de los nodos. Un bloque que no cabe no toma nada, así que los bloques que le siguen todavía se pueden leer.

Los bloques con otras etiquetas nunca se descomprimen y no toman nada. La memoria se cuenta así, sin construir ningún valor:

| Valor | Memoria |
| --- | --- |
| Cada valor, incluidas las claves de diccionario y los elementos de los arrays empaquetados | 80 bytes |
| Un string, además | 6 bytes por byte, porque JSON puede escribir un byte como `\u0000` |
| Bytes, además | `4 * ceil(n / 3) + n` para `n` bytes, su tamaño en base64 y la copia en la que un terreno o un precalculado los decodifica |
| Un nodo de un bloque `NODE`, además de sus valores | 5 KiB para una entidad, 12 KiB para un conjunto de scatter, 1 KiB para cualquier otro nodo |

Los costes de los nodos son una cota superior de lo que ocupan en el editor los nodos más pequeños de cada tipo una vez que abre el mapa (un grupo o un brush llega a unos 650 bytes y se le cobra 1 KiB), así que un archivo pequeño con muchos nodos diminutos no puede usar más memoria que el presupuesto. Los nodos anidados bajo `children` dentro de un bloque `NODE` se descartan y el lector informa de cuántos nodos del bloque tenían alguno, ya que el editor nunca los escribe ahí. Las instancias de scatter solo se cuentan por sus valores, que suman más de los 400 bytes aproximados que el editor guarda para una instancia construida, ya que todas las instancias de un conjunto comparten su modelo en la GPU.

El mapa de muestra más grande ocupa 79 MiB, una instancia de scatter de 640 a 720 bytes, y la luz, la sombra y la oclusión de un precalculado al tamaño de atlas máximo 299 MiB. El editor se niega a guardar un mapa que pase cualquiera de los dos límites, así que todo archivo que escribe se lee completo.
