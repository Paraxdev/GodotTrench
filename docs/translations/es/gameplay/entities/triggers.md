---
source: c7fdbd8dcaa1ac8625c1d6e7014b686194c50f36
---

# Triggers { #triggers }

> [!NOTE]
> Estas entidades vienen con el paquete de entidades de gameplay, mira
> [Instalar y personalizar](README.md#installing-and-customizing).

Volúmenes de brush invisibles que reaccionan cuando un cuerpo entra en ellos. Así es como un nivel se entera del jugador: una puerta inicia una cinemática, un foso hace daño, una almohadilla lanza. Dibuja uno con la herramienta **Volumen** (Shift+E).

## Entradas, salidas y claves compartidas { #shared-inputs-outputs-and-keys }

Todos los triggers de abajo tienen esto además de lo suyo.

* **Entradas:** `enable`, `disable`, `toggle`
* **Salidas:** `triggered(activator)`, `entered(activator)`, `exited(activator)`
* **Claves:** `filter_group` player, `start_disabled` 0

`filter_group` limita el trigger a los cuerpos de ese grupo de Godot, para que los enemigos y las cajas no activen un trigger del jugador. Déjalo vacío para aceptar cualquier cuerpo.

| Salida | Se dispara |
| --- | --- |
| `entered` | En cada entrada aceptada mientras está activado |
| `triggered` | En una entrada aceptada, como mucho una vez por `cooldown` para cada cuerpo. La mayoría del cableado usa esta |
| `exited` | Al salir, incluso con el trigger desactivado, para deshacer algo cuando el cuerpo se va |

Los triggers detectan cuerpos en la capa de física 1. Si el jugador está en otra capa, añádela a **Trigger Collision Mask** en los ajustes del mapa (el recurso `FuncGodotMapSettings` en Godot), o los triggers nunca se dispararán.

## trigger_once { #trigger_once }

Se dispara una vez y se desactiva solo, para eventos de una sola vez como una cinemática o una pista. `enable` lo arma otra vez.

* **Claves:** `once` 1

## trigger_multiple { #trigger_multiple }

Se dispara en cada entrada, como mucho una vez por `cooldown` para cada cuerpo, para cosas que pasan cada vez, como la campanilla de una tienda.

* **Claves:** `cooldown` 0.5

## trigger_hurt { #trigger_hurt }

Hace daño a los cuerpos mientras permanecen dentro, para lava, pinchos o una zona mortal bajo el mapa. Les llama a `damage_method(amount, source)` cada `interval` segundos, con `damage * interval` como cantidad.

* **Salidas:** `hurt(activator)`, por cuerpo dañado y por tic
* **Claves:** `damage` 10 (por segundo), `damage_method` take_damage, `interval` 0.5, `filter_group` vacía

Con el `filter_group` vacío por defecto daña todo lo que tenga el método, no solo al jugador. Con un grupo establecido, `hurt` se dispara para cada cuerpo del grupo, incluso uno sin el método.

## trigger_teleport { #trigger_teleport }

Mueve los cuerpos a la entidad nombrada en `destination`, normalmente una [`info_teleport_destination`](actors.md#info_teleport_destination), y los gira para mirar según su yaw. Los detiene salvo que `keep_velocity` sea 1, para un portal que el jugador deba atravesar cayendo o corriendo sin perder velocidad.

* **Salidas:** `teleported(activator)`
* **Claves:** `destination`, `keep_velocity` 0, `cooldown` 0.5

## trigger_push { #trigger_push }

Empuja los cuerpos con `push`, una velocidad en unidades de mapa por segundo que puedes orientar con su tirador del viewport. Con `once` 1 cada entrada recibe un solo impulso, como una almohadilla de salto. Con 0 empuja de forma continua como un túnel de viento o una cinta transportadora, y un cuerpo rígido deja de acelerar a la velocidad `push`.

* **Salidas:** `pushed(activator)`, una vez por entrada
* **Claves:** `push` 0 384 0, `filter_group` vacía, `once` 1

## trigger_call { #trigger_call }

Llama a un método al activarse, sobre `!activator` por defecto, así que un trigger puede llamar directamente al script del jugador, por ejemplo para guardar un punto de control. `call_target` y `arguments` funcionan como en [`logic_call`](scripting.md#logic_call).

* **Entradas:** `trigger(activator)`
* **Salidas:** `called(result)`
* **Claves:** `call_target` `!activator`, `method`, `arguments` `[]`, `once` 0, `cooldown` 0.5

## trigger_spawn_area { #trigger_spawn_area }

Genera escenas en puntos aleatorios dentro de su volumen, por defecto cuando entra el jugador, para una emboscada cuando el jugador entra en una sala. Sus claves funcionan como en [`info_spawner`](actors.md#info_spawner). `start` activa un temporizador de generación cuando `interval` es mayor que 0, y `toggle` aquí conmuta ese temporizador en vez del trigger.

* **Entradas:** `spawn`, `start`, `stop`, `toggle`, `kill_all`
* **Salidas:** `spawned(node)`, `all_dead`, `exhausted` (una vez, la primera vez que se alcanza `total`)
* **Claves:** `scene`, `count` 3, `max_alive` 10, `total` 0, `interval` 0, `spawn_on_enter` 1, `spawn_group` enemies,
  `snap_to_ground` 1, `cooldown` 0.5

## trigger_area { #trigger_area }

Un volumen `Area3D` simple sin script, para cablear los eventos de entrada y salida de una sala directamente a otras entidades. No tiene ninguna de las claves compartidas de arriba y se dispara para todos los cuerpos, así que filtra en la entidad a la que llama cuando importe.

* **Salidas:** `body_entered(body)`, `body_exited(body)`
* **Claves:** `targetname`
