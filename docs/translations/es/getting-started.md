---
source: 18fb3370f830b9fda2728d12b95c3b5cef06be64
---

# Primeros pasos { #getting-started }

Pasos 1 a 3: la configuración inicial. Si ya tienes un proyecto de Godot configurado con TrenchDot, puedes saltártelos.

Pasos 4 y 5: crear la primera sala en TrenchDot y verla en Godot.

## Qué necesitas { #what-you-need }

* Godot 4.7 o más reciente, la versión .NET si quieres escribir entidades en C#
* La [versión de TrenchDot](https://github.com/Paraxdev/TrenchDot/releases/tag/beta) más reciente

## Comprueba que TrenchDot ve el ejecutable de Godot { #ensure-trenchdot-can-see-the-godot-executable }

Asegúrate de que TrenchDot ve tu instalación de Godot. Para comprobar si todo se detecta bien, ve a
*Archivo > Preferencias… > Ejecutable de Godot*.

Si TrenchDot encontró Godot por sí mismo, la línea bajo el campo de texto ya dice "Usando" y la ruta. Si dice
"No encontrado", tienes que localizar el ejecutable en tu disco.

Para ello, haz clic en el botón "…" junto al campo de texto

![El botón … junto al campo Ejecutable de Godot](assets/getting-started/godot-path-browse.png)

y busca el ejecutable que empieza por "Godot_v".

Si elegiste el ejecutable correcto, debería decir "Usando" y la ruta que acabas de elegir.

![La línea bajo el campo de texto, que dice Usando y la ruta al ejecutable de Godot](assets/getting-started/godot-path-using.png)

## 1. Crea un proyecto en Godot { #1-create-a-project-in-godot }

Abre Godot y haz clic en "Create" arriba a la izquierda.

![El gestor de proyectos de Godot, con el botón Create y un icono de más arriba a la izquierda](assets/getting-started/godot-create-project.png)

Elige un nombre de proyecto, para este tutorial será TrenchDotTest, y (opcionalmente) una carpeta donde guardarlo.

Después pulsa "Create".

## 2. Abre el proyecto en TrenchDot { #2-opening-the-project-within-trenchdot }

Elige *Godot > Abrir proyecto de Godot…*

![El menú Godot de TrenchDot con Abrir proyecto de Godot… resaltado](assets/getting-started/menu-open-godot-project.png)

o el botón **Abrir proyecto de Godot…** arriba a la derecha de la barra de herramientas

![El botón Abrir proyecto de Godot… arriba a la derecha de la barra de herramientas de TrenchDot, que muestra el nombre del proyecto cuando hay uno abierto](assets/getting-started/toolbar-open-godot-project.png)

y elige la carpeta del proyecto o cualquier carpeta dentro de él. El editor lo vuelve a abrir en el siguiente arranque.

## 3. La pantalla de bienvenida { #3-the-welcome-screen }

En cuanto TrenchDot detecta que es un proyecto de Godot nuevo, muestra la ventana *Configurar este proyecto*.

![La ventana Configurar este proyecto con las secciones del addon, las entidades de gameplay y el contenido ya hecho](assets/getting-started/setup-window.png)

Aquí puedes descargar el addon, que Godot necesita para convertir tus mapas en escenas y para hablar con TrenchDot, y
contenido extra si quieres. Por supuesto, también puedes empezar sin nada.

Algunas entidades básicas se incluyen siempre, como un sencillo `info_player_start` y `light`.

1. Instala el addon

    Haz clic en el botón "Instalar el addon" de arriba para descargar el fork de FuncGodot de TrenchDot en tu proyecto,
    algo imprescindible para TrenchDot.
    (Nota: esto descarga de las versiones de GitHub. Si no tienes acceso a internet, descarga antes
    `func_godot-trenchdot-addon.zip` y extráelo en la raíz del proyecto para instalarlo a mano.)

    Una vez hecho, haz clic en "Activar el plugin" para que Godot lo cargue.
    (Nota: cierra antes Godot si tiene el proyecto abierto, porque si no Godot vuelve a escribir su propia copia de
    `project.godot` y el plugin se queda desactivado.)

2. Instala las entidades de gameplay

    Haz clic en **Instalar entidades de gameplay**, debajo del addon.

    ![La sección de entidades de gameplay de la ventana Configurar este proyecto con el botón Instalar entidades de gameplay](assets/getting-started/setup-gameplay-entities.png)

    Con este paquete vienen las puertas, botones, ascensores, triggers, props y spawners. Sin él, Godot los construye como
    formas simples que no tienen colisión y no hacen nada. Los ayudantes de puertas y triggers del menú Gameplay instalan
    el paquete por sí solos la primera vez que los usas, pero instalarlo ahora, antes de que Godot abra el proyecto por
    primera vez, significa que ninguna escena se construye nunca sin él.

3. (Opcional) Contenido extra

    TrenchDot tiene contenido opcional que puedes precargar: texturas y modelos CC0 de Poly Haven y ambientCG, además de
    modelos hechos para TrenchDot. Se incluyen las licencias y una lista de dónde sale cada cosa.

    Si no quieres contenido extra, puedes elegir "Nada extra", las entidades mínimas se incluyen igualmente.

4. Abre el proyecto en Godot una vez

    Una vez activado el plugin, abre el proyecto en Godot (la ventana tiene un botón "Abrir proyecto en el editor de
    Godot" para eso). El addon crea entonces un archivo llamado `trenchdot_game.json` en la raíz del proyecto. De ahí
    aprende TrenchDot tus texturas y entidades, y lo carga en menos de un segundo.

    El addon lo vuelve a escribir cada vez que cambian los archivos, así que no hace falta editarlo a mano. Hasta que
    exista, la barra de estado dice que no hay `trenchdot_game.json`, y el editor ofrece las entidades básicas, más las
    entidades de gameplay si las instalaste.

Para añadir entidades propias, mira [Entidades propias en GDScript](gameplay/custom-entities.md) y
[Entidades en C#](gameplay/csharp.md). [Configuración del proyecto](godot/project-setup.md) explica los recursos que las
listan.

## 4. Construye una sala { #4-build-a-room }

El editor tiene 4 vistas principales: una vista 3D por la que puedes volar igual que en Godot, y 3 vistas de alambre
desde arriba, de frente y de lado.

Los brushes y las operaciones dependen del tamaño de la cuadrícula. En la barra de herramientas puedes ajustar el tamaño
de la cuadrícula (más pequeño es más fino).

![El ajuste del tamaño de la cuadrícula en la barra de herramientas, puesto en 16](assets/getting-started/toolbar-grid-size.png)

En un nivel vacío verás un pequeño texto de ayuda que explica cómo dibujar. Puedes ocultarlo con "Ocultar consejos" o
simplemente empezar a dibujar, desaparece en cuanto el mapa tiene algo dentro.

![Un mapa nuevo: la tarjeta de la vista 3D dice que arrastres en la vista superior para dibujar la primera caja, y la vista superior muestra una etiqueta amarilla "Arrastre aquí para dibujar su primera caja"](assets/getting-started/empty-map.png)

1. Dibuja tu primera caja

    Con la herramienta Seleccionar (Q), arrastra sobre un espacio vacío en cualquier vista.
    Deberías ver cómo se dibuja una caja. Mientras mantienes pulsado el clic izquierdo, mueve el ratón para darle forma de
    rectángulo.

    ![Las cuatro vistas con la primera caja dibujada, y la vista superior mostrando su tamaño, 512 por 256](assets/getting-started/draw-first-box.png)

2. Añade altura

    En la vista lateral, mantén el ratón sobre el borde superior de la caja hasta que se ponga amarillo, mantén pulsado el
    clic izquierdo y mueve el ratón hacia arriba hasta que la caja mida 256 unidades de alto.

    ![La vista lateral con el borde superior de la caja arrastrado hacia arriba, con 256 escrito encima y al lado](assets/getting-started/side-view-depth.png)

3. Vacíala

    Pulsa Ctrl+Shift+K, o usa el botón Vaciar de la barra de herramientas.

    ![La barra de herramientas con el botón Vaciar bajo el cursor y su tooltip](assets/getting-started/toolbar-hollow.png)

    La caja queda dividida en paredes, un suelo y un techo, lo ideal si quieres hacer una sala pequeña o un pasillo. Ahora,
    con los controles de vuelo (mantén pulsado el botón derecho del ratón y usa WASD), vuela al interior de la caja
    vaciada.

4. Corta una puerta

    Con Escape puedes deseleccionar los brushes seleccionados. Cuando no haya nada seleccionado, dibuja en la vista frontal
    un rectángulo vertical más pequeño dentro de la caja, con forma de puerta.

    ![El brush de la puerta dentro de la caja vaciada, de 64 de ancho y 96 de alto en la vista frontal](assets/getting-started/doorway-brush.png)

    Después, en la vista lateral, alarga un lado del brush de la puerta para que tenga profundidad, hasta atravesar toda
    la pared.

    ![Arrastrando un lado del brush de la puerta en la vista lateral para que atraviese la pared](assets/getting-started/doorway-extend.gif)

    Ahora, con el brush seleccionado, pulsa Ctrl+K o usa el botón Restar de la barra de herramientas.

    ![La barra de herramientas con el botón Restar de CSG bajo el cursor y su tooltip](assets/getting-started/toolbar-subtract.png)

    Deberías ver desaparecer el brush, y la puerta queda cortada en la pared.

    ![La pared en la vista 3D con la puerta cortada](assets/getting-started/doorway-cut.png)

5. Ponle textura

    Deberías tener 5 texturas incluidas para hacer prototipos. Para aplicar una textura a una sola cara (por ejemplo, el
    marco de la puerta), arrastra la textura desde la pestaña Materiales hasta la cara (mantén Shift para cubrir todo el
    brush).

    ![Arrastrando dev/blue desde la pestaña Materiales hasta una cara, con la vista previa de soltar](assets/getting-started/texture-drop.png)

6. Guarda el mapa

    Para ver los cambios en Godot hay que guardar el mapa. Pulsa Ctrl+S, o usa *Archivo > Guardar*.

    El cuadro de guardado se abre en la carpeta de tu proyecto de Godot. Guárdalo en cualquier sitio dentro del proyecto,
    una carpeta `maps` en la raíz del proyecto lo mantiene ordenado, por ejemplo `res://maps/room.tdm`.
    (Nota: el nombre de la carpeta es cosa tuya, pero deja el mapa dentro del proyecto, para que Godot pueda cargarlo por
    su ruta `res://`.)

    Una vez guardado, aparece en *Archivo > Mapas del proyecto*.

    ![El menú Archivo con Guardar y Mapas del proyecto, que lista room bajo res://maps](assets/getting-started/file-menu-maps-in-project.png)

## 5. Velo en Godot { #5-see-it-in-godot }

En Godot, un nivel es una escena: un árbol de nodos, guardado como archivo `.tscn`, donde cada nodo es una cosa como una
malla, una luz o una cámara. Godot escribe las rutas dentro del proyecto con `res://`, así que `res://maps/room.tdm` es
el archivo `maps/room.tdm` de la carpeta de tu proyecto. El mapa se convierte en nodos mediante un nodo del addon,
`FuncGodotMap`.

1. En el editor de Godot, elige *Scene > New Scene* y haz clic en **3D Scene** en el panel Scene de la izquierda.
2. Pulsa Ctrl+A (*Add Child Node*), busca `FuncGodotMap` y añádelo.
3. Con el nuevo nodo seleccionado, pon *Local Map File* en tu `.tdm` en el Inspector propio de Godot, el panel de la
   derecha del editor de Godot. Funciona como el Inspector de TrenchDot, pero es de Godot.
4. Pulsa **Build Map** arriba de ese Inspector y guarda la escena con Ctrl+S.

Los brushes se convierten en mallas con colisión y las entidades en nodos con sus scripts, guardados con la escena.
[Construir mapas](godot/building.md) explica los demás ajustes del nodo. Si una entidad sale como una malla desnuda sin
colisión, la barra de estado de TrenchDot dice por qué, mira [Entidades que no hacen nada](godot/entities-troubleshooting.md).

Deja esa escena abierta en Godot y cada guardado en TrenchDot la reconstruye, enseguida mientras su pestaña es la actual,
o cuando vuelves a ella. Guarda la escena en Godot para conservar el resultado. Si un guardado no reconstruye, la barra
de estado dice por qué, mira [Reconstruir al guardar](godot/live.md#rebuild-on-save).

> [!WARNING]
> Una construcción borra primero los hijos del `FuncGodotMap`. Pon tus propios nodos junto al nodo del mapa, o en
> un [overlay](godot/overlays.md).

## Prueba el proyecto de demostración { #try-the-demo-project }

La carpeta `godot` del repositorio, también en la versión publicada como `trenchdot-demo-project.zip`, es un proyecto con
el addon instalado. Su escena principal reproduce los mapas de muestra, que están guardados en `godot/demo/maps/showcase`.
Abre el proyecto en el editor y elige uno en *Archivo > Mapas del proyecto* para ver cómo se monta un mapa terminado.

La opción de demo de *Godot > Añadir contenido al proyecto…* pone esos mismos mapas y escenas en tu propio proyecto. En
Godot, abre `res://demo/demo.tscn` y pulsa F6 para reproducirlos. Abiertos en TrenchDot desde tu proyecto se muestran sin
sus texturas, porque usan la carpeta de texturas de la demo `res://demo/textures` en vez de la de tu proyecto. El proyecto
de demostración los muestra tal como se construyeron.

| Tecla | Acción |
| --- | --- |
| WASD, Space, Shift | Caminar, saltar, correr |
| E | Usar puertas y botones |
| 1 a 5 | Cambiar entre los mapas de muestra |
| F3 | Mostrar el overlay de depuración de I/O, un registro de cada input y output según se dispara, con los volúmenes trigger dibujados |

## Siguiente { #next }

Para que la sala haga algo, lee [Cómo funciona el I/O de entidades](gameplay/io.md) y construye
[Un botón abre una puerta](tutorials/button-door.md).

Tu propio jugador necesita dos cosas para las entidades del paquete. Los triggers solo reaccionan a cuerpos del grupo de
Godot `player` en la capa de física 1, así que pon ahí a tu jugador. Las puertas y los botones se mueven cuando el código
de tu jugador llama a `use()` sobre el que está mirando, mira [Deja que el jugador lo pulse](tutorials/button-door.md#let-the-player-press-it).

<div class="td-mcp" markdown>

## MCP { #mcp }

La pregunta del contenido no se abre sola en un editor que maneja un cliente MCP: la primera llamada a una herramienta
distinta de `get_state`, `screenshot` y `simulate_input` la cierra, y a un proyecto abierto con `open_project` no se le
pregunta. `project_content` añade ese mismo contenido, mira [MCP](mcp.md).

</div>
