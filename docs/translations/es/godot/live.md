---
source: 15ac2f37b7d278fa849fb066b91734f671f35658
---

# Guardado, modo en vivo y recarga en caliente { #saving-live-mode-and-hot-reload }

## Reconstruir al guardar { #rebuild-on-save }

Deja abierta en el editor de Godot la escena con tu `FuncGodotMap` y guarda en TrenchDot con Ctrl+S. Godot reconstruye esa escena enseguida, siempre que su pestaña sea la que Godot muestra y el nodo del mapa tenga *Auto Rebuild On Save* activado. Una escena en otra pestaña que use el mapa se reconstruye cuando cambias a su pestaña. Cerrar y reabrir una pestaña o recargar una escena no lo rompe. Guardar un [prefab](../editor/prefab-library.md) reconstruye los mapas de esa escena que lo usan, directamente o a través de otro prefab.

La barra de estado de abajo a la izquierda te dice qué hizo Godot. Cuando funcionó, debería decir:

![La barra de estado diciendo "Enlace en vivo: Godot reconstruyó 1 nodo(s) de mapa"](../assets/live/status-rebuilt.png)

Guarda la escena en Godot para conservar el resultado. Si la barra de estado dice otra cosa, el mensaje te dice por qué:

![La barra de estado diciendo que room.tscn usa room.tdm pero no es la pestaña de escena actual en Godot, se reconstruye al cambiar a ella](../assets/live/status-not-current-tab.png)

| La barra de estado dice | Haz esto |
| --- | --- |
| La escena no es la pestaña de escena actual en Godot | Cambia a esa pestaña, se reconstruye entonces |
| La pestaña de escena actual no tiene un `FuncGodotMap` para el mapa | Abre la escena que construye el mapa, o activa *Auto Rebuild On Save* en su nodo de mapa |
| Godot no tiene ninguna escena abierta | Abre en Godot la escena con el mapa, y si ya está abierta, sigue leyendo |
| Godot no se está ejecutando con el addon de TrenchDot | Abre el proyecto en el editor de Godot con el addon activado |
| Godot reconstruyó el mapa, pero los ajustes del mapa no tienen definición para algunas clases, así que se construyen como nodos simples | Añade las definiciones, mira [Entidades que no hacen nada](entities-troubleshooting.md) |
| Godot reconstruyó el mapa con advertencias, mira el panel Problemas | Léelas en el panel Problemas, donde empiezan por "Godot:" |

Solo un editor de Godot puede tener el puerto del Live Link. Un segundo editor con el mismo proyecto avisa en su panel Output de que el Live Link no pudo escuchar, y TrenchDot sigue hablando con el primero, que puede no tener ninguna escena abierta. Cierra el editor que no uses y el otro se hace cargo del enlace en unos segundos. (Nota: *Godot > Abrir proyecto en el editor de Godot* trae al frente un editor que ya tiene el proyecto, en lugar de iniciar un segundo.)

## Modo en vivo { #live-mode }

El modo en vivo envía tus ediciones a la escena abierta en el editor de Godot antes de guardar, para que juzgues la iluminación y la escala en el renderizador real.

1. Abre la escena en Godot

    El mapa tiene que estar guardado una vez dentro del proyecto, y la escena abierta en Godot necesita un `FuncGodotMap` que lo use.

2. Actívalo

    Haz clic en el botón de enlace del extremo derecho de la barra de herramientas, junto al robot de Godot, o usa *Godot > Modo en vivo*. El botón se ilumina, y su tooltip dice qué sigue esperando, si espera algo.

    ![La barra de herramientas con el botón de enlace del modo en vivo activado y su tooltip diciendo que las ediciones llegan a Godot antes de guardar](../assets/live/toolbar-live-mode-on.png)

3. Edita como siempre

    Cada cambio aparece en Godot al instante.

    ![Ediciones en TrenchDot apareciendo en el editor de Godot antes de guardar](../screenshots/TrenchDot-demo-live-view.gif)

El modo en vivo necesita el Live Link de Godot, que viene activado por defecto. Los dos están en *Archivo > Preferencias*, donde *Modo en vivo de Godot* es el mismo interruptor que el botón de la barra de herramientas.

