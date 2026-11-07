---
source: 84383292fb022d6e6369e9ad8b643a67a07d4a67
---

# Si vienes de Unity o Blender { #coming-from-unity-or-blender }

TrenchDot es un editor de brushes en la línea de TrenchBroom y Hammer. Un nivel empieza como bloques sólidos que dibujas sobre una rejilla, no como mallas importadas. Casi todo lo que sabes de Unity y Blender sigue teniendo su sitio aquí, solo con otros nombres y otras teclas. Esta página los relaciona, y [Primeros pasos](../getting-started.md) te guía por una primera sala.

## Dónde está cada cosa { #where-things-are }

| En Unity o Blender | En TrenchDot |
| --- | --- |
| Hierarchy, Outliner | El **Outliner**, con [capas y grupos](organizing.md) en lugar de objetos padre |
| Inspector, panel N | El **Inspector**. Sus campos de posición y tamaño aceptan valores escritos, Enter los aplica |
| Vista de escena, viewport | La vista 3D más las vistas Superior, Frontal y Lateral. Dibujar y alinear es más fácil en 2D |
| Formas de ProBuilder, menú Add | Arrastra en cualquier vista para dibujar un [brush](brushes.md) caja, Ctrl+Shift+B para arcos, escaleras y cilindros |
| Modificador Boolean | *Brush > CSG*: restar (Ctrl+K), fusionar, intersecar y vaciar. Cambian los brushes al instante, no hay pila de modificadores |
| Modo de edición | La herramienta **Vértice** para brushes, la herramienta **Malla** para [mallas](meshes.md), ambas con Tab |
| Prefabs | [Prefabs](organizing.md#prefabs), guardados como su propio mapa y actualizados en cada instancia |
| Duplicado enlazado (Alt+D) | Duplicar enlazado, Ctrl+Shift+D, para que las copias compartan su geometría |
| Unidades, 1 m | 32 unidades de mapa son un metro, el paso de rejilla predeterminado es 16 |

Los números exactos van en el Inspector de la derecha. Selecciona algo, escribe una posición o un tamaño nuevos y pulsa Enter. La posición es la esquina más baja de la selección, no su centro ni un pivote.

![El Inspector con un brush seleccionado, su posición y tamaño como campos X, Y y Z sobre la sección Selección](../assets/coming-from-unity-blender/inspector-position-size.png)

## Teclas que cambian { #keys-that-differ }

Las teclas predeterminadas siguen a TrenchBroom, y tres de ellas pillan por sorpresa a quienes vienen de Blender:

* **G** es la herramienta Esculpir del terreno, no agarrar.
* **R** y **T** eligen las herramientas Girar y Escalar, que trabajan sobre la selección con tiradores en lugar de seguir al ratón.
* **Tab** edita la selección donde está.

Para mover algo, arrástralo en cualquier vista o usa el gizmo de la vista 3D, y usa el Inspector para los números exactos.

¿Prefieres conservar tus teclas de Blender? Cambia *Archivo > Preferencias > Preajuste de atajos* a Blender.

![El desplegable Preajuste de atajos en la ventana Preferencias, abierto con TrenchBroom, Hammer y Blender](../assets/coming-from-unity-blender/keymap-preset.png)

Añade A para seleccionar todo, X para eliminar, Shift+D para duplicar, H y Alt+H para ocultar y mostrar, S para la herramienta Escalar y Shift+A para el generador de formas. G sigue siendo Esculpir, y escribir un número durante un movimiento solo funciona dentro de la herramienta Malla. La lista completa está en [Otros preajustes](shortcuts.md#other-presets).

No hay vistas de teclado numérico. Las vistas Superior, Frontal y Lateral están siempre abiertas, Shift+Space maximiza la que está bajo el puntero y *Ver > Vistas* cambia entre una, dos y cuatro.

## Hábitos que se mantienen { #habits-that-carry-over }

En la herramienta Malla las teclas siguen a Blender: 1, 2 y 3 para vértices, aristas y caras, G, R y S con X, Y o Z para bloquear un eje, E para extruir, Ctrl+R para un corte en bucle y Ctrl+B para biselar.

Arrastrar con el botón derecho y WASD vuela la cámara 3D como la vista de escena de Unity, y F encuadra la selección. Ctrl+Z y Ctrl+Shift+Z deshacen y rehacen, y F2 renombra.
