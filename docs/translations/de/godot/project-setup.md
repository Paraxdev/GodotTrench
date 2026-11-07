---
source: 0b6155b82bf9e0fff2345e05627d7c675a868912
---

# Projekteinrichtung { #project-setup }

Die Standardwerte des Addons reichen für deine ersten Karten. Sobald du eigene Entitäten schreibst oder einen anderen Texturordner willst, gib dem Projekt eigene Ressourcen, denn ein Addon-Update ersetzt die Standardwerte.

## Das Addon installieren und aktualisieren { #installing-and-updating-the-addon }

Jedes Mal, wenn du ein Projekt öffnest, prüft TrenchDot sein Addon. Fehlt `addons/func_godot` oder hat es eine andere Version als der Editor, erscheint einmal das Fenster *Dieses Projekt einrichten*, und unten rechts in der Statusleiste bleibt ein gelber Hinweis. Fahr mit der Maus darüber, um beide Versionen zu sehen, und klick darauf, um das Fenster zurückzuholen.

![Die Statusleiste mit dem gelben Hinweis "Addon 0.0.9, Editor 0.1.0" unter dem Mauszeiger und seinem Tooltip](../assets/project-setup/status-addon-mismatch.png)

Du kannst das Fenster auch jederzeit über *Godot > Addon installieren oder aktualisieren…* öffnen.

![Das Menü Godot mit Addon installieren oder aktualisieren… unter dem Mauszeiger](../assets/project-setup/menu-install-update-addon.png)

Es bietet nur den Schritt an, den dein Projekt als Nächstes braucht:

| Das Projekt hat | Das Fenster bietet |
| --- | --- |
| Kein Addon | **Addon installieren**, dann **Plugin aktivieren** |
| Ein älteres Addon | **Addon auf 0.1.0 aktualisieren**, die Version des Editors |
| Ein neueres Addon oder eines ohne Version | **Addon durch 0.1.0 ersetzen**, die Version des Editors |
| Das Addon, nicht aktiviert | **Plugin aktivieren**, das es unter `[editor_plugins]` in `project.godot` einträgt und sonst nichts ändert |
| Das Addon als Git-Checkout oder Submodul | Nichts. Aktualisiere es mit Git, der Editor ersetzt es nie |

![Das Fenster Dieses Projekt einrichten mit der Meldung, dass das Addon 0.0.9 älter ist als der Editor, und dem Knopf Addon auf 0.1.0 aktualisieren](../assets/project-setup/setup-update-addon.png)

> [!WARNING]
> Schließ Godot, bevor du aktualisierst. Es führt das alte Addon weiter aus, bis es neu startet, deshalb bleibt der Knopf ausgegraut, solange der Live Link Godot mit dem Projekt offen sieht. Aktiviere das Plugin ebenfalls bei geschlossenem Godot, sonst schreibt Godot seine eigene Kopie von `project.godot` zurück. Es in Godot unter *Project > Project Settings > Plugins* anzuhaken funktioniert genauso.

Das alte Addon wird nicht gelöscht, es wandert nach `.trenchdot/addon-backups/func_godot-<version>` in deinem Projekt. Godot überspringt Ordner, die mit einem Punkt beginnen, es importiert die Sicherung also nicht und listet sie nicht als zweites Plugin. Lösch sie, sobald das neue Addon funktioniert. (Hinweis: Der Download wird zuerst entpackt und erst dann eingetauscht, ein fehlgeschlagenes oder abgebrochenes Update lässt das alte Addon also, wie es war.)

