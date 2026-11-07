---
source: 059c4546c8d47d8cdd4e33a900932ff4ff2b3833
---

# Colocar y cablear { #placing-and-wiring }

## Colocar entidades { #placing-entities }

Abre la pestaña **Entidades**, está junto al Inspector a la derecha. Cada clase que conoce tu proyecto tiene allí una tarjeta, agrupada por categoría. Pasa el cursor sobre una tarjeta para ver qué hace.

![El panel Entidades con la tarjeta func_door bajo el cursor, su tooltip dice entidad brush, puerta corredera](../assets/placing/entities-panel.png)

Arrastra una tarjeta a cualquier vista para colocarla, o haz doble clic en ella para soltarla en el cursor 3D. (Ctrl o Mayús y clic eligen varias tarjetas, arrástralas juntas y caen en fila.) La tarjeta de una entidad brush viene con una caja. Con brushes seleccionados, haz doble clic en ella para envolver esos brushes en la entidad.

La entidad cae en la primera superficie sólida bajo el puntero. Los volúmenes trigger, los brushes hechos solo de texturas de herramienta y otras entidades se atraviesan. Mientras arrastras, un marcador cian muestra dónde caerá. Una vista 2D no puede mostrar la profundidad, así que allí el marcador lleva la altura (o la profundidad) a la que cae. En la vista Arriba el techo de una sala estorba, así que mantén Alt mientras sueltas y la entidad atraviesa el techo y cae en el suelo de dentro. En las vistas Frontal y Lateral, Alt atraviesa la pared cercana y deja la entidad a mitad de la sala.

### Entidades brush { #brush-entities }

Las puertas, los botones y los triggers son entidades brush, toman su forma de brushes que construiste.

1. Construye la forma, por ejemplo una caja fina donde va la puerta, y selecciónala.
2. Haz clic derecho en una vista y elige una clase en *Crear entidad de brush*. *Brush > Entidad brush* tiene la misma lista.

![El menú contextual de una vista con Crear entidad de brush abierto y trigger_once bajo el cursor](../assets/placing/context-create-brush-entity.png)

Las más comunes tienen rutas más rápidas:

| Herramienta o menú | Qué crea |
| --- | --- |
| *Gameplay > Puertas y móviles* | Una puerta batiente o corredera con su trigger, o un elevador, a partir de los brushes seleccionados |
| *Gameplay > Triggers* | Un volumen trigger del tamaño de la selección |
| Herramienta **Volumen** (Mayús+E) | Dibuja arrastrando cualquier volumen trigger de una vez, sin necesidad de un brush antes |
| Herramienta **Ruta** (Mayús+P) | Clic a clic, una cadena de entidades `path_corner` que siguen los trenes y los caminantes, Intro termina |

![El menú Gameplay con Puertas y móviles abierto: puertas batientes, puertas correderas y Elevador, Plataforma móvil](../assets/placing/gameplay-doors-menu.png)

![La barra de herramientas con la herramienta Volumen bajo el cursor y su tooltip, Herramienta de volumen (Mayús+E), arrastra un volumen de trigger, aparición, daño, teletransporte o empuje](../assets/placing/toolbar-volume-tool.png)

### Tiradores { #handles }

Selecciona una puerta, una verja, una plataforma, un botón, un trigger de empuje, un spawner, una explosión o un prop de físicas y muestra tiradores en las vistas. Arrástralos para fijar su bisagra, hasta dónde se abre o recorre, su radio o su tamaño, en lugar de escribir números. El ángulo de una bisagra salta en pasos de 15 grados, mantén Mayús para grados sueltos.

![Arrastrando el tirador de ángulo de una puerta batiente en la vista Arriba, la etiqueta baja de 95° a 30°](../assets/placing/door-hinge-handle.gif)

## Cablear { #wiring }

Una conexión vive en la entidad que la activa. Selecciona esa entidad y mira **Salidas** al final del Inspector. **+ Añadir salida** añade una conexión, y luego rellenas salida, destino y entrada. Qué hace cada campo está en [Cómo funciona el I/O de entidades](io.md).

![La sección Salidas del Inspector para un botón: salida pressed, destino front_door, entrada open](../assets/io/inspector-outputs.png)

Para conectar dos entidades más rápido, selecciona las dos y elige *Gameplay > Lógica > Enlazar dos entidades seleccionadas…*. El Inspector también muestra un botón **Enlazar…** mientras haya exactamente dos entidades seleccionadas.

![El menú Gameplay con Lógica abierto y Enlazar dos entidades seleccionadas… bajo el cursor](../assets/placing/menu-link-two-entities.png)

La ventana *Vincular entidades* muestra el origen a la izquierda y el destino a la derecha. Elige la salida en *Cuando*, la entrada en *llamar*, y pulsa **Vincular**.

![La ventana Vincular entidades: func_button a la izquierda, light a la derecha, Cuando pressed, llamar turn_on, y el tooltip de Intercambiar](../assets/placing/link-dialog.png)

> [!WARNING]
> El origen no siempre es la entidad en la que hiciste clic primero. Comprueba los dos nombres de arriba, y pulsa el botón **Intercambiar** que hay entre ellos si están al revés.

El [grafo lógico](logic-graph.md) muestra todo el cableado a la vez y conecta entidades arrastrando de pin a pin.

## Comprobar el cableado sin Godot { #checking-wiring-without-godot }

Selecciona una entidad en el [grafo lógico](logic-graph.md#simulate), pulsa **Simular** y elige una salida. El grafo resalta cada paso `source.output → target.input` que seguiría y los lista con sus retardos, así que puedes seguir una cadena larga sin construir ni ejecutar el juego. El rojo marca un destino que no existe, normalmente una errata en un targetname. El azul marca un destino en tiempo de ejecución como `!player` o `@enemies`, que solo existe en el juego y no se sigue más allá.

## Código para cualquier entidad { #code-for-any-entity }

El panel **Referencia** muestra GDScript y C# de cualquier clase de entidad. Viene bien cuando el código del juego tiene que hablar con una entidad del mapa, por ejemplo para abrir una puerta o escuchar un botón. Ábrelo con *Ver > Paneles > Referencia*, con *Ayuda > Referencia de entidades y código*, o con **Referencia de código** en el Inspector, que salta directo a la clase de la entidad seleccionada. Arrastra el divisor entre la lista de clases y los detalles para cambiar el tamaño de la lista, haz doble clic en él para ajustarla a los nombres.

![El panel Referencia con func_button elegido: sus entradas y salidas, y el Uso desde GDScript](../assets/placing/reference-panel.png)

Elige una clase a la izquierda, y luego una pestaña. *Uso desde GDScript* y *Uso desde C#* muestran cómo llamar a la entidad, *Clase GDScript*, *Clase C#* y *FGD (.tres)* son esqueletos para escribir la tuya. **Copiar** pone el código en el portapapeles. En las pestañas de esqueleto **Crear en el proyecto** escribe el archivo en `res://entities/`, y deja en paz un archivo que ya exista.
