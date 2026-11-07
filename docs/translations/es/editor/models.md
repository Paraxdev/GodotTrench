---
source: d2dc61c8ee306263eda3f36520611e0790a27732
---

# Modelos { #models }

Los modelos son archivos 3D hechos en otro programa, como Blender o Blockbench. TrenchDot lee archivos glTF, OBJ, Blockbench, STL, MD2 y MD3.

Los encuentras en la pestaña **Modelos** de abajo, junto a Materiales y Prefabs. Lista todo lo que hay bajo `res://models`, además de los modelos de naturaleza de `res://trenchdot/nature` una vez que los hayas instalado.

![La pestaña Modelos: campo de búsqueda, filtros de carpeta y de origen, Ordenar, Slugs, el control de tamaño de las miniaturas, Ajustar, el multiplicador de tamaño, el botón de volver a escanear e Importar…, sobre una cuadrícula de miniaturas de modelos de naturaleza](../assets/models/models-panel.png)

Un proyecto nuevo empieza sin modelos. *Godot > Añadir contenido al proyecto…* añade los modelos de naturaleza, y la opción de demo trae además los props de Poly Haven en `res://models/polyhaven`. O copia tus propios archivos en `res://models` y pulsa el botón ⟳ para volver a escanear.

## Colocar un modelo { #placing-a-model }

Un modelo entra en el mapa de una de dos maneras. Elige según quieras o no cambiar su forma en el editor.

| Colocado como | Guardado en el mapa | Úsalo para |
| --- | --- | --- |
| Malla editable | Una copia de la geometría, que ya no sigue al archivo | Formas que quieres editar con las [herramientas de malla](meshes.md) |
| Entidad `prop_model` | Solo una ruta `res://`, Godot carga el archivo | Props que dejas tal cual. Cada copia sigue al archivo cuando lo cambias |

### Como malla editable { #as-an-editable-mesh }

1. (Opcional) Ajusta el tamaño

    Ajustar, en la barra de herramientas de Modelos, viene marcado por defecto. Escala a 2 metros un modelo cuyo lado más largo mide menos de un cuarto de metro o más de 256 metros, para que los archivos diminutos o enormes entren igualmente usables. El campo × junto a él multiplica el tamaño además de eso. Los dos se aplican al colocar, así que ajústalos antes de arrastrar.

2. Arrástralo a una vista

    Arrastra una miniatura de la pestaña Modelos a cualquier vista. Cae sobre la superficie bajo el puntero.

    ![Un barril de vino arrastrado de la pestaña Modelos a la vista 3D, donde cae sobre el suelo como una malla editable](../assets/models/drag-model-into-view.gif)

Para un archivo en cualquier lugar del disco, usa **Importar…** al final de la barra de herramientas de Modelos.

### Como prop { #as-a-prop }

1. Elige el archivo

    Ve a *Archivo > Importar > Prop de modelo (.bbmodel, .glb)…* y elige el modelo.

    ![El menú Archivo con Importar abierto y Prop de modelo (.bbmodel, .glb)… resaltado](../assets/models/file-import-model-prop.png)

