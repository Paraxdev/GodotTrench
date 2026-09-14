extends SceneTree
## Exports the project's GodotTrench game config without opening the editor:
## godot --headless --path <project> --script res://addons/func_godot/src/godottrench/export_game_config_cli.gd

func _initialize() -> void:
	var path: String = ProjectSettings.get_setting(GodotTrenchEditorIntegration.SETTING_CONFIG, GodotTrenchEditorIntegration.DEFAULT_CONFIG)
	var config := load(path) as GodotTrenchGameConfig
	if not config:
		printerr("No GodotTrenchGameConfig at ", path)
		quit(1)
		return
	quit(0 if config.export_file() == OK else 1)
