---
source: 1cc5b4057e3a84e4d2bcf666b68e70eb94fb265b
---

# Editor UV { #uv-editor }

El Editor UV dibuja las caras seleccionadas de brushes y mallas en plano sobre su textura en mosaico, para que veas qué parte de la textura muestra cada cara. Úsalo para lo que la [herramienta Textura](texturing.md#the-texture-tool) no puede hacer en la vista 3D, como colocar las UV de una malla al píxel.

## Abrirlo { #opening-it }

El Editor UV es una pestaña del panel derecho, junto al Inspector, y *Textura > Editor UV* también lo abre. Después selecciona caras: Shift+clic en una en una vista, o usa la herramienta Textura. La línea de arriba nombra el material y cuántas caras tienes.

![El Editor UV con una cara de pared seleccionada, dibujada como un rectángulo naranja sobre la textura showcase/plaster en mosaico](../assets/uv-editor/uv-editor-face.png)

> [!TIP]
> Mientras el Editor UV es la pestaña visible de su panel, Ctrl+clic en un brush o una malla selecciona todas sus caras, así puedes texturizar cada lado a la vez. Los brushes y mallas que ya estaban seleccionados también se convierten en sus caras. Cuando la selección contiene otra cosa, como un terreno o una entidad, Ctrl+clic añade y quita objetos como siempre.

Una cara de brush normalmente saca su textura de un desplazamiento, una escala y una rotación. Una cara con **UV explícitas** guarda en cambio una posición de textura para cada esquina, que es lo que necesitan las mallas. El Editor UV dibuja esas caras en azul, y puedes elegir sus esquinas una a una. Las caras naranjas no tienen UV explícitas y se mueven enteras.

## Controles { #controls }

La **?** al final de la línea de arriba muestra esto también en el editor.

| Entrada | Resultado |
| --- | --- |
| Arrastrar el espacio vacío | Mueve la textura, o las caras enteras cuando tienen UV explícitas |
| Rueda, arrastre con el botón derecho | Escala, rota |
| Clic en una esquina | La selecciona junto con las esquinas de las caras vecinas cosidas a ella |
| Shift+clic, Ctrl+clic | Añade, quita |
| Alt+clic | Selecciona solo la esquina de esa cara, para separar una costura |
| Shift o Ctrl+arrastre en el espacio vacío | Selección por caja, añadiendo o quitando |
| Arrastrar una esquina seleccionada | Mueve toda la selección |
| Doble clic en una esquina | Gizmo que mueve solo a lo largo de U o V, Escape lo oculta |
| Arrastre con el botón central, Ctrl+rueda | Desplaza, acerca el lienzo |

Con esquinas seleccionadas, la rueda y el arrastre con el botón derecho las escalan y las rotan alrededor de su centro. Los campos U y V bajo el lienzo colocan la selección al píxel.

Los botones pequeños tras los menús activan el ajuste a píxeles, muestran los hotspots de la textura, hacen que un arrastre simple sea una selección por caja, muestran el gizmo, alternan el bloqueo UV y vuelven a encuadrar las caras. Pasa el cursor por uno para ver cuál es cuál.

## Proyecciones UV de mallas { #mesh-uv-projections }

Con caras de malla seleccionadas, el menú **Proyección** coloca UV explícitas por ti. Usa la densidad de texels que tendría una cara de brush con la misma textura, así las mallas encajan con los brushes que las rodean. Pasa el cursor por una entrada para ver una explicación de una línea.

![El menú Proyección del Editor UV abierto, con Cilindro Y bajo el cursor y su tooltip, Envuelve este eje del mundo por el centro de las caras, u gira a su alrededor](../assets/uv-editor/projection-menu.png)

| Comando | Resultado |
| --- | --- |
| *Planar*, *Planar (X)*, *(Y)*, *(Z)* | Proyecta en la dirección hacia la que miran sobre todo las caras, o a lo largo de un eje del mundo, manteniéndolas en una pieza |
| *Caja* | Proyecta cada cara desde su eje del mundo más cercano, para formas de bloques |
| *Cilindro X*, *Y*, *Z*, *Esfera* | Envuelven la textura alrededor de las caras, para tuberías, columnas y cúpulas |
| *Ver* | Proyecta recto a lo largo de la cámara 3D |
| *Desplegar* | Extiende las caras en plano arista por arista, así las tiras y las cuadrículas siguen siendo continuas |
| *Restablecer al mundo* | La proyección de una cara de brush recién restablecida |
| *Precalcular planar* | Convierte la textura planar en UV de esquina editables, se muestra mientras algunas caras aún no tienen |
| *Borrar UV* | Vuelve a quitar las UV explícitas |
| *Normalizar* | Escala las UV para que quepan una vez en la textura |
| *Empaquetar* | Empaqueta las islas en la textura con una sola densidad de texels |

Aquí una columna de 12 lados recibió *Cilindro Y*, y sus lados ahora van uno junto a otro como una sola tira alrededor de la textura:

![El Editor UV tras Cilindro Y en una columna de malla de 12 lados, con sus caras laterales dispuestas como una tira continua](../assets/uv-editor/uv-editor-cylinder.png)

El menú **UV** voltea, gira, ajusta, alinea, endereza y encaja a píxeles. El menú **Hotspot** ajusta a un rectángulo de hotspot, mira [Hotspots](texturing.md#hotspots).
