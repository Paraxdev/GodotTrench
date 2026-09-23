# Running Godot from the editor

Saving, builds and live mode talk to the addon over the live link and work with any running Godot editor. The editor
only needs the Godot executable for *Godot > Run Project* (F5), *Godot > Open Project in Godot Editor* and the Godot
button in the toolbar.

## Finding the executable

The editor takes the first match:

1. *Godot executable* in *File > Preferences*.
2. The `GODOT` environment variable.
3. `godot` or `godot4` on `PATH`, then downloaded builds like `Godot_v4.7.2-stable_win64.exe`, newest first. Mono
   builds win when the project folder has a `.csproj`.
4. Common install folders: WinGet, Scoop, Steam and `Programs/Godot` on Windows, `~/.local/bin` and Steam on Linux,
   `/Applications/Godot.app` on macOS.

Preferences shows which executable is in use.

## The Godot button

The Godot robot at the right end of the toolbar shows the connection.

| Robot | Meaning | Click |
| --- | --- | --- |
| Grey | No project open, or Godot not found | Nothing |
| Plain | Godot found | Opens the project in the Godot editor |
| Cyan | A Godot editor has this project open | Brings it to the front |

A spinner next to it shows while Godot is building. If the tooltip says the running Godot has an older addon, update
the addon to get live mode.
