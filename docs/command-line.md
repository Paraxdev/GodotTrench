# Command line

```sh
trenchdot [options] [map]
```

Without options the editor just opens, with the map if you give one. `trenchdot --help` (or `-h`) lists the options
and exits.

| Option | Does |
| --- | --- |
| `--project <dir>` | Loads the Godot project in `<dir>` or a folder above it. Without it the editor loads the last project |
| `--default-prefs` | Starts with default preferences and never saves them. Useful for scripts and tests, and to check whether a problem comes from your own settings |
| `--offscreen` | Opens the window outside every screen, without focus or a taskbar button. It still renders, so scripts and tests can take screenshots while you keep working |
| `--dump <map>` | Prints a map as JSON and exits |
| `--to-json <map> <out>` | Writes a map as JSON and exits |
| `--to-gtm <map> <out>` | Writes a map as a binary `.tdm` and exits |
| `--export-glb <map> <out.glb>` | Writes the map's geometry and materials as glTF binary for Blender and other 3D tools, and exits |
| `--export-obj <map> <out.obj>` | Writes the same as OBJ, with a `.mtl` and a folder of textures next to it, and exits |

An unknown option, or one missing its value, prints an error and exits with code 2, no window opens. A conversion or
export that fails prints why and exits with code 1.

The conversions and exports never open a window, so they're safe in scripts and git hooks, see
[Map files in git](development.md#map-files-in-git).

The exports find the map's Godot project from its folder and use the defaults of *File > Export*: the whole map,
without hidden objects and scatter instances. Entity logic, triggers and scripts are left out, see
[Exporting to Blender and other 3D tools](editor/exporting.md).

<div class="td-mcp" markdown>

## MCP

These options set up the [MCP server](mcp.md), so scripts and AI agents can drive the editor.

| Option | Does |
| --- | --- |
| `--mcp` | Serves MCP over stdin and stdout |
| `--mcp-http[=<port>]` | Serves MCP over HTTP on 127.0.0.1, port 7841 by default |
| `--mcp-token=<token>` | Makes HTTP clients send this token, whatever Preferences says. It wins over `TRENCHDOT_MCP_TOKEN`, see [The HTTP token](mcp.md#the-http-token) |

`--default-prefs` pairs well with either, so a scripted session starts from the same settings every time.

</div>
