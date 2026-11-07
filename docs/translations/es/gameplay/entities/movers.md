---
source: 63266bb349fb8b02f543d43498bfdee3d8c3f3cd
---

# Puertas, movers y botones { #doors-movers-and-buttons }

> [!NOTE]
> Estas entidades vienen con el paquete de entidades de gameplay, mira
> [Instalar y personalizar](README.md#installing-and-customizing).

Entidades de brush que se mueven como un solo cuerpo y llevan consigo todo lo que está encima, así que un ascensor se lleva al jugador.

> [!NOTE]
> Las puertas y los botones no reaccionan al jugador por sí solos. El script de tu jugador llama a `use(activator)` sobre ellos, por ejemplo desde un rayo de interacción, mira [Un botón abre una puerta](../../tutorials/button-door.md). Pon `interact` en 0 para uno que solo deba moverse por I/O o por un script.

## func_door { #func_door }

Una puerta corredera, una escotilla o una persiana. Se mueve `travel` al abrirse y vuelve al cerrarse, arrastra el tirador en el viewport para establecerlo. Una puerta bloqueada ignora `open` y `use` y dispara `locked_use` en su lugar, para un mensaje de "bloqueada".

| Clave | Por defecto | Qué hace |
| --- | --- | --- |
| `travel` | 0 64 0 | Desplazamiento que recorre al deslizarse, en unidades de mapa |
| `speed` | 2.0 | Metros por segundo |
| `wait` | -1 | -1 se queda abierta, un valor positivo la cierra otra vez tras esos segundos |
| `locked` | 0 | Empieza bloqueada |
| `start_open` | 0 | Empieza en la posición abierta |
| `interact` | 1 | Se abre cuando el jugador llama a `use` sobre ella |

* **Entradas:** `open`, `close`, `toggle`, `lock`, `unlock`, `use(activator)`
* **Salidas:** `opened`, `closed`, `started_opening`, `started_closing`, `locked_use(activator)`

## func_door_rotating { #func_door_rotating }

Una puerta o portón que gira alrededor de una bisagra que arrastras a su sitio en el viewport. `open` recibe al activador para que `open_away` pueda abrirla alejándola de él en vez de hacia su cara.

| Clave | Por defecto | Qué hace |
| --- | --- | --- |
| `hinge` | 0 0 0 | Posición de la bisagra relativa al centro de la puerta |
| `axis` | y | Eje de giro. `y` es una bisagra vertical, `x` o `z` hacen una escotilla o un puente levadizo |
| `open_angle` | 95 | Grados que gira, un valor negativo gira al otro lado |
| `speed` | 120 | Grados por segundo |
| `open_away` | 1 | Gira alejándose de quien la abre |
| `wait`, `locked`, `start_open`, `interact` | -1, 0, 0, 1 | Como en `func_door` |

* **Entradas:** `open(activator)`, `close`, `toggle(activator)`, `lock`, `unlock`, `use(activator)`
* **Salidas:** las mismas que `func_door`

`func_gate` es un alias antiguo que solo tiene como claves `hinge`, `open_angle` 100 y `speed` 90. Usa `func_door_rotating`.

## func_platform { #func_platform }

Un ascensor o plataforma móvil entre su inicio y `start + travel`, para ascensores o suelos que cruzan un foso.

* **Entradas:** `start`, `stop`, `toggle`, `go_to_end`, `go_to_start`
* **Salidas:** `reached_end`, `reached_start`, `started`
* **Claves:** `travel` 0 128 0, `speed` 2.0, `wait` 1.0, `mode` 0, `start_active` 0 (se mueve desde que se carga el mapa)

| `mode` | Qué hace `start` | Uso típico |
| --- | --- | --- |
| 0, alternar | Se mueve al otro extremo | Un ascensor llamado por un botón |
| 1, ping pong | Va y viene hasta `stop`, esperando `wait` segundos en cada extremo | Una plataforma que no deja de ciclar |
| 2, una vez | Se mueve hasta el final y se queda ahí | Un puente que se extiende una sola vez |

## func_train { #func_train }

Sigue una cadena de entidades `path_corner` a partir de `target`, para cualquier cosa que recorra una ruta con varias paradas, como un tranvía o una vagoneta sobre un raíl. El `wait` y el `speed` de cada esquina se aplican al llegar. Empieza a moverse al cargar el mapa y da vueltas en bucle, así que pon `start_active` o `loop` en 0 para un viaje único.

* **Entradas:** `start`, `stop`, `toggle`
* **Salidas:** `arrived(corner)`, `finished` (solo cuando no hace bucle)
* **Claves:** `target`, `speed` 3.0, `loop` 1, `start_active` 1, `orient` 0 (1 lo gira para mirar hacia donde va)

## func_button { #func_button }

Se hunde `travel` mientras está pulsado y dispara `pressed`. Vuelve a salir tras `wait` segundos, mientras que -1 lo mantiene pulsado para un interruptor de un solo uso. Un botón bloqueado dispara `locked_use` en su lugar, útil para un panel que necesita corriente primero.

* **Entradas:** `press(activator)`, `release`, `lock`, `unlock`, `use(activator)`
* **Salidas:** `pressed(activator)`, `released`, `locked_use(activator)`
* **Claves:** `travel` 0 -2 0, `wait` 1.0, `locked` 0, `interact` 1
