---
source: 80f0288013e71dba5bd67c791a5e3e3272a71653
---

# Props, luces y efectos { #props-lights-and-effects }

> [!NOTE]
> Estas entidades vienen con el paquete de entidades de gameplay, mira
> [Instalar y personalizar](README.md#installing-and-customizing). `light`, `light_spot` y las sondas
> (`light_probe`, `env_reflection_probe` y `env_voxel_gi`) forman parte del núcleo del addon y funcionan sin él.

## prop_physics { #prop_physics }

Una caja o un barril que se puede lanzar, dañar y romper. Se rompe al llegar a 0 de salud, con un impacto más rápido que `impact_speed`, con `smash`, o `fuse` segundos después de `ignite`. Un prop explosivo estalla al romperse.

| Clave | Por defecto | Qué hace |
| --- | --- | --- |
| `model` | | La escena a la que se parece, solo en apariencia |
| `size` | 16 16 16 | Semiextensiones de su caja de colisión, así que por defecto mide un metro de ancho |
| `collision` | default | default y box usan `size`, convex hull y convex decomposition siguen el modelo, mira [Colisión](collision.md) |
| `collision_layer`, `collision_mask` | vacías | Números de capa como `1, 3`, vacío conserva el valor por defecto del proyecto |
| `mass` | 5.0 | Masa en kilogramos |
| `health` | 30 | Daño que aguanta antes de romperse |
| `explosive` | 0 | Estalla al romperse |
| `explosion_radius`, `explosion_damage` | 192, 40 | Tamaño y fuerza de esa explosión |
| `fuse` | 0.6 | Segundos que arde un prop encendido antes de romperse |
| `impact_speed` | 0 | Se rompe al golpear algo más rápido que esto, en m/s. 0 nunca se rompe por impacto |
| `debris_scene` | | Una escena que se genera donde se rompió, como tablones |

* **Entradas:** `smash`, `ignite(activator)`, `push(direction)`, `take_damage(amount, source)`
* **Salidas:** `damaged(hp)` con la salud restante, `broken`

`push` recibe una velocidad en unidades de mapa por segundo. `broken` se dispara después de la explosión, así que lo que active ya ve el daño.

El prop construye su modelo y su caja de colisión cada vez que se carga la escena. El editor de Godot los muestra como vista previa, que no se guarda con la escena.

## prop_model { #prop_model }

Un modelo estático con colisión opcional, para muebles, rocas y otros props que no deben moverse.

* **Claves:** `model` (.bbmodel, .glb, .gltf o .tscn), `model_node`, `collision`, `collision_layer`,
  `collision_mask`, `scale` 1.0

`collision` default da a cada malla un casco convexo, bien para formas con aspecto de caja. Elige convex decomposition o trimesh para formas por las que el jugador pasa o sobre las que camina, como un arco, o box para la forma más barata, mira [Colisión](collision.md). La colisión solo se genera cuando el modelo no trae ninguna propia. Los modelos colocados con *Prop de modelo* y los modelos dentro de prefabs son entidades prop_model, así que colisionan igual. Para un archivo que contiene varias variantes una junto a otra, `model_node` conserva solo el nodo nombrado y lo mueve al origen del prop.

## env_explosion { #env_explosion }

Una explosión con guion, como una carga en una pared. Con `explode` llama a `damage_method` en cada nodo dentro de `radius` con `damage * (1 - distance / radius)`, empuja los cuerpos rígidos lejos y genera `effect_scene` para el destello y el estallido.

* **Entradas:** `explode(activator)`
* **Salidas:** `exploded`
* **Claves:** `radius` 256, `damage` 50, `force` 8 (m/s en el centro, sea cual sea la masa), `damage_method` take_damage,
  `effect_scene`

La distancia se mide hasta el origen de cada nodo sin comprobar la línea de visión, así que hace daño a través de las paredes.

## game_text { #game_text }

Muestra una línea de texto en el mundo, en el HUD o en ambos, para pistas, carteles o subtítulos de cinemáticas.

| Clave | Por defecto | Qué hace |
| --- | --- | --- |
| `text` | | El texto que se muestra |
| `place` | world | `world` flota en la entidad, `hud` lo muestra en pantalla, `both` hace las dos cosas |
| `start_visible` | 0 | Visible desde que se carga el mapa |
| `duration` | 0 | Lo oculta tras esos segundos, 0 lo mantiene hasta `hide` |
| `text_color` | 255 255 255 | Color del texto |
| `world_size` | 0.01 | Tamaño de la etiqueta del mundo en metros por píxel |
| `billboard` | 1 | La etiqueta del mundo mira a la cámara y se dibuja a través de las paredes. Desactívalo para carteles, que entonces conservan los ángulos de la entidad y se ocultan tras las paredes |

* **Entradas:** `show`, `hide`, `set_text(text)`, `flash(seconds)`
* **Salidas:** `shown`, `hidden`

`show` y `hide` conmutan la etiqueta, no el nodo. `flash` la muestra durante su parámetro en segundos, si no `duration`, si no 2.

## light y light_spot { #light-and-light_spot }

Luces omni y de foco que las entradas encienden y apagan, para interruptores de luz, alarmas y lámparas que parpadean. `switched` solo se dispara cuando el estado realmente cambia. Forman parte del núcleo del addon, así que todo proyecto las tiene.

* **Entradas:** `turn_on`, `turn_off`, `toggle`
* **Salidas:** `switched(on)`
* **Claves:** `light_energy` 1.0 (1.5 para los focos), `light_color` 255 255 255, `start_on` 1, `shadows` 0 (cuesta más
  renderizar), `fixture`, `fixture_off` hide, y `omni_range` 10, o `spot_range` 15 y `spot_angle` 35 (el semiángulo
  del cono en grados), `bake_mode` auto, `light_size` 0, `light_indirect_energy` 1, y `omni_attenuation` 1, o
  `spot_attenuation` 1 y `spot_angle_attenuation` 1. Estas últimas claves establecen las propiedades de luz de Godot
  del mismo nombre, y el precalculado también las usa

Los **fixtures** mantienen el modelo de la lámpara en sintonía con la luz. Dale un targetname a la geometría luminosa de la lámpara y ponlo en `fixture`, un `*` al final coincide con un prefijo, así que `hall_tube_*` cubre todos los tubos. Se muestra mientras la luz está encendida y se oculta mientras está apagada, sin cableado. Con `fixture_off` en `dark` sigue visible con su emisión apagada. `fixture` admite también las otras formas de [destino](../parameters.md#targets), `@lamps` para un grupo de nodos y `/root/...` para una ruta de nodo. El editor previsualiza todas las formas salvo la ruta de nodo, que solo Godot puede resolver.

`bake_mode` y `light_size` deciden cómo trata la luz [Precalcular iluminación](../../editor/light-baking.md#light-keys). Con `auto` una luz que el I/O puede conmutar, una con un targetname o con `start_on` 0, sigue en tiempo real.

## env_sound { #env_sound }

Un sonido que se reproduce desde un punto del nivel, para alarmas, maquinaria o un estruendo cableado a un evento.

* **Entradas:** `play`, `stop`, `toggle`
* **Salidas:** `finished`
* **Claves:** `sound` (un archivo de audio `res://`), `volume_db` 0, `max_distance` 0 (sin límite), `autoplay_on` 0 (suena
  desde que se carga el mapa), `loop_sound` 0

## env_particles { #env_particles }

Un efecto de partículas para fuego, humo, chispas, polvo y lluvia, con aspectos incluidos para que no tengas que construir uno tú. `burst` emite una ráfaga, y luego un efecto en marcha sigue su curso.

* **Entradas:** `start`, `stop`, `toggle`, `burst`
* **Salidas:** `finished`
* **Claves:** `amount` 32 (partículas vivas a la vez), `lifetime` 2.0, `one_shot` 0, `start_emitting` 0, `effect` rise,
  `color` 255 255 255, `size` 0, `blend` auto, `area` 10 1 10

`effect` es `rise` (polvo luminoso o brasas), `fire`, `smoke`, `sparks` o `rain` (cae desde una caja de `area` metros de ancho, alto y profundo). `color` la tiñe, y el blanco conserva los colores propios del efecto. `size` establece el tamaño de las partículas en metros, 0 usa el propio del efecto. `blend` es `add` para brillo o `mix` para humo. Los valores por defecto de cada efecto se ven bien por sí solos, así que empieza solo con `effect`.

## infodecal { #infodecal }

Un decal proyectado, una caja que proyecta un material sobre todo lo que hay dentro, para manchas que se adaptan a escalones, tuberías o terreno irregular. [Decals proyectados](../../editor/projected-decals.md) te muestra cómo colocar uno y a qué prestar atención.

* **Claves:** `material` (un nombre como `base/stain` o una ruta `res://`), `texture`, `normal_texture`, `size` 64 32 64,
  `modulate` 255 255 255

`material` trae consigo sus texturas y su color, `texture` y `normal_texture` son imágenes `res://` que se usan sin uno. `size` es el ancho, la profundidad y la altura de la caja en unidades de mapa. Si se deja en 64 32 64, el [tamaño de textura](../../godot/materials.md#texture-size) del material establece el ancho y el alto en su lugar. `modulate` tiñe el decal.

## light_probe, env_reflection_probe y env_voxel_gi { #light_probe-env_reflection_probe-and-env_voxel_gi }

Sondas que iluminan y reflejan los objetos en movimiento, mira [Iluminación precalculada en Godot](../../godot/baked-lighting.md). Ninguna tiene entradas ni salidas.

| Entidad | Qué es | Claves |
| --- | --- | --- |
| `light_probe` | Un punto que añade una sonda de luz al precalculado, para sitios que la cuadrícula de sondas no alcanza | Ninguna |
| `env_reflection_probe` | Un volumen de brush que se convierte en un `ReflectionProbe` del tamaño del brush | `intensity` 1.0, `update` once, `box_projection` 1, `interior` 0, `max_distance` 0 |
| `env_voxel_gi` | Un volumen de brush que se convierte en un `VoxelGI` del tamaño del brush, precalculado en el editor de Godot en cada compilación | `subdiv` 128, `energy` 1.0, `bake_on_build` 1 |

`update` en `always` captura una sonda de reflejo en cada fotograma para objetos en movimiento, lo que cuesta mucho. `box_projection` alinea los reflejos con las paredes de una sala, `interior` deja fuera el cielo en salas sin ventanas, y `max_distance` son los metros que ve la sonda más allá de su caja. `subdiv` es 64, 128, 256 o 512 vóxeles a lo largo del lado más largo, más es más nítido y más lento de precalcular.
