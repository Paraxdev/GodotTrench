extends Node3D
## Records what I/O inputs receive, for the argument placement tests.

signal reported(value: int)
signal reported_by(activator: Node, value: int)

var calls: Array = []

func take_damage(amount: float, source: Node) -> void:
	calls.append(["take_damage", amount, source])

func record(value: Variant = null, activator: Node = null) -> void:
	calls.append(["record", value, activator])

func aim(target: Node, strength: float = 1.0) -> void:
	calls.append(["aim", target, strength])

func speed(value: float) -> void:
	calls.append(["speed", value])
