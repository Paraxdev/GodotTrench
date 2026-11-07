---
source: 6e5c9bc3787f689328791dca9df024446698b311
---

# Brushes y CSG { #brushes-and-csg }

Los brushes son con lo que construyes la mayor parte de un nivel. Se dibujan rápido, encajan limpiamente en la rejilla y en Godot se convierten en mallas con su colisión. Todo lo de aquí usa la herramienta **Seleccionar** (Q), salvo que una sección nombre otra.

## Dibujar y mover { #draw-and-move }

Arrastra en cualquier sitio fuera de la selección, en cualquier vista, y una caja sigue al ratón. Las vistas 2D imprimen su tamaño junto a ella. En la vista 3D empieza en la superficie bajo el cursor y tiene un paso de rejilla de grosor. En una vista 2D toma la profundidad del último brush que dibujaste, 128 unidades para el primero de todos.

![Las cuatro vistas mientras se dibuja una caja en 3D, la caja nueva mide 144 por 144 unidades y tiene un paso de rejilla, 16 unidades, de grosor](../assets/brushes/draw-box-3d.png)

Con un brush seleccionado puedes moverlo:

| Entrada | Resultado |
| --- | --- |
| Arrastrar la selección | La mueve, Shift la bloquea al eje principal |
| Ctrl+arrastrar la selección | Mueve una copia, el original se queda |
| Alt+arrastrar la selección (3D) | Mueve en vertical. Si empiezas fuera de la selección, Alt+arrastrar orbita la cámara |
| Teclas de flecha, Page Up, Page Down | Empujan un paso de rejilla |
| `[` y `]` | Reducen a la mitad y duplican la rejilla |

En 3D la selección también recibe un gizmo de mover, girar y escalar. La rotación encaja a 15°, o a 1° con Shift. Si te estorba, desactívalo en *Ver > Gizmo de transformación*.

## Posición y tamaño exactos { #exact-position-and-size }

A veces quieres números exactos, como un cortador de puerta justo tan profundo como la pared. Selecciona el brush y escríbelos en el **Inspector** de la derecha.

![El Inspector con un brush seleccionado, con sus campos de posición y tamaño](../assets/brushes/inspector-position-size.png)

La posición es la esquina más baja, la X, Y y Z más pequeñas, que es la esquina que suele estar sobre la rejilla. Un tamaño nuevo hace crecer la selección alejándose de esa esquina, así que la posición se queda donde está. Arrastrar un campo avanza por la rejilla mientras el ajuste está activado, y un valor escrito se conserva exacto. Los mismos campos aparecen para mallas, una entidad brush y una selección de varios objetos.

## Remodelar { #reshape }

Puedes remodelar un brush seleccionado sin salir de la herramienta Seleccionar:

| Entrada | Resultado |
| --- | --- |
| Shift+arrastrar una cara seleccionada (3D) | Empuja o tira de esa cara |
| Ctrl+Shift+arrastrar una cara seleccionada (3D) | Extruye un brush nuevo a partir de ella |
| Arrastrar un borde de la selección (2D) | Redimensiona. El borde se ilumina en amarillo y el cursor cambia donde un arrastre redimensiona |

![Shift+arrastrar la cara lateral de una caja seleccionada en la vista 3D, la caja se estira en pasos de rejilla](../assets/brushes/push-face.gif)

Para pendientes, rampas y paredes inclinadas, pulsa C para la herramienta **Clip**:

1. Haz clic en dos o tres puntos a lo largo del corte. Encajan en la rejilla, y en 3D caen sobre la superficie en la que haces clic.
2. Pulsa Tab para elegir qué conservar, la parte delantera, la trasera o ambas mitades. El texto en la parte inferior de la vista te dice cuál está activa.
3. Pulsa Enter para cortar. Backspace quita el último punto si te equivocaste al hacer clic.

![La herramienta Clip con dos puntos colocados en la vista Frontal a través de la esquina superior de una caja, la línea de corte también se ve en la vista 3D](../assets/brushes/clip-points.png)

Para arrastrar esquinas sueltas, pulsa V para la herramienta **Vértice**, o Tab con brushes seleccionados. Muestra tiradores en las esquinas, y puntos azules pequeños en los puntos medios de las aristas y en los centros de las caras. Haz clic en un punto para seleccionarlo, o arrástralo lejos del brush para añadir una esquina ahí. Delete elimina los vértices seleccionados.

![La herramienta Vértice sobre una caja cortada, tiradores amarillos en las esquinas y puntos azules en las aristas y las caras](../assets/brushes/vertex-handles.png)

> [!NOTE]
> Un brush no puede tener abolladuras, así que se rechaza un movimiento que la haría. Ves un fantasma rojo donde caería y una línea en la barra de estado. Tira del punto hacia fuera en su lugar, o convierte el brush en [malla](meshes.md) para abolladuras y voladizos.

