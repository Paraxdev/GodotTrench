---
source: 755d17f52aebf0b32513b6e22a04abb368badeab
---

# Construir mapas { #building-maps }

TrenchDot guarda un archivo `.tdm` y nunca escribe escenas de Godot por sí mismo. En Godot, el nodo `FuncGodotMap` del addon lee ese archivo y genera las mallas, la colisión y los nodos de entidad. El resultado se guarda con tu escena como cualquier nodo hecho a mano, así que nada se reconstruye en tiempo de ejecución a menos que lo pidas.

## El nodo FuncGodotMap { #the-funcgodotmap-node }

1. Añade el nodo

    En el editor de Godot, pulsa Ctrl+A en tu escena, busca `FuncGodotMap` y añádelo.

2. Apúntalo a tu mapa

    Con el nodo seleccionado, fija *Local Map File* en el Inspector de Godot al `.tdm` que guardaste.

3. Constrúyelo

    Haz clic en **Build Map** arriba del Inspector. Deberías ver el mapa en el viewport de Godot, y el nodo recibe hijos para las mallas y las entidades. Guarda la escena con Ctrl+S para conservarlos. **Clear Map** los vuelve a quitar.

Estos son los ajustes que ves en el Inspector:

| Propiedad | Qué hace |
| --- | --- |
| Local Map File | La ruta `res://` del `.tdm` que se construye. Es la opción normal. |
| Global Map File | Un archivo de mapa fuera del proyecto. Cuando se fijan los dos, gana este. |
| Auto Rebuild On Save | Reconstruye el nodo cada vez que TrenchDot guarda este mapa, mientras su escena es la pestaña actual del editor de Godot, mira [Reconstruir al guardar](live.md#rebuild-on-save). Activado por defecto. |
| Map Settings | De dónde salen las texturas, los materiales y las definiciones de entidad, y cuánto mide una unidad del mapa. Si se deja vacío, usa el ajuste de proyecto `func_godot/default_map_settings`, mira [Configuración del proyecto](project-setup.md). Un nodo de mapa añadido con un addon antiguo guardaba aquí los ajustes de mapa por defecto del addon, y su construcción avisa de ello en el Output de Godot, vacía el campo para seguir el ajuste de proyecto. |
| Use Baked Lighting | Ilumina el mapa con la iluminación [precalculada en el editor](baked-lighting.md). Activado por defecto, desmárcalo para usar otro lightmapper. |
| Warm Up Shaders | Compila los shaders del mapa mientras carga, para que los primeros segundos de juego no den tirones. Mira [Precalentamiento de shaders](warm-up.md). |

El addon solo construye archivos `.tdm`. Para usar un `.map` de Quake o un `.vmf` de Hammer, ábrelo con *Archivo > Importar* en TrenchDot y guárdalo antes como `.tdm`.

> [!WARNING]
> Una construcción borra todos los hijos del nodo del mapa salvo los [overlays](overlays.md). Pon tus propios nodos junto al nodo del mapa, o en un overlay.

## Tres formas de construir { #three-ways-to-build }

| Disparador | Qué se construye | Cuándo usarlo |
| --- | --- | --- |
| Guardar en TrenchDot | Cada `FuncGodotMap` que use ese archivo en la escena abierta en el editor de Godot | El bucle de cada día, guardar y mirar el resultado en Godot |
| *Godot > Construir en Godot* | El mapa tal como está ahora en TrenchDot, guardado o no, sin escribir el archivo | Para probar en Godot un cambio sin guardar sin comprometerte con él |
| **Build Map** en Godot | El archivo guardado | Cuando TrenchDot está cerrado, o para reconstruir una escena que acabas de abrir |

Las dos primeras necesitan un editor de Godot con este proyecto abierto, y solo tocan la escena abierta allí. Una reconstrucción marca esa escena como modificada, así que guárdala en Godot para conservar el resultado.

Hasta que Godot tenga el proyecto abierto, *Construir en Godot* sigue en gris. También necesita que el mapa se haya guardado una vez dentro del proyecto, para que Godot sepa qué `FuncGodotMap` lo usa.

![El menú Godot con Construir en Godot resaltado](../assets/building/menu-build-in-godot.png)

## Construir desde código { #building-from-code }

También puedes construir un mapa mientras el juego se ejecuta, por ejemplo para cargar un nivel elegido desde un menú:

```gdscript
var map := FuncGodotMap.new()
map.map_settings = load("res://demo/demo_map_settings.tres")
map.local_map_file = "res://demo/maps/scripted_scene.tdm"
add_child(map)
map.build()
```

El nodo emite `build_complete` cuando termina, o `build_failed` cuando falta el archivo del mapa o no es un `.tdm`.

## Entorno y sol { #environment-and-sun }

Las claves de cielo, niebla y sol de worldspawn construyen un `WorldEnvironment` y una luz llamada `sun` dentro del mapa, así que un mapa se ve iluminado sin ninguna configuración en Godot. Pulsa Escape para que no haya nada seleccionado, y el Inspector muestra entonces *Map (worldspawn)* con estas claves listadas. Su clave `environment` elige qué se construye. Una clave que dejas sin fijar se queda fuera del mapa, así que un mapa sin ninguna de ellas no construye ningún entorno. Si un proyecto hecho con un addon antiguo no lista las claves, actualiza el addon y [exporta otra vez la configuración de juego](project-setup.md#the-game-config).

| `environment` | Construye |
| --- | --- |
| `1` | El entorno y el sol. Es el valor por defecto. |
| `sun_only` | Solo el sol, para un juego que trae su propio `WorldEnvironment` |
| `0` o `none` | Nada |

Si la escena ya tiene un `WorldEnvironment` o un `DirectionalLight3D` propios, el mapa los conserva y no añade ninguno, así que un juego con su propio aspecto siempre gana. Cuando varios mapas comparten una escena, el primero que se construye la ilumina.

## Informe de construcción { #build-report }

`build()` devuelve un Dictionary, que también se guarda en `build_report`. Te dice cuánto tardó cada paso, cuántas entidades de cada clase se construyeron y cuántos vértices tiene el mapa. Lo más útil es la lista de lo que la construcción no pudo encontrar, que suele ser una errata o un archivo que se renombró o nunca se subió al repositorio:

| Clave | Qué lista | Qué ves en el juego |
| --- | --- | --- |
| `missing_classes` | Clases de entidad que ninguna definición de entidad describe | Una entidad brush conserva su malla pero no tiene script ni colisión, una entidad punto es un `Marker3D` vacío, mira [Entidades que no hacen nada](entities-troubleshooting.md) |
| `missing_models` | Rutas de modelo que no existen, con las entidades que usan cada una | El prop no tiene modelo |
| `missing_materials` | Texturas de cara sin archivo de material ni imagen | Las caras muestran la textura de marcador |
| `unknown_tool_textures` | Texturas de herramienta como `special/clip` que los ajustes del mapa nombran de otra forma | Las caras siguen sin dibujar nada, pero un brush de origen no fija el origen de su entidad |
| `unresolved_targets` | Outputs y claves target cuyo destino no coincide con ningún targetname del mapa ni de sus overlays | El output se dispara contra nada |

Para imprimirlo tras cada construcción, marca *Print Build Report* bajo *Build Flags* en el nodo, o añade `FuncGodotMap.BuildFlags.PRINT_REPORT` a `build_flags` en código. Un test o una comprobación de CI puede leer el Dictionary en su lugar:

```gdscript
var report := map.build()
if not report["missing_models"].is_empty():
	push_error("missing models: %s" % [report["missing_models"]])
```

Cada construcción también imprime una advertencia en el Output de Godot por lo que no pudo encontrar: una para todas las clases que faltan, y una por cada modelo, material o textura de herramienta que falta, por muchos props que compartan la ruta. Tras un guardado, TrenchDot lista las mismas advertencias en su panel Problemas.

> [!NOTE]
> Los overlays que añades después de `build()` no están cuando el informe busca destinos, así que sus targetnames aparecen en `unresolved_targets`.

## Mapas dañados { #damaged-maps }

Un mapa dañado aún se construye a partir de las partes que se pudieron leer, con una advertencia `[TDM]` por cada parte que no. Mira [el formato de mapa .tdm](../format/recovery.md).
