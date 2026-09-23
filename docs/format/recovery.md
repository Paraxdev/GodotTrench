# Damaged files

A damaged binary map loses only the chunks that are damaged, and everything else opens. A reader walks the
[chunks](container.md#chunks) from byte 12. Where it finds no valid chunk header, meaning the sync marker, the header
check, a raw size of at most 1 GiB and a printable tag, it searches forward for the next sync marker. A `HEAD`, `NODE`
or `END` chunk whose payload does not decompress, fails its checksum, has the wrong size or does not decode is dropped.

| Damage | Result |
| --- | --- |
| A `NODE` chunk | The nodes stored in it are lost |
| A node's parent was lost | The node moves, with everything below it, into a layer named `Recovered` at the end of the map. The layer gets a fresh id on load |
| Nesting deeper than 60 levels | Moved to `Recovered` as well, so reading cannot run out of stack. Only a crafted file has this |
| The `HEAD` chunk | Worldspawn properties and editor state are lost, and the map version comes from `END` |
| Both `HEAD` and `END` | The file is refused |
| No `END` chunk | The file was cut off. Everything up to the last complete chunk loads |
| `END` counts above what was read | Reported as the number of nodes, or chunks, that are missing |

Each reader reports every loss:

| Reader | Report |
| --- | --- |
| Editor | Opens what it could read, lists the losses in the status bar and marks the map unsaved. Damaged prefabs are reported in the status bar too |
| Godot addon | Builds what it could read and prints a warning per loss, prefixed with `[GTM]` and the file path |
| `godottrench --dump` | Prints what it could read and writes the losses to stderr |

Before every save the editor copies the old file to `<map>.gtm.bak`, so saving a recovered map keeps the damaged
original next to it. The new file is written to `<map>.gtm.tmp` and renamed over the map, so a crash during a save
never leaves a truncated map.
