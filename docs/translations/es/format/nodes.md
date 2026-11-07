---
source: d21533e024f6fb1ea2fed11362601fee5c4efd4d
---

# Nodos { #nodes }

El contenido de un mapa es un árbol de nodos. Todos los nodos tienen la misma forma exterior, con sus claves en este orden:

| Clave | Tipo | Se escribe | Significado |
| --- | --- | --- | --- |
| `id` | integer | siempre | Único en el mapa |
| `type` | string | siempre | `layer`, `group`, `entity`, `brush`, `mesh`, `terrain`, `scatter` o `instance` |
| campos del tipo | | | Las claves de ese tipo de nodo, mira más abajo, [Geometría](geometry.md) y [Terreno y scatter](terrain-scatter.md) |
| `label` | string | cuando está definida | Nombre dado con *Renombrar*, mira más abajo |
| `graph` | array de 2 números | cuando está definida | Dónde está el nodo de una entidad en el [grafo Logic](../gameplay/logic-graph.md) |
| `hidden` | bool | cuando es `true` | Oculto en el editor |
| `locked` | bool | cuando es `true` | No se puede seleccionar ni editar en el editor |
| `children` | array de nodos | cuando no está vacío | Nodos hijos, en orden |

`hidden` y `locked` son solo estado del editor, así que Godot construye igualmente un nodo oculto. Para dejar algo fuera de la construcción, ponlo en una capa con `omit_from_export`.

`label` es el nombre que diste en el editor a un brush, una malla, una entidad, una instancia o un terreno, y se muestra en lugar de un nombre por defecto como `brush12`. Las capas, los grupos y los conjuntos de scatter nunca tienen una, guardan la suya en `name`.

Godot solo usa `label` para los terrenos, nombra el nodo `terrain_<id>_<label>`. Los brushes y las mallas se fusionan en su entidad, y una entidad toma el nombre de su nodo de `targetname`. Los editores anteriores a la clave todavía leen archivos que la tienen, pero la descartan al guardar.

`graph` también es estado del editor. Es dónde está el nodo de una entidad en el grafo Logic, en puntos del grafo, y solo lo tienen los nodos que alguien movió. Godot lo ignora, y un valor que no sean dos números se descarta al cargar.

## Ids { #ids }

Los ids son enteros positivos que se mantienen entre guardados, para que otros datos puedan apuntar a un nodo. El Live Link envía las ediciones por id, los conjuntos de scatter nombran sus superficies por id, y Godot guarda el id de cada nodo generado en su metadato `_td_id`. Un id `0` o duplicado, que solo tiene un archivo editado a mano, recibe un id nuevo al cargar.

