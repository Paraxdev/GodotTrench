---
source: fb9984bb419921807428d1c050d90fa918ed7b18
---

# Texturizado { #texturing }

## Aplicar materiales { #applying-materials }

Tus texturas viven en el panel **Materiales**, en la parte inferior de la ventana bajo las vistas. El cuadro de búsqueda arriba a la izquierda las filtra por nombre, **Usados en el mapa** y **Favoritos** acortan la lista, y el control deslizante fija el tamaño de las miniaturas.

![El panel Materiales con el cuadro de búsqueda, la lista de carpetas, Usados en el mapa, Favoritos y el control de tamaño, y debajo las miniaturas de materiales con dev/blue resaltado mostrando su nombre y tamaño](../assets/texturing/materials-panel.png)

| Haz esto | Resultado |
| --- | --- |
| Clic en una miniatura | La aplica a la selección y la usa para los brushes nuevos |
| Arrastrar una miniatura sobre una cara | La aplica a esa cara |
| Shift al soltar | La aplica a todo el brush o malla |
| Alt al soltar | La coloca como [decal](decals.md) |

Mientras arrastras, una tarjetita junto al cursor te recuerda los modificadores, y la cara cambia en cuanto sueltas.

![Arrastrando withered/mossy_plaster sobre el frente de una pared de ladrillo, y después la misma pared con el enlucido solo en su cara frontal](../assets/texturing/drop-on-face.png)

Para texturizar una cara sin arrastrar, haz antes Shift+clic en ella en una vista y luego clic en la miniatura. Ctrl+Shift+clic añade o quita más caras.

Haz clic derecho en una miniatura para más opciones: añadirla a tus favoritos, seleccionar las caras que la usan, ponerla en lugar del material actual en todo el mapa, abrir el editor de hotspots, o *Mostrar en el gestor de archivos* para abrir la carpeta de su textura con el archivo seleccionado, por ejemplo para editarlo.

![El menú del clic derecho de una miniatura con Aplicar a la selección, Añadir a favoritos, Seleccionar caras que lo usan, Reemplazar showcase/red_bricks por él en el mapa, Editor de hotspots, Copiar nombre y Mostrar en el gestor de archivos](../assets/texturing/material-context-menu.png)

Soltar un material sobre un terreno fija una capa del terreno en su lugar, mira [Capas de terreno](terrain-layers.md).

El bloqueo UV (Ctrl+Shift+U) viene activado por defecto, así que las texturas siguen unidas a un brush mientras lo mueves.

## Colores incorporados { #built-in-colors }

El panel Materiales siempre ofrece `dev/grey`, `dev/dark`, `dev/orange`, `dev/blue` y `dev/green`, texturas de rejilla para esbozar un nivel antes de que tenga arte. Codificar las salas por colores con ellas sobrevive a la construcción, porque el addon incluye las mismas rejillas y Godot las usa cuando el proyecto no tiene ninguna textura con ese nombre. Una textura tuya en, por ejemplo, `res://textures/dev/grey.png` sustituye a una.

## Texturas de herramienta { #tool-textures }

El editor dibuja las texturas de herramienta translúcidas y con rayas, y la construcción de Godot no les da malla visible. Cuenta cualquier textura de la carpeta `special/`, `tools/` o `td/`, y también cualquiera que lleve el nombre de una de las herramientas de abajo, como `common/caulk`.

![Un bloque dev/dark, un muro morado translúcido special/clip delante y una caja naranja a rayas special/trigger](../assets/texturing/tool-textures.png)

| Textura | En Godot |
| --- | --- |
| `special/clip`, `playerclip`, `nodraw`, `caulk`, `trigger`, `sky` y cualquier otra herramienta | Colisiona pero no dibuja nada. Un trigger conserva su forma de trigger |
| `special/skip`, `origin`, `hint`, `hintskip`, `null`, `areaportal` | No construye nada. Un brush con solo estas caras no colisiona |
| `special/occluder` | No dibuja nada y no oculta a los vecinos, pero suma al oclusor de una entidad que construye uno, como `func_geo` |

Los ajustes del mapa nombran las texturas de clip, skip, origin y sky, `special/clip` y así sucesivamente por defecto. Un brush origin solo fija el origen de su entidad con la textura que nombran los ajustes del mapa.

## La herramienta Textura { #the-texture-tool }

La alineación de una cara es dónde se sitúa su textura sobre ella: el desplazamiento, la escala y la rotación. Pulsa **Shift+T** para la herramienta Textura, que la edita en el sitio, como el modo de edición de caras de Hammer. Haz clic en una cara para seleccionarla. Se tiñe de rojo y muestra una caja azul alrededor de una repetición de la textura, además de dos líneas cortas para las direcciones U y V. La cara bajo el cursor recibe un contorno amarillo.

![La herramienta Textura con la cara frontal de una pared de ladrillo seleccionada, una caja azul marca una repetición de la textura](../assets/texturing/texture-tool-face.png)

