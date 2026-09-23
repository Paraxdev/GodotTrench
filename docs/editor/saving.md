# Saving and recovery

## Backups and autosave

| File | Written |
| --- | --- |
| `<map>.gtm.bak` | On every save, a copy of the previous version |
| `<map>.gtm.autosave` | Every three minutes, for every tab with unsaved changes |

Change the autosave interval in *File > Preferences*, 0 turns it off. Autosaves sit next to the map with an extension
Godot does not import.

## Recovering a map

1. *File > Open Map*.
2. Pick the **Autosave (recover a map)** filter and open the autosave file.
3. Save. This writes the real `.gtm`.

Untitled maps autosave to a temporary file of their own. Find those under *File > Open Recent*.

## Tabs

Several maps can be open at once.

| Key | Action |
| --- | --- |
| Ctrl+Shift+N | New tab |
| Ctrl+Tab | Next tab |
| Ctrl+W, or × on the tab | Close tab |

The tab bar appears once two maps are open.

Unsaved work is never thrown away without asking. *New Map* (Ctrl+N) and importing a `.map` open in a new tab when the
current map has changes. Closing a modified tab asks to save, discard or cancel, and quitting offers *Save All* or
*Discard All*.
