---
source: 1b2a2ceda473607c8db087c552aff86851d08ef2
---

# Entidades que no hacen nada { #entities-that-do-nothing }

Cuando Godot construye una entidad sin su definición, la entidad es solo una cáscara. Una entidad brush como una puerta conserva su malla, pero se convierte en un `Node3D` normal sin script ni colisión, y una entidad punto se convierte en un `Marker3D` vacío. Lo mismo ocurre cuando una definición no encaja con su entidad, por ejemplo una entidad brush cuya definición no construye ningún cuerpo físico. TrenchDot y Godot te avisan los dos cuando esto pasa, y esta página explica cada mensaje.

## Dónde lo ves { #where-you-see-it }

1. La barra de estado

    Guarda el mapa en TrenchDot mientras su escena está abierta en Godot. Cuando Godot construyó clases para las que no tiene definición, la barra de estado de abajo a la izquierda las nombra, junto con los ajustes del mapa con los que construyó Godot:

    ![La barra de estado diciendo que Godot reconstruyó 1 nodo de mapa, pero res://crypt_settings.tres no tiene definición para func_button y func_door, así que se construyen como nodos normales](../assets/entities-troubleshooting/status-missing-classes.png)

2. El panel Problemas

    Abre la pestaña **Problemas**, junto a Outliner e Historial a la izquierda. Lista cada clase que Godot construyó como un nodo normal, con la solución, y las demás advertencias de esa construcción empiezan por "Godot:". Haz clic en una fila para seleccionar la entidad.

    ![El panel Problemas con dos advertencias, Godot construyó func_button y func_door como nodos normales, cada una indicando qué archivo añadir a Base Fgd Files](../assets/entities-troubleshooting/issues-missing-classes.png)

    Antes de que Godot haya construido el mapa, el panel Problemas compara tus entidades con el FGD con el que construye Godot, que el addon exporta con la [configuración de juego](project-setup.md#the-game-config). Una entidad que ese FGD no tiene recibe la advertencia "Godot construye con ..., cuyo FGD ... no tiene ...".

    Cuando Godot muestra una escena que construyó antes de que hubiera una definición, la barra de estado y el panel Problemas dicen "Godot muestra ..., construido con otras definiciones de entidades ...". Elige *Godot > Construir en Godot*, o pulsa **Build Map** en Godot y guarda, para construirla de nuevo. **Ejecutar proyecto** dice lo mismo sobre las escenas que ejecuta.

3. El panel Output de Godot

    Pulsa **Build Map** en el nodo `FuncGodotMap` en Godot. El panel Output de abajo del editor de Godot muestra una advertencia `[TrenchDot]` por las clases sin definición y una por cada definición que no encaja. Marca *Print Build Report* bajo *Build Flags* en el nodo para ver el [informe de construcción](building.md#build-report) completo.

## Entidades sin definición { #entities-without-a-definition }

| El mensaje dice | Por qué | Haz esto |
| --- | --- | --- |
| Instala el paquete de entidades de gameplay | Las puertas, los botones, los triggers, los props y los spawners vienen con el paquete, y el proyecto no lo tiene | Elige *Gameplay > Instalar entidades de gameplay* en TrenchDot, mira [Configuración del proyecto](project-setup.md#installing-and-updating-the-pack) |
| A tu FGD le falta el paquete de entidades de gameplay, añade `trenchdot_default_fgd.tres` a sus Base Fgd Files | Tu FGD no tiene *Base Fgd Files* | Añade `res://addons/func_godot/fgd/trenchdot_default_fgd.tres` a *Base Fgd Files* del FGD que los ajustes del mapa nombran en *Entity Fgd* |
| El paquete está instalado, pero el *Entity Fgd* es un FGD del addon | Los ajustes del mapa usan el FGD propio de FuncGodot u otro archivo del addon, que una actualización del addon reemplaza | Fija *Entity Fgd* de esos ajustes del mapa al FGD que usa tu configuración de juego |
| Una clase propia no tiene definición | Está en el FGD de tu configuración de juego, pero no en el *Entity Fgd* de los ajustes del mapa con los que construye Godot | Fija *Entity Fgd* de esos ajustes del mapa al FGD que usa tu configuración de juego, mira [Tus propios recursos](project-setup.md#your-own-resources) |
| El nodo del mapa usa los ajustes de mapa por defecto del addon, no los de Project Settings | El nodo se añadió con un addon antiguo, que guardaba los ajustes de mapa por defecto en la escena | Selecciona el nodo del mapa en Godot y vacía *Map Settings* en el Inspector |
| La escena se construyó con otras definiciones de entidades | Se construyó antes de que estuvieran el paquete o una definición, y no desde entonces | Pulsa **Build Map** y guarda la escena. En el [modo en vivo](live.md#when-live-mode-rebuilds-the-whole-map) Godot lo hace por sí mismo |

## Definiciones que no encajan { #definitions-that-do-not-fit }

Estos vienen del panel Output de Godot, y tras un guardado del panel Problemas como advertencias "Godot:". El panel Problemas también marca los dos últimos antes de guardar.

| El mensaje dice | Por qué | Haz esto |
| --- | --- | --- |
| `func_crusher builds as a Marker3D, so its brushes have no collision` | La definición no tiene *Node Class*, y su script extiende un nodo que no es un cuerpo físico | Fija *Node Class* a `StaticBody3D`, `AnimatableBody3D` o `Area3D`, o haz que el script extienda uno de ellos |
| `func_crusher: Node Class AnimatedBody3D not found, building it as a Node3D` | Una errata, o un `class_name` que Godot aún no ha registrado | Corrige el nombre. Para C#, mira [Entidades en C#](../gameplay/csharp.md) |
| `alarm: res://entities/alarm.gd has no @tool` | El editor solo ejecuta scripts tool, así que las claves del mapa nunca llegan al nodo | Añade `@tool` al principio del script |
| `light is a point entity, so its brushes are dropped` | Se usa un `FuncGodotFGDPointClass` para una entidad brush | Haz que la definición sea un `FuncGodotFGDSolidClass` |
| `func_door is a brush entity without brushes` | Una entidad brush sin nada con lo que construir | Dale brushes, o haz que la definición sea una clase punto |

## Entidades que se construyen pero no reaccionan { #entities-that-build-but-do-not-react }

| Lo que ves | Por qué | Haz esto |
| --- | --- | --- |
| Los triggers nunca se disparan, y el Output de Godot dice que el trigger ignoró a tu jugador porque ningún nodo está en su `filter_group` | Por defecto los triggers solo reaccionan a cuerpos del grupo `player` en la capa de física 1 | Añade tu jugador al grupo `player`. En otra capa, añade esa capa a *Trigger Collision Mask* en los ajustes del mapa |
| Las puertas y los botones nunca se mueven | Se mueven cuando el código de tu juego llama a `use()` sobre ellos | Mira [Deja que el jugador lo pulse](../tutorials/button-door.md#let-the-player-press-it) |
| `info_spawner` y `npc_walker` se ven vacíos en el editor de Godot | Solo hacen aparecer o cargan su modelo en el juego en ejecución | Mira [Spawners y rutas](../gameplay/entities/actors.md) |
| El Output de Godot muestra `Parse Error` para un script en `res://trenchdot/entities` | Godot compiló un script del paquete antes de conocer las clases que usa | Reinicia Godot, luego pulsa **Build Map** y guarda |
