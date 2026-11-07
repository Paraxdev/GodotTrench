---
source: 0c1b9cb979394402d977a6f647062a1558b8af7c
---

# Eigene Entitäten in GDScript { #custom-entities-in-gdscript }

Jedes Skript kann eine Entität sein. Deklariere für jeden Output ein **Signal** und für jeden Input eine **Methode**, und sende das Signal aus, wenn das Ding passiert. Dieser Alarm hat einen Output, `tripped`, und zwei Inputs, `trip` und `disarm`:

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

Mach den Aktivator zum ersten `Node`-Argument deiner Signale, damit Ketten weiter unten noch wissen, wer sie gestartet hat.

Karten-Schlüssel kommen in `_func_godot_apply_properties` an. Lies sie über `TrenchDotIO.to_bool`, `to_vector3` und `to_color`, die sowohl Text als auch bereits umgewandelte Werte akzeptieren. Nodes, die du dort hinzufügst, werden mit der Szene gespeichert. Nodes, die du in `_ready` hinzufügst, nicht, sie werden bei jedem Laden der Szene neu erzeugt, was zu allem passt, was nur das laufende Spiel braucht. Bei einer Entität mit einer *Scene File* speichert Godot die Nodes, die aus dieser Szene stammen, so, wie die Szenendatei sie enthält, Änderungen, die du dort an ihnen vornimmst, gehen also verloren. Nodes, die du hinzufügst, werden trotzdem gespeichert, und Änderungen an den eigenen Nodes der Szene gehören in `_ready`.

Das Skript braucht `@tool`, denn Karten werden im Godot-Editor gebaut, wo nur Tool-Skripte ihre Methoden ausführen.

## Platzierbar machen { #making-it-placeable }

Der Editor und Godot lernen die Entitätsklassen aus dem FGD deines Projekts kennen, der Liste der Entitätsdefinitionen. Jede Definition ist eine eigene Ressource, und die Art, die du wählst, entscheidet, was Godot baut:

| Deine Entität | Definition | Was du setzt |
| --- | --- | --- |
| Eine Punkt-Entität, an einem Ort, wie eine Lampe oder ein Pickup | `FuncGodotFGDPointClass` | *Script Class* auf dein Skript. Um stattdessen eine Szene zu spawnen, setze *Scene File*, eine *Script Class* ersetzt dann das Skript des Wurzel-Nodes der Szene |
| Eine Brush-Entität, aus Brushes gebaut, wie eine Tür oder ein Trigger | `FuncGodotFGDSolidClass` | *Script Class* auf dein Skript und *Node Class* auf den Body, den es erweitert: `StaticBody3D`, `AnimatableBody3D` oder `Area3D`. Bleibt es leer, baut Godot den Body, den das Skript erweitert, oder einen `StaticBody3D`, wenn das Skript `Node3D` erweitert oder es keins gibt. Siehe [Kollision](entities/collision.md) |

Das ist die Definition des Alarms oben, gespeichert als `res://entities/alarm.tres` neben seinem Skript `alarm.gd`:

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

Füge sie zu *Entity Definitions* des FGD hinzu, das sowohl deine Karteneinstellungen als auch deine Spielkonfiguration nutzen, siehe [Projekteinrichtung](../godot/project-setup.md#your-own-resources). Die Spielkonfiguration exportiert sie dann in den Editor, mit ihren Signalen als Outputs und ihren Methoden, die nicht mit `_` beginnen, als Inputs. Wird eine Entität trotzdem als einfacher Node gebaut, siehe [Entitäten, die nichts tun](../godot/entities-troubleshooting.md).

Für einen schnellen Start öffne das Panel **Referenz** und wähle eine Klasse, die deiner nahekommt. Auf dem Tab *GDScript-Klasse* schreibt **Im Projekt erstellen** dieses Gerüst unter `res://entities/`.

![Das Panel Referenz auf dem Tab GDScript-Klasse, Im Projekt erstellen unter dem Mauszeiger mit seinem Tooltip, Schreibt die Datei unter res://entities/, außer sie existiert](../assets/custom-entities/reference-create-in-project.png)

Auf dem Tab *FGD-.tres* schreibt **Im Projekt erstellen** eine Definition und das Skript, das sie ausführt, beide nach der Klasse benannt mit `my_` davor, `func_button` ergibt also `res://entities/my_func_button.tres` und `my_func_button.gd`. Füge die `.tres` zu *Entity Definitions* deines FGD hinzu, dann baut Godot sie.

![Das Panel Referenz auf dem Tab FGD-.tres, Im Projekt erstellen unter dem Mauszeiger mit seinem Tooltip, Schreibt res://entities/my_func_button.tres, eine Definition mit eigenem Classname, und das Skript, das sie ausführt](../assets/custom-entities/reference-create-definition.png)

## I/O aus dem Code aufrufen { #calling-io-from-code }

Spielcode kann dieselbe Maschinerie nutzen wie eine Karten-Verbindung:

```gdscript
for door in TrenchDotIO.find_targets(self, "main_door", null):
	TrenchDotIO.invoke(door, &"open", "", null)

TrenchDotIO.fire_output(self, &"tripped", activator)
```

Bevorzuge `invoke` gegenüber dem direkten Aufruf der Methode, es passt den Parameter wie eine Karten-Verbindung an die Argumente an und behandelt die eingebauten Inputs. `fire_output` löst jede Verbindung an diesem Output von Hand aus, nützlich, wenn die Entität kein Signal dieses Namens hat.
