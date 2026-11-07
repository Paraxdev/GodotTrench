---
source: 8586c5cf175329bb5dcb3e351f21f472800d70ab
---

# El grafo lógico { #the-logic-graph }

El panel **Lógica** muestra el I/O de entidades del mapa como un grafo. Cada entidad cableada y cada entidad lógica es un nodo, y cada conexión es un cable desde un pin de salida a la derecha de un nodo hasta un pin de entrada a la izquierda de otro.

El grafo muestra las mismas salidas que edita el Inspector, así que un cambio en uno se ve en el otro al instante. El mapa, la construcción en Godot y el juego funcionan exactamente igual uses el grafo o no.

## Abrirlo { #opening-it }

Elige *Ver > Paneles > Lógica*.

![El menú Ver con Paneles abierto y Lógica bajo el cursor](../assets/logic-graph/menu-panels-logic.png)

Se abre junto a Materiales debajo de las vistas y agranda ese dock hasta más o menos la mitad del centro de la ventana, porque un grafo necesita espacio. Arrastra el divisor o la pestaña para cambiarlo.

![El panel Lógica del mapa de demostración Night District: botones cableados a puertas, un trigger y un temporizador, con cabeceras coloreadas por categoría, y un marco titulado Hall lights alrededor de un trigger que enciende tres luces con retardos](../assets/logic-graph/graph.png)

## Leer el grafo { #reading-the-graph }

