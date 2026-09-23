# Saving and recovery

## Backups and autosave

| File | Written |
| --- | --- |
| `<map>.gtm.bak` | On every save, a copy of the previous version |
| `<map>.gtm.autosave` | Every three minutes, for every tab with unsaved changes |

Change the autosave interval in *File > Preferences*, 0 turns it off. Autosaves use an extension Godot does not
import, and saving the map deletes its autosave.

## Recovering a map

1. *File > Open Map…*
2. Pick the **Autosave (recover a map)** filter and open the `.gtm.autosave` file.
3. Save. This writes the real `.gtm`.

Untitled maps autosave to a temporary file. Find those under *File > Open Recent*, below *Autosaved untitled maps*.

## Damaged files

A map file damaged on disk still opens. The editor reads every intact part, puts objects whose layer, group or entity
was lost into a layer named *Recovered*, and says in the status bar what could not be read. Saving keeps the damaged
original as `<map>.gtm.bak`, so you can still restore it from version control.
