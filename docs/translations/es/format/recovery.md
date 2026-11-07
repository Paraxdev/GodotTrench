---
source: b3ed6fe92fc841e610e206dd64cc4020bf717ee7
---

# Archivos dañados { #damaged-files }

Un mapa binario dañado solo pierde sus bloques dañados, todo lo demás todavía se abre. Si escribes un lector, esta página es lo que tiene que hacer para igualarlo.

Un lector recorre los [bloques](container.md#chunks) desde el byte 12. Una cabecera de bloque válida tiene el marcador de sincronización, una comprobación de cabecera que coincide, un tamaño sin comprimir de 1 GiB como máximo y una etiqueta imprimible. Donde no hay una cabecera válida, el lector busca hacia delante el siguiente marcador de sincronización y sigue desde ahí.

Un bloque `HEAD`, `NODE` o `END` se descarta cuando su carga no se descomprime, no pasa su checksum, tiene un tamaño incorrecto o no se decodifica.

| Daño | Resultado |
| --- | --- |
| Un bloque `NODE` | Se pierden los nodos guardados en él |
| Se perdió el padre de un nodo | El nodo pasa, con todo lo que cuelga de él, a una capa llamada `Recovered` al final del mapa. La capa recibe un id nuevo al cargar |
| Anidamiento de más de 60 niveles | También pasa a `Recovered`, para que la lectura no pueda agotar la pila. Solo un archivo fabricado a propósito tiene esto |
| El bloque `HEAD` | Se pierden las propiedades de worldspawn y el estado del editor, y la versión del mapa sale de `END` |
| `HEAD` y `END` a la vez | Se rechaza el archivo |
| Ningún bloque `END` | El archivo está cortado. Todo hasta el último bloque completo se carga |
| Los recuentos de `END` superan lo que se leyó | Se informa como el número de nodos, o de bloques, que faltan |

Cada lector informa de cada pérdida:

| Lector | Informe |
| --- | --- |
| Editor | Abre lo que pudo leer, lista las pérdidas en la barra de estado y marca el mapa como sin guardar. Los prefabs dañados también se informan en la barra de estado |
| Addon de Godot | Construye lo que pudo leer e imprime un aviso por cada pérdida, con el prefijo `[TDM]` y la ruta del archivo |
| `trenchdot --dump` | Imprime lo que pudo leer y escribe las pérdidas en stderr |

Antes de cada guardado el editor copia el archivo antiguo a `<map>.tdm.bak`, así que al guardar un mapa recuperado el original dañado sigue estando a su lado. El archivo nuevo va primero a `<map>.tdm.tmp` y después se renombra sobre el mapa, así que un cierre inesperado a mitad de un guardado nunca deja un mapa a medio escribir.
