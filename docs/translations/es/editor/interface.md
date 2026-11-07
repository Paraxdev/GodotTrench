---
source: 7982e4d62890afb0393dc32addda8d5b4d33f84a
---

# Interfaz y navegación { #interface-and-navigation }

TrenchDot sigue a TrenchBroom y Hammer, así que si has usado cualquiera de los dos, casi todos tus hábitos valen. Si no, esta página te da una vuelta por el editor, y [Si vienes de Unity o Blender](coming-from-unity-blender.md) relaciona las ideas que ya conoces.

## La ventana { #the-window }

![El editor con una sala pequeña: el Outliner a la izquierda, las vistas 3D, Superior, Frontal y Lateral en el centro, el Inspector a la derecha y el panel Materiales abajo](../assets/interface/window.png)

Los menús están arriba. Un comando está bajo el tipo de objeto que cambia, así que CSG está en *Brush* y el esculpido de terreno en *Terreno*.

¿No encuentras algo? Pulsa Ctrl+Shift+P (o F1) para la paleta de comandos y escribe su nombre. Los comandos con un nombre técnico, como *Vaciar* o *Bloqueo UV*, se explican solos al pasar el puntero por encima, y la paleta muestra la misma línea debajo de su lista. Escape o un clic fuera la cierra sin ejecutar nada.

![La paleta de comandos con vaciar escrito, Brush: Vaciar elegido con su atajo, y debajo la línea que lo explica](../assets/interface/command-palette.png)

Justo bajo la barra de herramientas está la **barra de opciones de la herramienta**. Contiene los ajustes de la herramienta activa, o una línea sobre cómo usar una herramienta que no tiene ninguno. Pasa el puntero por el nombre de la herramienta ahí, o por su botón en la barra de herramientas, para ver una descripción corta.

![La barra de opciones de la herramienta con la herramienta Seleccionar resaltada, su tooltip describe la herramienta y termina en F1: abrir manual](../assets/interface/tool-options-tooltip.png)

### La tarjeta del mapa vacío { #the-empty-map-card }

En un mapa nuevo y vacío, una tarjeta en la vista 3D te dice por dónde empezar: arrastra en la vista Superior para dibujar tu primera caja, y la vista Superior marca el sitio con una etiqueta amarilla. Debajo explica la cámara 3D y enlaza la [guía de primeros pasos](../getting-started.md). Cuando el proyecto abierto tiene mapas, **Abrir un mapa de este proyecto** los lista, para que puedas abrir uno terminado en su lugar.

![La tarjeta del mapa vacío en la vista 3D con Guía de inicio, Abrir un mapa de este proyecto y Ocultar consejos, junto a la etiqueta amarilla en la vista Superior](../assets/interface/empty-map-card.png)

La tarjeta desaparece en cuanto el mapa tiene algo dentro. **Ocultar consejos** desactiva los consejos, y *Consejos en un mapa vacío* en *Archivo > Preferencias* los devuelve.

### Paneles { #panels }

| Panel | Acoplado | Para qué sirve |
| --- | --- | --- |
| Outliner | Izquierda | El mapa como un árbol de [capas, grupos](organizing.md) y objetos |
| Historial | Izquierda | Cada edición. Hacer clic en una la deshace a ella y a todo lo posterior, hacer clic en una atenuada rehace hasta ella |
| Problemas | Izquierda | Problemas encontrados mientras trabajas, como caras sin material. Haz clic en uno para seleccionar el objeto, la mayoría ofrece una solución |
| Inspector | Derecha | Las propiedades de la selección, como las claves de una entidad o la posición y el tamaño exactos de un brush |
| Entidades | Derecha | Clases de entidad para arrastrar a una vista |
| Editor UV | Derecha | [Colocación exacta de texturas](uv-editor.md) |
| Scatter | Derecha | Los modelos que pinta la herramienta [Scatter](scatter.md) |
| Materiales | Abajo | Las texturas de tu proyecto |
| Modelos | Abajo | [Archivos de modelo](models.md) para colocar |
| Prefabs | Abajo | Piezas ya hechas de la [biblioteca de prefabs](prefab-library.md) para pegar |
| Lógica | Empieza cerrado | El [cableado de entidades como grafo](../gameplay/logic-graph.md), para conectar entidades y previsualizar qué desencadena un output |
| Referencia | Empieza cerrado | Código para usar una clase de entidad desde tus scripts |

