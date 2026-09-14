extends Node3D
## Stands in for game code in the I/O tests: a GDScript method, a C# style PascalCase method and a damage handler.

var calls: Array = []
var health := 100.0

func add_score(amount: int, tag: String) -> int:
	calls.append(["add_score", amount, tag])
	return amount * 2

func OpenVault(code: String) -> void:
	calls.append(["OpenVault", code])

func take_damage(amount: float, _source: Node) -> void:
	health -= amount