| Lo que ves | Qué significa |
| --- | --- |
| Título del nodo | El targetname de la entidad, o si no su nombre de *Renombrar*, o si no su classname. Debajo están la categoría y el classname |
| Líneas grises bajo el título | Los ajustes que difieren de los valores por defecto de la entidad, como `max 3`, `wait 2 s` u `once`, recortados a dos líneas cortas. Un nodo sin ellas está en sus valores por defecto. El Inspector los muestra todos |
| Color de la cabecera | La categoría: el grupo de la definición, o si no la primera palabra del classname, así que `logic_relay` es *logic*. Mira [Colores](#colors) |
| Pines | Entradas a la izquierda, salidas a la derecha, tomadas de la [definición de entidad](entities/README.md) más cualquier nombre que el mapa ya use en esa entidad. Un pin lleno tiene un cable. Pasa el cursor sobre un pin para ver su tipo y los valores que pasa |
| Nombre de pin en amarillo | El nombre no está en la definición de la entidad, el panel Problemas también avisa de ello |
| Etiqueta del cable | El retardo, `once` o cuántas veces puede activarse, y el parámetro, cuando están fijados. Pasa el cursor sobre un cable para ver su salida, destino, entrada y retardo completos |
| Nodo y cable azules | Un [destino](parameters.md#targets) que solo resuelve el juego en ejecución, como `!player`, `@enemies`, una ruta de nodo o un comodín como `door_*` |
| Nodo rojo, cable rojo discontinuo | Un targetname que ninguna entidad tiene, normalmente una errata |

Solo las entidades cableadas y las entidades de lógica y de matemáticas tienen un nodo. Selecciona cualquier otra entidad, en una vista o en el Outliner, y también aparece, lista para cablearse.

## Colores { #colors }

Los colores ayudan a escanear un grafo grande, pero cada color viene también con un nombre o una forma, y las paletas se mantienen distintas para los tipos comunes de daltonismo.

| Categoría | Color |
| --- | --- |
| logic | azul |
| math | naranja |
| trigger | verde azulado |
| func, puertas, botones y móviles | bermellón |
| light | amarillo |
| env | turquesa |
| game | azul cielo |
| prop | rosa |
| npc | púrpura rojizo |
| info | gris |

| Tipo de pin | Color |
| --- | --- |
| pulso, un evento sin valor | blanco |
| bool | bermellón |
| int | azul cielo |
| float | verde azulado |
| string | rosa |
| vector3 | amarillo |
| color | púrpura rojizo |
| nodo, como el activador | azul |
| variant, cualquier valor | gris |

Los pines que pasan un valor son círculos, los pulsos son triángulos. Un cable toma el color de la salida de la que parte. Una categoría propia, fijada con el `group` de una definición, también recibe un color propio.

## Cablear { #wiring }

Arrastra de un pin de salida a un pin de entrada. Mientras arrastras, los pines a los que encaja el cable reciben un anillo, y el pin en el que acabaría se rellena. Suelta y la conexión se añade a las salidas de la entidad de origen.

![Arrastrando un cable desde la salida released de button_1 hasta la entrada turn_off de hall_light, los pines a los que encaja muestran anillos](../assets/logic-graph/wire-drag.gif)

Un pulso encaja en cualquier pin y un valor encaja en un pin de su propio tipo, así que un nodo `activator` no encaja en una entrada `int`. (Un int también encaja en un pin float o bool y al revés, y un variant encaja en todo.) Cualquier pin acepta aun así el cable, los nombres del I/O de entidades solo se comprueban cuando se ejecuta el juego.

Un destino sin targetname recibe uno en el mismo paso: el nombre que le diste con *Renombrar* en el Outliner, así que una luz que llamaste `lamp` se convierte en el destino `lamp`, o si no uno libre como `door_2`. Un nombre que otra entidad ya tiene recibe un número detrás.

También puedes soltar el cable en otro sitio:

* Sobre el título de un nodo, para elegir entre todas sus entradas o escribir cualquier nombre.
* Sobre un espacio vacío, para añadir allí una entidad lógica, ya cableada al pin del que arrastraste.

### Añadir entidades lógicas { #adding-logic-entities }

Haz clic derecho en un espacio vacío, o pulsa **Añadir** arriba a la izquierda del panel, para añadir una entidad lógica sin un cable.

![La lista Añadir: un campo de búsqueda, luego el grupo logic con logic_animate resaltado, logic_auto, logic_branch y más](../assets/logic-graph/add-menu.png)

Sin una búsqueda la lista se agrupa por categoría, primero lógica y matemáticas, luego otras entidades que tengan entradas o salidas. Escribe para buscar en todas: primero van las clases cuyo nombre empieza por el texto o lo contiene, y luego aquellas cuya categoría o descripción lo hace. Las flechas arriba y abajo mueven el resaltado e Intro añade esa clase.

Las entidades nuevas son entidades punto, y la construcción en Godot crea un nodo para cada una, así que necesitan un lugar en el mapa. Van a una capa llamada *Logic*, junto a la entidad a la que están cableadas, o delante de la cámara de la vista 3D. Cada una recibe de inmediato un targetname libre, como `relay_2`, que titula su nodo. Cámbialo en el Inspector, ya que *Renombrar* solo fija el nombre del Outliner.

> [!NOTE]
> Sin el paquete de entidades de gameplay no hay entidades lógicas que añadir. El menú Añadir ofrece entonces instalarlo.

### Editar una conexión { #editing-a-connection }

Haz clic en un cable y un cuadro arriba a la derecha del grafo te deja editar su salida, destino, entrada, parámetro, retardo y veces. Su botón **Eliminar**, o la tecla Supr con el puntero sobre el grafo, lo quita.

![Un cable seleccionado de button_1 a hall_light y el cuadro Conexión desde button_1 arriba a la derecha con salida released, destino hall_light, entrada turn_off, retardo 0.00 s, veces cada vez y Eliminar](../assets/logic-graph/connection-box.png)

Haz clic derecho en cualquier punto de un cable, no solo en su etiqueta, para **Editar conexión** y **Eliminar conexión**. Cada cambio es un paso en el *Historial*, así que deshacer funciona como en todas partes.

### El menú del nodo { #the-node-menu }

Haz clic derecho en un nodo para ver qué hacer con su entidad.

![El menú contextual del nodo hall_light: Encuadrar en las vistas, Renombrar, Duplicar, Seleccionar conectados, Desconectar todo, Eliminar entidad, y Simular con su salida switched](../assets/logic-graph/node-menu.png)

| Elemento | Qué hace |
| --- | --- |
| Encuadrar en las vistas | La selecciona y lleva las vistas hasta ella, como un doble clic |
| Renombrar | Renombra la entidad en el Outliner. El nodo se titula con el targetname cuando lo hay, así que el nombre solo se ve allí en una entidad sin él |
| Duplicar | Copia la entidad un paso de cuadrícula al lado, con los cables que parten de ella |
| Seleccionar conectados | Selecciona el nodo y cada nodo al que lo une un cable |
| Desconectar todo | Elimina de una vez todos los cables que entran o salen del nodo. Las entidades se quedan |
| Eliminar entidad | Quita la entidad del mapa, mira [Copiar, pegar y eliminar](#copy-paste-and-delete) |
| Simular | Activa una de sus salidas sobre el papel, mira [Simular](#simulate) |

Con varios nodos seleccionados, Duplicar y Eliminar actúan sobre todos ellos cuando haces clic derecho en uno de los seleccionados.

## Copiar, pegar y eliminar { #copy-paste-and-delete }

El grafo muestra entidades del mapa, así que estas teclas actúan sobre las entidades seleccionadas, brushes incluidos, mientras el puntero está sobre el grafo.

| Tecla | Qué hace |
| --- | --- |
| Ctrl+C, Ctrl+V | Copiar y pegar. La copia cae a un paso de cuadrícula del original, con sus cables, ya que no hay ninguna vista bajo el puntero donde ponerla. Pega con el puntero sobre una vista para colocarla allí |
| Ctrl+D | Duplicar, también a un paso de cuadrícula del original |
| Supr | Quita el cable o el marco seleccionado. Sin ninguno seleccionado, elimina las **entidades seleccionadas del mapa**, no solo sus nodos. Los cables de otras entidades hacia una eliminada se ponen rojos. Deshacer lo trae todo de vuelta |
| Ctrl+A | Selecciona todo el mapa, no solo los nodos mostrados |

## Moverse { #moving-around }

| Para | Haz esto |
| --- | --- |
| Zoom | Gira la rueda |
| Desplazar la vista | Arrastra con el botón central o derecho. Un clic derecho sin mover abre un menú, así que un desplazamiento tembloroso puede abrirlo, Esc lo cierra |
| Seleccionar | Haz clic en un nodo, Mayús o Ctrl y clic añade, arrastra sobre un espacio vacío para seleccionar con una caja. Las entidades también se seleccionan en las vistas y en el Outliner, y al revés |
| Encuadrar en las vistas | Doble clic en un nodo |
| Mover | Arrastra nodos. Sus posiciones se guardan en el mapa |
| Ordenar | **Ordenar** coloca los nodos seleccionados según el flujo de la señal. Con menos de dos seleccionados, cada nodo vuelve a la disposición automática |
| Verlo todo | **Ajustar**, o Inicio con el puntero sobre el grafo |
| Ver la selección | F con el puntero sobre el grafo encuadra los nodos seleccionados |

Los nodos que nadie movió siguen la disposición automática, de izquierda a derecha según fluyen las señales, y la vista hace zoom para ajustarse al panel hasta que haces zoom o desplazas tú mismo. Cuando mueves un nodo o cableas algo en el grafo, los demás conservan su sitio, y un nodo nuevo aparece junto a aquello a lo que está cableado. Los nodos marcadores azules y rojos no son entidades, así que no puedes moverlos. Se sitúan junto al nodo que los apunta.

## Marcos { #frames }

Un marco es una caja con título detrás de un grupo de nodos, para mantener organizado un grafo grande: las puertas de un edificio, un puzle, una cinemática. Selecciona los nodos y pulsa **C** sobre el grafo, o **Encuadrar** en la barra de herramientas del panel, y luego escribe su título. La caja *Hall lights* de la imagen de arriba es uno.

| Para | Haz esto |
| --- | --- |
| Mover un marco con todo lo de dentro | Arrastra su título |
| Cambiar su tamaño | Arrastra la esquina de abajo a la derecha |
| Renombrarlo | Doble clic en su título, o selecciónalo y pulsa F2 |
| Cambiarle el color o eliminarlo | Clic derecho en su título. Eliminar un marco conserva sus nodos |

Los marcos se guardan en el mapa como datos del editor, la construcción en Godot los ignora.

## Simular { #simulate }

Selecciona una entidad, pulsa **Simular** y elige una de sus salidas, o haz clic derecho en un nodo. El grafo numera en amarillo cada cable que desencadenaría la salida, y una lista bajo el grafo muestra cada paso con los títulos de los nodos, en el orden en que llegan los pasos, con el tiempo en segundos tras los retardos. **Borrar** termina la vista previa. Haz clic en un paso para seleccionar su cable.

![Simular en button_2: los cables hacia garage_door y de ahí a garage_beacon son amarillos y están numerados del 1 al 3, y la lista bajo el grafo nombra cada paso](../assets/logic-graph/simulate.png)

Sigue las reglas del tiempo de ejecución: los destinos se encuentran por targetname, y los relés, temporizadores, contadores, puertas lógicas, puertas y las demás entidades comunes pasan la cadena. Los destinos en tiempo de ejecución como `!player` se muestran pero no se siguen.

Una entidad que activa una de varias salidas muestra cada cable que podría desencadenar. Esos pasos, y todo lo que viene después, acaban en `(maybe)` en la lista y en `?` en el número del cable, y al pasar el cursor por uno dice qué espera. Un contador alimentado por `add` o `subtract` continúa hacia `hit_max` y `hit_min` de este modo, ya que la vista previa no cuenta las pulsaciones: el paso está ahí, y ocurre cuando el contador llega a su `max` o `min`. Un `logic_sequence` activa sus pasos en los tiempos de sus propiedades `interval`, `times` y `steps`, así que una cinemática se lee como una línea de tiempo.

Simular funciona sobre el papel, sin Godot. Lo que pasa de verdad en el juego se cubre en [Depurar el cableado](debugging.md).
