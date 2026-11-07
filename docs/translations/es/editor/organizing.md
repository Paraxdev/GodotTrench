---
source: b549cb4d1d3fd2ab8e0ac144848e50eac2b168fc
---

# Organizar un mapa { #organizing-a-map }

Cuando un mapa crece quieres ordenarlo en partes, volver a encontrar las cosas y tener en un solo sitio las piezas que reutilizas. Casi todo eso empieza en el Outliner, el panel de la izquierda.

## Capas { #layers }

El Outliner muestra el mapa como un árbol de capas, grupos y objetos. Las capas te dejan dividir un mapa en partes, por ejemplo un edificio por capa, para poder ocultar o bloquear una parte mientras trabajas en el resto.

Haz clic en **Añadir capa** arriba del Outliner para crear una nueva, que se convierte enseguida en la capa actual. Los objetos nuevos van a la capa actual, la marcada con *(actual)*, y al hacer clic en una capa esta pasa a ser la actual. El ojo delante de una fila la oculta, el candado impide seleccionarla o editarla.

Pasa el ratón sobre una fila para ver sus materiales, su tamaño y sus límites. El objeto se resalta en todas las vistas a la vez, así que puedes ver qué pared es `brush7`.

![El Outliner con la capa House abierta y la fila brush7 bajo el ratón, un tooltip muestra su material, tamaño y límites mientras las vistas 3D y Frontal resaltan la pared del fondo](../assets/organizing/outliner-hover.png)

Los brushes y las mallas se listan por tipo e id, como `brush12` o `mesh40`, para poder distinguir las paredes que dejan un Vaciar o un Restar. El id sigue igual cuando el objeto se mueve y entre guardados.

### Poner nombres { #naming-things }

Selecciona un objeto y pulsa F2, o haz clic derecho sobre él en el Outliner o en una vista y elige *Renombrar*. La fila se convierte en un campo de texto. Enter conserva el nombre, Escape cancela, y un nombre vacío devuelve el predeterminado.

![Una fila de brush en el Outliner convertida en un campo de texto con "front wall" escrito](../assets/organizing/outliner-rename.png)

Los nombres se guardan con el mapa. Las entidades conservan su `targetname` aparte de este nombre, así que renombrar nunca cambia a qué llega una salida. Las piezas que deja un Clip, una operación CSG o una conversión conservan el nombre, y poner nombre a un objeto dentro de un grupo enlazado se lo pone en todas las copias.

(Las capas, los grupos y los conjuntos de scatter siempre tienen nombre, así que para ellos uno vacío no cambia nada. Renombrar un conjunto de scatter también renombra la capa que la herramienta Scatter creó para él.)

### Elegir objetos en el Outliner { #picking-objects-in-the-outliner }

Ctrl+clic en una fila la añade a la selección o la quita. Shift+clic selecciona todas las filas entre la última en la que hiciste clic y esta, y Ctrl+Shift+clic añade ese rango a lo que ya está seleccionado. El Outliner suele ser el sitio más fácil para elegir objetos que quedan ocultos tras otros en las vistas.

### Mantener una capa fuera de Godot { #keeping-a-layer-out-of-godot }

Para geometría de referencia que nunca debe llegar al juego, haz clic derecho en su capa y elige *Alternar omitir al exportar*. La capa muestra entonces *[omitida]* tras su nombre. Ocultarla no basta, Godot sigue construyendo los objetos ocultos.

![El menú de clic derecho de la capa de referencia Blockout con Alternar omitir al exportar resaltado](../assets/organizing/layer-menu-omit.png)

## Grupos { #groups }

Un grupo mantiene objetos juntos para que se seleccionen y se muevan como uno, como los brushes de una mesa. Todo lo relacionado con los grupos está en *Editar > Grupos*.

![El menú Editar con Grupos abierto y Duplicar enlazado resaltado](../assets/organizing/menu-groups.png)

| Entrada | Resultado |
| --- | --- |
| Ctrl+G, Ctrl+Shift+G | Agrupar la selección, desagruparla |
| Clic | Seleccionar el objeto dentro del grupo |
| Doble clic | Seleccionar el grupo entero |

**Duplicar enlazado** (Ctrl+Shift+D) hace una copia que se mantiene sincronizada con su original, así que arreglar un marco de ventana arregla todos. Las copias enlazadas muestran *(enlazado)* en el Outliner, y un clic en una selecciona el grupo entero, ya que sus copias se editan como una sola. *Editar > Grupos > Desenlazar grupos* rompe el enlace cuando una copia tiene que ser distinta.

## Prefabs { #prefabs }

Para piezas que reutilizas entre mapas, selecciónalas y elige *Editar > Prefabs > Crear desde la selección…*. Eso guarda la selección como su propio `.tdm` y pone una instancia en su lugar. Cada instancia se actualiza cuando cambia el archivo del prefab.

