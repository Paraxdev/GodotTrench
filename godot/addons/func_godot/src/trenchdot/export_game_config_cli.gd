extends SceneTree
## Exports the project's TrenchDot game config without opening the editor:
## godot --headless --path <project> --script res://addons/func_godot/src/trenchdot/export_game_config_cli.gd

func _initialize() -> void:
	var path: String = ProjectSettings.get_setting(TrenchDotEditorIntegration.SETTING_CONFIG, TrenchDotEditorIntegration.DEFAULT_CONFIG)
	var config := load(path) as TrenchDotGameConfig
	if not config:
		printerr("No TrenchDotGameConfig at ", path)
		quit(1)
		return
	quit(0 if config.export_file() == OK else 1)
