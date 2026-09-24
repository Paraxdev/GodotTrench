# Damaged files

A damaged binary map loses only its damaged chunks, and everything else still opens. This page is what a reader has to
do to match that behavior.

A reader walks the [chunks](container.md#chunks) from byte 12. A valid chunk header has the sync marker, a matching
header check, a raw size of at most 1 GiB and a printable tag. Where the reader finds no valid header, it searches
forward for the next sync marker and carries on from there. A `HEAD`, `NODE` or `END` chunk is dropped when its payload
does not decompress, fails its checksum, has the wrong size or does not decode.

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

Before every save the editor copies the old file to `<map>.gtm.bak`, so saving a recovered map still keeps the damaged
original next to it. The new file is written to `<map>.gtm.tmp` first and then renamed over the map, so a crash during
a save never leaves a half written map behind.
