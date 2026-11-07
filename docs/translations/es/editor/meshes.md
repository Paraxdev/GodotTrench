---
source: d32761c1e423392e984f1e0d2875aadc77d595ce
---

# Mallas { #meshes }

A diferencia de los brushes, las mallas pueden tener abolladuras, agujeros y caras curvadas, así que úsalas para rocas, tuberías, arcos suaves y otras formas que los brushes no pueden hacer.

## Editar una malla { #editing-a-mesh }

Selecciona los brushes que quieras remodelar y elige *Malla > Editar malla*. Los convierte en una malla y empieza a editar de inmediato.

![El menú Malla con Editar malla resaltado, y debajo Convertir brushes en malla (Ctrl+Shift+E), Convertir mallas en brushes, Unir mallas (Ctrl+Shift+J) y los submenús Topología, Componentes, Normales y sombreado y Transformar](../assets/meshes/menu-edit-mesh.png)

Con una malla seleccionada, **Tab** hace lo mismo, y otra vez Tab te devuelve a los objetos. Tab sobre brushes abre en cambio la herramienta Vértice, así que una pulsación accidental nunca convierte un brush en malla. El botón de la herramienta Malla en la barra de herramientas empieza a editar sin convertir nada. Ctrl+Shift+E solo convierte, y Ctrl+Shift+J une mallas en una sola.

Mientras editas, la barra de opciones de la herramienta arriba cambia entre **Vértice**, **Arista** y **Cara**, y tiene botones para las operaciones habituales.

![La barra de opciones de la herramienta Malla con Vértice, Arista y Cara, y después Subdividir, Fusionar, Rellenar, Eliminar, Suavizar y Plano](../assets/meshes/mesh-tool-options.png)

## Teclas { #keys }

Las teclas siguen a Blender. Por ejemplo, pulsa 3, haz clic en la cara superior de una caja y pulsa E, luego mueve el ratón hacia arriba y haz clic para extruirla:

![Una caja de malla en modo cara, con su cara superior extruida hacia arriba en pasos de rejilla](../assets/meshes/extrude-face.gif)

| Tecla | Acción |
| --- | --- |
| 1, 2, 3 | Trabajar con vértices, aristas o caras |
| A, Alt+A | Seleccionar todo, no seleccionar nada |
| Alt+clic, Ctrl+Alt+clic | Seleccionar el bucle de aristas que pasa por la arista pulsada, o el anillo que la cruza |
| Ctrl+L | Seleccionar todo lo que está conectado a la selección |
| G, R, S | Mover, rotar, escalar |
| E | Extruir geometría nueva a partir de la selección |
| I | Inset, una cara más pequeña dentro de cada cara seleccionada |
| Ctrl+B, Ctrl+Shift+B | Biselar aristas o vértices, cortando la esquina |
| Ctrl+R | Corte en bucle, un anillo nuevo de aristas a lo largo de la malla. La rueda o + y - fijan el número de cortes |
| K | Cuchillo, haz clic en dos puntos para cortar a lo largo de esa línea |
| M | Fusionar los vértices seleccionados en su centro |
| F | Rellenar, una cara nueva a través de los vértices seleccionados |
| X o Delete | Eliminar |
| Shift+D | Duplicar y mover |
| P | Separar la selección en una malla nueva |
| Alt+F | Invertir normales, para caras que se ven desde el lado equivocado |

Mientras mueves, rotas o escalas, pulsa X, Y o Z para bloquear ese eje, o Shift con la letra para excluirlo. Escribe un número para una cantidad exacta. Un clic o Enter confirma, el clic derecho o Escape cancela. La línea de ayuda en la parte inferior de la vista las enumera sobre la marcha.

## Gizmo { #gizmo }

Haz doble clic en un elemento para ponerle un gizmo de mover, rotar y escalar. Es la forma limpia de deslizar un vértice a lo largo de un eje, y también funciona en las vistas 2D. Escape lo oculta y mantiene la selección.

El menú *Malla* contiene el resto, como subdividir, triangular, solidificar, espejo y sombreado suave.
