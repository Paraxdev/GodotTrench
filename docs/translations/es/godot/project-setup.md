---
source: 0b6155b82bf9e0fff2345e05627d7c675a868912
---

# Configuración del proyecto { #project-setup }

Los valores por defecto del addon sirven para tus primeros mapas. Cuando escribas tus propias entidades o quieras otra carpeta de texturas, dale al proyecto sus propios recursos, porque una actualización del addon reemplaza los valores por defecto.

## Instalar y actualizar el addon { #installing-and-updating-the-addon }

Cada vez que abres un proyecto, TrenchDot revisa su addon. Cuando falta `addons/func_godot` o tiene una versión distinta a la del editor, la ventana *Configurar este proyecto* aparece una vez, y una nota amarilla se queda abajo a la derecha de la barra de estado. Pasa el ratón por encima para ver las dos versiones, haz clic para traer la ventana de vuelta.

![La barra de estado con la nota amarilla "Addon 0.0.9, el editor 0.1.0" resaltada y su tooltip](../assets/project-setup/status-addon-mismatch.png)

También puedes abrir la ventana cuando quieras con *Godot > Instalar o actualizar addon…*.

![El menú Godot con Instalar o actualizar addon… resaltado](../assets/project-setup/menu-install-update-addon.png)

Solo ofrece el paso que tu proyecto necesita a continuación:

| El proyecto tiene | La ventana ofrece |
| --- | --- |
| Ningún addon | **Instalar el addon**, y luego **Activar el plugin** |
| Un addon más antiguo | **Actualizar el addon a** la versión del editor |
| Un addon más nuevo, o uno sin versión | **Reemplazar el addon por** la versión del editor |
| El addon, sin activar | **Activar el plugin**, que lo añade a `[editor_plugins]` en `project.godot` y no cambia nada más |
| El addon como checkout de git o submódulo | Nada. Actualízalo con git, el editor nunca lo reemplaza |

![La ventana Configurar este proyecto diciendo que el addon 0.0.9 es más antiguo que el editor, con el botón Actualizar el addon a 0.1.0](../assets/project-setup/setup-update-addon.png)

> [!WARNING]
> Cierra Godot antes de actualizar. Sigue ejecutando el addon antiguo hasta que se reinicie, así que el botón queda en gris mientras el Live Link ve a Godot con el proyecto abierto. Activa también el plugin con Godot cerrado, o Godot vuelve a escribir su propia copia de `project.godot`. Marcarlo en Godot, en *Project > Project Settings > Plugins*, funciona igual.

El addon antiguo no se borra, pasa a `.trenchdot/addon-backups/func_godot-<version>` en tu proyecto. Godot se salta las carpetas que empiezan por un punto, así que ni importa la copia ni la lista como un segundo plugin. Bórrala cuando el addon nuevo funcione. (Nota: la descarga se desempaqueta primero y solo después se cambia, así que una actualización fallida o cancelada deja el addon antiguo como estaba.)

