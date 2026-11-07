---
source: 0c1b9cb979394402d977a6f647062a1558b8af7c
---

# Собственные сущности на GDScript { #custom-entities-in-gdscript }

Любой скрипт может быть сущностью. Объявите **сигнал** для каждого выхода и **метод** для каждого входа, затем испускайте сигнал, когда что-то происходит. У этой сигнализации один выход, `tripped`, и два входа, `trip` и `disarm`:

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

Сделайте активатор первым аргументом `Node` ваших сигналов, чтобы цепочки дальше по ходу по-прежнему знали, кто их запустил.

Ключи карты приходят в `_func_godot_apply_properties`. Читайте их через `TrenchDotIO.to_bool`, `to_vector3` и `to_color`, которые принимают и текст, и уже преобразованные значения. Узлы, которые вы добавляете там, сохраняются вместе со сценой. Узлы, которые вы добавляете в `_ready`, не сохраняются, они создаются заново при каждой загрузке сцены, что подходит для того, что нужно только запущенной игре. У сущности с *Scene File* Godot сохраняет узлы, пришедшие из этой сцены, в том виде, в каком их хранит файл сцены, поэтому сделанные там изменения теряются. Добавленные вами узлы всё равно сохраняются, а изменения собственных узлов сцены место в `_ready`.

Скрипту нужен `@tool`, потому что карты собираются в редакторе Godot, где методы выполняют только скрипты-инструменты.

## Как сделать её размещаемой { #making-it-placeable }

Редактор и Godot узнают о классах сущностей из FGD вашего проекта, списка определений сущностей. Каждое определение это отдельный ресурс, и выбранный вид решает, что построит Godot:

| Ваша сущность | Определение | Что задать |
| --- | --- | --- |
| Точечная сущность, в одном месте, например лампа или подбираемый предмет | `FuncGodotFGDPointClass` | *Script Class* на ваш скрипт. Чтобы вместо этого создавать сцену, задайте *Scene File*, тогда *Script Class* заменит скрипт корня сцены |
| Браш-сущность, собранная из брашей, например дверь или триггер | `FuncGodotFGDSolidClass` | *Script Class* на ваш скрипт и *Node Class* на тело, которое он расширяет: `StaticBody3D`, `AnimatableBody3D` или `Area3D`. Если оставить пустым, Godot построит тело, которое расширяет скрипт, или `StaticBody3D`, если скрипт расширяет `Node3D` или его нет. См. [Коллизию](entities/collision.md) |

Это определение сигнализации выше, сохранённое как `res://entities/alarm.tres` рядом с её скриптом `alarm.gd`:

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

Добавьте его в *Entity Definitions* того FGD, который используют и настройки вашей карты, и конфигурация игры, см. [Настройку проекта](../godot/project-setup.md#your-own-resources). Затем конфигурация игры экспортирует его в редактор, а его сигналы становятся выходами, и методы, не начинающиеся с `_`, входами. Если сущность всё равно собирается как обычный узел, см. [Сущности, которые ничего не делают](../godot/entities-troubleshooting.md).

Для быстрого старта откройте панель **Справка** и выберите класс, близкий к вашему. На вкладке *Класс GDScript* **Создать в проекте** записывает эту заготовку в `res://entities/`.

![Панель «Справка» на вкладке «Класс GDScript», «Создать в проекте» наведена, подсказка гласит: записывает файл в res://entities/, если его там нет](../assets/custom-entities/reference-create-in-project.png)

На вкладке *FGD (.tres)* **Создать в проекте** записывает определение и скрипт, который оно запускает, оба с именем класса и приставкой `my_`, так что `func_button` даёт `res://entities/my_func_button.tres` и `my_func_button.gd`. Добавьте `.tres` в *Entity Definitions* вашего FGD, и Godot его построит.

![Панель «Справка» на вкладке «FGD (.tres)», «Создать в проекте» наведена, подсказка гласит: записывает res://entities/my_func_button.tres, определение со своим classname, и скрипт, который оно запускает](../assets/custom-entities/reference-create-definition.png)

## Вызов I/O из кода { #calling-io-from-code }

Код игры может использовать ту же машинерию, что и соединение на карте:

```gdscript
for door in TrenchDotIO.find_targets(self, "main_door", null):
	TrenchDotIO.invoke(door, &"open", "", null)

TrenchDotIO.fire_output(self, &"tripped", activator)
```

Предпочитайте `invoke` прямому вызову метода, он подгоняет параметр под аргументы так же, как соединение на карте, и обрабатывает встроенные входы. `fire_output` вручную запускает каждое соединение этого выхода, что полезно, когда у сущности нет сигнала с таким именем.
