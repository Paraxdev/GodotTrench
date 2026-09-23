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

## Tabs

| Key | Action |
| --- | --- |
| Ctrl+Shift+N | New tab |
| Ctrl+Tab | Next tab |
| Ctrl+W, or × on the tab | Close tab |

The tab bar appears once two maps are open. *New Map* (Ctrl+N) and importing a `.map` open in a new tab when the
current map has unsaved changes. Closing a modified tab asks to save, discard or cancel, and quitting lists every unsaved
map and offers to save or discard them all.
