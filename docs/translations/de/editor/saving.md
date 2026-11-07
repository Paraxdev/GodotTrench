---
source: 320dd2516530d568c7e2acda31113f93ead3f05b
---

# Speichern und Wiederherstellung { #saving-and-recovery }

Drück Ctrl+S zum Speichern oder Ctrl+Shift+S für *Speichern unter…*. Dateidialoge starten im Ordner der aktuellen Karte, bei einer Karte, die du noch nie gespeichert hast, im Godot-Projekt, denn das Addon lädt Karten über ihren `res://`-Pfad. *Datei > Karten im Projekt* listet jede Karte im Projekt auf, wenn du eine nur schnell öffnen willst.

## Backups und Autospeichern { #backups-and-autosave }

Der Editor legt zwei Arten von Ersatzkopien neben deiner Karte ab:

| Datei | Geschrieben |
| --- | --- |
| `<map>.tdm.bak` | Bei jedem Speichern, eine Kopie der vorherigen Version. Benenn sie in `.tdm` um, um einen Speicherstand zurückzugehen |
| `<map>.tdm.autosave` | Alle drei Minuten, für jeden Tab mit ungespeicherten Änderungen |

Wie oft automatisch gespeichert wird, stellst du unter *Datei > Einstellungen* bei *Autospeichern (Minuten, 0 = aus)* ein.

![Das Feld Autospeichern (Minuten, 0 = aus) in den Einstellungen, auf 3 gesetzt](../assets/saving/prefs-autosave.png)

Autosaves haben eine Endung, die Godot nicht importiert, deshalb taucht eine halb fertige Karte nie in deinem Spiel auf. Wenn du die Karte speicherst, wird ihr Autosave gelöscht.

## Eine Karte wiederherstellen { #recovering-a-map }

Wenn der Editor geschlossen wurde, bevor du gespeichert hast, öffne den Autosave:

1. Wähl *Datei > Karte öffnen…*.
2. Nimm den Dateityp **Autospeicherung (Karte wiederherstellen)** und öffne die `.tdm.autosave`-Datei. Die Statusleiste sollte sagen, dass die Karte aus ihrem Autosave wiederhergestellt wurde.
3. Speichere. Das schreibt die echte `.tdm` neu.

Karten, die du nie gespeichert hast, werden stattdessen in eine temporäre Datei autogespeichert. Die findest du unter *Datei > Zuletzt geöffnet*, unterhalb von *Automatisch gesicherte unbenannte Karten*.

![Datei > Zuletzt geöffnet mit Keine letzten Karten ganz oben und darunter einer Liste automatisch gesicherter unbenannter Karten](../assets/saving/open-recent-autosaves.png)

## Beschädigte Dateien { #damaged-files }

Eine Kartendatei, die auf der Platte beschädigt wurde, lässt sich trotzdem öffnen. Der Editor liest jeden intakten Teil, steckt Objekte, deren Ebene, Gruppe oder Entität verloren ging, in eine Ebene namens *Recovered*, und sagt in der Statusleiste, was sich nicht lesen ließ. Beim Speichern bleibt das beschädigte Original als `<map>.tdm.bak` erhalten, du kannst es also immer noch aus der Versionsverwaltung zurückholen.
