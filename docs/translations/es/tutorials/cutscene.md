---
source: 12dfe8079d66dd10bb5ca7191aa90e4b3c01fe5b
---

# Cinemática con script { #scripted-cutscene }

El jugador entra en una sala, aparece un cartel, un guía se aleja caminando, un barril explota, se enciende una lámpara y se abre la salida. Nada de esto necesita código propio: una entidad temporizador dispara una salida por tiempo, y cada salida está conectada a la entidad que debe actuar. Lo encuentras como el mapa `scripted_scene` del proyecto de demo.

![El mapa scripted_scene en el panel Lógica: start_zone dispara cutscene.start, las salidas de paso de cutscene van al guía, al barril y a la pista, y la salida broken del barril va a hp_readout. La lámpara y la puerta de salida quedan más abajo, fuera de la imagen](../assets/cutscene/logic-graph.png)

## La línea de tiempo { #the-timeline }

Todas las entidades de aquí vienen con el paquete de entidades de gameplay. Instálalo primero si tu proyecto todavía no lo tiene, con *Gameplay > Instalar entidades de gameplay*.

1. Cubre la entrada

    Dibuja un brush sobre la entrada y conviértelo en un `trigger_once` llamado `start_zone` (clic derecho, *Crear entidad de brush > trigger_once*). Dispara `triggered` la primera vez que el jugador entra y después se desactiva, así que la escena solo se reproduce una vez.

2. Añade la línea de tiempo

    Elige `logic_sequence` en *Gameplay > Lógica*, aparece en el cursor 3D. Llámala `cutscene`, pon `steps` en 5 y `interval` en 1.5. Una vez iniciada dispara `step_1`, `step_2` y así sucesivamente, con 1,5 segundos de separación.

    ![El menú Gameplay con Lógica abierto y logic_sequence resaltado bajo Colocar en el cursor](../assets/cutscene/menu-logic-sequence.png)

3. Ponla en marcha desde el trigger

    Enlaza la salida `triggered` de `start_zone` con `cutscene.start`, con *Gameplay > Lógica > Enlazar dos entidades seleccionadas…* como en [Un botón abre una puerta](button-door.md).

4. Conecta cada tiempo

    Enlaza cada salida de paso de `cutscene` con la entidad que actúa en ese momento, un par cada vez. Arrastrar de un pin de salida a un pin de entrada en el [panel Lógica](../gameplay/logic-graph.md) también funciona.

| Salida | Destino y entrada | Qué ocurre |
| --- | --- | --- |
| `step_1` | `hint.show` | Un `game_text` dice "Follow the guide" |
| `step_2` | `guide.start` | Un `npc_walker` echa a andar por su cadena de `path_corner`, una fila de marcadores que nombran cada uno al siguiente |
| `step_3` | `barrel.ignite` | Un `prop_physics` explosivo quema su mecha, luego estalla y se rompe |
| `step_4` | `lamp.turn_on` | La lámpara se enciende |
| `step_5` | `exit_door.open` | La puerta de salida se desliza hacia arriba |

Selecciona `cutscene` y deberías ver las cinco en *Salidas* de su Inspector.

![El Inspector de la logic_sequence llamada cutscene: steps 5, interval 1.5 y Salidas (E/S, 5) que empiezan con step_1 a hint.show](../assets/cutscene/inspector-sequence.png)

La secuencia espera antes de cada paso, también antes del primero, así que el paso 1 se dispara a 1,5 s y el paso 5 a 7,5 s. Para un ritmo irregular, pon en `times` una espera por paso, como `0.5 2 1 1 3`.

## Leer la salud del jugador { #reading-the-players-health }

Algunas reacciones necesitan un poco de lógica que ninguna entrada integrada cubre. Un `logic_script` ejecuta un fragmento corto de GDScript cuando se dispara. Enlaza la salida `broken` del barril con `hp_readout.run`, un `logic_script` cuyo `source` escribe la salud del jugador en un `game_text` de HUD llamado `hpmsg`:

```gdscript
var players = io.find_targets(this, "!player", null)
if players.is_empty():
	return
var hp = int(players[0].health)
for label in io.find_targets(this, "hpmsg", null):
	label.set_text("HP " + str(hp))
	label.show()
```

Usa `!player` aquí, no `!activator`. `!activator` es quien puso en marcha la cadena, y las salidas de paso y `broken` no llevan a nadie. `!player` siempre encuentra el nodo del grupo `player`. `broken` se dispara después de que la explosión haya hecho su daño, así que la salud ya está actualizada.

## Reproducirla { #playing-it }

El script de la escena,
[godot/demo/scenes/scripted_scene.gd](https://github.com/Paraxdev/TrenchDot/blob/main/godot/demo/scenes/scripted_scene.gd),
solo construye el mapa, hace aparecer al jugador en el `info_player_start` y añade un `TrenchDotDebugOverlay`. Pulsa **F3** en el juego para ver cómo se dispara cada salida. Es la forma más rápida de detectar un paso conectado con el nombre equivocado.

<div class="td-mcp" markdown>

## MCP { #mcp }

El mapa de demo se construye controlando el editor a través del [servidor MCP](../mcp.md). Las llamadas están en
[examples/mcp/scripted_scene.json](https://github.com/Paraxdev/TrenchDot/blob/main/examples/mcp/scripted_scene.json).

</div>
