@tool
extends Node3D
## A test entity that adds a child with a child of its own while its map properties are applied, and one more when it
## enters the tree.

func _func_godot_apply_properties(_properties: Dictionary) -> void:
	var added := Node3D.new()
	added.name = "added"
	var inner := Node3D.new()
	inner.name = "inner"
	added.add_child(inner)
	add_child(added)

func _ready() -> void:
	if not has_node("from_ready"):
		var extra := Node3D.new()
		extra.name = "from_ready"
		add_child(extra)
