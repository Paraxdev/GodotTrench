---
source: e0934aeaf4aaffb804a36e5ff5821a38614fc413
---

# Importar mapas de Hammer y TrenchBroom { #importing-hammer-and-trenchbroom-maps }

¿Tienes un nivel de Hammer o TrenchBroom? Tráelo en lugar de reconstruirlo. *Archivo > Importar* abre un `.vmf` de Hammer, o un `.map` de TrenchBroom, Quake o Qodot, como un mapa nuevo sin guardar (en una pestaña nueva si el mapa abierto tiene cambios sin guardar). Guárdalo como `.tdm` y Godot lo construye como cualquier otro mapa.

## Pasos { #steps }

1. Abre primero el proyecto de Godot

    Abre el proyecto al que pertenece el mapa con *Godot > Abrir proyecto de Godot…*, ya que las texturas convertidas van a su carpeta de texturas.

2. Importa el mapa

    Ve a *Archivo > Importar* y elige *Archivo .vmf (Hammer)…* o *Archivo .map (TrenchBroom, Quake)…*, y luego el archivo.

    ![El menú Archivo con Importar abierto y Archivo .vmf (Hammer)… resaltado, debajo Convertir texturas y las importaciones de modelos](../assets/importing/file-import-menu.png)

3. Convierte las texturas

    Cuando hay texturas del mapa cerca de él, el editor pregunta "¿Convertir las texturas del mapa?" por las que le faltan al proyecto. Di que sí, mira [Convertir texturas](importing-textures.md).

4. Revisa la pestaña Problemas

    Abre la pestaña Problemas arriba a la izquierda, junto a Outliner e Historial. Cada material que aún le falta al proyecto aparece como una advertencia, y sus caras se dibujan como un tablero de ajedrez.

    ![La pestaña Problemas con dos advertencias, El material 'castle/brick_wall' no está en el proyecto (12 caras), y la vista 3D con esas paredes y escalones dibujados como un tablero gris](../assets/importing/issues-missing-materials.png)

    Las caras conservan el nombre del material, así que añadir la textura más tarde las arregla. Cuando estén todas las texturas, esas advertencias desaparecen.

    ![El mismo mapa tras convertir sus texturas: la pestaña Problemas dice No se encontraron problemas y las paredes muestran su textura de ladrillo](../assets/importing/textures-converted.png)

5. Guárdalo

    Usa *Archivo > Guardar como…* y guárdalo como un `.tdm` dentro del proyecto.

> [!NOTE]
> Las unidades se conservan tal cual. Los mapas de Source se construyen para unas 40 unidades por metro y los de Quake para unas 32, así que un mapa de Source sale algo más grande que en su juego, a menos que el proyecto use otra escala.

## Qué conserva la importación { #what-the-import-keeps }

| Hammer | En TrenchDot |
| --- | --- |
| Materiales de herramienta (`tools/toolsnodraw`, `toolsclip`, `toolsplayerclip`, `toolstrigger`, `toolsorigin`, ...) | Las texturas de omitir, clip y origen del proyecto y `special/trigger` |
| Caras de cielo | La textura de cielo del proyecto, `special/sky` por defecto |
| Desplazamientos | Desplazamientos con sus alturas y sus alfas de mezcla |
| `func_instance` | Un grupo con el contenido de la instancia, movido a su sitio. Los nombres de dentro reciben el fixup de la instancia, se rellenan sus reemplazos `$` y `#`, y las salidas dirigidas a `instance:name;Input` se conectan a la entidad real |
| Visgroups | Capas |
| Objetos ocultos | Nodos ocultos |
| Claves y salidas de entidades | Propiedades y salidas de entidades |

Las capas y los grupos de TrenchBroom siguen siendo capas y grupos, y las texturas de herramienta de Quake se convierten en las texturas de herramienta del proyecto de la misma forma.

¿Solo necesitas unos pocos brushes? Cópialos en TrenchBroom y pulsa Ctrl+V en TrenchDot, llegan convertidos del mismo modo, mira [Biblioteca de prefabs](prefab-library.md#pasting-from-trenchbroom).

Las caras de cielo colisionan en Godot pero no dibujan nada. La clave `sky_source` de worldspawn conserva la textura que tenían, y al convertir las texturas se convierte en un cielo procedural, mira [Convertir texturas](importing-textures.md#skies).

## Limitaciones conocidas { #known-limitations }

| Limitación | Qué hacer |
| --- | --- |
| Las texturas dentro de archivos `.vpk` o `.pak` no se leen | Extráelas primero, por ejemplo con VPKEdit |
| Las instancias se buscan junto al mapa y en las carpetas superiores | Conserva la estructura de la carpeta `maps` del juego. Las instancias que no se encuentran siguen siendo entidades `func_instance` |
| Las clases de entidad de Valve y Quake no tienen definiciones en un proyecto nuevo, así que el editor no conoce sus claves | Añádelas a tu FGD, o reemplázalas por tus propias clases. La pestaña Problemas las lista |
| Se omiten algunas funciones de materiales de Source: mezclas `$basetexture2`, texturas de detalle, env maps y proxies | Configúralas en Godot |
| Los parches de Quake 3 (superficies curvas) y las primitivas de brush se omiten. La barra de estado dice cuántos | Reconstrúyelos como mallas |
| Los desplazamientos cuyos vértices se mueven de lado, no solo arriba y abajo, se convierten en mallas sin alfas de mezcla | Edítalos como mallas |
