---
source: 757483e9586cbd895b01c35597f84b4f54089ad0
---

# Cómo funciona el I/O de entidades { #how-entity-io-works }

El I/O de entidades es la forma en que un mapa hace que pasen cosas sin escribir código, cableado como lo hace Hammer para Source.

Cada entidad tiene **salidas**, eventos que anuncia, como un botón al pulsarse, y **entradas**, cosas que le puedes ordenar hacer, como que una puerta se abra. Una conexión dice: cuando se active esta salida, llama a esa entrada en un destino. El cableado se guarda en el mapa, así que conectas un botón a una puerta en el editor y la puerta se abre en el juego.

## Una conexión { #a-connection }

Una salida puede tener cualquier número de conexiones. Cada una tiene estos campos:

| Campo | Qué hace |
| --- | --- |
| Salida | El evento de esta entidad que inicia la conexión, por ejemplo `pressed` en un botón |
| Destino | Qué entidades reciben la llamada, normalmente un targetname, mira [Destinos](parameters.md#targets) |
| Entrada | Lo que se ordena a los destinos, por ejemplo `open` en una puerta |
| Parámetro | Un valor opcional que se pasa a la entrada, como una cantidad de daño, mira [Parámetros](parameters.md#parameters) |
| Retardo | Segundos que esperar antes de llamar a la entrada, 0 la llama al instante. Úsalo para escalonar eventos, como luces que se encienden una tras otra |
| Veces | Cuántas veces puede activarse la conexión. -1 significa cada vez, 1 hace un evento de un solo uso |

Las editas en el Inspector, bajo **Salidas** de la entidad seleccionada, o como cables en el [grafo lógico](logic-graph.md). Ambos cambian las mismas salidas. (El Inspector solo tiene una casilla **una sola vez** para *Veces*, para cualquier otra cantidad usa el cuadro de conexión del grafo lógico.)

![La sección Salidas del Inspector para un botón: salida pressed, destino front_door, entrada open, un parámetro vacío, retardo 0.00 s y la casilla una sola vez](../assets/io/inspector-outputs.png)

En Godot una salida es una señal y una entrada es un método: `func_button` tiene `signal pressed(activator)` y `func_door` tiene `func open()`. Tus propios scripts participan de la misma manera, mira [Entidades propias](custom-entities.md).

## Cuándo se activa una salida { #when-an-output-fires }

Pasados *Retardo* segundos, se llama a la entrada en cada nodo que encuentra el destino. Un destino que no encuentra nada no hace nada. Para un targetname o una ruta de nodo eso casi siempre es una errata, así que imprime una advertencia la primera vez que se activa. Una llamada con retardo se descarta si su entidad salió del árbol mientras tanto, por ejemplo porque la mataron.

Sin retardo, toda la cadena se ejecuta dentro del `emit` de la señal, antes de que siga el código que la emitió. Eso importa cuando tu script lee estado justo después de emitir, las puertas y los contadores de más abajo ya han reaccionado.

## El activador { #the-activator }

El activador es quien inició la cadena, normalmente el jugador que pulsó el botón o entró en el trigger. Más abajo en la cadena, `!activator` significa ese nodo, así que un trigger puede herir o teletransportar exactamente a quien lo tocó.

Se toma del primer argumento `Node` de la señal que se activó, así que solo sobrevive mientras cada señal de la cadena pase uno. Las señales integradas se comportan así:

| Señales | Qué le pasa al activador |
| --- | --- |
| `triggered`, `pressed`, `entered`, `exited`, `hurt`, `teleported`, `pushed` | Se pasa sin cambios, estas señales llevan el nodo que las causó |
| `arrived(corner)`, `spawned(node)`, `reached(train)` | Se sustituye por el nodo entre paréntesis, así que más abajo `!activator` es ese corner, ese nodo generado o ese tren |
| `opened`, `timer`, `step_N`, `on_true`, `on_case_N`, `result`, `broken`, `switched`, `map_spawn` | Se pierde. Estas no llevan ningún nodo, así que desde aquí `!activator` no encuentra nada |

> [!TIP]
> ¿Necesitas al jugador varios pasos más abajo en una cadena que perdió el activador? Apunta a `!player` en su lugar, no depende de la cadena.
