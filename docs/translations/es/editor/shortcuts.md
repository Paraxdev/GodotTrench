---
source: c1ad5f7b4051d96e108a3133de3b683670efbf64
---

# Atajos { #shortcuts }

Estas son las teclas por defecto, el preajuste de TrenchBroom.

¿Quieres otras teclas? *Ayuda > Atajos de teclado…* lista cada asignación. Escribe en el filtro para encontrar una, pulsa **Asignar** y luego el nuevo atajo. **Preajuste** arriba de esa ventana, o *Archivo > Preferencias > Preajuste de atajos*, cambia a Hammer o a Blender, mira [Otros preajustes](#other-presets).

![La ventana Atajos de teclado filtrada por csg, con el desplegable Preajuste, Restablecer personalizaciones y un botón Asignar y Quitar en cada fila](../assets/shortcuts/keyboard-shortcuts-window.png)

Los atajos esperan mientras escribes en un campo o hay un menú abierto. Mientras el puntero está sobre una ventana flotante, como el Generador de formas, solo funcionan guardar, deshacer y rehacer, así que una tecla pensada para la ventana nunca borra ni cambia tu mapa.

## Herramientas { #tools }

Pulsa otra vez la tecla de una herramienta para volver a Seleccionar. Esc cancela un arrastre o un trazo, y si no, vuelve a Seleccionar, y en Seleccionar borra la selección.

| Tecla | Herramienta |
| --- | --- |
| Q | Seleccionar, dibujar brushes y mover cosas |
| C | Clip, para cortar brushes a lo largo de un plano |
| V | Vértice, para arrastrar las esquinas de los brushes |
| R | Girar |
| T | Escalar |
| Tab | Editar la selección donde está: mallas en la herramienta Malla, brushes en la herramienta Vértice. Nunca convierte un brush, eso lo hace *Malla > Editar malla* |
| Shift+T | Textura, para alinear texturas en las caras |
| P | Pintar, para pintar colores de vértice en las caras |
| G | Esculpir terreno y desplazamientos |
| Shift+G | Mezclar capas de terreno |
| B | Scatter |
| Shift+E | Volumen, para arrastrar un volumen trigger |
| Shift+P | Ruta, para colocar una cadena de entidades `path_corner` |
| M | Medir distancias |

## Ratón en las vistas { #mouse-in-the-views }

| Entrada | Resultado |
| --- | --- |
| Arrastrar fuera de la selección | Dibujar un brush |
| Arrastrar la selección | Mover. Ctrl copia, Shift fija el eje, Alt mueve en vertical en 3D |
| Clic, doble clic | Seleccionar, seleccionar el grupo entero |
| Clic otra vez enseguida en el mismo punto, o Alt+clic (2D) | Seleccionar el siguiente objeto bajo el cursor, como una pared bajo su techo en la vista Superior |
| Ctrl+clic | Añadir a la selección o quitar de ella, en las vistas y en el Outliner |
| Shift+clic en el Outliner | Seleccionar las filas desde la última en la que hiciste clic |
| Shift+clic, Ctrl+Shift+clic | Seleccionar una cara, añadir o quitar una cara |
| Shift+arrastrar una cara seleccionada (3D) | Empujarla o tirar de ella |
| Ctrl+Shift+arrastrar una cara seleccionada (3D) | Extruir un brush nuevo |
| Arrastrar con el botón derecho (3D) | Mirar alrededor. Mantenlo y usa WASD para volar, Q y E para bajar y subir |
| Arrastrar con el botón central | Desplazar |
| Alt+arrastrar con el botón izquierdo (3D) | Orbitar. En la herramienta Seleccionar, un arrastre que empieza en la selección o en su gizmo la mueve en vertical en su lugar |
| Ctrl+rueda (Esculpir, Mezcla, Pintar, Scatter) | Cambiar el tamaño del pincel |

## Edición { #editing }

| Tecla | Acción |
| --- | --- |
| Ctrl+Z, Ctrl+Shift+Z o Ctrl+Y | Deshacer, rehacer |
| Ctrl+X, Ctrl+C, Ctrl+V | Cortar, copiar, pegar bajo el ratón |
| Ctrl+D, Ctrl+Shift+D | Duplicar, duplicar enlazado |
| F2 | Renombrar el objeto seleccionado en el Outliner |
| Ctrl+R | Repetir el último giro, volteo, empujón o duplicado. En la edición de mallas hace un corte de bucle |
| Supr o Retroceso | Borrar |
| Ctrl+A, Esc, Ctrl+I | Seleccionar todo, nada, invertir |
| Ctrl+T, Ctrl+Shift+T | Usar los brushes seleccionados como plantilla y seleccionar lo que tocan, o lo que queda dentro de ellos. Los brushes de la plantilla se borran |
| Ctrl+G, Ctrl+Shift+G | Agrupar, desagrupar |
| Ctrl+H, Ctrl+J, Ctrl+Shift+H | Ocultar la selección, aislarla ocultando todo lo demás, mostrar todo |
| Flechas, Re Pág, Av Pág | Empujar un paso de la rejilla |

## Brushes, mallas y texturas { #brushes-meshes-and-textures }

| Tecla | Acción |
| --- | --- |
| Ctrl+K, Ctrl+M, Ctrl+L | Restar con CSG (tallar la selección de lo que toca), fusión convexa, intersección |
| Ctrl+Shift+K | Vaciar, convertir un brush sólido en una sala con paredes |
| Ctrl+Shift+B | Generador de formas, para arcos, cilindros, escaleras y otras formas ya hechas |
| Ctrl+Shift+E, Ctrl+Shift+J | Convertir brushes en malla, unir mallas |
| Ctrl+Shift+W | Mover brushes al mundo, sacándolos de su entidad para que pasen a ser geometría de nivel normal |
| Ctrl+Shift+U | Alternar el bloqueo UV, que mantiene las texturas pegadas a un brush mientras se mueve |
| Alt+H | Ajuste de hotspot, que ajusta las caras seleccionadas al rectángulo que mejor encaja de un trim sheet |

## Vista y archivos { #view-and-files }

| Tecla | Acción |
| --- | --- |
| F o Ctrl+U | Encuadrar la selección, o el mapa entero |
| `[` y `]`, o `-` y `=` | Reducir a la mitad o duplicar la rejilla |
| F4, F3 | Recorrer el sombreado (texturizado, plano, iluminado, alambre), alternar la Vista previa iluminada |
| Shift+Espacio | Maximizar la vista bajo el puntero |
| Ctrl+Shift+1 a 9, Ctrl+1 a 9 | Guardar, recuperar marcadores de cámara |
| Ctrl+=, Ctrl+-, Ctrl+0 | Escala de la interfaz mayor, menor, restablecer |
| F1 | Abrir el [manual](interface.md#help-and-the-manual) en la herramienta, la pestaña de panel, la entrada de menú o la sección del Inspector bajo el puntero, y en cualquier otro sitio la paleta de comandos |
| Ctrl+clic en una herramienta de la barra, Cmd+clic en macOS | Abrir su página del manual |
| Ctrl+Shift+P | Paleta de comandos |
| Ctrl+N, Ctrl+O | Mapa nuevo, abrir mapa |
| Ctrl+S, Ctrl+Shift+S | Guardar, guardar como |
| Ctrl+Shift+N, Ctrl+Tab, Ctrl+W | Pestaña nueva, pestaña siguiente, cerrar pestaña |
| F5 | Ejecutar el proyecto de Godot |

> [!NOTE]
> F2 antes recorría el sombreado. Ahora renombra, como en Godot, Blender y la mayoría de los gestores de archivos, y el sombreado pasó a F4. Una asignación que hayas fijado tú en la ventana de atajos se queda como estaba.

## Otros preajustes { #other-presets }

Un preajuste solo cambia las teclas de abajo, todo lo demás sigue como arriba. Tus propias asignaciones van encima de cualquier preajuste, **Restablecer personalizaciones** en la ventana de atajos las borra.

| Tecla | Preajuste de Hammer |
| --- | --- |
| Shift+S, Shift+X, Shift+V, Shift+A | Herramienta Seleccionar, clip, vértice, textura |
| H, Ctrl+H, U | Ocultar, aislar, mostrar todo |
| Ctrl+W | Mover brushes al mundo |
| Ctrl+M | Generador de formas |
| Ctrl+L | Girar 90° en torno al eje vertical |
| Ctrl+Shift+M, Ctrl+Shift+L | Fusión convexa CSG, intersección |
| Ctrl+F4 | Cerrar pestaña |

| Tecla | Preajuste de Blender |
| --- | --- |
| A, Alt+A | Seleccionar todo, nada |
| X | Borrar |
| Shift+D, Alt+D | Duplicar, duplicar enlazado |
| S | Herramienta Escalar |
| H, Alt+H, / | Ocultar, mostrar todo, aislar |
| Ctrl+J | Unir mallas |
| Z | Recorrer el sombreado |
| Shift+A | Generador de formas |
| F3 | Paleta de comandos |
| F4 | Vista previa iluminada |
| Alt+Shift+H | Ajuste de hotspot |