¿Cerraste un panel por accidente? Vuelve a abrirlo desde *Ver > Paneles*. *Restablecer diseño* al final del mismo menú lo deja todo donde empezó.

![El menú Ver con Paneles abierto, los paneles visibles marcados y Restablecer diseño al final](../assets/interface/view-panels-menu.png)

## Ayuda y manual { #help-and-the-manual }

Los tooltips explican botones, pestañas y ajustes. Para más, apunta a una herramienta, a la pestaña de un panel, a una entrada de menú o a una sección del Inspector y pulsa **F1**, y se abre la página correspondiente de este manual. Un Ctrl+clic en una herramienta de la barra de herramientas (Cmd+clic en macOS) hace lo mismo. Un tooltip que termina en *F1: abrir manual* tiene una página detrás, sobre cualquier otra cosa F1 abre la paleta de comandos.

*Ayuda > Manual* abre el principio del manual.

![El menú Ayuda con Manual resaltado](../assets/interface/help-menu.png)

Cuando ya te muevas con soltura, desmarca *Texto de ayuda* en *Archivo > Preferencias* para ocultar las líneas cortas de orientación en la barra de opciones de la herramienta y en los paneles. Sus tooltips se quedan.

(Nota: los enlaces se abren en tu navegador web. Si no se abre ningún navegador, la barra de estado lo dice y el enlace se copia al portapapeles.)

## Vistas { #views }

Las vistas son 3D, Superior, Frontal y Lateral. Las vistas 2D miran en línea recta a lo largo de un eje, lo que facilita alinear cosas. Con algo seleccionado, imprimen su ancho encima y su alto al lado.

![La vista Frontal con un brush caja seleccionado, 96 impreso encima y 48 al lado](../assets/interface/front-view-size-labels.png)

Un clic en una vista 2D selecciona el objeto más cercano a lo largo de la vista, así que en la vista Superior un techo oculta las paredes que tiene debajo. Haz clic otra vez en el mismo sitio en menos de un segundo y medio, o Alt+clic en cualquier momento, para pasar al siguiente objeto bajo el cursor.

| Haz esto | Para |
| --- | --- |
| Arrastrar el punto donde se cruzan los divisores | Redimensionar las cuatro vistas a la vez |
| Shift+Space | Maximizar la vista bajo el puntero, otra vez para restaurar |
| *Ver > Vistas* | Cambiar entre una, dos y cuatro vistas, o recuperar una vista cerrada. Una vista conserva la vista en la que trabajaste por última vez, la vista 3D al principio, y mientras se muestra una sola vista, elegir el nombre de otra vista cambia a ella |

![El menú Ver con Vistas abierto: las cuatro vistas marcadas, Una vista, Dos vistas y Cuatro vistas, y Maximizar vista en Shift+Space](../assets/interface/view-views-menu.png)

## Pestañas { #tabs }

Cada mapa abierto recibe una pestaña, y la barra de pestañas aparece bajo la barra de opciones de la herramienta cuando hay dos abiertos. Ctrl+Shift+N (o el botón **+**) abre una pestaña nueva, Ctrl+Tab cambia y Ctrl+W cierra una.

![La barra de pestañas con room.tdm y untitled.tdm abiertos y el botón + resaltado, mostrando Nueva pestaña](../assets/interface/map-tabs.png)

*Nuevo mapa* y la importación se abren en una pestaña nueva cuando el mapa actual tiene cambios sin guardar. Cerrar una pestaña modificada pregunta si guardar, descartar o cancelar, y al salir se lista cada mapa sin guardar.

## Cámara { #camera }