2. Mira el Inspector

    Cae en el cursor 3D como una entidad `prop_model`, y su clave `model` guarda la ruta `res://`.

    ![El Inspector de una entidad prop_model con la clave model apuntando a una ruta res://models/polyhaven](../assets/models/inspector-prop-model.png)

Un prop tiene que apuntar a un archivo que Godot pueda cargar. Si eliges uno fuera del proyecto, el editor ofrece copiarlo antes en `res://models`.

Los modelos de Blockbench tienen dos maneras más de entrar, *Archivo > Importar > Modelo de Blockbench como malla…* y *Modelo de Blockbench como brushes…*. Como brushes, cada cubo se convierte en un brush, agrupado bajo el nombre del archivo.

## Texturas de una malla colocada { #textures-of-a-placed-mesh }

Una malla editable conserva el aspecto del modelo. Sus texturas se copian en `models/<name>` dentro de la carpeta de texturas del proyecto, donde `<name>` es la ruta del modelo en la pestaña Modelos, o su nombre de archivo para un modelo de otro sitio. Selecciona la malla y el Inspector lista los materiales con los que se dibuja:

![El Inspector de una malla colocada: 5966 vértices y el material models/polyhaven/containers/wine_barrel_01/tex0](../assets/models/inspector-mesh-materials.png)

Donde la imagen simple no se dibujaría como el modelo, también va un material de Godot a la carpeta de materiales del proyecto. Así las hojas recortadas, el pixel art y las partes brillantes (incluidos el color y el mapa de emisión de un modelo glTF) se dibujan en Godot como aquí. Para cambiar los materiales, haz clic en uno en la pestaña Materiales con la malla seleccionada, *Editar > Deshacer* los devuelve.

Colocar nunca sobrescribe un archivo. Si colocas el mismo modelo otra vez, reutiliza sus texturas y conserva un material que ajustaste en Godot. Si las texturas de otro modelo ya están en esa carpeta, como las de `rock.bbmodel` junto a `rock.glb`, las nuevas van a una carpeta numerada como `models/rock_2`. Deshacer deja los archivos donde están. Si no se pueden escribir las texturas, por ejemplo en un proyecto de solo lectura, la malla entra sin texturas y la barra de estado dice por qué.

Las texturas cuentan como iguales por sus píxeles, así que una que otro programa solo guardó de nuevo se reutiliza igualmente. ¿Pintaste encima de una? Las mallas colocadas antes conservan tu versión, y la siguiente colocación recibe copias nuevas en una carpeta numerada, con un material nuevo. Para dar tus cambios a colocaciones posteriores, aplícales tu material desde la pestaña Materiales.

> [!NOTE]
> FuncGodot solo construye el material de una cara a partir del `default_material` del proyecto cuando no hay un archivo de material con el mismo nombre. Las caras de un modelo colocado que recibieron un archivo de material se dibujan con ese archivo, así que un shader propio puesto como `default_material` no las alcanza. Edita el archivo en Godot para cambiarlas. Un proyecto cuya extensión de material no es `tres` no recibe archivos de material, y se aplica su material predeterminado.

## Más sobre una miniatura { #more-on-a-thumbnail }

Haz clic derecho en una miniatura para colocarla en el cursor, copiar su ruta o encontrar el archivo en el disco. *Abrir página del paquete* aparece cuando su paquete tiene una url.

![El menú del clic derecho en la miniatura wine_barrel_01 con Colocar en el cursor resaltado, y después Copiar ruta, Mostrar en el gestor de archivos y Abrir página del paquete](../assets/models/thumbnail-menu.png)

## Créditos del paquete { #pack-credits }

Los paquetes de modelos gratuitos suelen pedir que se les dé crédito. Pon un `pack.json` en la carpeta de un modelo, o en cualquier carpeta por encima (gana el más cercano), y el panel sabe de qué paquete viene el modelo, quién lo hizo y con qué licencia. Pasa el puntero por una miniatura para verlo:

![El tooltip de la miniatura wine_barrel_01: su ruta, Paquete: Poly Haven, el autor, Licencia: CC0, la url del paquete y el crédito del modelo](../assets/models/thumbnail-credit-tooltip.png)

```json
{
  "name": "Nature Kit",
  "short": "Nature",
  "author": "Some Artist",
  "license": "CC0",
  "url": "https://example.com/nature-kit",
  "credits": [{ "asset": "pine", "author": "Other Artist" }]
}
```

`short` es la etiqueta de cada miniatura mientras Slugs está marcado, y si falta se usa `name`. `credits` sustituye el autor, la licencia o la url de modelos concretos, buscados por el nombre de archivo sin extensión. Las carpetas sin `pack.json` se listan bajo el nombre de su carpeta superior.