Schlägt der Download fehl, verlinkt das Fenster die [Releases](https://github.com/Paraxdev/TrenchDot/releases) und [itch.io](https://paraxdev.itch.io/trenchdot), die dieselbe `func_godot-trenchdot-addon.zip` tragen. Entpack sie in den Projektordner, sodass du `addons/func_godot` bekommst, und ersetze den alten Ordner, statt darüber zu entpacken.

## Kernentitäten und das Gameplay-Entitätenpaket { #core-entities-and-the-gameplay-entities-pack }

Jedes Projekt bekommt vom Addon einen kleinen Kern: worldspawn, `func_detail`, `func_illusionary`, `func_geo`, `light`, `light_spot`, `light_probe`, `env_reflection_probe`, `env_voxel_gi` und `info_player_start`.

Türen, Buttons, Aufzüge, Züge, Trigger, Logik, Spawner, Teleporter, Wegpunkte, Props, Effekte und der Rest der [Entitätenreferenz](../gameplay/entities/README.md) kommen als Gameplay-Entitätenpaket. Du installierst es, wenn du es willst, und es wird nach `res://trenchdot/entities` kopiert:

| Pfad | Enthält |
| --- | --- |
| `gameplay_fgd.tres` | Die FGD-Datei des Pakets, die jede Definition auflistet |
| `definitions/` | Eine Definition pro Entität: ihre Schlüssel, Eingänge, Ausgänge und Editor-Gizmos |
| `scripts/` | Das GDScript, das jede Entität ausführt |
| `pack.json` | Was installiert wurde, damit Updates deine Änderungen erkennen. Lass es, wie es ist |

Die Kopien gehören deinem Projekt. Ändere ein Skript oder eine Definition, lösch die Entitäten, die du nicht nutzt, oder füg deine eigenen Definitionen zu `gameplay_fgd.tres` hinzu. Das Addon definiert diese Klassennamen oder Skriptklassen nie selbst, nichts, was du dort tust, kollidiert also damit, und ein Addon-Update macht es nicht rückgängig.

### Das Paket installieren und aktualisieren { #installing-and-updating-the-pack }

1. Das Paket installieren

    Klick im Fenster *Dieses Projekt einrichten* unter dem Addon auf **Gameplay-Entitäten installieren**.

    ![Das Fenster Dieses Projekt einrichten mit dem Abschnitt Gameplay-Entitäten und seinem Knopf Gameplay-Entitäten installieren](../assets/project-setup/setup-gameplay-entities.png)

    Solange das Projekt nur den Kern hat, steht derselbe Knopf oben im Menü Gameplay und im Panel Entitäten.

    ![Das Menü Gameplay mit Gameplay-Entitäten installieren unter dem Mauszeiger und seinem Tooltip](../assets/project-setup/menu-install-gameplay-entities.png)

2. Prüfen, ob es geklappt hat

    Das Fenster sollte jetzt in Grün "Installiert in res://trenchdot/entities" sagen, und der Editor bietet die neuen Entitäten sofort an.

    ![Der Abschnitt Gameplay-Entitäten nach der Installation mit der Meldung, dass 86 Dateien nach res://trenchdot/entities hinzugefügt wurden und Godot neu gestartet werden soll, falls es das Projekt offen hat](../assets/project-setup/setup-gameplay-installed.png)

    Hat ein Godot-Editor das Projekt offen, bittet TrenchDot ihn, die neuen Dateien zu übernehmen, und das Fenster sagt das. Eine Szene im [Live-Modus](live.md#when-live-mode-rebuilds-the-whole-map) baut dann von selbst neu. Drück in den anderen Szenen, die deine Karten nutzen, **Build Map** und speichere sie, denn sie wurden ohne das Paket gebaut. Läuft Godot nicht oder ein älteres Addon, sagt das Fenster, dass du Godot neu starten sollst, falls es das Projekt offen hat, und dann diese Szenen bauen und speichern.

3. (Optional) Es später aktualisieren

    Bringt ein Addon-Update ein neueres Paket, bietet das Fenster **Gameplay-Entitäten aktualisieren** an, und das Menü Gameplay zeigt es mit der Zahl der geänderten Dateien. Nichts wird aktualisiert, bis du darauf klickst. Das Update fügt neue Dateien hinzu und ersetzt die, die du nicht angefasst hast. Dateien, die du bearbeitet hast, bleiben, wie sie sind, und der Editor nennt sie, und gelöschte Dateien bleiben gelöscht.

Ältere Karten aus der Zeit, bevor die Entitäten das Addon verlassen haben, funktionieren weiter. Nutzen die Karten eines Projekts Paket-Entitäten, die nichts im Projekt definiert, installiert sich das Paket selbst, sowohl wenn der Editor das Projekt öffnet als auch wenn Godot mit dem aktualisierten Addon startet. Eine `.tdm`-Datei behält in jedem Fall jede Entität, mit oder ohne Definition.

> [!NOTE]
> Die Vorlage im Addon, `addons/func_godot/gameplay_pack`, hat eine `.gdignore`-Datei, damit Godot sie überspringt. Behalte diese Datei, sonst findet Godot jede Skriptklasse des Pakets doppelt, sobald es installiert ist.

### Welche Definition gewinnt { #which-definition-wins }

Die Kern-FGD, `res://addons/func_godot/fgd/trenchdot/trenchdot_fgd.tres`, liest das installierte Paket selbst, jede auf den Standardwerten des Addons gebaute FGD bekommt das Paket also, ohne es aufzulisten. Teilen zwei Definitionen einen Klassennamen, gewinnt die spätere, in dieser Reihenfolge:

1. Die Klassen von FuncGodot und der Kern des Addons
2. Das installierte Paket in `res://trenchdot/entities`
3. Die Entitäten deiner eigenen FGD-Datei

Um eine Entität zu ändern, bearbeite ihre Kopie im Paket oder definiere denselben Klassennamen in deiner eigenen FGD. Beides funktioniert, aber ein doppelt definierter Klassenname ist leicht vergessen, deshalb sagt es dir der Editor: Beim Öffnen des Projekts nennt die Statusleiste die Definition, die gewinnt, und der Bereich Probleme warnt vor jeder doppelt definierten Klasse, die eine Karte nutzt. Lösch eine der beiden, um es zu klären.

## Deine eigenen Ressourcen { #your-own-resources }

Drei Ressourcen arbeiten zusammen. Die FGD-Datei listet die Entitäten auf, die deine Karten nutzen können. Die Karteneinstellungen sagen einem Build, wo die Texturen liegen und welche FGD er lesen soll. Die Spielkonfiguration erzählt TrenchDot von beiden, damit der Editor beim Bearbeiten dieselben Entitäten und Texturen anbietet.

Am schnellsten kopierst du `demo_fgd.tres`, `demo_map_settings.tres` und `demo_game_config.tres` aus [`godot/demo`](https://github.com/Paraxdev/TrenchDot/tree/main/godot/demo) in dein Projekt und korrigierst die Pfade. Um sie selbst anzulegen, geh im Godot-Editor so vor:

1. Leg eine `FuncGodotFGDFile` an. Setz *Base Fgd Files* auf `res://addons/func_godot/fgd/trenchdot_default_fgd.tres`, damit du den Kern und das installierte Paket behältst, und füg dann unter *Entity Definitions* deine eigenen Entitäten hinzu. Ohne diese Basis hat deine FGD keine davon, und selbst worldspawn baut ohne Kollision.
2. Leg eine `FuncGodotMapSettings` an. Setz *Entity Fgd* auf die FGD aus Schritt 1 und *Base Texture Dir* auf deinen Texturordner (Standard `res://textures`). Eine neue Karteneinstellungs-Ressource startet auf `trenchdot_default_fgd.tres`, die den Kern und das Paket hat, aber nicht deine eigenen Entitäten.
3. Leg eine `TrenchDotGameConfig` an. Setz ihre *Fgd File* und *Map Settings* auf die beiden Ressourcen oben. (Optional) Setz *Game Name*, der Editor zeigt ihn oben rechts in der Werkzeugleiste neben dem Projektordner.
4. Lass in *Project > Project Settings* `func_godot/default_map_settings` auf deine Karteneinstellungen und `trenchdot/game_config` auf deine Spielkonfiguration zeigen. Jeder Karten-Node, dessen Feld *Map Settings* leer ist, folgt dem. Ein mit einem älteren Addon hinzugefügter Node hat die Standardwerte des Addons in diesem Feld gespeichert, leere es dort also.

Der Editor bietet die Entitäten der FGD der Spielkonfiguration an, aber Godot baut mit der FGD der Karteneinstellungen, halte die beiden also gleich. Unterscheiden sie sich, warnt Godot in seinem Bereich Output bei jedem Export der Spielkonfiguration, und der Bereich Probleme von TrenchDot nennt jede Entität, die der FGD fehlt, mit der Godot baut.

> [!TIP]
> Setz *Entity Name Property* in den Karteneinstellungen auf `targetname`, wie es die Demo tut. Gebaute Nodes heißen dann nach ihren targetnames, was den Szenenbaum lesbar hält.

## Die Spielkonfiguration { #the-game-config }

TrenchDot lernt deine Entitäten, Texturordner und den Einheitenmaßstab aus `res://trenchdot_game.json`, die die Spielkonfiguration schreibt. Das Addon exportiert sie erneut, sobald Godot Dateiänderungen bemerkt, und der Editor lädt sie binnen einer Sekunde neu. Der Export nennt auch die Standard-Karteneinstellungen und die Klassen, die ihre FGD definiert, womit der Editor vor Entitäten warnt, die Godot ohne Definition bauen würde. Um von Hand zu exportieren, nutze in Godot *Project > Tools > TrenchDot: Export Game Config* oder führe es headless aus, zum Beispiel in der CI:

```sh
godot --headless --path <project> --script res://addons/func_godot/src/trenchdot/export_game_config_cli.gd
```

## Projekteinstellungen { #project-settings }

Die meisten Projekte ändern nur die erste und die dritte. Die des Live Links sind wichtig, wenn ein anderes Programm den Port schon nutzt.

| Einstellung | Standard | Zweck |
| --- | --- | --- |
| `func_godot/default_map_settings` | die Standardwerte des Addons | Karteneinstellungen für jede Karte, die keine eigenen setzt |
| `func_godot/default_inverse_scale_factor` | 32 | Karteneinheiten pro Meter |
| `trenchdot/game_config` | die Konfiguration des Addons | Die Konfiguration, die `trenchdot_game.json` schreibt |
| `trenchdot/auto_export_game_config` | true | Die Spielkonfiguration erneut exportieren, wenn sich Dateien ändern |
| `trenchdot/live_link_enabled` | true | Den Live-Link-Server starten, mit dem TrenchDot spricht |
| `trenchdot/live_link_port` | 7842 | Muss zu *Godot Live Link* in den Einstellungen des Editors passen. Änderungen gelten sofort. Zwischen 1024 und 65535, alles andere fällt mit einer Warnung auf 7842 zurück |
| `trenchdot/threaded_build` | true | Brushes und Terrain-Chunks im WorkerThreadPool umwandeln, damit große Karten schneller bauen |
| `trenchdot/live_chunk_size` | 16.0 | Im Live-Modus wird die worldspawn-Geometrie in Chunks dieser Größe in Metern geteilt, damit eine Änderung nur ihren Chunk neu baut |
| `trenchdot/csharp_entity_dirs` | `res://` | Wo C#-Entitäten beim Bauen gefunden werden |
