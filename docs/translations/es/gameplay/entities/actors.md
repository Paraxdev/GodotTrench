---
source: 978a2cda95dff6236396e32b6d494bed11f3f94b
---

# Spawners y rutas { #spawners-and-paths }

> [!NOTE]
> Estas entidades vienen con el paquete de entidades de gameplay, mira
> [Instalar y personalizar](README.md#installing-and-customizing).

## info_spawner { #info_spawner }

Genera una escena, como un enemigo o un objeto recogible, en puntos aleatorios dentro de `radius`. Dispara `spawn` desde un trigger para una emboscada, o establece un `interval` y haz `start` para un flujo constante. Para repartir las generaciones por una sala, usa [`trigger_spawn_area`](triggers.md#trigger_spawn_area).

| Clave | Por defecto | Qué hace |
| --- | --- | --- |
| `scene` | | La escena `res://` que se genera |
| `count` | 1 | Cuántas se generan cada vez |
| `max_alive` | 5 | Nunca hay más vivas a la vez, las generaciones de más se saltan |
| `total` | 0 | Se detiene para siempre tras tantas, 0 es sin límite |
| `interval` | 0 | Segundos entre generaciones mientras corre el temporizador. 0 genera solo con `spawn` |
| `radius` | 64 | A qué distancia del spawner pueden aparecer. Arrastra su tirador en el viewport |
| `start_active` | 0 | Corre el temporizador desde que se carga el mapa |
| `spawn_on_ready` | 0 | Genera una vez al cargar el mapa |
| `spawn_group` | enemies | Grupo de Godot que se añade a cada nodo generado |
| `snap_to_ground` | 1 | Deja caer cada generación sobre el suelo que tiene debajo |

* **Entradas:** `spawn`, `start`, `stop`, `toggle`, `kill_all`
* **Salidas:** `spawned(node)`, `all_dead`, `exhausted` (una vez, la primera vez que se alcanza `total`)

`all_dead` se dispara cuando todos los nodos generados han desaparecido, para "limpia la sala para abrir la puerta". Los nodos generados no tienen targetname, llega a ellos por su `spawn_group`, como `@enemies`.

Con las claves por defecto no genera nada hasta que algo le dispara `spawn`. Para generar sin un trigger, establece `spawn_on_ready`, o un `interval` con `start_active`. La generación solo ocurre en el juego en ejecución, en el editor de Godot el spawner sigue siendo un nodo vacío.

## info_teleport_destination { #info_teleport_destination }

Marca adónde envía `trigger_teleport` los cuerpos. Oriéntala hacia donde debe mirar el jugador al llegar, su yaw pasa a ser hacia dónde miran. Su única clave es `targetname`.

## path_corner { #path_corner }

Una parada de la ruta de un `func_train` o un `npc_walker`, con `target` nombrando la siguiente. Coloca una cadena con un clic usando la herramienta **Ruta** (Shift+P). Dispara `reached` cuando llega un tren o un caminante, para un sonido o un evento en esa parada.

* **Salidas:** `reached(train)`
* **Claves:** `target`, `wait` 0 (segundos que esperar aquí), `speed` 0 (nueva velocidad del tren a partir de aquí, 0 mantiene la actual)

`speed` solo afecta a los trenes, los caminantes conservan la suya.

## npc_walker { #npc_walker }

Un actor con guion que recorre una cadena de esquinas de ruta reproduciendo animaciones, para cinemáticas en vez de IA, como un guardia que pasa por una ventana. Se mueve en línea recta, ignorando la gravedad y las paredes, así que coloca las esquinas por donde pueda caminar.

| Clave | Por defecto | Qué hace |
| --- | --- | --- |
| `model` | | Una escena que le da una malla y un `AnimationPlayer` |
| `target` | | La primera esquina de ruta a la que camina |
| `speed` | 2.0 | Metros por segundo |
| `loop` | 0 | Vuelve a la primera esquina tras la última |
| `start_active` | 0 | Camina desde que se carga el mapa |
| `walk_anim`, `idle_anim` | walk, idle | Animaciones que se reproducen al caminar y al estar parado |
| `face_travel` | 1 | Se gira para mirar hacia donde camina |
| `collision` | default | La forma construida a partir de su modelo, para que otros cuerpos choquen con él, mira [Colisión](collision.md) |
| `collision_layer`, `collision_mask` | vacías | Números de capa como `1, 3`, vacío conserva el valor por defecto del proyecto |

* **Entradas:** `start`, `stop`, `walk_to(corner)`, `play_anim(name)`, `face(target)`
* **Salidas:** `arrived(corner)`, `reached_goal`, `finished` (ambas en la última esquina cuando no hace bucle)

`face` y `walk_to` aceptan cualquier destino, como `!player`. `walk_to` abandona el tramo actual y va directo allí.

Carga su `model` solo en el juego en ejecución, así que en el editor de Godot es un nodo vacío.
