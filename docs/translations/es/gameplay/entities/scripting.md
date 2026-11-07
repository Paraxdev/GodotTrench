---
source: 2bf109094dcc42ebb11cf6c736e210c0e4e79b02
---

# Scripts y llamadas { #scripts-and-calls }

> [!NOTE]
> Estas entidades vienen con el paquete de entidades de gameplay, mira
> [Instalar y personalizar](README.md#installing-and-customizing).

Entidades de punto para lo que el simple cableado no alcanza: el estado del juego, el código de tu propio juego, las animaciones y descubrir por qué una cadena no se dispara.

## logic_script { #logic_script }

Ejecuta un fragmento de GDScript cuando se dispara, para un poco de lógica que ninguna entidad incluida ofrece, como leer la salud del jugador, sin escribir una entidad propia.

* **Entradas:** `run(activator)`, `run_with(parameter)`
* **Salidas:** `ran(result)`, con el valor devuelto por el fragmento
* **Claves:** `source` vacía, `expression` vacía (una sola línea, se usa cuando `source` está vacía)

El fragmento ve `this` (la propia entidad), `activator` (quien inició la cadena), `parameter` (el valor entrante), `io` (los ayudantes de `TrenchDotIO`, por ejemplo para encontrar entidades por nombre) y `tree` (el `SceneTree`). `run_with` siempre tiene un `activator` nulo. Un error de compilación se registra como advertencia al cargar el mapa. Este fragmento hace daño al jugador:

```gdscript
var p = io.find_targets(this, "!player", null)
if not p.is_empty():
	p[0].take_damage(25, this)
```

> [!WARNING]
> Sangra las líneas anidadas con tabuladores. Los espacios no compilan y dan un error de sangría mixta.

> [!WARNING]
> El fragmento es código que viene dentro del mapa. Se ejecuta siempre que corre el juego, con los mismos permisos que los scripts propios del proyecto, así que trata un mapa descargado que contiene código como cualquier programa que descargas: puede hacer cualquier cosa en tu ordenador. El panel Problemas avisa de cada `logic_script` que contiene código, de cada salida que puede entregar código a uno y de cada prefab con alguna de las dos cosas, léelos antes de jugar el mapa.

## logic_call { #logic_call }

Llama a un método de cualquier destino con argumentos fijos, para llegar al código de tu juego, como sumar puntuación en un autoload `Game`. A diferencia de una salida simple, avisa cuando el destino o el método no existen, así que los errores tipográficos aparecen en el registro. Los métodos de C# funcionan con sus nombres en PascalCase.

* **Entradas:** `trigger(activator)`, `call_with(parameter)`
* **Salidas:** `called(result)`
* **Claves:** `call_target` `/root/Game`, `method` vacía, `arguments` `[]`

`call_target` es una ruta de nodo, un `@group`, un targetname, `!activator` o `!self`. `arguments` es un array JSON, como `["key_red", "$activator"]`, y admite los marcadores de [Parámetros](../parameters.md), resueltos en el propio `logic_call`. `call_with` pone su parámetro donde está `"$parameter"`, o lo pasa como único argumento.

## logic_animate { #logic_animate }

Reproduce animaciones en el primer `AnimationPlayer` en `target` o bajo él, para personajes y props que llevan sus propias animaciones. Pon `target` en `!activator` para animar a quien inició la cadena.

* **Entradas:** `play(name)`, `stop`, `queue(name)`, `seek(time)`
* **Salidas:** `finished(anim)`
* **Claves:** `target` `!self`, `animation` vacía (se reproduce cuando `play` no tiene nombre), `speed` 1.0

## logic_debug { #logic_debug }

Imprime una línea con `write` y muestra sus últimas líneas como una etiqueta flotante en las compilaciones de depuración, para ver en el juego si una cadena llega hasta ella y con qué parámetro. Con `trace_all` activado registra todas las salidas del juego, una forma rápida de vigilar una cadena entera. Mira [Depuración](../debugging.md).

* **Entradas:** `write(parameter)`, `toggle` (la etiqueta), `show`, `hide`
* **Salidas:** `printed(text)`
* **Claves:** `message` `{name} received {parameter} from {activator}` (`{time}` también funciona), `show_label` 1,
  `trace_all` 0
