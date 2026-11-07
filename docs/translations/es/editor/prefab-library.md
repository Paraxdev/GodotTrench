---
source: be5056a246dacf54d3a4aecafff90bff3f0d1aad
---

# Biblioteca de prefabs { #prefab-library }

La pestaña **Prefabs** de abajo, junto a Materiales y Modelos, guarda piezas ya hechas para pegar en un mapa: escaleras, puertas, pilares, salas pequeñas y detalles. Piensa en las colecciones de prefabs de las que copian los mappers de TrenchBroom, pero integradas en el editor.

![La pestaña Prefabs con Buscar prefabs, la lista de categorías, el control deslizante del tamaño de las tarjetas y una tarjeta por cada pieza integrada, con la escalera en zigzag bajo el cursor y su tooltip: Stairs/Switchback stair, 192 × 128 × 352 unidades, Integrado](../assets/prefab-library/prefabs-panel.png)

Cada tarjeta dibuja su pieza desde el lado que más muestra: desde arriba para plantas como una escalera, desde el frente para paredes como una puerta. Más claro significa más cerca.

## Pegar una pieza { #pasting-a-piece }

1. Apunta al sitio

    Pasa el cursor por el sitio de una vista donde debe ir la pieza, eso fija el cursor 3D.

2. Haz clic en la tarjeta

    Escribe en *Buscar prefabs* o elige una categoría para encontrarla, y haz clic en ella. La pieza se pega centrada en ese sitio y ajustada a la rejilla, como Ctrl+V.

    ![La escalera en zigzag pegada en una sala pequeña, con sus escalones en los materiales dev grises](../assets/prefab-library/pasted-switchback-stair.png)

Solo el punto de la planta sigue al cursor. La altura se queda como se guardó la pieza, así que en un piso superior súbela después.

El pegado es una copia simple sin enlace de vuelta a la biblioteca, así que remodélala o retexturízala con libertad. Las piezas integradas usan los materiales `dev/` que tiene todo proyecto, retexturízalas desde la pestaña Materiales.

Clic derecho en una tarjeta para *Pegar* y *Copiar*. Copiar pone la pieza en el portapapeles, para pegarla en otro mapa o compartirla como texto.

Para cubrir una zona entera con piezas elegidas al azar, usa [Relleno aleatorio](random-fill.md).

> [!TIP]
> Una pieza es contenido de mapa normal una vez pegada. Para algo que deba mantenerse sincronizado en todos los sitios donde se use, como un marco de ventana repetido por un edificio, haz en cambio una [instancia de prefab](organizing.md#prefabs).

## Tus propias piezas { #your-own-pieces }

1. Selecciona los objetos

    Selecciona lo que debe convertirse en la pieza, con un proyecto de Godot abierto.

2. Añade la selección

    Abre **Añadir la selección** arriba de la pestaña Prefabs, ponle un nombre y una categoría a la pieza, y haz clic en **Añadir**.

    ![Añadir la selección abierto, con Grand stair como nombre, Stairs como categoría, Mantener posición sin marcar y el tooltip de Añadir: Guarda la selección como .tdm en res://prefab_library, con su centro inferior en el origen](../assets/prefab-library/add-the-selection.png)

3. Busca la tarjeta nueva

    Aparece la primera, con un marco azul. Clic derecho en ella para *Abrir para editar*, que abre la pieza como un mapa, y *Mostrar en el gestor de archivos*. Las piezas integradas no se pueden editar.

    ![Menú de clic derecho en la tarjeta Grand stair con Pegar, Copiar, Abrir para editar bajo el cursor y Mostrar en el gestor de archivos](../assets/prefab-library/card-menu.png)

La pieza se guarda como un `.tdm` en `res://prefab_library/<category>/`, movida para que el centro inferior de la selección quede en el origen. Marca **Mantener posición** para dejarla donde está, para una pieza que construiste en torno al origen del mapa a propósito, como las piezas de un kit de [relleno aleatorio](random-fill.md).

Cada `.tdm` de esa carpeta es una pieza. Guarda mapas ahí tú mismo con *Archivo > Guardar como…* y aparecen como tarjetas al instante. Para archivos que copiaste a mano, o una carpeta de piezas compartida entre proyectos, pulsa el botón ⟳.

## Pegar desde TrenchBroom { #pasting-from-trenchbroom }

TrenchDot también pega lo que TrenchBroom pone en el portapapeles. Copia brushes en TrenchBroom, o de una colección de prefabs de TrenchBroom en la web, y pulsa Ctrl+V en un mapa. Los grupos llegan como grupos, las entidades como entidades, y los ejes se giran igual que al [importar un .map](importing.md). Los nombres de textura se conservan, las texturas de herramienta de otros juegos pasan a ser las del proyecto.

<div class="td-mcp" markdown>

## MCP { #mcp }

La herramienta `prefab_library` lista piezas (`op: list`), pega una por su `category/name` (`op: paste`, con `origin` u `offset`), devuelve su texto del portapapeles (`op: copy`) y añade objetos a la biblioteca del proyecto (`op: add`, la selección salvo que se den `ids`). `run_action` con `paste` acepta texto del portapapeles de TrenchBroom además del de TrenchDot.

</div>
