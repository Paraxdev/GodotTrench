---
source: be4d0912557053b84612829a7948190773c48b4d
---

# Isla en el mar 1: el terreno { #sea-island-1-terrain }

Una isla grande con dos colinas, acantilados, una playa, caminos, bosque y un muelle, construida en cinco partes. Los números son los usados en las imágenes, en unidades de mapa a 32 por metro.

![La isla terminada vista desde el sur](../assets/sea-island/island-finished.jpg)

## Crea el terreno { #create-the-terrain }

Ve a *Terreno > Crear terreno…* en la barra de menús.

![El menú Terreno con Crear terreno… resaltado](../assets/sea-island/menu-create-terrain.png)

Rellena el cuadro así:

| Ajuste | Valor y por qué |
| --- | --- |
| Resolución, Tamaño de celda | 257 x 257 puntos, a 64 unidades uno de otro. La fila Tamaño de abajo muestra lo que obtienes, 16384 unidades (512 m) |
| Forma | Isla, Altura 1600, Tamaño de rasgos 3000, Erosión 20, Semilla 1. La isla sube en el centro y baja hacia los bordes. El tamaño de rasgos es el ancho de los bultos más grandes, y la erosión deja que las pendientes empinadas se desmoronen para que parezcan desgastadas |
| Capa base, Capa de pendiente, Capa de cima, Capa baja | `showcase/grass`, `showcase/cliff`, `showcase/dirt`, `showcase/sand`. Mosaico 256 para las dos primeras, 192 para las demás. La base cubre todo lo que no pintes, las otras sirven para terreno empinado, alto y bajo |
| Centro | Desmarca **Cursor 3D**, luego `0 -576 0` |
| Pintar capas según pendiente y altura | Desmarcado, la parte 3 pinta a mano |

(El valor de mosaico es cuántas unidades cubre una repetición de la textura.)

![El cuadro Crear terreno rellenado: 257 x 257, tamaño de celda 64, Isla, altura 1600, tamaño de rasgos 3000, erosión 20, semilla 1, las cuatro capas showcase, centro 0 -576 0 y Pintar capas según pendiente y altura desmarcado](../assets/sea-island/create-terrain-dialog.png)

Pulsa **Crear** abajo del todo. Deberías ver aparecer una isla verde, ya seleccionada, y el editor cambia solo a la herramienta Esculpir.

El mar estará a altura 0, así que bajar el terreno 576 inunda su anillo exterior y deja seco el centro. Las partes siguientes se refieren a las capas por su número de ranura: 0 hierba, 1 acantilado, 2 tierra, 3 arena.

## Añade el mar { #add-the-sea }

El mar es un único brush plano en el que el jugador puede meterse.

1. Dibuja una caja enorme

    Pulsa **Q** para la herramienta Seleccionar y **Escape** para que el terreno deje de estar seleccionado, si no tu arrastre lo esculpe o lo mueve. Luego arrastra en la vista superior para dibujar una caja muchas veces mayor que el terreno, o su borde se verá en el horizonte.

2. Aplánala

    En la vista frontal, dale 32 unidades de grosor con su cara superior a altura 0.

3. Conviértela en agua

    Con la caja seleccionada, haz clic en `showcase/water` en la pestaña Materiales. Luego Shift+clic en el agua en la vista 3D para seleccionar solo su cara superior, y pon *escala* en 8 y 8 en la sección *Alineación* del Inspector. Así la textura del agua no se repite cada pocos metros.

    ![El Inspector con 1 cara seleccionada, material showcase/water y la escala de Alineación en 8.00 y 8.00](../assets/sea-island/inspector-alignment-scale.png)

4. Deja que el jugador se meta

    Haz clic una vez en el agua, lejos de la isla, para que vuelva a estar seleccionada toda la caja y no solo su cara superior. Luego haz clic derecho y elige *Crear entidad de brush > func_illusionary*. Se dibuja sin colisión, así que el jugador puede entrar en el agua.

Ahora deberías tener una isla en el mar, lista para esculpir:

![La isla generada en el mar, antes de esculpir](../assets/sea-island/fresh-island.jpg)

Siguiente: [Isla en el mar 2: esculpir](sea-island-sculpt.md).

<div class="td-mcp" markdown>

## MCP { #mcp }

El [servidor MCP](../mcp.md) deja que agentes y scripts controlen el editor.
[examples/mcp/sea_island.json](https://github.com/Paraxdev/TrenchDot/blob/main/examples/mcp/sea_island.json) es un script MCP que construye la misma isla, así que puedes reproducirlo para comprobar un valor.

</div>
