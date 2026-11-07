---
source: b890aefb21f6d8de673ab4ebc70a71d93fd598c5
---

# Destinos, parámetros y entradas { #targets-parameters-and-inputs }

Una conexión nombra su destino, su entrada y un parámetro opcional como texto plano. Esta página muestra cómo ese texto se convierte en los nodos, el método y los valores que se usan en el juego.

## Destinos { #targets }

El destino de una conexión, y de claves como `call_target`, decide qué nodos reciben la entrada. Casi siempre es un targetname que diste a otra entidad. Las otras formas llegan a grupos de cosas y a nodos fuera del mapa.

| Destino | Llega a | Úsalo para |
| --- | --- | --- |
| `main_door` | Cada entidad con ese targetname en el mismo mapa | El caso normal, una entidad con nombre o varias que comparten nombre |
| `door_*` | Cada targetname que empiece por `door_` | Abrir un conjunto entero a la vez. Solo funciona un `*` final |
| `@enemies` | Cada nodo del grupo de Godot `enemies`, en cualquier parte del árbol | Cosas generadas en tiempo de ejecución o colocadas por el código del juego |
| `!player` | El primer nodo del grupo `player` | Llegar al jugador cuando la cadena perdió el activador |
| `!activator` | Quien inició la cadena, `!caller` significa lo mismo | Herir o teletransportar a quien tocó un trigger |
| `!self` | La entidad dueña de la salida | Una entidad que actúa sobre sí misma, como un botón que se desactiva |
| `/root/Game` | Una ruta de nodo absoluta, tiene que empezar por `/` | Autoloads y otro código del juego fuera del mapa |

Los targetnames se buscan dentro del `FuncGodotMap` más cercano, así que dos mapas en una escena no pueden alcanzar los nombres del otro. Los grupos, las rutas de nodo y `!player` funcionan en todo el árbol, úsalos cuando un mapa tiene que hablar con otro.

> [!NOTE]
> En Source, `!caller` es la entidad que activó la salida. Aquí significa lo mismo que `!activator`, usa `!self` para la propia entidad.

### Cambios en tiempo de ejecución { #runtime-changes }

Las búsquedas de targetname se guardan en caché por mapa para ir más rápido. Después de generar o renombrar entidades con nombre en tiempo de ejecución, llama a `TrenchDotIO.invalidate(node)` con cualquier nodo de ese mapa para que la siguiente búsqueda las vea. Renombrar significa cambiar el meta `td_targetname`, el `name` del nodo no interviene. Matar una entidad no necesita ninguna llamada, la caché nota los nodos liberados.

Los nodos creados por un spawner no tienen targetname. Llega a ellos por su grupo, por ejemplo `@enemies`.

## Parámetros { #parameters }

El parámetro es un valor que escribes en la conexión y se pasa a la entrada, como `25` para `take_damage` o una velocidad para una puerta. El texto se convierte en el primer tipo al que encaja:

| Escribes | La entrada recibe |
| --- | --- |
| `25` | El entero 25 |
| `0.5` | El float 0.5 |
| `true` o `false` | Un bool |
| `0 64 0` | Un `Vector3`, a partir de tres números separados por espacios |
| `hello` | El string `"hello"`, cuando nada más encaja |

Un array JSON como `[10, "$activator"]` se reparte en varios argumentos, para entradas que toman más de uno. Los valores se convierten a los tipos que declara el método, así que `1234` llega a un argumento `String` como texto. Cuando el método toma un nodo antes que cualquier otro argumento, un texto como `!player` o un targetname se busca y se pasa como ese nodo.

Un parámetro también puede ser uno de estos marcadores, que se rellenan cuando se llama a la entrada:

| Marcador | Se convierte en |
| --- | --- |
| `$activator` | El activador, el nodo que inició la cadena |
| `$self`, `$caller` | La propia entidad receptora |
| `$position` | La posición global de la entidad receptora |
| `$caller_name` | El targetname de la entidad receptora, o si no el nombre de su nodo |

Los marcadores se rellenan en la entidad **receptora**, no en la que activó la salida. Tienen que ser el parámetro entero o un elemento entero del array, no se sustituyen dentro de un texto más largo.

## Sin parámetro { #no-parameter }

Deja el parámetro vacío y el activador va al primer argumento tipado como objeto, o sin tipo y llamado `activator`, esté donde esté. Por eso `take_damage(amount, source)` lo recibe en `source`. Los demás valores que llevaba la salida rellenan el resto de argumentos en orden, así que `changed(value)` cableado a `set_value` pasa el valor.

Eso reenvía también `false`, así que `switched(false)` cableado al `add` de un contador suma 0. Pon un parámetro cuando quieras un valor fijo.

## Cómo se encuentra una entrada { #how-an-input-is-found }

Cuando se activa una conexión, el nombre de la entrada se compara con el destino en este orden:

1. Un método con ese nombre, o su grafía en PascalCase o camelCase, así que un `AddScore` de C# responde a `add_score`.
2. Una propiedad con ese nombre, fijada al parámetro o al valor que llevaba la salida. La entrada `speed` con parámetro `4` cambia la velocidad de una puerta, sin necesidad de un método extra.
3. Una de las entradas integradas de abajo, que entiende todo nodo.

| Entrada | Efecto |
| --- | --- |
| `kill` | Libera el nodo, quitándolo del juego |
| `show`, `enable` | Lo hace visible y deja que vuelva a procesar |
| `hide`, `disable` | Lo hace invisible y pausa su procesamiento |
| `toggle` | Solo alterna la visibilidad, el procesamiento se queda como está |

Los métodos propios de la entidad ganan, así que `disable` en un trigger apaga el trigger en vez de ocultarlo. `show` y `hide` son los integrados salvo que el script de la entidad defina los suyos. Cualquier otra entrada imprime `[TD I/O] <node> has no input '<name>'`.
