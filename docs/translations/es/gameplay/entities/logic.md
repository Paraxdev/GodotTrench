---
source: 5168ac40e48ba63850a1be1795dae420bfe6bf5f
---

# Lógica { #logic }

> [!NOTE]
> Estas entidades vienen con el paquete de entidades de gameplay, mira
> [Instalar y personalizar](README.md#installing-and-customizing).

Entidades de punto invisibles que deciden cuándo pasan las cosas. Se colocan entre un trigger o un botón y lo que afecta, y dan forma a la señal por el camino. Las entidades que trabajan con números están en [Matemáticas y valores](math.md), y las que ejecutan código o animaciones en [Scripts y llamadas](scripting.md).

## logic_relay { #logic_relay }

Pasa `trigger` como `triggered` mientras está activado. Cablea varias cosas a un mismo relé y luego usa `disable` para apagar toda la cadena de una vez.

* **Entradas:** `trigger(activator)`, `enable`, `disable`, `toggle`
* **Salidas:** `triggered(activator)`
* **Claves:** `start_disabled` 0

## logic_branch { #logic_branch }

Recuerda verdadero o falso y dispara `on_true` u `on_false` con `test`, para comprobaciones como "la puerta solo se abre si hay corriente". El generador lo pone en verdadero, el botón de la puerta lo comprueba.

* **Entradas:** `set_true`, `set_false`, `toggle`, `test`, `set_and_test(value)`
* **Salidas:** `on_true`, `on_false`
* **Claves:** `start_value` 0

## logic_gate { #logic_gate }

Guarda dos valores verdadero o falso, `a` y `b`, y los combina con una puerta lógica, para comprobaciones como "la puerta se abre cuando hay corriente **y** la palanca está accionada". Cada cambio de entrada que invierte el resultado dispara `changed(on)` y luego `on_true` u `on_false`, así que nadie tiene que preguntar. `test` vuelve a disparar el resultado actual, para cuando otra cosa necesita saberlo ahora.

* **Entradas:** `set_a(value)`, `set_b(value)`, `toggle_a`, `toggle_b`, `test`
* **Salidas:** `on_true`, `on_false`, `changed(on)`
* **Claves:** `gate` and, `start_a` 0, `start_b` 0

| `gate` | Verdadero cuando |
| --- | --- |
| `and` | ambos son verdaderos |
| `or` | al menos uno es verdadero |
| `xor` | exactamente uno es verdadero |
| `nand` | no son ambos verdaderos |
| `nor` | ninguno es verdadero |
| `xnor` | ambos son iguales |
| `not` | `a` es falso, `b` se ignora |

`set_a` y `set_b` sin valor ponen verdadero, así que la salida `entered` de un trigger cableada a `set_a` activa `a`. Un número es verdadero salvo que sea 0, y un texto es verdadero solo para `true`, `yes` o `1`. Los valores iniciales no disparan nada, ni tampoco un cambio que deja el resultado como estaba.

## logic_flipflop { #logic_flipflop }

Alterna cada vez que se activa, así que un solo botón puede encender algo y volver a apagarlo. `on_a` se dispara cuando se enciende y `on_b` cuando se apaga. `reset` lo devuelve a `start_on` sin disparar ninguna de las dos, y dispara solo `changed` si eso fue un cambio.

* **Entradas:** `trigger`, `reset`
* **Salidas:** `on_a`, `on_b`, `changed(on)`
* **Claves:** `start_on` 0 (encendido, así que el primer trigger dispara `on_b`)

## logic_case { #logic_case }

Compara el valor que recibe con hasta ocho casos y dispara la salida del primero que coincide, o `on_default` cuando ninguno lo hace. Convierte un número, como el índice de un `logic_random` o la puntuación de un `math_value`, en eventos distintos.

* **Entradas:** `in_value(value)`
* **Salidas:** `on_case_1` a `on_case_8`, `on_default`
* **Claves:** `case_1` a `case_8`, vacías

Los números coinciden por valor, así que el caso `2` coincide con un valor de 2 y de `2.0`. El texto tiene que coincidir exactamente, `Red` no es `red`, y los espacios alrededor de un caso se ignoran. Los casos vacíos se saltan, gana la primera coincidencia, y un valor que falta va a `on_default`.

## logic_random { #logic_random }

Elige al azar una de hasta ocho salidas, o tira una probabilidad. Cablea `out_1` a `out_3` a tres luces y `pick` enciende una de ellas. `roll` decide por probabilidad en cambio, por ejemplo si un cofre contiene botín.

* **Entradas:** `pick`, `roll`
* **Salidas:** `out_1` a `out_8`, `picked(index)`, `on_success`, `on_fail`
* **Claves:** `count` 3 (1 a 8), `no_repeat` 0, `chance` 50 (porcentaje)

`pick` dispara `picked` con el número de la salida que eligió y luego esa salida, así que `picked` puede alimentar un [`logic_case`](#logic_case). Con `no_repeat` nunca elige la salida que eligió la vez anterior, salvo que `count` sea 1. `roll` dispara `on_success` con una probabilidad de `chance` por ciento y `on_fail` en caso contrario, así que 0 nunca tiene éxito y 100 siempre lo tiene.

## logic_counter { #logic_counter }

Cuenta entre `min` y `max`, para puzles como "pulsa los tres botones": cada botón envía `add`, y `hit_max` abre la puerta. El valor se limita al rango y no se dispara nada a menos que cambie. `hit_max` y `hit_min` se disparan cuando un cambio cae en un límite, `reset` dispara solo `changed`.

* **Entradas:** `add(amount)` (1 por defecto), `subtract(amount)`, `set_value(value)`, `reset`
* **Salidas:** `hit_max`, `hit_min`, `changed(value)`
* **Claves:** `min` 0, `max` 3, `start_value` 0

## logic_timer { #logic_timer }

Dispara `timer` una y otra vez, para cualquier cosa que se repita, como una luz que parpadea u oleadas de enemigos. Espera `interval` segundos entre disparos y funciona desde que se carga el mapa, salvo que `start_on` sea 0. Con `random_max` por encima de `random_min` cada espera es aleatoria dentro de ese rango, así que las repeticiones se sienten menos mecánicas. `once` lo detiene tras el primer disparo, lo que lo convierte en un simple retardo.

* **Entradas:** `start`, `stop`, `toggle`, `fire` (una vez, ahora)
* **Salidas:** `timer`
* **Claves:** `interval` 1.0, `random_min` 0, `random_max` 0, `start_on` 1, `once` 0

## logic_auto { #logic_auto }

Dispara `map_spawn` una vez, cuando todas las entidades del mapa están listas. Cablea aquí cualquier cosa que deba funcionar desde el principio, como la música o un texto de introducción.

* **Salidas:** `map_spawn`

## logic_sequence { #logic_sequence }

Una línea de tiempo de cinemática. Dispara hasta ocho pasos numerados en orden y espera antes de cada uno, el primero incluido. Cablea cada paso a lo que ocurre en ese momento, una puerta que se abre, una línea de texto, un actor que camina.

* **Entradas:** `start`, `stop`, `reset`
* **Salidas:** `step_1` a `step_8`, `step(index)`, `finished`
* **Claves:** `steps` 3 (1 a 8), `interval` 1.0, `times` vacía, `start_active` 0 (funciona desde que se carga el mapa), `loop` 0

`times` enumera una espera por paso, como `0 1 0.5`, y los pasos sin una usan `interval`. Una secuencia en bucle nunca dispara `finished`. `start` se ignora mientras funciona, y tras `stop` empieza otra vez en el paso 1. Mira el [tutorial de la cinemática](../../tutorials/cutscene.md).
