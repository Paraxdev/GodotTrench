---
source: c6245792b90999ca4b48812eb3d96e7a9c2389d6
---

# Matemáticas y valores { #math-and-values }

> [!NOTE]
> Estas entidades vienen con el paquete de entidades de gameplay, mira
> [Instalar y personalizar](README.md#installing-and-customizing).

Entidades de punto invisibles que guardan, calculan y comparan números, para que una puntuación, un conteo o un precio puedan dirigir el mapa sin un script. Se cablean como cualquier entidad. Un número viaja por una conexión como el valor de una salida, como `changed(value)` o `result(value)`, y una entrada sin parámetro propio lo recibe, mira [Sin parámetro](../parameters.md#no-parameter). Un parámetro escrito en la conexión gana al valor.

Los números enteros siguen siendo enteros, `2 + 3` es `5` y no `5.0`, y un número que llega como texto, como `2.5`, cuenta como número. Todo lo que no es un número cuenta como texto.

## math_value { #math_value }

Guarda un número o un fragmento de texto, igual que una variable. Los botones le hacen `add`, y `changed` avisa al resto del mapa de cuál es ahora. Dispara `changed` solo cuando el valor es distinto del anterior.

* **Entradas:** `set_value(value)`, `add(amount)` (1 por defecto), `get_value`, `reset`
* **Salidas:** `value(value)`, `changed(value)`
* **Claves:** `start_value` 0, un número o un texto

`add` suma dos números y concatena cuando uno de ellos es texto, así que añadir `!` a `hello` da `hello!`. `get_value` dispara `value` con lo guardado, para cuando algo necesita el número sin esperar a un cambio. `reset` vuelve a `start_value`. Un `set_value` sin valor conserva lo guardado.

## math_calc { #math_calc }

Hace un cálculo con el número que recibe y dispara la respuesta. No guarda nada, así que la misma entrada siempre da la misma respuesta. Encadena varias para resolver algo por pasos.

* **Entradas:** `calculate(value)`, un valor que falta cuenta como 0
* **Salidas:** `result(value)`
* **Claves:** `operation` add, `operand` 1

| `operation` | `result` |
| --- | --- |
| `add`, `subtract`, `multiply` | el valor más, menos o por `operand` |
| `divide` | el valor dividido entre `operand` |
| `modulo` | el resto de la división, nunca negativo para un `operand` positivo, así que `-1` módulo `3` es `2` |
| `power` | el valor elevado a `operand` |
| `min`, `max` | el menor o el mayor de los dos |
| `abs`, `negate` | el valor sin su signo, o con el signo invertido |
| `round`, `floor`, `ceil` | el número entero más cercano (las mitades se alejan de 0), redondeado hacia abajo, redondeado hacia arriba |

`abs`, `negate`, `round`, `floor` y `ceil` ignoran `operand`. Cuando no hay respuesta, como en `divide` o `modulo` entre 0 o la raíz de un número negativo, no se dispara nada y Godot imprime una advertencia una sola vez.

## math_compare { #math_compare }

Compara un valor guardado con `compare_value` y dispara de qué lado cayó. `on_not_equal` se dispara junto a `on_less` y `on_greater`, para cuando solo importa una diferencia.

* **Entradas:** `set_value(value)`, `set_compare_value(value)`, `compare`, `set_and_compare(value)`
* **Salidas:** `on_less`, `on_equal`, `on_not_equal`, `on_greater`
* **Claves:** `compare_value` 0

`set_value` y `set_compare_value` solo guardan, `compare` compara lo guardado, y `set_and_compare` hace ambas cosas, así que es la entrada que hay que cablear a una salida `changed(value)`. Los números se comparan por valor, y `10` es mayor que `9`. Dos textos se comparan en orden de diccionario, con las mayúsculas antes que las minúsculas. Los decimales casi iguales, como `0.1 + 0.2` y `0.3`, cuentan como iguales.

## Ejemplo: una puntuación que abre una puerta { #example-a-score-that-opens-a-door }

Tres botones de monedas y una palanca abren la puerta de una cámara acorazada cuando hay al menos tres monedas.

1. Coloca un `math_value` llamado `score`, un `math_compare` llamado `enough` con `compare_value` 3, y un `logic_gate` llamado `door_gate` con `gate` and.
2. Cablea el `pressed` de cada botón de moneda a `score`, entrada `add`. Cada pulsación suma 1.
3. Cablea el `changed` de `score` a `enough`, entrada `set_and_compare`. Pasa la nueva puntuación.
4. Cablea el `on_less` de `enough` a `door_gate`, entrada `set_a`, parámetro `false`. Cablea `on_equal` y `on_greater` igual con el parámetro `true`. El `a` de la puerta lógica ahora dice si hay monedas suficientes.
5. Cablea el `pressed` de la palanca a `door_gate`, entrada `toggle_b`.
6. Cablea el `on_true` de `door_gate` a la puerta con la entrada `open`, y `on_false` con la entrada `close`.

La puerta se abre cuando cae la tercera moneda con la palanca accionada, o al revés. Devolver la palanca o reiniciar la puntuación la cierra. Para mostrar la puntuación, cablea el `changed` de `score` a un `game_text` con la entrada `set_text`, o pon un `math_calc` en medio para mostrar puntos en vez de monedas.

El mapa logic playground construye este vestíbulo, y añade una lotería de luces con `logic_random` y `logic_case` y un interruptor de lámpara con `logic_flipflop`. Abre `godot/demo/maps/logic_playground.tdm` en el proyecto de demostración, o lee cómo está hecho en [`examples/mcp/logic_playground.json`](https://github.com/Paraxdev/TrenchDot/blob/main/examples/mcp/logic_playground.json).

<div class="td-mcp" markdown>

## MCP { #mcp }

El script anterior coloca las entidades con `gameplay place`, una llamada cada una:

```json
{ "tool": "gameplay", "args": { "op": "place", "classname": "math_value", "origin": [-280, 40, 40],
  "properties": { "targetname": "score" },
  "outputs": [ { "output": "changed", "target": "enough", "input": "set_and_compare" } ] } }
{ "tool": "gameplay", "args": { "op": "place", "classname": "math_compare", "origin": [-280, 40, -40],
  "properties": { "targetname": "enough", "compare_value": "3" },
  "outputs": [ { "output": "on_greater", "target": "door_gate", "input": "set_a", "parameter": "true" } ] } }
```

</div>
