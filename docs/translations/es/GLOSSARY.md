---
source: 62dc81f5978f85583c7c7cd6083e20459a306923
---

# Glosario { #glossary }

## Cursor 3D { #3d-cursor }
El punto del mundo que queda bajo el ratón, en la vista sobre la que pasaste por última vez. Todo lo que colocas en el cursor aparece ahí, como una entidad del botón Colocar del panel Entidades o un terreno nuevo.

## activador { #activator data-forms="activadores" }
El jugador o cuerpo que inició una cadena de I/O de entidades, normalmente quien pulsó el botón o entró en el trigger. Los pasos posteriores de la cadena pueden apuntar a !activator para actuar justo sobre ese.

## albedo { #albedo }
El color base de un material, la textura simple antes de añadir iluminación, mapas normales o brillo. Godot llama albedo a la textura de color de un material.

## alpha scissor { #alpha-scissor }
Un modo de transparencia de Godot que recorta la textura de golpe en un umbral de alfa, de modo que cada píxel se ve entero o se oculta del todo. Va bien para rejillas, hojas y decals de bordes duros, y a diferencia del alpha blending todavía proyecta sombras.

## autoload { #autoload data-forms="autoloads" }
Un script o escena que Godot carga una vez al arrancar y mantiene vivo durante todo el juego, configurado en Project Settings > Globals > Autoload. El código de todo el juego, como un contador de puntos o el oyente de la recarga en caliente, vive en uno.

## Iluminación precalculada { #baked-lighting data-forms="iluminaciones precalculadas|iluminación horneada" }
Luz que se calcula una sola vez de antemano y se guarda, en lugar de calcularse en cada fotograma. Godot > Precalcular iluminación precalcula el sol, el cielo, las lámparas y los materiales brillantes en un lightmap, y Godot lo dibuja mediante un LightmapGI.

## bloom { #bloom }
El brillo suave que las lámparas intensas y los materiales brillantes extienden a los píxeles de alrededor. Se activa con la clave glow_intensity de worldspawn.

## booleano { #boolean data-forms="booleanos|booleana|booleanas" }
Un valor que es verdadero o falso, como si la corriente está encendida. Las entidades logic_branch, logic_gate y logic_flipflop guardan y combinan booleanos.

## brush { #brush data-forms="brushes" }
Un sólido convexo, como una caja o una cuña, el bloque básico de construcción de un nivel. Dibujas brushes en las vistas y los combinas con CSG, y Godot convierte cada uno en una malla con su colisión.

## entidad brush { #brush-entity data-forms="entidades brush" }
Una entidad que toma su forma de brushes que construiste, como una puerta, un botón o un trigger. Primero dibujas los brushes y luego los conviertes en la entidad, al contrario que una entidad punto, que está en un único lugar.

## convexa { #convex data-forms="convexo|convexos|convexas" }
Abultada hacia fuera por todas partes, sin hendiduras ni huecos, como una caja, una cuña o un cilindro. Todo brush tiene que ser convexo, por eso el editor rechaza los movimientos de vértices que lo abollarían. Para formas cóncavas, convierte el brush en malla.

## cordon { #cordon data-forms="cordons" }
Una caja que limita el mapa a una zona mientras trabajas en ella. Con el cordon activado, los objetos fuera de la caja se ocultan y se dejan fuera de una exportación con cordon.

## CSG { #csg }
Geometría sólida constructiva, construir formas combinando sólidos, por ejemplo restando un brush de otro para abrir una puerta. Los comandos CSG del editor son Restar, Vaciar, Fusión convexa e Intersecar.

## Decal { #decal data-forms="decals" }
Una textura puesta sobre una superficie, como una mancha, una grieta o un póster. En TrenchDot un decal es una malla fina que se asienta sobre la cara sin parpadear, y se coloca manteniendo Alt mientras sueltas un material sobre una cara. Un decal proyectado es, en cambio, una entidad que proyecta un material sobre todo lo que hay dentro de su caja.

## desrepetir { #detile data-forms="desrepetición" }
Un ajuste de capa de terreno de 0 a 1 que gira y desplaza al azar cada repetición de la textura, para que un campo grande no muestre una cuadrícula evidente. Cuesta lecturas de textura adicionales, así que déjalo en 0 donde la repetición no se note.

## Displacement { #displacement data-forms="displacements" }
Una cara de brush de cuatro lados dividida en una rejilla de vértices que se pueden subir o bajar uno a uno, como en Hammer. Da un suelo irregular dentro de un nivel hecho con brushes. Las zonas grandes al aire libre es mejor construirlas como un terreno.

