---
source: 5b022e12202fe97edce0105dd2916cbd30e66e36
---

# Terreno { #terrain }

Un terreno es una gran rejilla de alturas que esculpes como arcilla, con hasta cuatro texturas pintadas encima. Úsalo para suelo abierto como colinas, playas y valles, donde dar forma a mano con brushes llevaría una eternidad.

Esta página cubre cómo crear uno y darle forma con la herramienta Esculpir (G). Texturizarlo está en [Capas de terreno](terrain-layers.md) y [Pintar terreno](terrain-painting.md).

## Crear un terreno { #creating-a-terrain }

1. Abre el diálogo

    Ve a *Terreno > Crear terreno…* en la barra de menús.

    ![El menú Terreno abierto con Crear terreno… bajo el cursor](../assets/terrain/menu-create-terrain.png)

2. Elige un tamaño

    **Resolución** y **Tamaño de celda** deciden lo grande que queda, y la fila **Tamaño** bajo ellos muestra el resultado en unidades y metros. Los valores por defecto, 129 x 129 vértices con celdas de 64 unidades, dan 8192 unidades, 256 m. Mira [Elegir el tamaño de celda](#picking-the-cell-size) más abajo.

    ![El diálogo Crear terreno con sus valores por defecto: 129 x 129, celdas de 64 u, Colinas y los cuatro campos de capa](../assets/terrain/create-terrain-dialog.png)

3. (Opcional) Elige una forma inicial

    **Forma** es el suelo del que partes. Por defecto es Colinas, las otras son Plano, Montaña, Isla, Valle y Crestas. **Importar PNG…** lee en cambio un mapa de alturas en escala de grises, donde los píxeles más claros son más altos. Ambos solo funcionan aquí, al crear.

4. (Opcional) Elige las capas

    **Capa base**, **Capa de pendiente**, **Capa de cima** y **Capa baja** son las cuatro texturas del terreno, y **Usar actual** pone el material que pulsaste por última vez. Con **Pintar capas según pendiente y altura** marcado se pintan de inmediato, mira [Pintar auto](terrain-painting.md#auto-paint).

5. Haz clic en **Crear** abajo del todo

    El terreno se centra en el cursor 3D, o en el **Centro** que escribas con **Cursor 3D** sin marcar, y esa altura pasa a ser el cero del terreno. Deberías ver el terreno nuevo seleccionado, con la herramienta Esculpir ya activa.

## Elegir el tamaño de celda { #picking-the-cell-size }

| Ajuste | Significado |
| --- | --- |
| Resolución | Cuántos vértices hay a lo largo de cada lado, de 33 a 1025. Más vértices guardan más detalle pero cuestan más de dibujar y esculpir |
| Tamaño de celda | La distancia en unidades de mapa entre dos vértices vecinos |

257 vértices con celdas de 64 unidades, por ejemplo, dan 16384 unidades, 512 m.

No hay nada entre dos vértices, así que el tamaño de celda es el detalle más fino que puede guardar el terreno. Una cordillera pide celdas grandes, un jardín celdas pequeñas.

¿Te equivocaste? Selecciona el terreno y abre **Remuestrear** en el Inspector. Reconstruye la rejilla con 65, 129, 257 o 513 vértices a lo largo del lado más largo y conserva las alturas, la pintura y los agujeros. El terreno mantiene su tamaño, así que más vértices significan celdas más pequeñas.

![El Inspector del terreno con el menú Remuestrear abierto y 257 vértices bajo el cursor](../assets/terrain/inspector-resample.png)

**Celdas por chunk** (32 por defecto) divide el terreno en piezas cuadradas para dibujar y para la colisión. Los chunks más pequeños se reconstruyen más rápido al esculpir, los más grandes significan menos nodos en Godot.

## Esculpir { #sculpting }

Pulsa G, elige un modo en la barra de opciones de herramienta arriba a la izquierda y arrastra sobre el terreno. Funciona sobre los terrenos seleccionados, o sobre todos cuando no hay nada seleccionado.

![El menú de modos de la herramienta Esculpir abierto en la barra de opciones, junto a los campos de radio y fuerza](../assets/terrain/sculpt-modes.png)

Mantén el botón del ratón pulsado y el suelo sigue subiendo bajo el pincel.

![Mantener Elevar con un pincel de 900 unidades levanta una colina en la isla](../assets/terrain/sculpt-raise.gif)

| Modo | Qué hace |
| --- | --- |
| Elevar, Bajar | Sube altura o la cava mientras mantengas el botón. Un clic hace un bultito |
| Suavizar | Relaja cada vértice hacia sus vecinos, para suavizar bultos y crestas afiladas |
| Aplanar | Lleva el suelo a la altura donde empezó el trazo, para hacer explanadas, caminos y mesetas |
| Ruido | Añade pequeños bultos aleatorios, para que el suelo plano parezca menos artificial |
| Aterrazar | Ajusta las alturas a escalones de **paso** unidades, para arrozales o acantilados escalonados. El campo aparece junto al modo |
| Pintar alfa, Borrar alfa, Pintar capa | Pintan en lugar de dar forma. Los modos alfa están pensados para displacements, Pintar capa pinta la capa de terreno fijada junto al modo. En un terreno la [herramienta Mezcla](terrain-painting.md) hace esto con más control |
| Agujero, Tapar agujero | Recorta celdas enteras o las vuelve a rellenar, colisión incluida. Úsalo para entradas de cuevas o búnkeres |

Ctrl+rueda cambia el tamaño del pincel, Shift intercambia elevar y bajar, y mantener Ctrl suaviza sea cual sea el modo.

La **fuerza** marca lo rápido que trabaja el pincel. Elevar y Bajar mueven el centro del pincel 8 unidades por segundo por punto de fuerza, sea cual sea el radio, así que el 4 por defecto levanta 32 unidades por segundo. Para colinas e islas sube la fuerza en lugar de mantener pulsado un minuto.

Mantén el radio por encima del tamaño de celda del terreno, un pincel más pequeño solo empuja el vértice que tiene debajo (la barra de estado te avisa cuando pasa).

Para empezar de cero, *Terreno > Esculpir > Aplanar*, o **Aplanar** en el Inspector, devuelve todo el terreno seleccionado a su altura cero.

> [!TIP]
> Da forma con un pincel grande y suaviza después de cada pasada brusca. Los pinceles pequeños dejan bultos.

## En Godot { #in-godot }

Cada terreno se convierte en un nodo `TrenchDotTerrain`, un `StaticBody3D` (el nodo de Godot para suelo sólido que nunca se mueve) cuya colisión coincide con lo que dibuja. Para el código de juego, `height_at(x, z)` devuelve la altura del suelo en una posición local, útil para colocar cosas en el suelo en tiempo de ejecución.

<div class="td-mcp" markdown>

## MCP { #mcp }

La operación `sculpt` de la herramienta `terrain_edit` aplica un solo toque, y su fuerza es cuánto mueve ese toque el centro, otra vez sea cual sea el radio. Mantener un trazo un segundo con fuerza 4 equivale a un toque de fuerza 32.

</div>
