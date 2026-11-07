---
source: 9167db67d391d56c5d5d5ba31e5f260c3c7d0f58
---

# Entitätsreferenz { #entity-reference }

Die Entitäten des Gameplay-Entitätenpakets, gruppiert danach, wofür du sie einsetzt. Jeder Eintrag sagt, wofür die Entität da ist, und listet dann ihre Inputs (was andere Entitäten ihr auftragen können), ihre Outputs (Ereignisse, die du weiterverdrahten kannst) und ihre Schlüssel (die Einstellungen im Inspektor). Wie Outputs mit Inputs verbunden werden, steht unter [Inputs und Outputs](../io.md).

| Seite | Was du dort findest |
| --- | --- |
| [Logik](logic.md) | Unsichtbare Helfer, die entscheiden, wann etwas passiert: Relays, "nur wenn"-Prüfungen, Gatter, Flipflops, Zufallswahl und Fallunterscheidungen, Zähler, Timer, ein Ereignis beim Kartenstart und Zeitleisten für Zwischensequenzen |
| [Mathe und Werte](math.md) | Unsichtbare Helfer, die Zahlen speichern, berechnen und vergleichen, etwa ein Punktestand, der eine Tür öffnet |
| [Skripte und Aufrufe](scripting.md) | Ein GDScript-Snippet ausführen, deinen Spielcode aufrufen, eine Animation abspielen oder ausgeben, was eine Kette gerade tut |
| [Türen, Mover und Knöpfe](movers.md) | Brush-Entitäten, die sich bewegen: Schiebe- und Drehtüren, Aufzüge, Loren und Knöpfe |
| [Trigger](triggers.md) | Unsichtbare Volumen, die reagieren, wenn ein Körper hineinläuft, um Ereignisse zu starten, Schaden zuzufügen, zu teleportieren, zu schubsen oder etwas zu spawnen |
| [Spawner und Pfade](actors.md) | Gegner oder Pickups spawnen, Teleportziele, Pfadecken und Figuren für Zwischensequenzen |
| [Props, Lichter und Effekte](effects.md) | Zerbrechliche Physik-Props, statische Modelle, Explosionen, Text auf dem Bildschirm, schaltbare Lichter, Sound, Partikel und Lichtsonden |
| [Gestreamte Abschnitte](sections.md) | Landmarken und Verbindungsvolumen, um mehrere Karten ohne Ladebildschirm als eine Welt zu spielen |

## Installieren und anpassen { #installing-and-customizing }

Das Paket ist nicht Teil des Addons selbst. Klicke im Fenster *Dieses Projekt einrichten* auf **Gameplay-Entitäten installieren** oder wähle *Gameplay > Gameplay-Entitäten installieren*, dann werden seine Definitionen und Skripte nach `res://trenchdot/entities` kopiert. Bis dahin listet das Panel Entitäten nur den Kern: worldspawn, die `func_`-Helfer, `light`, `light_spot`, die Licht- und Reflexionssonden und `info_player_start`.

Die Kopien gehören zu deinem Projekt. Ändere das Skript einer Entität in `scripts/` oder ihre Schlüssel und Standardwerte in `definitions/`, lösche die Entitäten, die du nicht brauchst, oder füge eigene hinzu. Ein Addon-Update bietet ein neueres Paket an, überschreibt aber nie eine Datei, die du bearbeitet hast. [Projekteinrichtung](../../godot/project-setup.md#core-entities-and-the-gameplay-entities-pack) erklärt Updates und was passiert, wenn sich zwei Definitionen einen Klassennamen teilen.

## Die Einträge lesen { #reading-the-entries }

* Argumente in Klammern sind das, was ein Input annimmt oder ein Output weitergibt, zum Beispiel empfängt `trigger(activator)` den Körper oder die Entität, die die Kette gestartet hat.
* Jede Entität mit Inputs oder Outputs hat einen Schlüssel `targetname`, außer `logic_auto`. Mit diesem Namen erreichen andere Entitäten sie.
* Ein Schlüssel, den du nicht setzt, nimmt den angegebenen Standardwert.
* Entfernungen sind in Karteneinheiten angegeben, 32 pro Meter. Geschwindigkeiten sind in Metern pro Sekunde angegeben, und `omni_range`, `spot_range` und `max_distance` in Metern.
* Die Listen der Inputs und Outputs sind das, was die Auswahllisten des Editors anbieten. Zur Laufzeit funktioniert jede Methode am Skript der Entität als Input, die Auswahlliste ist also nicht die vollständige Liste.

Wie du eigene Klassen hinzufügst, steht unter [Eigene Entitäten in GDScript](../custom-entities.md).

## Nicht im Paket { #not-in-the-pack }

**Der Kern** kommt mit dem Addon und braucht kein Paket. Worldspawn enthält Sonne, Umgebungslicht, Himmel, Nebel und die Streaming-Einstellungen, und der Inspektor zeigt es, wenn nichts ausgewählt ist. Die FuncGodot-Klassen `func_geo`, `func_detail`, `func_detail_illusionary` und `func_illusionary` bauen einfache Geometrie. `info_player_start` markiert, wo der Spieler startet, spawnt aber selbst nichts, dein Spielcode setzt den Spieler dorthin. [`light` und `light_spot`](effects.md) sind die schaltbaren Lichter.

Brush-Entitäten und Modell-Entitäten kollidieren standardmäßig, die Schlüssel, die das ändern, stehen unter [Kollision](collision.md).

**Der Spieler** ist keine Entität, sondern eine Szene aus deinem eigenen Spiel. Das Paket erwartet ihn in der Gruppe `player` und auf Kollisionsebene 1, wo Trigger suchen, mit einer Methode `take_damage(amount, source)`, die Schaden-Trigger und Explosionen aufrufen. Ein Spieler auf einer anderen Ebene braucht diese Ebene in *Trigger Collision Mask*, siehe [Trigger](triggers.md#shared-inputs-outputs-and-keys). Ein vollständiges Beispiel ist [`godot/demo/player.gd`](https://github.com/Paraxdev/TrenchDot/blob/main/godot/demo/player.gd).