## definición de entidad { #entity-definition data-forms="definiciones de entidad" }
La descripción de una clase de entidad en un FGD: su classname, claves, inputs y outputs, y lo que Godot construye para ella, como la clase de nodo y el script. En Godot es un recurso FuncGodotFGDPointClass o FuncGodotFGDSolidClass, listado en Entity Definitions de un archivo FGD.

## Atenuación { #falloff data-forms="atenuaciones" }
Cómo se desvanece un pincel, como el de la herramienta Mezcla, del centro al borde. Una atenuación suave da bordes blandos, una constante pinta todo el disco con la misma fuerza.

## FGD { #fgd }
La definición de las clases de entidad de un juego, con sus nombres, claves, inputs y outputs. El nombre viene de los archivos Forge Game Data de Hammer. En Godot es un recurso FuncGodotFGDFile, y la configuración de juego se lo pasa al editor.

## fixup { #fixup data-forms="fixups" }
Un prefijo de nombre que se fija en una instancia de prefab. Con un fixup de p1, una entidad llamada door dentro de ella pasa a llamarse p1-door, así varias copias de un mismo prefab mantienen su cableado separado.

## func_detail { #func_detail }
Una entidad brush de FuncGodot para geometría estática con colisión, construida como su propio nodo aparte de los brushes del mundo. No añade ningún oclusor, así que nunca oculta del renderizado los objetos que quedan detrás.

## func_illusionary { #func_illusionary }
Una entidad brush de FuncGodot que se ve pero no tiene colisión, así que los jugadores la atraviesan. El tutorial de la isla marina usa una para el agua.

## FuncGodot { #funcgodot }
El plugin de Godot para construir niveles a partir de archivos de mapa al estilo Quake. El addon de TrenchDot es un fork suyo, por eso su nodo se llama FuncGodotMap y sus recursos empiezan por FuncGodot.

## Configuración de juego { #game-config data-forms="configuraciones de juego" }
El archivo trenchdot_game.json en la raíz del proyecto de Godot, escrito por el addon. Le dice al editor qué entidades, carpetas de texturas y escala de unidades usa tu proyecto, y el editor lo recarga cada vez que el addon lo vuelve a escribir.

## paquete de entidades de gameplay { #gameplay-entities-pack data-forms="paquete de entidades Gameplay|paquetes de entidades de gameplay|paquete Gameplay" }
Las puertas, botones, ascensores, triggers, lógica, spawners, props y efectos de la referencia de entidades. No forman parte del addon en sí. Al instalar el paquete se copian sus definiciones y scripts a res://trenchdot/entities, donde pasan a ser de tu proyecto y puedes cambiarlos con libertad.

## glTF { #gltf }
Un formato de archivo abierto para modelos y escenas 3D que leen Blender, Godot y la mayoría de las herramientas 3D. Un archivo .glb es su forma binaria, que guarda la geometría y las texturas en un solo archivo.

## grupo de Godot { #godot-group data-forms="grupos de Godot" }
Una etiqueta con nombre en los nodos de Godot, que se pone en Node > Groups o desde código, y permite al código y al I/O de entidades encontrarlos todos a la vez. Un destino como @enemies alcanza a todos los nodos del grupo enemies.

## Hammer { #hammer }
El editor de niveles de Valve para juegos del motor Source, como Half-Life 2. TrenchDot toma prestado su modelo de inputs y outputs de entidades y puede importar sus mapas .vmf.

## recarga en caliente { #hot-reload data-forms="hot reload" }
Reconstruir los mapas de un juego que ya está en marcha cada vez que guardas en TrenchDot, para no tener que reiniciarlo y ver un cambio. Necesita el autoload trenchdot_hot_reload.gd en el juego.

## Hotspot { #hotspot data-forms="hotspots" }
Un rectángulo marcado en una trim sheet y guardado en el archivo .hotspots.json de la textura. Ajuste de hotspot encaja las caras seleccionadas en el rectángulo que mejor les corresponde, así una trim sheet puede texturizar una sala rápidamente.

## lightmap { #light-map data-forms="lightmaps" }
Una textura que guarda la luz que llega a cada punto de la geometría estática, rebotes incluidos. Cada cara recibe su propia zona de ella.

## sonda de luz { #light-probe data-forms="sondas de luz" }
Un punto en el espacio abierto que guarda la luz que llega desde todas las direcciones. Los objetos en movimiento toman su luz de las sondas de alrededor, para coincidir con la iluminación precalculada de la geometría estática.

