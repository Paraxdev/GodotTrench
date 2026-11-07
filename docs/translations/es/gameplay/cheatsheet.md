---
source: 414dc5a3711ba57511bd4ecc18dc19f159af44d7
---

# Chuleta de I/O { #io-cheatsheet }

Un resumen de una página de [Cómo funciona el I/O de entidades](io.md) y [Destinos, parámetros y entradas](parameters.md), para cuando ya conoces la idea y solo necesitas los detalles.

## Cuándo se activa una salida { #when-an-output-fires }

La señal se activa, cuenta contra *Veces*, espera el *Retardo*, y luego llama a la entrada en cada nodo que encuentra el destino. Si no encuentra ninguno no pasa nada, y un targetname o ruta de nodo que no encuentra nada avisa una vez.

## Destinos { #targets }

| Escribe | Llega a |
| --- | --- |
| `main_door` | Cada entidad con ese targetname, en el mismo mapa |
| `door_*` | Cada targetname que empiece por `door_` |
| `@enemies` | Cada nodo de ese grupo de Godot, en cualquier parte del árbol |
| `!player` | El primer nodo del grupo `player` |
| `!activator` | Quien inició la cadena, `!caller` significa lo mismo |
| `!self` | La entidad dueña de la salida |
| `/root/Game` | El nodo en esa ruta absoluta |

## Parámetros { #parameters }

Un valor simple se prueba como int, float, bool, luego `x y z` como un `Vector3`, y si no se queda como string. Un `[a, b]` JSON se reparte en varios argumentos. `$activator`, `$self`, `$position` y `$caller_name` se rellenan en la entidad **receptora**.

Sin parámetro, el activador va al primer argumento de tipo objeto, y los demás valores que llevaba la salida rellenan el resto en orden.

## Quién conserva el activador { #who-keeps-the-activator }

| Señales | Activador |
| --- | --- |
| `triggered`, `pressed`, `entered`, `exited`, `hurt`, `teleported`, `pushed` | Se pasa sin cambios |
| `arrived(corner)`, `spawned(node)`, `reached(train)` | Se sustituye por el nodo entre paréntesis |
| `opened`, `timer`, `step_N`, `on_true`, `on_case_N`, `result`, `broken`, `switched`, `map_spawn` | Se pierde |

Si la cadena lo perdió y necesitas al jugador, apunta a `!player`.

## Entradas que tiene todo nodo { #inputs-every-node-has }

`kill`, `show`, `hide`, `enable`, `disable` y `toggle` funcionan en cualquier nodo, siempre que la entidad no tenga un método con ese nombre. Una entrada que coincide con una propiedad la fija al parámetro.

## Alcance de logic_script { #logic_script-scope }

Un [`logic_script`](entities/scripting.md) ejecuta un fragmento de GDScript cuando se activa. Dentro, `this` es el propio logic_script, `activator` el nodo que inició la cadena, `parameter` el valor pasado, `io` los ayudantes de `TrenchDotIO` y `tree` el `SceneTree`. Indenta con tabuladores.

```gdscript
var p = io.find_targets(this, "!player", null)
if not p.is_empty():
	p[0].take_damage(25, this)
```

## ¿No pasó nada? { #nothing-happened }

Mira el panel Problemas, luego el grafo lógico, luego F3 en el juego, y luego registra `TrenchDotIO.events()`. Mira [Depurar el cableado](debugging.md).
