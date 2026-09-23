"""Writes crates/gt_formats/src/godot_methods.json: the methods and properties of the Godot node classes entity
definitions use, which validate_map accepts as I/O inputs.

    godot --headless --dump-extension-api      # writes extension_api.json in the working directory
    python tools/gen_godot_methods.py extension_api.json
"""

import json
import pathlib
import sys

CLASSES = [
    "AnimatableBody3D", "AnimationPlayer", "Area3D", "AudioStreamPlayer3D", "Camera3D", "CharacterBody3D", "CollisionShape3D",
    "CPUParticles3D", "Decal", "DirectionalLight3D", "GPUParticles3D", "Label3D", "Marker3D", "MeshInstance3D", "Node",
    "Node3D", "OmniLight3D", "Path3D", "PathFollow3D", "RayCast3D", "ReflectionProbe", "RigidBody3D", "SpotLight3D",
    "Sprite3D", "StaticBody3D", "Timer", "VehicleBody3D", "WorldEnvironment",
]


def main() -> None:
    api = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
    classes = {c["name"]: c for c in api["classes"]}
    out = {}

    def add(name: str) -> None:
        if name in out or name not in classes:
            return
        c = classes[name]
        members = {m["name"] for m in c.get("methods", [])} | {p["name"] for p in c.get("properties", [])}
        out[name] = {"inherits": c.get("inherits", ""), "members": sorted(m for m in members if not m.startswith("_") and "/" not in m)}
        add(c.get("inherits", ""))

    for name in CLASSES:
        add(name)
    target = pathlib.Path(__file__).resolve().parent.parent / "crates/gt_formats/src/godot_methods.json"
    target.write_text(json.dumps(out, indent=None, separators=(",", ":"), sort_keys=True) + "\n", encoding="utf-8", newline="\n")
    print(f"wrote {len(out)} classes to {target}")


if __name__ == "__main__":
    main()
