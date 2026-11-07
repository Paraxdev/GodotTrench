---
source: 7e5d26a9fedbc9f43e9edcb92409992b19d24676
---

# Navegación { #navigation }

`TrenchDotNav` precalcula una malla de navegación a partir de un mapa construido, para que los enemigos y los NPC encuentren el camino por él.

```gdscript
map.build()
var region := await TrenchDotNav.bake_region(map)
```

`bake_region` precalcula en un hilo de trabajo, añade un `NavigationRegion3D` llamado `navigation` bajo el mapa y vuelve cuando ya funcionan los caminos sobre él. Pásale un nodo que contenga varios mapas para precalcularlos todos en una sola malla. Una reconstrucción libera la región, así que precalcula de nuevo después.

## Qué cuenta { #what-counts }

El cálculo lee la colisión del mapa, no sus mallas, así que un agente rodea todo aquello contra lo que un jugador puede chocar.

| En el mapa | En la malla de navegación |
| --- | --- |
| Brushes del mundo, `func_detail`, terreno y colisión de props | Sí |
| Brushes clip | Sí, bloquean como paredes |
| Brushes ilusorios, triggers y props con físicas | No |
| Puertas, botones, plataformas y trenes | No, así que una puerta cerrada no sella su hueco |
| Cuerpos del grupo `navigation_ignore` | No |

Eso significa que los caminos atraviesan las puertas cerradas, así que un agente tiene que abrir la puerta con la que se topa.

## Tamaño del agente { #agent-size }

`TrenchDotNav.profile(radius, height, max_climb, max_slope)` te da los ajustes para un tamaño de agente. El radio mantiene los caminos lejos de las paredes, la altura descarta los techos bajos, *max_climb* es el escalón más alto que el agente sube y *max_slope* la rampa más empinada. Los tamaños van en metros, la pendiente en grados:

| Argumento | Por defecto |
| --- | --- |
| `radius` | 0.3 |
| `height` | 1.8 |
| `max_climb` | 0.3 |
| `max_slope` | 45 |

La malla usa celdas de 0.1 m y ajusta los tamaños a ellas. Pasa un perfil como segundo argumento, por ejemplo para una criatura ancha y baja:

```gdscript
var region := await TrenchDotNav.bake_region(map, TrenchDotNav.profile(0.5, 1.2))
```

(`bake_region` fija el tamaño de celda del mapa de navegación al del cálculo, y las demás regiones de ese mapa tienen que compartirlo.)

## Caché { #caching }

El primer cálculo de un mapa grande tarda unos segundos. El resultado se guarda en caché en `user://trenchdot_navigation`, con la colisión y los ajustes como clave, así que el siguiente arranque con el mismo mapa lo carga en una fracción de segundo. Pasa `false` como tercer argumento para calcular sin la caché.

## Comprobar un mapa { #checking-a-map }

`TrenchDotNav.bake(map)` calcula en el hilo que llama y devuelve el `NavigationMesh`, útil para herramientas y tests. `TrenchDotNav.islands(nav)` lista sus partes sin conexión como índices de polígonos, de la más grande a la más pequeña.

Una sala que debería ser alcanzable pero queda en una isla propia suele tener un hueco bloqueado por algo bajo, o un escalón más alto que `max_climb`. Las partes superiores de las paredes y el suelo dentro de paredes gruesas también salen como islas pequeñas.

> [!WARNING]
> Godot detiene una búsqueda de camino a los 4096 polígonos y devuelve el camino hasta ahí, lo que en un mapa grande parece un punto inalcanzable. `TrenchDotNav.path(node, from, to)` no tiene límite. En los agentes, pon `path_search_max_polygons` a 0 en el `NavigationAgent3D`.

## Verlo en TrenchDot { #seeing-it-in-trenchdot }

Puedes ver por dónde camina un agente directamente en TrenchDot, sin ejecutar el juego.

1. Abre la escena en Godot

    Guarda el mapa dentro del proyecto y abre el proyecto en el editor de Godot con una escena cuyo `FuncGodotMap` construya ese mapa.

2. Activa el área transitable

    Ve a *Ver > Área transitable > Mostrar área transitable*. El tamaño del agente está justo debajo, y la línea de abajo te dice cuánto es transitable y cuántas islas quedan aisladas.

    ![El menú Ver con el submenú Área transitable abierto: Mostrar área transitable, luego Radio 0.30 m, Altura 1.80 m, Subida máx. 0.30 m, Pendiente máx. 45.00 grados y Aún sin precalcular](../assets/navigation/menu-walkable-area.png)

Godot precalcula la escena por el [Live Link](live-link.md) y TrenchDot dibuja el resultado en las vistas 3D y 2D. (Si no puede precalcular, la barra de estado dice por qué, por ejemplo que el mapa tiene que guardarse antes en el proyecto.)

Los colores significan:

| Color | Significado |
| --- | --- |
| Verde azulado | La mayor área conectada, donde estarán la mayoría de los agentes |
| Bermellón | Islas a las que un agente del área verde no llega. Busca un escalón más alto que *Subida máx.*, un hueco más estrecho que el doble de *Radio* o una pendiente más empinada que *Pendiente máx.* |

El overlay vuelve a precalcular un momento después de cada edición. Sin [modo en vivo](live.md) Godot construye primero el mapa tal como se ve, igual que *Construir en Godot*.

> [!TIP]
> Las partes superiores de paredes y techos salen como islas pequeñas en bermellón. Busca suelo bermellón junto a suelo verde.

<div class="td-mcp" markdown>

## MCP { #mcp }

El [servidor MCP](../mcp.md) deja que agentes y scripts controlen el editor. Comprueban un mapa con la herramienta `walkability`, que devuelve las islas con sus áreas y límites en unidades de mapa.

</div>