## LightmapGI { #lightmapgi }
El nodo de Godot que dibuja un lightmap y sus sondas de luz. Un mapa construido a partir de un .tdm precalculado recibe uno llamado baked_lighting.

## Vista previa iluminada { #lit-preview data-forms="vista previa iluminada" }
Un modo de sombreado de la vista 3D del editor, que se activa con F3, y que muestra las luces del mapa, las sombras del sol, el cielo y la niebla. Sirve para juzgar la iluminación antes de construir en Godot, pero solo dibuja las 64 luces puntuales y focos más fuertes.

## Live Link { #live-link }
La conexión entre TrenchDot y el editor de Godot, un servidor local que ejecuta el addon en el puerto 7842. Reconstruir al guardar, el modo en vivo y el estado de Godot en la barra de herramientas pasan todos por ahí.

## Modo en vivo { #live-mode }
Envía tus ediciones a la escena abierta en el editor de Godot mientras las haces, antes de guardar, para que puedas juzgar la iluminación y la escala con el propio renderizador de Godot. Actívalo con el botón de enlace de la barra de herramientas o en Godot > Modo en vivo.

## puerta lógica { #logic-gate data-forms="puertas lógicas" }
Una regla que convierte dos entradas verdaderas o falsas en una sola respuesta, como and (ambas son verdaderas) o xor (exactamente una lo es). La entidad logic_gate ofrece las más habituales.

## escena principal { #main-scene data-forms="escenas principales" }
La escena que Godot reproduce al ejecutar el proyecto, configurada en Project > Project Settings > Application > Run. Ejecutar proyecto en TrenchDot y el botón Play de Godot la inician las dos.

## ajustes del mapa { #map-settings }
Un recurso FuncGodotMapSettings que dice de dónde salen las texturas, los materiales y las definiciones de entidad y cuántas unidades de mapa son un metro. Un FuncGodotMap puede tener los suyos, y si no se usan los del proyecto.

## MCP { #mcp }
Model Context Protocol, una forma estándar de que agentes de IA y scripts llamen a las herramientas de un programa. El editor puede ejecutar un servidor MCP para que un agente inspeccione y edite mapas. No lo necesitas para construir niveles, y esta documentación oculta las partes de MCP hasta que activas el interruptor MCP de la cabecera.

## MultiMeshes { #multimeshes }
La forma de Godot de dibujar muchas copias de una malla en una sola pasada. Los árboles, rocas y hierba de scatter se construyen como MultiMeshes, lo que mantiene barato renderizar miles de ellos.

## malla de navegación { #navigation-mesh data-forms="mallas de navegación" }
Un contorno simplificado de los suelos por los que pueden caminar los agentes de IA, que Godot usa para encontrar rutas. TrenchDotNav precalcula una a partir de la colisión de un mapa construido.

## mapa normal { #normal-map data-forms="mapas normales" }
Una textura que guarda hacia dónde mira cada píxel de una superficie, para que las caras planas capten la luz como si tuvieran bultos y surcos. Añade detalle fino sin geometría adicional.

## operando { #operand data-forms="operandos" }
El número con el que trabaja un cálculo además del que recibe. En un math_calc que suma 5, el operando es 5.

## ORM { #orm }
Una textura que empaqueta tres mapas en escala de grises en sus canales de color, oclusión ambiental en el rojo, rugosidad en el verde y metálico en el azul. Los materiales de Godot y las capas de terreno de TrenchDot la leen como un solo archivo.

## entidad punto { #point-entity data-forms="entidades punto" }
Una entidad que está en un único lugar en vez de tomar su forma de brushes, como una luz, un sonido o un punto de aparición. Coloca una arrastrándola del panel Entidades a una vista.

## Prefab { #prefab data-forms="prefabs" }
Una pieza reutilizable de un mapa guardada como su propio archivo .tdm y colocada en otros mapas como instancias. Editar el archivo del prefab actualiza todas sus instancias.

## res:// { #res }
La forma que tiene Godot de escribir una ruta dentro de la carpeta del proyecto, se llame como se llame la carpeta en el disco. res://maps/room.tdm es room.tdm en la carpeta maps de tu proyecto de Godot. Por eso los mapas y modelos tienen que guardarse dentro del proyecto para que el addon los encuentre.

## Modo restringido { #restricted-mode }
Cómo abre TrenchDot un proyecto de Godot que contiene código y en el que todavía no has confiado. Puedes ver y editar sus mapas, pero el editor no inicia Godot con él, no construye en Godot ni envía ediciones en vivo, ya que Godot ejecuta los scripts de un proyecto al abrirlo, reproducirlo o construirlo. Haz clic en Modo restringido en la barra de estado para confiar en el proyecto.

