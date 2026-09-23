# Command line

```sh
godottrench [options] [map]
```

With no options the editor opens, with the map when one is given. `godottrench --help` prints the options and exits.

| Option | Does |
| --- | --- |
| `--project <dir>` | Loads the Godot project in `<dir>` or a folder above it. Without it the editor loads the last project |
| `--mcp` | Serves the [MCP server](mcp.md) over stdin and stdout |
| `--mcp-http[=<port>]` | Serves MCP over HTTP on 127.0.0.1, port 7841 by default |
| `--default-prefs` | Starts with default preferences and never saves them, for scripts and tests |
| `--dump <map>` | Prints a map as JSON and exits |
| `--to-json <map> <out>` | Writes a map as JSON and exits |
| `--to-gtm <map> <out>` | Writes a map as a binary `.gtm` and exits |

An unknown option, or one missing its value, prints an error and exits with code 2 instead of opening a window. The
three conversions never open a window either, so they are safe in scripts and git hooks, see
[Map files in git](development.md#map-files-in-git).
