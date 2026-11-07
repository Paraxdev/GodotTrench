---
source: 7757565706801cf3511cdef72eaa79c5eb0dc3d0
---

# Pintar terreno { #terrain-painting }

La herramienta **Mezcla** (Shift+G) pinta las [capas](terrain-layers.md) de un terreno, por ejemplo arena a lo largo de una playa o roca en un acantilado. Como Esculpir, funciona sobre los terrenos seleccionados, o sobre todos cuando no hay nada seleccionado.

## La herramienta Mezcla { #the-blend-tool }

1. Elige la herramienta

    Pulsa Shift+G, o usa *Terreno > Mezcla > Herramienta de mezcla*. Sus opciones aparecen en la barra de opciones de herramienta arriba a la izquierda.

    ![Las opciones de la herramienta Mezcla en modo pendiente: modo, atenuación, capa, radio, fuerza y el rango de pendiente](../assets/terrain-painting/blend-toolbar.png)

2. Elige un modo

    El primer menú de la barra decide qué pinta el pincel, los modos se enumeran abajo.

    ![El menú de modos de la herramienta Mezcla abierto con pendiente bajo el cursor](../assets/terrain-painting/blend-modes.png)

3. Elige la capa y pinta

    Pon **capa** (de 0 a 3) en la capa que quieras y arrastra sobre el terreno. Varias pasadas suaves dan bordes más suaves que una sola pasada fuerte.

    ![Pintando un camino de tierra por la isla con la capa 2 en modo pintar](../assets/terrain-painting/blend-paint-path.gif)

| Modo | Qué pinta |
| --- | --- |
| pintar | Añade la capa elegida bajo el pincel |
| borrar | Vuelve a pintar hacia la capa 0, el suelo base |
| suavizar | Suaviza los bordes entre capas |
| nitidez | Deja que gane la capa más fuerte, para bordes nítidos |
| ruido | Solo en manchas con forma de nube del **tamaño** dado, para musgo, hierba seca o grava |
| pendiente | Solo donde el suelo está entre dos ángulos. `30` a `90` pone roca en todo lo más empinado que 30° |
| altura | Solo entre dos alturas del mundo, para arena en una línea de agua o nieve sobre el límite de los árboles |

Pendiente y altura hacen seguros los pinceles enormes, un pincel del tamaño de una colina en modo pendiente solo convierte los acantilados en roca.

El segundo menú de la barra es la atenuación, cómo se desvanece la fuerza desde el centro del pincel hasta su borde.

| Atenuación | Forma |
| --- | --- |
| suavizar | Un desvanecimiento suave, la elección habitual |
| lineal | Un desvanecimiento uniforme del centro al borde |
| constante | Todo el disco con la misma fuerza, para bordes duros |
| spray | Motas aleatorias, para guijarros y flores |

Shift convierte pintar, ruido, pendiente y altura en borrar, y Ctrl suaviza. Ctrl+rueda cambia el tamaño del pincel.

(Nota: **Fijar material de mezcla**, **repetición**, **desrepetir**, **nitidez** y **Aplicar mosaico**, más a la derecha en la barra, son para mezclar caras de brush, un terreno toma esos ajustes de sus capas.)

## Pintar auto { #auto-paint }

Pintar auto vuelve a pintar un terreno a partir de su forma, una primera pasada rápida antes de pintar a mano.

1. Selecciona el terreno

    Pintar auto solo toca los terrenos seleccionados.

2. Ejecútalo

    Ve a *Terreno > Mezcla > Pintar capas automáticamente*, o haz clic en **Pintar auto** en el Inspector.

    ![El menú Terreno > Mezcla con Pintar capas automáticamente bajo el cursor y Bandas desde el nivel del mar marcado en y 0](../assets/terrain-painting/menu-auto-paint.png)

| Capa | Va en |
| --- | --- |
| 1 | Pendientes más empinadas que unos 50° |
| 2 | El 20% superior del rango de altura |
| 3 | El 8% inferior del rango de altura |

La capa 0 cubre todo lo demás. Un terreno con menos capas recibe las bandas que faltan en su última capa.

El rango empieza en el punto más bajo, que en una isla es el fondo del mar. Marca **Bandas desde el nivel del mar** para medir en cambio desde una altura que des, como la línea de agua en y 0 de arriba.

![La isla del mar antes de Pintar auto, toda de hierba](../assets/terrain-painting/auto-paint-before.jpg)

![La misma isla después de Pintar auto: roca en los acantilados, tierra en la cima y arena a lo largo de las playas](../assets/terrain-painting/auto-paint-after.jpg)

> [!WARNING]
> Pintar auto reemplaza toda la pintura, así que esculpe primero, pinta auto, y pinta a mano al final.

## Qué tan estrecho puedes pintar { #how-narrow-you-can-paint }

La pintura vive en los vértices, así que un trazo necesita un radio de unas dos celdas para una banda limpia. Por debajo de una celda se convierte en una escalera. Para un sendero estrecho, usa celdas más pequeñas, o coloca encima una malla o un [decal](decals.md).

![Trazos con radio 40, 96 y 160 sobre celdas de 64 unidades, junto a la rejilla de vértices que guarda la pintura](../assets/terrain-painting/brush-width-grid.jpg)