## conjunto de scatter { #scatter-set data-forms="conjuntos de scatter" }
Un grupo de modelos pintados juntos con la herramienta Scatter, como un bosque o un prado de hierba, con su propia capa, densidad y superficies de destino. Godot construye cada conjunto como MultiMeshes.

## reflejos en espacio de pantalla { #screen-space-reflections data-forms="reflexiones en espacio de pantalla" }
Un efecto de Godot que refleja en las superficies brillantes lo que ya está en pantalla, así las calles mojadas reflejan las lámparas. Se activa poniendo la clave ssr de worldspawn en 1.

## señal { #signal data-forms="señales" }
El nombre que da Godot a un evento que anuncia un nodo, como pulsar un botón. Cada output de una entidad es una señal, y cablear un output conecta esa señal con un método de input del destino.

## targetname { #targetname }
El nombre que le das a una entidad para que otras puedan alcanzarla, como main_door. Los outputs, los fixups de prefab y claves como fixture encuentran las entidades por él.

## texel { #texel data-forms="texels" }
Un píxel de una textura tal como cae sobre una superficie. Por defecto un texel cubre una unidad de mapa, y la densidad de texels, cuántos caben en un área, decide lo nítida que se ve una textura. Un lightmap también tiene texels, y su tamaño de texel dice cuántas unidades de mapa cubre cada uno.

## tamaño de textura { #texture-size data-forms="tamaños de textura" }
El tamaño en el mundo que cubre una repetición de una textura, que se define como metadata/texture_size en un material de Godot. Sin él un píxel cubre una unidad de mapa, lo que estira una foto de alta resolución sobre un área enorme.

## texturas de herramienta { #tool-textures data-forms="textura de herramienta" }
Texturas especiales que le dicen a la construcción para qué sirve una cara de brush en vez de dibujarla. Las caras clip solo colisionan, las caras skip no construyen nada y las caras origin fijan dónde está el origen de una entidad brush. Toda textura de la carpeta special/, tools/ o td/ no dibuja nada en Godot, y lo mismo vale para nodraw, hint, trigger y los demás nombres de herramienta en cualquier carpeta.

## TrenchBroom { #trenchbroom }
Un editor de niveles gratuito para juegos al estilo Quake. TrenchDot se inspira mucho en él y puede importar sus archivos .map.

## volumen trigger { #trigger-volume data-forms="volúmenes trigger|volumen de trigger|volúmenes de trigger" }
Una entidad brush invisible que dispara outputs cuando un cuerpo entra o sale de ella, por ejemplo para iniciar una cinemática cuando el jugador entra en una sala. Dibuja una con la herramienta Volumen (Shift+E).

## trim sheet { #trim-sheet data-forms="trim sheets" }
Una textura que empaqueta tiras de detalle, como bordes, tuberías y paneles, en una sola imagen. Las caras se ajustan a sus partes con hotspots, así unas pocas trim sheets pueden texturizar un edificio entero.

## triplanar { #triplanar }
Una forma de proyectar una textura sobre una superficie desde tres direcciones, desde arriba y desde dos lados, mezcladas según el ángulo de la superficie. Las capas de terreno la usan para que los acantilados muestren roca en vez de un borrón estirado.

## Bloqueo UV { #uv-lock }
Un ajuste, activado por defecto y que se alterna con Ctrl+Shift+U, que mantiene las texturas pegadas a un brush mientras lo mueves. Con él desactivado, la textura se queda quieta en el mundo y se desliza por las caras movidas.

## UV { #uv }
Las coordenadas que dicen qué parte de una textura cae en cada punto de una cara, U a lo ancho de la textura y V hacia abajo. Las caras de brush suelen obtenerlas de una proyección, mientras que las mallas y algunas caras guardan las suyas para cada esquina.

## worldspawn { #worldspawn }
La entidad raíz del mapa, que contiene los brushes del mundo. Sus claves guardan ajustes de todo el mapa, como el cielo, la niebla, el sol y el streaming, y el Inspector la muestra cuando no hay nada seleccionado.

## yaw { #yaw }
Rotación alrededor del eje vertical, la dirección hacia la que mira algo visto desde arriba. El yaw de un destino de teletransporte fija hacia dónde miran los cuerpos al llegar.

## z-fighting { #z-fighting }
Franjas parpadeantes donde dos superficies están en el mismo plano o muy cerca. La GPU no puede saber cuál está delante, así que cada píxel muestra una u otra, y el patrón cambia al mover la cámara. Los decals lo evitan, los brushes finos puestos sobre una pared no.
