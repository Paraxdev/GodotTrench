---
source: 078f68d4f428db30dc2dfcb8561a8991ada14154
---

# Convertir texturas { #converting-textures }

Godot no puede cargar archivos `.vtf`, `.wad` ni `.wal`, así que el editor los convierte en PNG, más un `.tres` donde una imagen simple no puede llevar los ajustes. Cada uno acaba en la carpeta de texturas del proyecto con el nombre de material de la cara, para que las caras importadas lo encuentren enseguida.

Al importar un mapa se ofrece esto para las texturas que usa, mira [Importar mapas de Hammer y TrenchBroom](importing.md). Para convertir una carpeta entera, abre primero el proyecto de Godot y luego:

1. Abre el conversor

    Ve a *Archivo > Importar > Convertir texturas (VTF/VMT, WAD, WAL)…*.

    ![El menú Archivo con Importar abierto y Convertir texturas (VTF/VMT, WAD, WAL)… resaltado](../assets/importing-textures/file-import-convert-textures.png)

2. Elige la carpeta

    Elige la carpeta con los archivos `.vmt` y `.vtf`, `.wad` o `.wal`. Si al mapa abierto le faltan algunas, el editor pregunta si convertir solo **Las del mapa** o **Todo**.

    La barra de estado debería decir entonces cuántas se convirtieron, como "Texturas: 12 converted", y aparecen en la pestaña Materiales con sus nombres de material.

    ![La pestaña Materiales buscando castle, con brick_wall y stone_step de un mapa importado, y el tooltip castle/brick_wall, 1024 x 1024 px](../assets/importing-textures/converted-in-materials.png)

## Dónde se encuentran { #where-they-are-found }

El editor busca en los sitios habituales donde cada juego guarda sus texturas, así que un mapa copiado junto con las carpetas de su juego las encuentra sin configurar nada.

| Origen | Dónde se busca | Nombre que reciben |
| --- | --- | --- |
| `.vmt` y `.vtf` de Valve | una carpeta `materials` junto al mapa o hasta cuatro carpetas más arriba | la ruta bajo `materials`, como `brick/brickwall001a` |
| `.wad` de Quake y Half-Life | la clave `wad` de worldspawn, y luego el mismo nombre de archivo en carpetas cercanas | el nombre de la textura, como `{grate` |
| `.wal` de Quake 2 | una carpeta `textures` cercana | la ruta bajo `textures`, como `e1u1/floor1_1` |
| `.png`, `.jpg`, `.tga` sueltos | una carpeta `textures` cercana, como las traen los proyectos de Qodot y FuncGodot | la ruta bajo `textures`, copiada tal cual |

## Materiales de Valve { #valve-materials }

Un archivo `.vmt` de Valve describe un material como una lista de parámetros. Estos pasan al material de Godot, el resto se omite:

| Material de Valve | Material de Godot |
| --- | --- |
| `$basetexture` | albedo, con su alfa descartado salvo que el material sea transparente |
| `$bumpmap`, o `$normalmap` en el agua | mapa normal, con el verde invertido de DirectX a Godot |
| `$translucent`, `$additive` | mezcla alfa para `$translucent`, mezcla aditiva para `$additive` |
| `$alphatest`, `$alphatestreference` | alpha scissor con ese umbral, 0,5 por defecto |
| `$selfillum`, `$selfillummask` | una textura de emisión brillante, enmascarada por el alfa de la textura base o por la máscara |
| `$nocull`, `UnlitGeneric` | doble cara para `$nocull`, sin sombreado para `UnlitGeneric` |
| `$color` | color de albedo |
| `$surfaceprop` | `metadata/surfaceprop` en el material |
| `patch` con `include` | un patch reutiliza otro material con unos pocos cambios, así que se convierte en el material incluido con sus cambios `insert` y `replace` aplicados |

## Texturas de Quake { #quake-textures }

Las texturas que empiezan por `{` se recortan con alpha scissor, los cielos no llevan sombreado y `*water` se guarda como `water`. Las texturas animadas, incluidos los VTF de varios fotogramas, conservan su primer fotograma.

## Cielos { #skies }

Convertir las texturas de un mapa también fija su cielo, así que el mapa importado recibe un cielo parecido al original. Se omite cuando el mapa ya tiene `sky_top_color`.

Un mapa de Source con su skybox `skyname` en `materials/skybox` recibe un panorama, `skybox/<skyname>_panorama.png` en la carpeta de texturas, fijado como `sky_panorama`. Los demás mapas reciben `sky_top_color` y `sky_horizon_color` a partir de la textura de `sky_source`. Cuando la escena no tiene un WorldEnvironment propio (el nodo de Godot que guarda el cielo, la niebla y la luz ambiental de una escena), `TrenchDotEnvironment` construye uno a partir de estas claves.

## Formatos VTF { #vtf-formats }

El conversor lee VTF de las versiones 7.0 a 7.5 en estos formatos de píxel: DXT1 (con alfa de un bit), DXT3, DXT5, los órdenes RGB y RGBA de 8 bits, I8, IA88, A8, RGB565, BGR565, BGRA4444, BGRA5551, BGRX5551, UV88, UVWQ8888, UVLX8888 y los formatos RGBA de 16 bits. Los demás formatos, y VTF 7.6, se rechazan indicando el formato, para que sepas qué archivo convertir a mano.

<div class="td-mcp" markdown>

## MCP { #mcp }

Por el [servidor MCP](../mcp.md), convierte una carpeta de texturas con `map_file {op: convert_textures, path}`. Añade `only_missing` para convertir solo lo que le falta al mapa abierto, y `overwrite` para reemplazar los archivos que ya están en el proyecto.

</div>
