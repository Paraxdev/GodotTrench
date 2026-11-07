---
source: 9167db67d391d56c5d5d5ba31e5f260c3c7d0f58
---

# Referencia de entidades { #entity-reference }

Las entidades del paquete de entidades de gameplay, agrupadas por para qué las usarías. Cada entrada dice para qué sirve la entidad y luego enumera sus entradas (lo que otras entidades pueden ordenarle hacer), sus salidas (eventos que puedes cablear hacia otras entidades) y sus claves (los ajustes del Inspector). Cómo se conectan las salidas a las entradas se explica en [Entradas y salidas](../io.md).

| Página | Qué encuentras ahí |
| --- | --- |
| [Lógica](logic.md) | Ayudantes invisibles que deciden cuándo pasan las cosas: relés, comprobaciones "solo si", puertas lógicas, flip flops, elecciones y casos aleatorios, contadores, temporizadores, un evento de inicio del mapa y líneas de tiempo de cinemáticas |
| [Matemáticas y valores](math.md) | Ayudantes invisibles que guardan, calculan y comparan números, como una puntuación que abre una puerta |
| [Scripts y llamadas](scripting.md) | Ejecuta un fragmento de GDScript, llama al código de tu juego, reproduce una animación o imprime lo que está haciendo una cadena |
| [Puertas, movers y botones](movers.md) | Entidades de brush que se mueven: puertas correderas y batientes, ascensores, trenes y botones |
| [Triggers](triggers.md) | Volúmenes invisibles que reaccionan cuando un cuerpo entra en ellos, para iniciar eventos, hacer daño, teletransportar, empujar o generar |
| [Spawners y rutas](actors.md) | Generar enemigos o objetos recogibles, destinos de teletransporte, esquinas de ruta y personajes de cinemáticas con guion |
| [Props, luces y efectos](effects.md) | Props de física rompibles, modelos estáticos, explosiones, texto en pantalla, luces conmutables, sonido, partículas y sondas de luz |
| [Secciones en streaming](sections.md) | Puntos de referencia y volúmenes conectores para jugar varios mapas como un solo mundo sin pantalla de carga |

## Instalar y personalizar { #installing-and-customizing }

El paquete no forma parte del addon en sí. Pulsa **Instalar entidades de gameplay** en la ventana *Set up this project*, o elige *Gameplay > Instalar entidades de gameplay*, y sus definiciones y scripts se copian en `res://trenchdot/entities`. Hasta entonces el panel Entidades solo lista el núcleo: worldspawn, los ayudantes `func_`, `light`, `light_spot`, las sondas de luz y de reflejo y `info_player_start`.

Las copias pertenecen a tu proyecto. Cambia el script de una entidad en `scripts/` o sus claves y valores por defecto en `definitions/`, borra las entidades que no uses o añade las tuyas. Una actualización del addon ofrece un paquete más nuevo pero nunca sobrescribe un archivo que hayas editado. [Configuración del proyecto](../../godot/project-setup.md#core-entities-and-the-gameplay-entities-pack) explica las actualizaciones y qué pasa cuando dos definiciones comparten un classname.

## Leer las entradas { #reading-the-entries }

* Los argumentos entre paréntesis son lo que acepta una entrada o lo que pasa una salida, por ejemplo `trigger(activator)` recibe el cuerpo o la entidad que inició la cadena.
* Toda entidad con entradas o salidas tiene una clave `targetname`, salvo `logic_auto`. Es el nombre que usan las demás entidades para llegar a ella.
* Una clave que no establezcas usa el valor por defecto indicado.
* Las distancias están en unidades de mapa, 32 por metro. Las velocidades están en metros por segundo, y `omni_range`, `spot_range` y `max_distance` están en metros.
* Las listas de entradas y salidas son lo que ofrecen los selectores del editor. En tiempo de ejecución cualquier método del script de la entidad funciona como entrada, así que el selector no es la lista completa.

Para añadir tus propias clases, mira [Entidades propias en GDScript](../custom-entities.md).

## Lo que no está en el paquete { #not-in-the-pack }

**El núcleo** viene con el addon y no necesita el paquete. Worldspawn contiene el sol, la luz ambiental, el cielo, la niebla y los ajustes de streaming, y el Inspector lo muestra cuando no hay nada seleccionado. Las clases de FuncGodot `func_geo`, `func_detail`, `func_detail_illusionary` y `func_illusionary` construyen geometría simple. `info_player_start` marca dónde aparece el jugador pero no genera nada por sí misma, el código de tu juego coloca ahí al jugador. [`light` y `light_spot`](effects.md) son las luces conmutables.

Las entidades de brush y las de modelo colisionan por defecto, mira [Colisión](collision.md) para las claves que lo cambian.

**El jugador** no es una entidad, es una escena de tu propio juego. El paquete lo espera en el grupo `player` y en la capa de colisión 1, donde miran los triggers, con un método `take_damage(amount, source)` que llaman los triggers de daño y las explosiones. Un jugador en otra capa necesita esa capa en *Trigger Collision Mask*, mira [Triggers](triggers.md#shared-inputs-outputs-and-keys). Mira [`godot/demo/player.gd`](https://github.com/Paraxdev/TrenchDot/blob/main/godot/demo/player.gd) para ver uno completo.
