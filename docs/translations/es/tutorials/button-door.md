---
source: 19de62e149bf31bc7f0453a04737b8fbd5510db4
---

# Un botón abre una puerta { #button-opens-a-door }

Un botón en la pared que desliza una puerta cuando el jugador lo pulsa. Es el fragmento más pequeño y útil de E/S de entidades: el botón dispara una salida llamada `pressed`, y un enlace la convierte en una llamada a la entrada `open` de la puerta.

> [!TIP]
> Si las salidas, las entradas y `targetname` son nuevas para ti, lee primero [Cómo funciona la E/S de entidades](../gameplay/io.md). Solo lleva unos minutos y esta página se apoya en ella.

## Conéctalos { #wire-it }

`func_door` y `func_button` vienen con el paquete de entidades de gameplay. Si tu proyecto todavía no lo tiene, usa *Gameplay > Instalar entidades de gameplay*, o **Instalar entidades de gameplay** en la ventana *Configurar este proyecto* (*Godot > Instalar o actualizar addon…*).

1. Crea la puerta

    Dibuja un brush del tamaño de una puerta en un vano, [Primeros pasos](../getting-started.md#4-build-a-room) muestra cómo abrir uno. Haz clic derecho en el brush y elige *Crear entidad de brush > func_door*.

    ![El menú del clic derecho sobre un brush de puerta naranja, con Crear entidad de brush abierto y func_door resaltado](../assets/button-door/create-brush-entity-func-door.png)

2. Ponle nombre

    En el Inspector de la derecha, escribe `main_door` en `targetname`. Con ese nombre las demás entidades encuentran la puerta. Deja `travel` en `0 64 0`, así la puerta se desliza 64 unidades (2 m) hacia arriba al abrirse.

    ![El Inspector de func_door con targetname main_door y travel 0 64 0](../assets/button-door/inspector-targetname.png)

3. Crea el botón

    Dibuja un brush pequeño en la pared junto a la puerta y conviértelo en un `func_button` de la misma forma.

4. Selecciona los dos

    Haz clic en el botón y luego Ctrl+clic en la puerta, en una vista o en el Outliner de la izquierda. ¿Hay paredes en medio? Escribe `func` en el cuadro Filtro de la parte superior del Outliner, y solo aparecerán estos dos.

    ![El Outliner filtrado por func, con func_door (main_door) y func_button seleccionados](../assets/button-door/outliner-filter-func.png)

5. Enlázalos

    Ve a *Gameplay > Lógica > Enlazar dos entidades seleccionadas…*, o pulsa **Enlazar…** en la parte superior del Inspector.

    ![El menú Gameplay con Lógica abierto y Enlazar dos entidades seleccionadas… resaltado](../assets/button-door/menu-link-two-selected.png)

    La entidad que dispara tiene que estar a la izquierda, aquí el botón. El cuadro pone a la izquierda la que creaste primero, que es la puerta, así que pulsa el pequeño botón Intercambiar entre los dos nombres. Elige `pressed` en *Cuando* y `open` en *llamar*, y pulsa **Vincular**.

    ![La ventana Enlazar entidades con func_button a la izquierda, func_door (main_door) a la derecha, Cuando pressed y llamar open](../assets/button-door/link-dialog.png)

La barra de estado de abajo a la izquierda debería decir ahora `pressed > main_door.open`. Léelo como "cuando se pulse, llama a open en main_door".

![La barra de estado con el texto pressed > main_door.open](../assets/button-door/status-linked.png)

Si seleccionas el botón solo, el enlace aparece en *Salidas* de su Inspector, donde puedes cambiarlo o quitarlo más adelante.

![La sección Salidas de func_button: salida pressed, destino main_door, entrada open](../assets/button-door/inspector-button-output.png)

> [!TIP]
> *Gameplay > Puertas y móviles*, también en la sección *Gameplay* del Inspector, convierte el brush seleccionado en una puerta con bisagra o corredera con un solo clic. Le pone nombre a la puerta, define cómo se abre y coloca delante un `trigger_multiple` que abre la puerta cuando el jugador se acerca. Borra ese trigger si solo debe abrirla el botón.
>
> ![El menú Gameplay con Puertas y móviles abierto: puerta con bisagra a izquierda o derecha, puerta corredera hacia arriba o hacia un lado, y Ascensor, plataforma móvil](../assets/button-door/menu-doors-and-movers.png)

## Deja que el jugador lo pulse { #let-the-player-press-it }

El addon no tiene un sistema de interacción propio, porque cada juego resuelve "usar" a su manera. Las puertas y los botones tienen un método `use(activator)` y el script de tu jugador lo llama. El jugador de la demo lo hace con **E**: lanza un rayo desde la cámara y llama a `use` en lo primero que golpea y que lo tenga ([godot/demo/player.gd](https://github.com/Paraxdev/TrenchDot/blob/main/godot/demo/player.gd)):

```gdscript
func use_target() -> void:
	var from := camera.global_position
	var query := PhysicsRayQueryParameters3D.create(from, from - camera.global_basis.z * use_distance)
	query.exclude = [get_rid()]
	query.collide_with_areas = false
	var hit := get_world_3d().direct_space_state.intersect_ray(query)
	var node: Node = hit.get("collider")
	while node:
		if node.has_method(&"use"):
			node.call(&"use", self)
			return
		node = node.get_parent()
```

El jugador se pasa a sí mismo como activador, así que `pressed(activator)` lleva al jugador consigo. Las salidas posteriores de la cadena pueden apuntar a `!activator` para actuar sobre quien pulsó el botón.

## Variaciones { #variations }

| Quieres | Qué cambiar |
| --- | --- |
| Que la puerta solo se abra con el botón | Desmarca `interact` de la puerta. Si no, pulsar **E** sobre la puerta también la abre |
| Que la puerta se cierre sola | Pon en `wait` de la puerta los segundos que se queda abierta. El valor por defecto, -1, la deja abierta |
| Una puerta con llave | Marca `locked`. La puerta ignora `open` y `use` y dispara `locked_use` en su lugar, hasta que una salida llame a su entrada `unlock`, por ejemplo desde la recogida de una llave |
| Un botón, varias puertas | Llama a las puertas `gate_1`, `gate_2` y así sucesivamente y apunta a `gate_*`. Un `*` final coincide con todos los nombres que empiezan por `gate_` |
| Una puerta batiente | Usa `func_door_rotating` y arrastra su tirador de bisagra en la vista hasta donde debe ir la bisagra |

## El mismo cableado en código { #the-same-wiring-in-code }

¿Prefieres conectar las cosas en GDScript antes que en el editor? Esto hace lo mismo que el enlace de arriba:

```gdscript
# pressed(activator) > main_door.open
button.pressed.connect(func(activator):
	for door in TrenchDotIO.find_targets(button, "main_door", activator):
		TrenchDotIO.invoke(door, &"open", "", activator))
```

Conectar `pressed` directamente a `door.open` falla, porque `pressed` pasa un argumento y `open()` no recibe ninguno. `invoke` adapta los argumentos al método, igual que hacen los enlaces del mapa.