## Girar, escalar y medir { #rotate-scale-and-measure }

| Herramienta | Entrada | Resultado |
| --- | --- | --- |
| **Girar** (R) | Arrastra un anillo alrededor de la selección | Gira en pasos de 15°, con Shift en pasos de 1°. Una vista 2D tiene un anillo, alrededor del eje por el que mira |
| **Escalar** (T) | Arrastra un tirador en los límites de la selección | Redimensiona sobre la rejilla, con Alt por ambos lados a la vez para que el centro se quede quieto. La vista 3D tiene un tirador en cada lado |
| **Medir** (M) | Arrastra, o haz clic en dos puntos | Muestra la distancia en unidades y metros, y abajo en la vista sus partes X, Y, Z y horizontal |

![La herramienta Medir en la vista 3D, una línea rotulada 230,76 u (7,21 m) y las partes X, Y y Z abajo](../assets/brushes/measure.png)

## Generador de formas { #shape-generator }

Para cilindros, arcos, escaleras, tejados y otras formas que llevarían muchos brushes dibujados a mano, usa el Generador de formas.

1. Dibuja una caja donde debe ir la forma y déjala seleccionada.
2. Abre *Brush > Generador de formas…* (Ctrl+Shift+B).
3. Elige la **Forma** y sus opciones. El diálogo indica los límites que rellena, y que reemplaza la selección.
4. Haz clic en **Crear**.

![El diálogo Generador de formas con Cilindro, 16 lados y el eje Y, rellenando una selección de 128 x 192 x 128](../assets/brushes/shape-generator.png)

![La caja seleccionada a la izquierda, el cilindro de 16 lados que la reemplazó a la derecha](../assets/brushes/shape-generator-box-to-cylinder.png)

Los cilindros y los conos están de pie por defecto. Elige **X** o **Z** en **Eje** para tumbar uno dentro de la misma caja, para tuberías, troncos o una viga, en lugar de girarlo después. El toro, el muro con arco, el tejado a dos aguas, la aguja y la cuadrícula se hacen siempre como [malla](meshes.md). Los cilindros, los conos y las esferas también pueden serlo con **Crear como malla editable**, y entonces **Sombreado suave** los redondea. La forma **Texto** construye letras de bloque con brushes, para carteles y rótulos en el suelo.

## CSG { #csg }

Las operaciones CSG están en *Brush > CSG*, que también contiene sus dos ajustes.

![El menú Brush con el submenú CSG abierto, con Restar, Fusión convexa, Intersecar y Vaciar y sus atajos, la casilla Las caras recortadas conservan el material del objetivo y el campo Grosor de pared](../assets/brushes/menu-brush-csg.png)

Las cuatro también tienen un botón en la barra de herramientas, a la derecha de los botones de ajuste y de bloqueo UV.

![Los cuatro botones CSG en la barra de herramientas, con Fusión convexa resaltado y su tooltip](../assets/brushes/toolbar-csg.png)

| Operación | Tecla | Resultado |
| --- | --- | --- |
| Restar | Ctrl+K | Recorta la selección de todos los brushes que toca y luego elimina la selección. Una selección que no toca ningún otro brush se conserva y la barra de estado lo dice |
| Vaciar | Ctrl+Shift+K | Convierte un brush en paredes alrededor de su volumen anterior, de 16 unidades de grosor salvo que cambies *Grosor de pared* en el mismo menú. Un brush demasiado fino para dos paredes se deja como está |
| Fusión convexa | Ctrl+M | Funde la selección en un único brush convexo que la envuelve |
| Intersecar | Ctrl+L | Conserva solo donde se solapan los brushes seleccionados |

Para abrir una puerta o una ventana, coloca un brush donde va el hueco y réstalo, [Primeros pasos](../getting-started.md) te guía por uno. Las caras que abre un corte toman el material del cortador, salvo que marques *Las caras recortadas conservan el material del objetivo* en el mismo menú.

## Displacements { #displacements }

Para suelo irregular dentro de un nivel de brushes, como una pendiente de escombros, puedes convertir una cara en un displacement y esculpirla.

1. Haz Shift+clic en una cara de cuatro lados, o selecciona el brush para usar su cara superior.
2. Abre *Brush > Displacement* y elige una de las entradas **Crear**. Una potencia mayor da una rejilla más fina.
3. La herramienta **Esculpir** (G) se activa sola, arrastra sobre la cara para darle forma.

![El menú Brush con el submenú Displacement abierto, Crear, potencia 2 (5x5), Crear, potencia 3 (9x9) y Crear, potencia 4 (17x17), luego Coser displacements y Quitar displacement](../assets/brushes/menu-brush-displacement.png)

Para zonas exteriores más grandes funciona mejor un [terreno](terrain.md).