![El menú Editar con Prefabs abierto y Crear desde la selección resaltado](../assets/organizing/menu-prefabs.png)

> [!NOTE]
> Guarda el mapa una vez antes de crear un prefab. Las rutas de los prefabs se guardan relativas al mapa, así que el editor se niega hasta que el mapa tenga un archivo.

Para piezas que pegas una vez y luego modelas libremente, usa en su lugar la [biblioteca de prefabs](prefab-library.md).

Colocar un prefab dos veces da a dos entidades el mismo targetname. El fixup de una instancia lo resuelve: con un fixup de `p1`, cada nombre de su interior recibe ese prefijo, así que `door` pasa a ser `p1-door` y cada copia mantiene su cableado aparte. El archivo del mapa y la construcción de Godot conocen el fixup, pero el editor todavía no tiene un campo para fijarlo.

## Cordon { #cordon }

Un cordon reduce el editor a una parte de un mapa grande.

1. Selecciona en lo que quieres trabajar.
2. Elige *Ver > Cordon > Fijar cordon desde la selección*, o haz clic en **Cordon** en el Inspector.

    ![El menú Ver con Cordon abierto y Fijar cordon desde la selección resaltado](../assets/organizing/menu-cordon.png)

3. Todo lo que no toca la caja amarilla alrededor de tu selección queda oculto y no se puede seleccionar, y la barra de estado dice *cordon*.

    ![Antes: el mapa entero con la casa, el césped, un muro de jardín, un pilar y un bloque de referencia alto. Después: solo la casa dentro de una caja amarilla y el césped que la toca](../assets/organizing/cordon-before-after.png)

*Ver > Cordon > Cordon activado* lo apaga y lo vuelve a encender, y *Quitar cordon* lo elimina. El cordon se guarda con el mapa.

Solo cambia lo que muestra el editor, Godot sigue construyendo el mapa entero. Para pasar una zona a otra herramienta, *Archivo > Exportar > .map, solo cordon…* escribe solo los objetos de su interior.

## Problemas { #issues }

El panel Problemas, la pestaña junto al Outliner y el Historial, revisa el mapa mientras trabajas. Lista los problemas que romperían o se verían mal en Godot, como brushes sin volumen, caras sin material u outputs dirigidos a un targetname que nada tiene.

![El panel Problemas con un error por un destino que falta, tres advertencias por un brush duplicado, una entidad sin definición y dos caras que parpadean por z-fighting, y una línea informativa por un grupo vacío, la mayoría con un botón Corregir](../assets/organizing/issues-panel.png)

Haz clic en un problema para seleccionar y encuadrar el objeto. Muchos tienen un botón **Corregir**, y **Corregir todo** en la cabecera arregla todos los que puede.

Una advertencia de que una cara se solapa con una cara de otro brush en el mismo plano significa que las dos parpadearán en Godot. Mueve una cara fuera del plano o recorta el solape.

Una advertencia de que un modelo no existe significa que el `model` de un prop nombra un archivo que no está, así que el prop se construye sin malla. Cuando existe un archivo con el mismo nombre y otra extensión, `chair.gltf` para `chair.glb`, la advertencia lo nombra.

Una advertencia de que algo ejecuta GDScript desde el mapa significa que el mapa lleva código propio, que se ejecuta en cuanto la escena se reproduce en Godot. Recibes una por cada `logic_script` con un `source` o `expression`, por cada output o `logic_call` que puede darle código a uno, como un output que le envía `source`, y por cada prefab que contiene alguno de estos. Léelas antes de reproducir un mapa que hizo otra persona.

Las advertencias que empiezan por "Godot" vienen de la última construcción en Godot, tras un guardado o *Construir en Godot*: las clases que construyó sin su definición, y las demás advertencias de esa construcción. Mira [Entidades que no hacen nada](../godot/entities-troubleshooting.md).

## Historial { #history }

El panel Historial lista cada edición, la más nueva arriba. Haz clic en una entrada más antigua para deshacer hasta ella, esa entrada incluida. Las ediciones deshechas quedan atenuadas sobre *actual*, haz clic en una para rehacer hasta ella.

![El panel Historial con Rename y Group atenuados sobre la marca actual y la entrada Hollow bajo el ratón](../assets/organizing/history-panel.png)

<div class="td-mcp" markdown>

## MCP { #mcp }

Un agente o script que controla el editor por el [servidor MCP](../mcp.md) ejecuta las mismas comprobaciones que el panel Problemas con la herramienta `validate_map`, que las nombra por código, como `coplanar_faces` y `missing_model`. La acción `insert_prefab` de `run_action` acepta un `fixup`, así que un agente puede darle a una instancia su fixup al colocarla.

</div>