Un prefab es un archivo `.tdm` aparte con ids propios, así que dos copias suyas chocarían. Para evitarlo, la construcción de Godot suma un millón por el número de aparición al id de cada nodo que trae una [instancia](#instance). Dos copias de un mismo prefab nunca comparten ids, por muy anidadas que estén.

## Dónde van los nodos { #where-nodes-go }

| Nodo | Padre permitido |
| --- | --- |
| Capa | Solo el nivel superior |
| Grupo | Capa o grupo |
| Brush, malla, terreno, conjunto de scatter, entidad punto, instancia | Capa o grupo |
| Brush o malla de una entidad brush | Esa entidad |

Una entidad con hijos es una entidad brush, por ejemplo una puerta hecha de brushes. Los brushes y las mallas que no están en ninguna entidad pertenecen a worldspawn.

## Capa { #layer }

Una parte de nivel superior del mapa, que se muestra en el Outliner del editor.

| Clave | Tipo | Por defecto | Significado |
| --- | --- | --- | --- |
| `name` | string | obligatoria | Nombre que se muestra en el Outliner |
| `color` | string | obligatoria | `#rrggbbaa`. Al leer también se acepta `#rrggbb` |
| `omit_from_export` | bool | `false` | Siempre se escribe. Cuando es true, la construcción se salta la capa entera, para geometría de referencia que nunca debe llegar a Godot |

## Grupo { #group }

Nodos reunidos para que los selecciones y los muevas juntos.

| Clave | Tipo | Por defecto | Significado |
| --- | --- | --- | --- |
| `name` | string | obligatoria | Nombre que se muestra en el Outliner |
| `link_id` | integer | se omite | Los grupos que comparten un link id son copias enlazadas, hechas con *Duplicar enlazado* |
| `transform` | 16 números | identidad, se omite | La colocación de esta copia respecto a las demás, una matriz 4x4 en orden de columnas |

El contenido de un grupo se guarda en espacio de mundo, ya colocado. Una [copia enlazada](../editor/organizing.md#groups) repite en todas las demás una edición hecha en una copia, y `transform` es justo lo que usa el editor para llevar la edición de una copia a la siguiente. La construcción de Godot ignora tanto `transform` como `link_id`.

## Entidad { #entity }

| Clave | Tipo | Por defecto | Significado |
| --- | --- | --- | --- |
| `classname` | string | obligatoria | Clase de entidad de la configuración de juego, por ejemplo `prop_physics` |
| `origin` | `[x, y, z]` | `[0, 0, 0]` | Siempre se escribe. Posición de una entidad punto |
| `angles` | `[pitch, yaw, roll]` | `[0, 0, 0]` | Siempre se escribe. Grados, orden YXZ como `rotation_degrees` |
| `properties` | object de string a string | se omite | Claves de la entidad, ordenadas por nombre |
| `outputs` | array | se omite | Conexiones de E/S, mira más abajo |

Los valores de las propiedades son siempre strings, sea cual sea su tipo en el FGD, por ejemplo `"travel": "0 168 0"`.

Una propiedad `angles`, `angle` o `mangle` gana sobre los `angles` del nodo y se lee a la manera de Quake: pitch y roll con el signo cambiado, yaw girado 180 grados. En las clases que empiezan por `light`, `mangle` es en cambio yaw, pitch y roll. Un `angle` de -1 o -2 apunta recto hacia arriba o hacia abajo. El editor dibuja la entidad mirando en la misma dirección, y al girarla allí la propiedad pasa a los `angles` del nodo.

La construcción ignora `origin` y `angles` de una entidad brush, ya que sus brushes ya están donde les corresponde.

Cada salida es una [conexión de E/S](../gameplay/io.md): cuando esta entidad dispara una señal, llama a un método de un destino.

| Clave | Tipo | Por defecto | Significado |
| --- | --- | --- | --- |
| `output` | string | obligatoria | Señal de esta entidad que activa la conexión |
| `target` | string | obligatoria | Quién recibe la llamada, mira [Destinos](../gameplay/parameters.md#targets) |
| `input` | string | obligatoria | Método al que llamar en el destino |
| `parameter` | string | `""`, se omite | Valor que se pasa a la entrada |
| `delay` | number | `0`, se omite | Segundos que esperar antes de la llamada |
| `times` | integer | `-1`, se omite | Cuántas veces puede dispararse, `-1` para todas |

## Instancia { #instance }

Una referencia a otro archivo `.tdm`, colocada con una posición y una rotación, como el `func_instance` de Hammer. Así se colocan los prefabs: el contenido vive una sola vez en su propio archivo, y cada instancia lo muestra.

| Clave | Tipo | Por defecto | Significado |
| --- | --- | --- | --- |
| `path` | string | obligatoria | El mapa al que se refiere, relativo al mapa que lo referencia, absoluto o `res://` |
| `origin` | `[x, y, z]` | obligatoria | Dónde se coloca la instancia |
| `angles` | `[pitch, yaw, roll]` | obligatoria | Grados, orden YXZ |
| `fixup` | string | `""`, se omite | Prefijo de nombre para el contenido de la instancia, mira más abajo |

La construcción toma los hijos de cada capa del mapa referenciado que no esté omitida de la exportación, y los mueve según `origin` y `angles`. Las proyecciones de textura se mueven junto con la geometría. El worldspawn del mapa referenciado no se usa. Las instancias se anidan hasta ocho niveles de profundidad.

Una construcción de Godot lee cada mapa referenciado una vez, y todas las instancias de un mapa juntas pueden añadir 2.097.152 valores, siendo un valor un elemento de array o una entrada de diccionario del mapa referenciado, y contando como uno más cada 8 caracteres de texto. Pasado eso, la construcción deja fuera las instancias restantes e informa de un error. Los mapas normales se quedan muy por debajo, solo lo alcanzan instancias de instancias multiplicadas muchas veces.

Con un fixup de `p1`, `door` pasa a ser `p1-door` en `targetname`, `target`, `destination`, `call_target`, en cada propiedad que el FGD declara como `target_source` o `target_destination`, y en cada `target` de una salida. Los valores que empiezan por `!`, `@` o `/` se dejan como están. Los fixups anidados se suman de fuera hacia dentro, así que `b` dentro de `a` da `a-b-door`, y *Explotar instancia* en el editor da los mismos nombres.