| Entrada | Resultado |
| --- | --- |
| Clic, Ctrl+clic | El clic selecciona una cara, Ctrl+clic añade o quita una |
| Arrastrar una cara seleccionada | Desliza la textura, el píxel bajo el cursor se queda bajo él |
| Teclas de flecha | Deslizan el tamaño de la rejilla en píxeles, con Shift un píxel |
| Ctrl+rueda | Escala por 2, con Shift en pasos finos |
| Alt+rueda | Gira 15°, con Shift 1° |
| Alt+clic | Recoge el material y la alineación de una cara |
| Clic derecho | Aplica el material actual |
| Ctrl+clic derecho | Lo aplica y restablece la alineación |
| Shift+clic derecho | Pega el material y la alineación recogidos |
| Alt+clic derecho | Aplica el material actual, continuando la alineación de la cara seleccionada a través de la arista |

Alt+clic derecho es la forma de que una textura, como un trim o una hilera de ladrillos, rodee limpiamente una esquina.

La barra de opciones de la herramienta arriba tiene los botones de **Justificar**. **Izquierda**, **Derecha**, **Arriba**, **Abajo** y **Centro** empujan la textura a ese lado de la cara, **Ajustar** la estira sobre la cara, **Ajustar An.** y **Ajustar Al.** solo a lo ancho o solo a lo alto. Con **Tratar como uno** marcado actúan sobre todas las caras seleccionadas como una sola área, así que una textura las abarca todas. **Alinear con la vista** proyecta la textura a lo largo de la cámara 3D.

![La barra de opciones de la herramienta Textura con Justificar Izquierda, Derecha, Arriba, Abajo, Centro, Ajustar, Ajustar An., Ajustar Al., Tratar como uno, Alinear con la vista y Ajuste de hotspot](../assets/texturing/texture-tool-options.png)

Si prefieres números, el **Inspector** muestra la misma cara bajo **Alineación**, con campos de desplazamiento, escala y rotación y botones para voltearla, girarla y restablecerla. **Herramientas de textura** debajo fija la densidad de texels y copia la alineación de una cara a otra.

![El Inspector con una cara seleccionada, la sección Alineación con desplazamiento, escala y rotación, los botones de Justificar y la sección Herramientas de textura](../assets/texturing/inspector-alignment.png)

Para caras con UV explícitas, y para mallas, usa el [Editor UV](uv-editor.md).

## Escala de textura { #texture-scale }

Por defecto un píxel de textura cubre una unidad de mapa, así que una textura de 64 píxeles se repite cada dos metros. Eso va bien para texturas de baja resolución, pero una foto de 1024 píxeles se estiraría sobre 32 metros. Pon `metadata/texture_size` en el material con el tamaño de mundo que debe cubrir una repetición, mira [Tamaño de textura](../godot/materials.md#texture-size). La línea del Inspector bajo el nombre del material te dice qué recibió una cara, como "1024 x 1024 px, 64 x 64 unidades por repetición".

Para fijarlo solo en un mapa, o para que una textura corra sin costuras por todas las caras, usa los [Ajustes de textura](light-baking.md#texture-settings) en el Inspector. Su tamaño de repetición gana al del material.

## Hotspots { #hotspots }

Los hotspots dejan que una sola textura trim sheet texturice muchas caras de formas distintas. Marcas sus rectángulos una vez, y luego TrenchDot elige el correcto para cada cara.

1. Haz clic derecho en la trim sheet en el panel Materiales y elige **Editor de hotspots**. *Textura > Editor de hotspots…* también lo abre, para el material de la cara seleccionada o, si no, el actual.

    ![El menú Textura con Editor de hotspots… resaltado, Ajuste de hotspot (Alt+H) abajo](../assets/texturing/menu-texture.png)

2. Arrastra sobre la textura para dibujar un rectángulo, o fija una cuadrícula y haz clic en **Dividir en cuadrícula**. Arrastra un rectángulo para moverlo, su tirador de la esquina para redimensionarlo, y el clic derecho lo elimina.
3. Haz clic en **Guardar**. Los rectángulos van a `<texture>.hotspots.json` junto al archivo de la textura.
4. Selecciona las caras y usa *Textura > Ajuste de hotspot* (Alt+H), o **Aplicar a la selección** en el editor.

![El Editor de hotspots sobre withered/concrete_panels dividido en una cuadrícula de 4 por 4, 16 rectángulos, sin guardar](../assets/texturing/hotspot-editor.png)

El ajuste de hotspot da a cada cara el rectángulo cuya forma mejor se le parece, prefiriendo uno que mantenga la textura cerca de su escala normal.

## Pintura de vértices { #vertex-paint }

La herramienta **Pintar** (P) pinta colores de vértice en las caras de los brushes, para suciedad, tintes e iluminación falsa. Elige el color en la barra de opciones de la herramienta, junto al radio y la fuerza del pincel, y arrastra sobre las caras. Pinta los brushes seleccionados, o todos los brushes cuando no hay nada seleccionado. Ctrl+rueda cambia el tamaño del pincel.

![La barra de opciones de la herramienta Pintar con la muestra de Color, radio 48 u y fuerza 4.0](../assets/texturing/paint-tool-options.png)
