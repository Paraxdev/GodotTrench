---
source: 0c1b9cb979394402d977a6f647062a1558b8af7c
---

# Entités personnalisées en GDScript { #custom-entities-in-gdscript }

N'importe quel script peut être une entité. Déclarez un **signal** pour chaque sortie et une **méthode** pour chaque
entrée, puis émettez le signal quand la chose arrive. Cette alarme a une sortie, `tripped`, et deux entrées, `trip` et
`disarm` :

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

Faites de l'activateur le premier argument `Node` de vos signaux, pour que les chaînes plus bas sachent toujours qui les a
lancées.

Les clés de la carte arrivent dans `_func_godot_apply_properties`. Lisez-les avec `TrenchDotIO.to_bool`, `to_vector3` et
`to_color`, qui acceptent aussi bien du texte que des valeurs déjà converties. Les nœuds que vous y ajoutez sont
enregistrés avec la scène. Ceux que vous ajoutez dans `_ready` ne le sont pas, ils sont refaits à chaque chargement de la
scène, ce qui convient à ce dont seul le jeu en cours a besoin. Pour une entité avec un *Scene File*, Godot enregistre les
nœuds qui viennent de cette scène tels que le fichier de scène les contient, donc les changements que vous leur faites là
sont perdus. Les nœuds que vous ajoutez sont tout de même enregistrés, et les changements aux nœuds propres de la scène
vont dans `_ready`.

Le script a besoin de `@tool`, parce que les cartes sont construites dans l'éditeur Godot, où seuls les scripts tool
exécutent leurs méthodes.

## La rendre plaçable { #making-it-placeable }

L'éditeur et Godot découvrent les classes d'entités par le FGD de votre projet, la liste des définitions d'entités.
Chaque définition est une ressource à part, et le type que vous choisissez décide de ce que Godot construit :

| Votre entité | Définition | Quoi régler |
| --- | --- | --- |
| Une entité point, à un seul endroit, comme une lampe ou un objet à ramasser | `FuncGodotFGDPointClass` | *Script Class* sur votre script. Pour faire apparaître une scène à la place, réglez *Scene File*, un *Script Class* remplace alors le script de la racine de la scène |
| Une entité brush, construite à partir de brushes, comme une porte ou un trigger | `FuncGodotFGDSolidClass` | *Script Class* sur votre script, et *Node Class* sur le corps qu'il étend : `StaticBody3D`, `AnimatableBody3D` ou `Area3D`. Laissé vide, Godot construit le corps que le script étend, ou un `StaticBody3D` quand le script étend `Node3D` ou qu'il n'y en a pas. Voir [Collision](entities/collision.md) |

Voici la définition de l'alarme ci-dessus, enregistrée sous `res://entities/alarm.tres` à côté de son script `alarm.gd` :

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

Ajoutez-la à *Entity Definitions* du FGD que vos réglages de carte et votre config de jeu utilisent tous les deux, voir
[Configurer le projet](../godot/project-setup.md#your-own-resources). La config de jeu l'exporte alors vers l'éditeur, avec
ses signaux comme sorties et ses méthodes qui ne commencent pas par `_` comme entrées. Si une entité se construit quand
même comme un nœud simple, voir [Entités qui ne font rien](../godot/entities-troubleshooting.md).

Pour démarrer vite, ouvrez le panneau **Référence** et choisissez une classe proche de la vôtre. Sur l'onglet *Classe
GDScript*, **Créer dans le projet** écrit ce squelette sous `res://entities/`.

![Le panneau Référence sur l'onglet Classe GDScript, Créer dans le projet survolé avec son infobulle, Écrit le fichier sous res://entities/ sauf s'il existe](../assets/custom-entities/reference-create-in-project.png)

Sur l'onglet *FGD (.tres)*, **Créer dans le projet** écrit une définition et le script qu'elle exécute, tous deux nommés
d'après la classe avec `my_` devant, donc `func_button` donne `res://entities/my_func_button.tres` et `my_func_button.gd`.
Ajoutez le `.tres` à *Entity Definitions* de votre FGD, et Godot le construit.

![Le panneau Référence sur l'onglet FGD (.tres), Créer dans le projet survolé avec son infobulle, Écrit res://entities/my_func_button.tres, une définition avec son propre classname, et le script qu'elle exécute](../assets/custom-entities/reference-create-definition.png)

## Appeler les E/S depuis le code { #calling-io-from-code }

Le code du jeu peut utiliser la même mécanique qu'une connexion de carte :

```gdscript
for door in TrenchDotIO.find_targets(self, "main_door", null):
	TrenchDotIO.invoke(door, &"open", "", null)

TrenchDotIO.fire_output(self, &"tripped", activator)
```

Préférez `invoke` à l'appel direct de la méthode, il adapte le paramètre aux arguments comme le fait une connexion de
carte et gère les entrées intégrées. `fire_output` active à la main chaque connexion de cette sortie, utile quand
l'entité n'a pas de signal de ce nom.
