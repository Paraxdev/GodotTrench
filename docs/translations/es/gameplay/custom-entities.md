---
source: 0c1b9cb979394402d977a6f647062a1558b8af7c
---

# Entidades propias en GDScript { #custom-entities-in-gdscript }

Cualquier script puede ser una entidad. Declara una **señal** por cada salida y un **método** por cada entrada, y emite la señal cuando ocurra la cosa. Esta alarma tiene una salida, `tripped`, y dos entradas, `trip` y `disarm`:

```gdscript
@tool
class_name Alarm extends Node3D

signal tripped(activator: Node)

@export var armed := true

func _func_godot_apply_properties(props: Dictionary) -> void:
	armed = TrenchDotIO.to_bool(props.get("armed", armed))

func trip(activator: Node = null) -> void:
	if armed:
		tripped.emit(activator)

func disarm() -> void:
	armed = false
```

Haz del activador el primer argumento `Node` de tus señales, para que las cadenas de más abajo sigan sabiendo quién las inició.

Las claves del mapa llegan a `_func_godot_apply_properties`. Léelas con `TrenchDotIO.to_bool`, `to_vector3` y `to_color`, que aceptan tanto texto como valores ya convertidos. Los nodos que añades ahí se guardan con la escena. Los que añades en `_ready` no, se vuelven a crear cada vez que se carga la escena, lo que sirve para lo que solo necesita el juego en ejecución. En una entidad con un *Scene File*, Godot guarda los nodos que vienen de esa escena tal como los guarda el archivo de escena, así que los cambios que les hagas ahí se pierden. Los nodos que añades sí se guardan, y los cambios a los nodos propios de la escena van en `_ready`.

El script necesita `@tool`, porque los mapas se construyen en el editor de Godot, donde solo los scripts tool ejecutan sus métodos.

## Hacerla colocable { #making-it-placeable }

El editor y Godot conocen las clases de entidad por el FGD de tu proyecto, la lista de definiciones de entidad. Cada definición es un recurso aparte, y el tipo que elijas decide lo que construye Godot:

| Tu entidad | Definición | Qué ajustar |
| --- | --- | --- |
| Una entidad punto, en un solo sitio, como una lámpara o un objeto recogible | `FuncGodotFGDPointClass` | *Script Class* a tu script. Para generar una escena en su lugar, pon *Scene File*, y entonces un *Script Class* reemplaza el script de la raíz de la escena |
| Una entidad brush, construida a partir de brushes, como una puerta o un trigger | `FuncGodotFGDSolidClass` | *Script Class* a tu script, y *Node Class* al cuerpo que extiende: `StaticBody3D`, `AnimatableBody3D` o `Area3D`. Si lo dejas vacío, Godot construye el cuerpo que extiende el script, o un `StaticBody3D` cuando el script extiende `Node3D` o no hay ninguno. Mira [Colisión](entities/collision.md) |

Esta es la definición de la alarma de arriba, guardada como `res://entities/alarm.tres` junto a su script `alarm.gd`:

```
[gd_resource type="Resource" script_class="FuncGodotFGDPointClass" load_steps=3 format=3]

[ext_resource type="Script" path="res://addons/func_godot/src/fgd/func_godot_fgd_point_class.gd" id="1_class"]
[ext_resource type="Script" path="res://entities/alarm.gd" id="2_script"]

[resource]
script = ExtResource("1_class")
classname = "alarm"
description = "Trips when told to, until disarmed"
script_class = ExtResource("2_script")
class_properties = Dictionary[String, Variant]({
"targetname": "",
"armed": true
})
node_class = "Node3D"
```

Añádela a *Entity Definitions* del FGD que usan a la vez tus ajustes del mapa y tu configuración de juego, mira [Configuración del proyecto](../godot/project-setup.md#your-own-resources). La configuración de juego la exporta entonces al editor, con sus señales como salidas y sus métodos que no empiezan por `_` como entradas. Si una entidad aun así se construye como un nodo simple, mira [Entidades que no hacen nada](../godot/entities-troubleshooting.md).

Para empezar rápido, abre el panel **Referencia** y elige una clase parecida a la tuya. En la pestaña *Clase GDScript*, **Crear en el proyecto** escribe ese esqueleto en `res://entities/`.

![El panel Referencia en la pestaña Clase GDScript, con Crear en el proyecto bajo el cursor y su tooltip, Escribe el archivo en res://entities/ salvo que ya exista](../assets/custom-entities/reference-create-in-project.png)

En la pestaña *FGD (.tres)*, **Crear en el proyecto** escribe una definición y el script que ejecuta, ambos con el nombre de la clase precedido de `my_`, así que `func_button` da `res://entities/my_func_button.tres` y `my_func_button.gd`. Añade el `.tres` a *Entity Definitions* de tu FGD, y Godot lo construye.

![El panel Referencia en la pestaña FGD (.tres), con Crear en el proyecto bajo el cursor y su tooltip, Escribe res://entities/my_func_button.tres, una definición con un classname propio, y el script que ejecuta](../assets/custom-entities/reference-create-definition.png)

## Llamar al I/O desde código { #calling-io-from-code }

El código del juego puede usar la misma maquinaria que una conexión del mapa:

```gdscript
for door in TrenchDotIO.find_targets(self, "main_door", null):
	TrenchDotIO.invoke(door, &"open", "", null)

TrenchDotIO.fire_output(self, &"tripped", activator)
```

Prefiere `invoke` a llamar al método directamente, ajusta el parámetro a los argumentos como lo hace una conexión del mapa y maneja las entradas integradas. `fire_output` activa a mano cada conexión de esa salida, útil cuando la entidad no tiene una señal con ese nombre.