Si la descarga falla, la ventana enlaza a las [releases](https://github.com/Paraxdev/TrenchDot/releases) y a [itch.io](https://paraxdev.itch.io/trenchdot), que traen el mismo `func_godot-trenchdot-addon.zip`. Extráelo en la carpeta del proyecto para obtener `addons/func_godot`, y reemplaza la carpeta antigua en lugar de desempaquetar encima.

## Entidades básicas y el paquete de entidades de gameplay { #core-entities-and-the-gameplay-entities-pack }

Todo proyecto recibe del addon un pequeño núcleo: worldspawn, `func_detail`, `func_illusionary`, `func_geo`, `light`, `light_spot`, `light_probe`, `env_reflection_probe`, `env_voxel_gi` e `info_player_start`.

Las puertas, los botones, los ascensores, los trenes, los triggers, la lógica, los spawners, los teletransportes, los path corners, los props, los efectos y el resto de la [referencia de entidades](../gameplay/entities/README.md) vienen como el paquete de entidades de gameplay. Lo instalas cuando lo quieras, y se copia en `res://trenchdot/entities`:

| Ruta | Contiene |
| --- | --- |
| `gameplay_fgd.tres` | El archivo FGD del paquete, que lista cada definición |
| `definitions/` | Una definición por entidad: sus claves, inputs, outputs y gizmos del editor |
| `scripts/` | El GDScript que ejecuta cada entidad |
| `pack.json` | Lo que se instaló, para que las actualizaciones distingan tus ediciones. Déjalo como está |

Las copias pertenecen a tu proyecto. Cambia un script o una definición, borra las entidades que no uses o añade tus propias definiciones a `gameplay_fgd.tres`. El addon nunca define estos classnames ni clases de script por sí mismo, así que nada de lo que hagas ahí choca con él, y una actualización del addon no lo deshace.

### Instalar y actualizar el paquete { #installing-and-updating-the-pack }

1. Instala el paquete

    Haz clic en **Instalar entidades de gameplay** en la ventana *Configurar este proyecto*, debajo del addon.

    ![La ventana Configurar este proyecto con la sección Entidades de gameplay y su botón Instalar entidades de gameplay](../assets/project-setup/setup-gameplay-entities.png)

    Mientras el proyecto solo tiene el núcleo, el mismo botón está arriba del menú Gameplay y en el panel Entidades.

    ![El menú Gameplay con Instalar entidades de gameplay resaltado y su tooltip](../assets/project-setup/menu-install-gameplay-entities.png)

2. Comprueba que funcionó

    La ventana debería decir ahora "Instalado en res://trenchdot/entities" en verde, y el editor ofrece las entidades nuevas al instante.

    ![La sección Entidades de gameplay después de instalar, diciendo que añadió 86 archivos a res://trenchdot/entities y que reinicies Godot si tiene el proyecto abierto](../assets/project-setup/setup-gameplay-installed.png)

    Cuando un editor de Godot tiene el proyecto abierto, TrenchDot le pide que recoja los archivos nuevos, y la ventana lo dice. Una escena en [modo en vivo](live.md#when-live-mode-rebuilds-the-whole-map) se reconstruye entonces por sí sola. Pulsa **Build Map** en las demás escenas que usan tus mapas y guárdalas, ya que se construyeron sin el paquete. Cuando Godot no está en marcha, o ejecuta un addon más antiguo, la ventana dice que reinicies Godot si tiene el proyecto abierto, y que luego construyas y guardes esas escenas.

3. (Opcional) Actualízalo más adelante

    Cuando una actualización del addon trae un paquete más nuevo, la ventana ofrece **Actualizar entidades de gameplay**, y el menú Gameplay lo muestra con el número de archivos cambiados. No se actualiza nada hasta que haces clic. La actualización añade archivos nuevos y reemplaza los que no tocaste. Los archivos que editaste se quedan como están y el editor los nombra, y los archivos que borraste siguen borrados.

Los mapas antiguos hechos antes de que las entidades salieran del addon siguen funcionando. Cuando los mapas de un proyecto usan entidades del paquete que nada en el proyecto define, el paquete se instala solo, tanto cuando el editor abre el proyecto como cuando Godot arranca con el addon actualizado. Un archivo `.tdm` conserva cada entidad de todos modos, con o sin definición.

> [!NOTE]
> La plantilla dentro del addon, `addons/func_godot/gameplay_pack`, tiene un archivo `.gdignore` para que Godot la salte. Conserva ese archivo, o Godot encuentra cada clase de script del paquete dos veces una vez instalado.

### Qué definición gana { #which-definition-wins }

El FGD básico, `res://addons/func_godot/fgd/trenchdot/trenchdot_fgd.tres`, lee el paquete instalado por sí mismo, así que todo FGD construido sobre los valores por defecto del addon recibe el paquete sin listarlo. Cuando dos definiciones comparten un classname, gana la posterior, en este orden:

1. Las clases de FuncGodot y el núcleo del addon
2. El paquete instalado en `res://trenchdot/entities`
3. Las entidades de tu propio archivo FGD

Para cambiar una entidad, edita su copia en el paquete, o define el mismo classname en tu propio FGD. Las dos formas funcionan, pero un classname definido dos veces es fácil de olvidar, así que el editor te lo dice: cuando se abre el proyecto, la barra de estado nombra la definición que gana, y el panel Problemas avisa de cada clase definida dos veces que usa un mapa. Borra una de las dos para resolverlo.

## Tus propios recursos { #your-own-resources }

Tres recursos trabajan juntos. El archivo FGD lista las entidades que pueden usar tus mapas. Los ajustes del mapa le dicen a una construcción dónde están las texturas y qué FGD leer. La configuración de juego le cuenta a TrenchDot ambas cosas, para que el editor ofrezca las mismas entidades y texturas mientras editas.

El camino más rápido es copiar `demo_fgd.tres`, `demo_map_settings.tres` y `demo_game_config.tres` de [`godot/demo`](https://github.com/Paraxdev/TrenchDot/tree/main/godot/demo) a tu proyecto y corregir las rutas. Para crearlos tú mismo, haz esto en el editor de Godot:

1. Crea un `FuncGodotFGDFile`. Pon *Base Fgd Files* a `res://addons/func_godot/fgd/trenchdot_default_fgd.tres`, para conservar el núcleo y el paquete instalado, y luego añade tus propias entidades en *Entity Definitions*. Sin esa base tu FGD no tiene ninguna de ellas, y hasta worldspawn se construye sin colisión.
2. Crea un `FuncGodotMapSettings`. Pon *Entity Fgd* al FGD del paso 1 y *Base Texture Dir* a tu carpeta de texturas (por defecto `res://textures`). Un recurso de ajustes del mapa nuevo empieza en `trenchdot_default_fgd.tres`, que tiene el núcleo y el paquete pero no tus propias entidades.
3. Crea un `TrenchDotGameConfig`. Pon su *Fgd File* y *Map Settings* a los dos recursos de arriba. (Opcional) pon *Game Name*, el editor lo muestra arriba a la derecha de la barra de herramientas junto a la carpeta del proyecto.
4. En *Project > Project Settings*, apunta `func_godot/default_map_settings` a tus ajustes del mapa y `trenchdot/game_config` a tu configuración de juego. Todo nodo de mapa con el campo *Map Settings* vacío la sigue. Un nodo añadido con un addon antiguo guardó los valores por defecto del addon en ese campo, así que vacíalo ahí.

El editor ofrece las entidades del FGD de la configuración de juego, pero Godot construye con el FGD de los ajustes del mapa, así que mantén los dos iguales. Cuando difieren, Godot avisa en su panel Output cada vez que exporta la configuración de juego, y el panel Problemas de TrenchDot nombra cada entidad que le falta al FGD con el que construye Godot.

> [!TIP]
> Pon *Entity Name Property* en los ajustes del mapa a `targetname`, como hace la demo. Los nodos construidos se llaman entonces como sus targetnames, lo que mantiene legible el árbol de escena.

## La configuración de juego { #the-game-config }

TrenchDot aprende tus entidades, carpetas de texturas y escala de unidades de `res://trenchdot_game.json`, que escribe la configuración de juego. El addon lo exporta de nuevo cada vez que Godot nota cambios en los archivos, y el editor lo recarga en un segundo. La exportación también nombra los ajustes del mapa por defecto y las clases que define su FGD, que es como el editor avisa de las entidades que Godot construiría sin definición. Para exportar a mano, usa *Project > Tools > TrenchDot: Export Game Config* en Godot, o ejecútalo sin ventana, por ejemplo en CI:

```sh
godot --headless --path <project> --script res://addons/func_godot/src/trenchdot/export_game_config_cli.gd
```

## Ajustes del proyecto { #project-settings }

La mayoría de los proyectos solo cambian el primero y el tercero. Los del Live Link importan cuando otro programa ya usa el puerto.

| Ajuste | Por defecto | Para qué sirve |
| --- | --- | --- |
| `func_godot/default_map_settings` | los valores por defecto del addon | Ajustes del mapa para todo mapa que no pone los suyos |
| `func_godot/default_inverse_scale_factor` | 32 | Unidades de mapa por metro |
| `trenchdot/game_config` | la configuración del addon | La configuración que escribe `trenchdot_game.json` |
| `trenchdot/auto_export_game_config` | true | Exportar de nuevo la configuración de juego cuando cambian los archivos |
| `trenchdot/live_link_enabled` | true | Ejecutar el servidor del Live Link con el que habla TrenchDot |
| `trenchdot/live_link_port` | 7842 | Debe coincidir con *Live Link de Godot* en las preferencias del editor. Los cambios se aplican al instante. Entre 1024 y 65535, cualquier otro valor vuelve a 7842 con una advertencia |
| `trenchdot/threaded_build` | true | Convertir brushes y chunks de terreno en el WorkerThreadPool, para que los mapas grandes se construyan más rápido |
| `trenchdot/live_chunk_size` | 16.0 | En modo en vivo, la geometría de worldspawn se divide en chunks de este tamaño en metros, así que una edición solo reconstruye su chunk |
| `trenchdot/csharp_entity_dirs` | `res://` | Dónde se encuentran las entidades C# al construir |