| Entrada | Vista 3D | Vistas 2D |
| --- | --- | --- |
| Arrastrar con el botón derecho | Mirar alrededor. WASD mueve, Q y E bajan y suben, Shift triplica la velocidad | Desplazar |
| Arrastrar con el botón central | Desplazar | Desplazar |
| Rueda | Avanza y retrocede, más rápido cuanto más lejos está la superficie justo delante | Zoom hacia el cursor |
| Alt+arrastrar con el botón izquierdo | Orbitar, salvo que el arrastre empiece sobre la selección, que entonces se mueve en vertical | |

WASD solo vuela mientras mantienes pulsado el botón derecho, si no esas teclas son atajos. ¿Olvidaste los controles? Pasa el puntero por la pestaña de una vista. Hasta que miras alrededor por primera vez, la vista 3D también los muestra bajo su nombre mientras el puntero está sobre ella (**Ocultar consejos** en la tarjeta del mapa vacío lo desactiva).

**F** encuadra la selección en todas las vistas, o todo el mapa cuando no hay nada seleccionado. Una muesca de la rueda hace zoom en una vista 2D con el mismo factor a cualquier zoom, así que pasar de un terreno grande entero a una sola sala lleva unas 20 muescas, y **F** sobre la sala es más rápido.

Ctrl+Shift+1 a 9 guarda la cámara 3D como un marcador y Ctrl+1 a 9 salta de vuelta. Los marcadores se guardan en el mapa, y deshacer no los toca.

## Sombreado { #shading }

F4 recorre la vista 3D por Con textura, Plano, Vista previa iluminada y Alambre, y la vista imprime el modo arriba a la izquierda. Cuando el mapa tiene un [precalculado](light-baking.md), **Iluminación precalculada** viene después de Vista previa iluminada. F3 salta directo a **Vista previa iluminada** y de vuelta a Con textura, sin pasar por los demás.

![La vista 3D recorriendo el sombreado con textura, plano, iluminado y alambre](../assets/interface/shading-cycle.gif)

La vista previa iluminada muestra las luces del mapa, las sombras del sol, el cielo y la niebla sin construir en Godot. Solo dibuja las 64 luces puntuales y focales más fuertes, así que un mapa con muchas más puede verse más oscuro aquí que en Godot.

## Tema { #theme }

TrenchDot viene con un aspecto oscuro y uno claro. Una instalación nueva empieza en oscuro.

1. Abre **Archivo > Preferencias**. **Tema** es la primera fila, arriba de la ventana.
2. Haz clic en el desplegable y elige **Oscuro**, **Claro** o **Sistema**.

    ![El desplegable Tema en Preferencias, abierto con Oscuro, Claro y Sistema](../assets/interface/preferences-theme.png)

3. Todo el editor cambia en el siguiente fotograma, sin reiniciar. La elección se guarda con tus otras preferencias.

    ![El editor en el tema claro, con la barra de menús, el Outliner, la vista 3D y la vista Superior](../assets/interface/theme-light-window.png)

**Sistema** sigue el modo claro u oscuro de las aplicaciones de tu sistema operativo y cambia con él. Mientras está elegido, el desplegable dice *Sistema (Claro)* o *Sistema (Oscuro)* para que veas qué te dio tu sistema. Donde el sistema no informa de un modo, el editor se queda en oscuro.

También puedes elegirlo en **Ver > Tema**, justo debajo de *Escala de la interfaz*, o en la paleta de comandos como *Ver: Tema: Oscuro*, *Claro* o *Sistema*. No hay ninguna tecla asignada por defecto, pero puedes dar una tecla a cada uno en el editor de atajos.

En Claro, las vistas 2D (Superior, Frontal y Lateral) se vuelven papel, un fondo gris claro con líneas de rejilla y bordes de brush oscuros, para que las líneas que dibujas se lean de un vistazo. La vista 3D mantiene su entorno oscuro en los dos temas, y también los lienzos de textura del editor UV y del editor de hotspots. Una textura se ve igual en cualquiera de los temas, y un entorno claro cambiaría cómo juzgas sus colores.

<div class="td-mcp" markdown>

