---
source: 5b25efdda7a0b559f9cff81871758a2f55e33a9c
---

# Exportar a Blender y otras herramientas 3D { #exporting-to-blender-and-other-3d-tools }

Puedes guardar el aspecto de tu mapa como un único archivo de modelo 3D: brushes, mallas, terreno y modelos, con sus texturas dentro. Práctico para renderizar un nivel en Blender, montar un plano para un tráiler, o pasarle un blockout a un artista que construye las mallas finales encima.

> [!WARNING]
> Solo se exportan la geometría y los materiales. La lógica de entidades, el cableado de I/O, los triggers, los scripts y el comportamiento de juego se quedan fuera, y el archivo no se puede volver a abrir en TrenchDot como mapa. El `.tdm` sigue siendo el mapa, sigue editando ese.

## Pasos { #steps }

1. Abre la exportación

    Ve a *Archivo > Exportar > glTF Binary (.glb) para Blender y otras herramientas 3D…*, o escribe "glTF" en la paleta de comandos (Ctrl+Shift+P o F1).

    ![El menú Archivo con Exportar abierto y glTF Binary (.glb) para Blender y otras herramientas 3D resaltado](../assets/exporting/menu-export.png)

2. Elige qué entra

    Se abre la ventana *Exportar para herramientas 3D*. Marca lo que quieras, mira [las opciones](#options) más abajo. Los números junto a *Modelos de props* e *Instancias de scatter* te dicen cuántos objetos añade cada una. Puedes cambiar el formato a OBJ arriba.

    ![La ventana Exportar para herramientas 3D con glTF Binary elegido, Mapa completo, Modelos de props (5) marcado e Instancias de scatter (4081) sin marcar](../assets/exporting/export-dialog.png)

3. Guárdalo

    Haz clic en **Exportar…** y elige dónde va el archivo. La exportación se ejecuta en segundo plano con una ventana de progreso, así que puedes seguir editando, y **Cancelar** la detiene sin escribir nada. Cuando termina, la barra de estado dice cuántos objetos, materiales y triángulos salieron.

4. Ábrelo en Blender

    Usa *Archivo > Importar > glTF 2.0 (.glb/.gltf)* y elige el archivo. Los valores por defecto son correctos, el importador pone el modelo con Z hacia arriba por sí solo.

El archivo contiene el mapa tal como estaba cuando hiciste clic en **Exportar…**. Los cambios sin guardar se autoguardan antes, salvo que el autoguardado esté desactivado en *Archivo > Preferencias*. Si la barra de estado nombra un material para el que el proyecto no tiene imagen, ese sale sin textura.

## Opciones { #options }

| Opción | Hace |
| --- | --- |
| Mapa completo, Solo la selección | Todo, o solo los objetos seleccionados, aún dentro de sus capas y grupos |
| Capas y objetos ocultos | También exporta lo que está oculto en las vistas. Desactivada, obtienes lo que ves |
| Modelos de props | Los modelos que muestran los props y otras entidades punto, en su sitio, con su rotación y escala |
| Instancias de scatter | Cada árbol, roca y mata de los [conjuntos de scatter](scatter.md) como su propio objeto. Desactivada por defecto, ya que miles de ellos hacen un archivo grande y lento de abrir |
| Entidades puntuales como marcadores vacíos | Un objeto vacío con el nombre de la entidad donde está cada entidad punto sin modelo, práctico para colocar luces o cámaras en Blender |
| Combinar brushes sin nombre | Los brushes a los que no has puesto nombre se unen en un objeto por capa, grupo o entidad, lo que mantiene corto el outliner de Blender. Los brushes con nombre siguen separados |

## Qué se exporta { #what-is-exported }

| En el mapa | En el archivo |
| --- | --- |
| Capas y grupos | Objetos padre vacíos con los mismos nombres, así que la estructura del Outliner se conserva |
| Brushes | Un objeto por cada uno, con el nombre del Outliner, `brush12` o el nombre que le diste |
| Entidades de brush como puertas y ascensores | Un objeto padre con el nombre de la entidad que contiene sus brushes, donde están en el editor |
| Mallas, decals | Un objeto por cada uno. Los decals reciben materiales mezclados y de doble cara |
| Terreno | Un objeto. Cada triángulo toma la textura de su capa pintada más fuerte, así que las mezclas pintadas se ven escalonadas |
| Instancias de prefab | El contenido del prefab movido a su sitio, bajo un objeto con el nombre de la instancia |
| Materiales | Uno por material con su textura de albedo, más los mapas de normales, rugosidad, metálico, oclusión ambiental y emisión que tenga. Los materiales recortados y transparentes conservan su alfa, el pixel art se mantiene nítido. Los colores de desarrollo integrados salen como sus texturas de cuadrícula |

El archivo deja fuera lo que la construcción de Godot no dibuja: las caras con texturas de herramienta (clip, skip, origin, sky, trigger, nodraw, hint), las caras ocultas contra otro brush, los triggers y otras entidades de volumen, y las capas con *Omitir al exportar*. Tampoco se exportan las luces, los sonidos, los puntos de aparición, el cielo ni la niebla.

Solo los objetos que entran en el archivo se ocultan caras unos a otros. Así, un pilar exportado con *Solo la selección* conserva la base que descansa en el suelo, y una capa con *Omitir al exportar* no deja agujeros en sus vecinas.

## Unidades y ejes { #units-and-axes }

Un metro en Godot es un metro en Blender. Las unidades del mapa se dividen por las unidades por metro del proyecto, 32 salvo que los ajustes del mapa digan otra cosa. El archivo tiene Y hacia arriba como Godot, que es lo que espera glTF. Cada brush, malla y terreno tiene su origen en el centro de sus límites, y los modelos conservan el origen del archivo del modelo.

## Cuándo usar OBJ { #when-to-use-obj }

*Archivo > Exportar > Archivo Wavefront OBJ (.obj)…* es para herramientas que no leen glTF. Escribe tres cosas una al lado de otra, por ejemplo `level.obj`, `level.mtl` con los materiales, y una carpeta `level_textures` con copias de las texturas. Mantenlas juntas cuando las muevas. En Blender, usa *Archivo > Importar > Wavefront (.obj)*.

OBJ no tiene objetos padre ni forma de compartir una malla entre copias, así que cada objeto se escribe plano en el espacio del mundo y cada instancia de scatter es una copia completa. Con las instancias de scatter marcadas, la ventana avisa cuando suman un millón de triángulos y dice cuánto crece el archivo. Una exportación OBJ rechaza más de 10 millones de triángulos, un archivo de más de un gigabyte. Los materiales conservan las texturas de albedo, normales, rugosidad y emisión. Prefiere glTF siempre que la otra herramienta lo lea.

Un archivo `.glb` guarda como máximo 4 GB. glTF escribe cada modelo una sola vez por muchas instancias de scatter que lo muestren, así que solo un mapa enorme llega ahí, y entonces la exportación lo dice en lugar de escribir un archivo roto.

## Línea de comandos { #command-line }

`trenchdot --export-glb <map> <out.glb>` y `--export-obj <map> <out.obj>` exportan con las opciones por defecto sin abrir una ventana. El proyecto de Godot del mapa se encuentra a partir de su carpeta. Mira [Línea de comandos](../command-line.md).

<div class="td-mcp" markdown>

## MCP { #mcp }

`map_file {op: export_glb, path}` escribe un `.glb` y `map_file {op: export_obj, path}` un `.obj` con su `.mtl` y su carpeta de texturas, a partir del mapa abierto tal como está, guardado o no. Las opciones son indicadores: `selection_only`, `hidden`, `scatter`, `models` (true salvo que se indique), `markers` y `merge_brushes`. El resultado lista `objects`, `meshes`, `materials`, `images`, `triangles`, `bytes` y `missing_textures`. `run_action` rechaza `export_glb` y `export_obj`, ya que las entradas del menú abren un diálogo de archivo.

</div>
