# Command line

```sh
godottrench [options] [map]
```

With no options the editor opens, with the map when one is given. `godottrench --help` prints the options and exits.

| Option | Does |
| --- | --- |
| `--project <dir>` | Loads the Godot project in `<dir>` or a folder above it. Without it the editor loads the last project |
| `--default-prefs` | Starts with default preferences and never saves them. Useful for scripts and tests, and to check whether a problem comes from your own settings |
| `--dump <map>` | Prints a map as JSON and exits |
| `--to-json <map> <out>` | Writes a map as JSON and exits |
| `--to-gtm <map> <out>` | Writes a map as a binary `.gtm` and exits |
| `--export-glb <map> <out.glb>` | Writes the map's geometry and materials as glTF binary for Blender and other 3D tools, and exits |
| `--export-obj <map> <out.obj>` | Writes the same as OBJ, with a `.mtl` and a folder of textures next to it, and exits |

An unknown option, or one missing its value, prints an error and exits with code 2 instead of opening a window. The
conversions and exports never open a window either, so they are safe in scripts and git hooks, see
[Map files in git](development.md#map-files-in-git).

The exports find the map's Godot project from its folder and use the default options of *File > Export*: the whole map
without hidden objects and scatter instances. They leave entity logic, triggers and scripts out, see
[Exporting to Blender and other 3D tools](editor/exporting.md).

{% mcp %}

## MCP

Two more options start the [MCP server](mcp.md), so scripts and AI agents can drive the editor.

| Option | Does |
| --- | --- |
| `--mcp` | Serves MCP over stdin and stdout |
| `--mcp-http[=<port>]` | Serves MCP over HTTP on 127.0.0.1, port 7841 by default |

`--default-prefs` pairs well with either, so a scripted session starts from the same settings every time.

{% endmcp %}