Por el [servidor MCP](../mcp.md), `set_editor` con `"theme": "light"` cambia el tema. Los valores son `dark`, `light` y `system`, y uno desconocido es un error que no cambia nada. `get_state` informa de la elección como `ui.theme` y de lo que se ve en pantalla como `ui.theme_active`.

</div>

## Idioma { #language }

Los menús, paneles, tooltips y mensajes de estado vienen en inglés, alemán, francés, español, portugués, ruso, chino simplificado y japonés. Una instalación nueva empieza en inglés.

1. Abre **Archivo > Preferencias**. **Idioma** es la segunda fila, justo debajo de **Tema**.
2. Haz clic en el desplegable y elige tu idioma. Cada idioma aparece con su propio nombre, así que puedes encontrar el tuyo aunque el editor esté en un idioma que no lees.

    ![El desplegable Idioma en Preferencias, abierto con English, Deutsch, Français, Español, Português, Русский, 简体中文 y 日本語](../assets/interface/preferences-language.png)

3. Todo el editor cambia al instante, sin reiniciar. La elección se guarda con tus otras preferencias.

    ![El menú Archivo en alemán](../assets/interface/language-file-menu-de.png)

Mientras hay otro idioma distinto del inglés activo, la fila dice por ejemplo *Sprache (Language)*, así que siempre puedes encontrar el camino de vuelta. También puedes elegir un idioma en la paleta de comandos como *Ver: Idioma: Deutsch*, y el mismo comando funciona en todos los idiomas, porque la paleta también busca los nombres en inglés. No hay ninguna tecla asignada por defecto, pero puedes dar una tecla a cada idioma en el editor de atajos.

Las traducciones las escribe una IA y todavía ningún hablante nativo las ha revisado, una nota bajo la fila lo dice. Un texto que aún no tiene traducción se muestra en inglés, y también un idioma que esta versión no conoce. Si una traducción suena mal, puedes corregirla en [el archivo de idioma de ese idioma](https://github.com/Paraxdev/TrenchDot/tree/main/crates/td_editor/locales), mira [Traducciones](../development.md#translations).

> [!NOTE]
> El manual sigue el idioma del editor. Con un idioma elegido, **F1** sobre una herramienta, un panel o un menú, y las entradas del menú **Ayuda**, abren esa página del manual en tu idioma. El botón de idioma en la cabecera del manual cambia el idioma ahí. Una página que aún no está traducida, o que ha cambiado desde que se tradujo, se muestra en inglés con una nota arriba. Las capturas muestran el editor en inglés, así que busca la misma entrada en tu idioma, o escribe el nombre en inglés en la paleta de comandos. Los classnames de entidades, las claves de mapa, los formatos de archivo, el log y el [servidor MCP](../mcp.md) se quedan en inglés en todos los idiomas.

<div class="td-mcp" markdown>

Por el [servidor MCP](../mcp.md), `set_editor` con `"language": "de"` cambia el idioma. Los valores son `en`, `de`, `fr`, `es`, `pt`, `ru`, `zh-Hans` y `ja`, en mayúsculas o minúsculas, y uno desconocido es un error que los nombra todos y no cambia nada. `get_state` informa de la elección como `ui.language`. Los nombres de herramientas, las respuestas y los errores se quedan en inglés en todos los idiomas. Un editor iniciado con `--default-prefs` está en inglés, de lo que dependen los tests e2e.

</div>

## Escala de la interfaz { #interface-scale }

Ctrl+= y Ctrl+- escalan la interfaz y Ctrl+0 la restablece. Lo mismo lo encuentras como *Escala de la interfaz* arriba de *Archivo > Preferencias*. Si el monitor informa de un escalado de pantalla equivocado, desmarca *seguir al monitor* justo debajo.

![La parte superior de la ventana Preferencias: Tema en Oscuro, Idioma en English, Escala de la interfaz al 100 %, Escalado de pantalla con seguir al monitor marcado, y debajo Consejos en un mapa vacío y Texto de ayuda](../assets/interface/preferences-scale-and-tips.png)
