---
source: 4d58791b780474701e360cec324c41c44739f30d
---

# Isla en el mar 4: scatter { #sea-island-4-scatter }

El scatter coloca muchos árboles, arbustos, rocas o hierba con un pincel en lugar de uno a uno. Cada tipo de plantación de abajo es su propio conjunto de scatter, y un conjunto solo crece en sus destinos, las superficies que eliges para él.

## Prepara un conjunto { #set-up-a-set }

Todos los conjuntos de esta página empiezan igual.

1. Elige la herramienta Scatter

    Pulsa **B**, o haz clic en el botón Scatter de la barra de herramientas. La pestaña Scatter se abre a la derecha, junto al Inspector. (Si la cierras, **Panel scatter** en la barra de opciones de la herramienta la devuelve.)

    ![La barra de herramientas con el botón Herramienta scatter resaltado y su tooltip: arrastre para pintar los modelos del conjunto de scatter activo](../assets/sea-island/toolbar-scatter.png)

2. Empieza desde un preajuste

    Haz clic en **Preajuste** en la parte superior del panel Scatter y elige el preajuste que nombra el paso.

    ![El panel Scatter con el menú Preajuste abierto y bosque mixto resaltado](../assets/sea-island/scatter-preset-menu.png)

3. Apunta al terreno

    Haz clic en el cuentagotas junto a **Destinos**, luego haz clic en el terreno en la vista 3D. Deberías verlo listado bajo Destinos, como `terrain #2` o similar.

    ![La fila Destinos del panel Scatter con el cuentagotas resaltado y su tooltip](../assets/sea-island/scatter-eyedropper.png)

Deja **Seguir el cursor** sin marcar, ya que pinta sobre lo que haya bajo el pincel e ignora los destinos. Deja **Mantener espaciado con otros conjuntos** marcado, para que los conjuntos guarden la distancia entre sí y nada crezca a través de un tronco.

## Bosque { #woodland }

1. **Preajuste** > *bosque mixto*, luego haz clic en **Renombrar** arriba y llámalo `woodland`. Apunta al terreno.
2. En la sección Brush pon **Densidad** 0.16 y **Pendiente** de 0 a 30, y marca **Altura**, de 120 a 2000. El rango de pendiente mantiene los árboles lejos de los acantilados, y el de altura los mantiene lejos de la playa.

    ![La parte baja del panel Scatter: terrain #2 como destino, Seguir el cursor sin marcar, Densidad 0.16, Pendiente de 0 a 30, Altura marcada de 120 a 2000, Mantener espaciado con otros conjuntos marcado y el botón Rellenar objetivos resaltado](../assets/sea-island/scatter-woodland-settings.png)

3. Haz clic en **Rellenar objetivos** abajo del panel para plantar todo el terreno de una vez.

La densidad cuenta intentos por cada 64 x 64 unidades, así que 0.16 es más o menos un árbol cada 5 m.

![El bosque después de Rellenar objetivos](../assets/sea-island/scatter-woodland.jpg)

Mantén **Shift** y arrastra a lo largo de cada camino con radio 260 para borrar los árboles de ahí. Borra un claro en la cima de la colina principal (radio de unos 520) y un prado detrás de la playa (unos 700).

![Borrando árboles del camino](../assets/sea-island/scatter-erase-paths.jpg)

## Árboles destacados, arbustos y peñascos { #hero-trees-bushes-and-boulders }

Cada fila es un conjunto nuevo del preajuste indicado, con el terreno como destino igual que el bosque. Los árboles destacados son unos pocos árboles grandes y detallados colocados donde el jugador mira de cerca.

| Conjunto | Preajuste | Cómo colocarlo | Radio | Densidad | Pendiente | Altura |
| --- | --- | --- | --- | --- | --- | --- |
| Árboles destacados | bosque detallado | Clics sueltos donde los caminos salen de la playa y alrededor del claro de la cima | 260 | 0.08 | 0 a 30 | desactivada |
| Arbustos | arbustos | **Rellenar objetivos**, luego borra a lo largo de los caminos con radio 190 | | 0.35 | 0 a 32 | 45 a 200 |
| Peñascos | peñascos | Un trazo a lo largo de lo alto de los acantilados | 1100 | 0.035 | 12 a 50 | 20 a 2000 |

![Árboles destacados al pie del camino](../assets/sea-island/scatter-hero-trees.jpg)

![Arbustos en una franja sobre la arena](../assets/sea-island/scatter-bushes.jpg)

![Peñascos a lo largo de los acantilados](../assets/sea-island/scatter-boulders.jpg)

## Hierba { #grass }

1. Empieza un conjunto con el preajuste *hierba*. La hierba es follaje, se dibuja sin colisión ni sombras, así que puede ser densa sin ralentizar el juego.
2. Pinta los prados abiertos con radio 650, densidad 2.5, pendiente de 0 a 28, altura de 45 a 2000. Bórrala de los caminos con radio 170.

    ![Hierba del prado detrás de la playa](../assets/sea-island/scatter-meadow.jpg)

3. Para tener hierba sobre los peñascos, empieza un conjunto más de *hierba*. Con el cuentagotas, haz clic en un peñasco en lugar de en el terreno, para que el conjunto de peñascos pase a ser su destino. Deberías verlo bajo Destinos con `(set)` tras su nombre. Luego **Rellenar objetivos** con densidad 6 y pendiente de 0 a 45.

    ![La fila Destinos con cliff boulders (set) como destino](../assets/sea-island/scatter-target-boulders.png)

> [!NOTE]
> Rellenar coloca la hierba sobre los peñascos tal como están en ese momento, así que termina antes los peñascos.

![Hierba creciendo sobre un peñasco](../assets/sea-island/scatter-boulder-grass.jpg)

Siguiente: [Isla en el mar 5: muelle y Godot](sea-island-finish.md).
