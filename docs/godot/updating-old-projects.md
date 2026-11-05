# Updating a GodotTrench project

TrenchDot used to be called GodotTrench, and beta builds were briefly called LevelBench. If you set up a project or
saved maps back then, they keep working. You only update the addon once.

## Steps

1. Start the new editor. On the first start it copies your preferences, trusted projects and recent files from the old
   settings folder. The old folder stays where it is, so the old editor still works too.
2. Open your Godot project. The setup window opens and says the addon in the project is older than the editor, even
   when the version numbers match. Close Godot first, then press **Update the addon to** followed by the version.
3. Open the project in Godot once. The addon moves its project settings from `godottrench/` to `trenchdot/` and writes
   `trenchdot_game.json` in place of `godottrench_game.json`.

## What carries over

| You had | What happens |
| --- | --- |
| `.gtm` maps | They open as they are. The status line says "Updated room.gtm from the GodotTrench format, save to keep it". Saving keeps the `.gtm` name, so your scenes still find the map. New maps are saved as `.tdm`. |
| `GODOTTRENCH_` environment variables | They still work. When both names are set, the `TRENCHDOT_` one wins. |
| The Gameplay pack in `res://godottrench/entities` | It keeps working, and your own scripts that extend its classes still compile. If you also install the new pack, the new one wins where both define an entity. Leave the old folder in place while your scenes use it. |
| Nature packs and models in `res://godottrench/` | Maps that place them keep working. The scatter presets look in `res://trenchdot/nature`, so add the nature pack again to paint new scatter. |
| Scenes built by the old addon | They load, because Godot finds the moved addon scripts by their uid. Building the map again refreshes them. |

<div class="td-mcp" markdown>

## MCP clients

The MCP server is now called `trenchdot`. Rename it in your client's config. The token variable is
`TRENCHDOT_MCP_TOKEN`, and the old `GODOTTRENCH_MCP_TOKEN` still works.

</div>