![Las filas Live Link de Godot, Modo en vivo de Godot y Recarga en caliente del juego en marcha en Preferencias](../assets/live/prefs-live-link.png)

Los arrastres mueven nodos mientras arrastras, y la mayoría de las ediciones reconstruyen solo lo que tocaron, así que la vista previa sigue el ritmo. Alternar *Omit From Export* en una capa y editar instancias de prefab reconstruyen en cambio el mapa entero, cuando haces una pausa.

La vista previa se parece mucho a una construcción completa, pero no siempre es idéntica. Guardar siempre hace una construcción completa, y salir sin guardar devuelve a Godot al archivo guardado, así que nada de lo que probaste en modo en vivo se queda si no lo guardas.

### Cuándo el modo en vivo reconstruye el mapa entero { #when-live-mode-rebuilds-the-whole-map }

Godot reconstruye el mapa entero por sí mismo cuando la escena ya no coincide con lo que daría una construcción completa. Cuando el modo en vivo arranca sobre una escena lo hace al instante, y si no, cuando haces una pausa de edición:

| Cuándo | Por qué |
| --- | --- |
| El modo en vivo arranca sobre una escena construida con otras definiciones de entidades | La escena se construyó antes de instalar el paquete de entidades de gameplay o de añadir una definición, así que algunas entidades son nodos simples |
| El modo en vivo arranca sobre una escena que cambiaron ediciones en vivo anteriores | La escena aún guarda ediciones en vivo de una sesión anterior que nunca guardaste |
| Las definiciones de entidades cambian mientras corre el modo en vivo | Guardaste un FGD o una definición en el Inspector de Godot, cambió una clase de script, o llegaron archivos nuevos, como un paquete instalado |
| Guardas un prefab que usa el mapa | La escena aún muestra el prefab como estaba |

Las escenas abiertas fuera del modo en vivo conservan sus entidades como se construyeron, y el panel Output de Godot avisa cuando una de ellas se construyó con otras definiciones de entidades. Pulsa **Build Map** en ella y guarda.

### Entidades sin definición { #entities-without-a-definition }

Cuando el modo en vivo construye una entidad cuya clase Godot no tiene definida, la barra de estado nombra la clase y el FGD al que añadirla:

![La barra de estado diciendo que Godot no tiene definición para crypt_lamp, muestra un nodo simple, añade la clase al FGD de res://crypt_settings.tres](../assets/live/status-no-definition.png)

El tooltip del botón del modo en vivo mantiene la misma nota mientras el mapa use esa clase:

![El botón del modo en vivo activado, su tooltip diciendo que las ediciones llegan a Godot antes de guardar, y que Godot no tiene definición para crypt_lamp](../assets/live/toolbar-live-mode-no-definition.png)

Añade la definición y guárdala, y Godot reconstruye el mapa por sí mismo. La nota desaparece con tu siguiente guardado. Mira [Entidades que no hacen nada](entities-troubleshooting.md) para saber dónde va una definición.

## Recarga en caliente en un juego en marcha { #hot-reload-in-a-running-game }

Para ver tus guardados en un juego que ya está en marcha:

1. Añade el autoload

    En Godot, ve a *Project > Project Settings > Globals > Autoload* y añade este script:

    ```
    res://addons/func_godot/src/trenchdot/runtime/trenchdot_hot_reload.gd
    ```

2. Ejecuta el juego desde el editor de Godot

    El autoload solo escucha en juegos iniciados desde el editor de Godot, salvo que pongas su `enabled_in_exported_builds` a true.

3. Guarda en TrenchDot

    El juego reconstruye los mapas que usan el archivo guardado y emite `map_reloaded(map)`. Usa esa señal para devolver al jugador a un punto de aparición o reiniciar el estado del juego. La barra de estado debería terminar con "el juego en ejecución recargó 1 mapa(s)".

Escucha en el puerto 7843, que debe coincidir con *Recarga en caliente del juego en marcha* en las preferencias del editor. TrenchDot solo le envía los guardados mientras el Live Link está activado.
