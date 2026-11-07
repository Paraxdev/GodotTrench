---
source: 46b4723cec5ef9109b70a2a54e850458f119c7e9
---

# Godot aus dem Editor starten { #running-godot-from-the-editor }

Speichern, Builds und der Live-Modus sprechen über den Live Link mit dem Addon, sie funktionieren also mit jedem Godot-Editor, der das Projekt offen hat. TrenchDot muss das Godot-Programm nur für *Godot > Projekt im Godot-Editor öffnen*, den Godot-Roboter in der Werkzeugleiste und *Godot > Projekt ausführen* (F5) finden, solange kein Godot-Editor das Projekt offen hat, denn diese starten Godot selbst.

![Das Menü Godot mit dem Eintrag Projekt ausführen unter dem Mauszeiger, seinem Kürzel F5 rechts und Projekt im Godot-Editor öffnen darunter](../assets/running-godot/menu-run-project.png)

## Projekt ausführen { #run-project }

Hat ein Godot-Editor das Projekt offen, bittet *Projekt ausführen* diesen Editor, seine offenen Szenen zu speichern und die Hauptszene zu spielen, genau wie ein Klick auf Play in Godot. Das Spiel hat dann alles, was Godot gebaut hat, auch Live-Änderungen. Die Statusleiste sollte "Godot hat seine offenen Szenen gespeichert und führt die Hauptszene aus" sagen. Hat das Projekt keine Hauptszene, sagt sie das, setz dann in Godot unter *Project > Project Settings > Application > Run* eine.

Ohne Godot-Editor startet *Projekt ausführen* das Spiel mit dem Godot-Programm. Es führt dann die Szenen so aus, wie sie zuletzt in Godot gespeichert wurden. Dasselbe passiert mit einem älteren Addon, und die Statusleiste fügt hinzu, dass dem Spiel deine neuesten Kartenänderungen fehlen können. Speichere sie in Godot oder drück dort Play.

## Das Programm finden { #finding-the-executable }

Geh zu *Datei > Einstellungen* und sieh dir *Godot-Programm* an. Hat TrenchDot eines gefunden, steht unter dem Eingabefeld "Verwendet" und der Pfad.

![Das Feld Godot-Programm in den Einstellungen mit der Zeile "Verwendet" und dem Pfad darunter](../assets/running-godot/prefs-godot-using.png)

Steht dort in Gelb "Nicht gefunden, ...", klick auf den Knopf "…" neben dem Feld und wähl das Godot-Programm auf deiner Festplatte, das mit "Godot_v" beginnt. Tu dasselbe, falls es das falsche Godot gewählt hat. Bis es eines findet, bleiben Projekt im Godot-Editor öffnen und Projekt ausführen ausgegraut, Letzteres, solange kein Godot-Editor das Projekt offen hat.

![Das leere Feld Godot-Programm mit dem Knopf … und der gelben Zeile Nicht gefunden](../assets/running-godot/prefs-godot-not-found.png)

Ohne einen funktionierenden Pfad in den Einstellungen nimmt der Editor den ersten Treffer aus:

1. Der Umgebungsvariable `GODOT`.
2. `godot` oder `godot4` im `PATH`, dann heruntergeladene Builds wie `Godot_v4.7.2-stable_win64.exe`, die neueste zuerst. Mono-Builds gewinnen, wenn der Projektordner eine `.csproj` hat, da ein C#-Projekt sie braucht. Konsolen-Builds werden übersprungen.
3. Übliche Installationsordner: WinGet, Scoop, Steam und `Programs/Godot` unter Windows, `~/.local/bin` und Steam unter Linux, `/Applications/Godot.app` unter macOS.

## Der Godot-Knopf { #the-godot-button }

Der Godot-Roboter am rechten Ende der Werkzeugleiste zeigt die Verbindung. Fahr mit der Maus darüber, um zu sehen, was er tun würde.

Grau heißt, es ist kein Projekt offen oder Godot wurde nicht gefunden, ein Klick tut also nichts.

![Der graue Godot-Roboter unter dem Mauszeiger, sein Tooltip sagt, dass Godot nicht gefunden wurde](../assets/running-godot/toolbar-godot-not-found.png)

Normal heißt, Godot wurde gefunden. Klick darauf, um das Projekt im Godot-Editor zu öffnen.

![Der normale Godot-Roboter unter dem Mauszeiger, sein Tooltip sagt Projekt im Godot-Editor öffnen](../assets/running-godot/toolbar-godot-found.png)

Cyan heißt, ein Godot-Editor hat dieses Projekt offen. Klick darauf, um diesen Editor nach vorn zu holen.

![Der cyanfarbene Godot-Roboter unter dem Mauszeiger, sein Tooltip sagt, dass Godot 4.7.2-stable (official) dieses Projekt offen hat und ein Klick es anzeigt](../assets/running-godot/toolbar-godot-open.png)

Daneben dreht sich ein Spinner, während Godot baut. Sagt der Tooltip, dass das laufende Godot ein älteres Addon hat, aktualisiere es mit *Godot > Addon installieren oder aktualisieren…*, um den Live-Modus zu bekommen.

## Projektvertrauen { #project-trust }

Godot führt die Skripte eines Projekts aus, sobald es das Projekt öffnet, spielt oder baut, und ein heruntergeladenes Projekt oder Kartenpaket kann eigene Skripte mitbringen. Öffnest du also ein Projekt, das TrenchDot noch nicht kennt und das Code enthält, fragt TrenchDot zuerst nach.

TrenchDot sucht zuerst im Projekt nach Code. Währenddessen zeigt die Statusleiste *Suche nach Code*, und in Godot wird auf dem Projekt nichts gestartet, ein großes Projekt braucht also einen Moment, bevor es fragt oder öffnet.

![Die Abfrage "Vertraust du den Autoren der Dateien in diesem Projekt?" mit dem Ordner des Projekts, was der eingeschränkte Modus blockiert, dem Kontrollkästchen für den übergeordneten Ordner und den Knöpfen Projekt vertrauen und Im eingeschränkten Modus öffnen](../assets/running-godot/trust-prompt.png)

Klick bei deinen eigenen Projekten und Projekten von Leuten, denen du vertraust, auf **Projekt vertrauen**. Hak zuerst *Den Autoren aller Dateien im übergeordneten Ordner vertrauen* an, um jedem Projekt in diesem Ordner zu vertrauen, etwa dem, in dem du alle deine Spiele ablegst. Ohne das vertraust du nur diesem einen Projekt, und ein darin entpacktes Projekt, etwa ein Kartenpaket mit eigener `project.godot`, wird nicht mit vertraut.

Klick auf **Im eingeschränkten Modus öffnen**, wenn du dir nicht sicher bist. Du kannst die Karten weiter ansehen und bearbeiten, aber TrenchDot startet Godot nicht auf dem Projekt, baut nicht in Godot und sendet keine Live-Änderungen. *Eingeschränkter Modus* steht dann in Gelb rechts in der Statusleiste, klick darauf, um die Frage noch einmal zu beantworten.

![Der gelbe Chip Eingeschränkter Modus in der Statusleiste, sein Tooltip sagt, dass TrenchDot dieses Projekt erst in Godot ausführt, wenn du ihm vertraust](../assets/running-godot/restricted-mode-chip.png)

Im eingeschränkten Modus fragen *Projekt ausführen*, der Godot-Roboter, *In Godot bauen* und das Einschalten des Live-Modus "Dabei werden seine Skripte ausgeführt. Vertrauen und fortfahren?" und machen weiter, sobald du auf **Projekt vertrauen** klickst.

Öffnest du eine Karte, die zu einem anderen Projekt gehört, dem du nicht vertraust, bleibt das offene Projekt erhalten, und die Abfrage bekommt zusätzlich den Knopf **Aktuelles Projekt behalten**. Eine `project.godot` im Ordner des offenen Projekts wird ignoriert, so wie Godot sie ignoriert. Führt eine Karte von außerhalb des offenen Projekts eigenes GDScript aus, sagt das die Statusleiste, und der Bereich Probleme zeigt, wo.

Projekte in *Letzte Projekte* aus der Zeit, bevor TrenchDot nach Vertrauen fragte, gelten als vertrauenswürdig, ebenso ein Projekt, mit dem du den Editor über `--project` startest, für diese Sitzung.

> [!NOTE]
> Der eingeschränkte Modus deckt nur ab, was TrenchDot startet. Öffnest du das Projekt selbst in Godot, laufen seine Skripte wie gewohnt.

### Projekte ohne Code { #projects-without-code }

Ein Projekt ohne Code öffnet ohne die Frage, denn Godot hat darin nichts auszuführen. Das ist ein leerer Ordner, ein frisch angelegtes Projekt oder eines mit nur dem Addon von TrenchDot und dem Gameplay-Entitätenpaket, das es installiert hat. Code heißt hier Skripte, C#-Projekte, native Bibliotheken, Szenen und Ressourcen mit Skripten darin, Autoloads, Editor-Plugins und Karten, deren `logic_script` GDScript enthält. Das Addon und das Paket zählen nur so lange als TrenchDots eigene, wie jede Datei darin die ist, die diese Version installiert, eine bearbeitete oder ältere Kopie zählt also als Code, ebenso ein Projekt, das zu groß ist, um es schnell zu durchsuchen.

TrenchDot sieht bei jedem Öffnen des Projekts neu nach und merkt sich nichts, fragt also, sobald das Projekt Skripte bekommen hat. Damit dein eigenes Projekt vertrauenswürdig bleibt, speichere eine Karte mit *Datei > Speichern* darin, solange es keinen Code enthält. Ab dann vertraut TrenchDot ihm, auch nachdem du in Godot Skripte hinzufügst. Eine Karte im eingeschränkten Modus in ein Projekt zu speichern vertraut nichts.

### Vertrauenswürdige Projekte { #trusted-projects }

Öffne *Datei > Einstellungen…* und sieh unten bei *Vertrauenswürdige Projekte* nach, um jedes Projekt und jeden Ordner zu sehen, denen du vertraust. Klick neben einem auf **Entfernen**, um das Vertrauen zurückzunehmen. Ist das das offene Projekt, geht es sofort in den eingeschränkten Modus, und der gelbe Chip erscheint in der Statusleiste, außer das Projekt enthält keinen Code.

![Der Teil Vertrauenswürdige Projekte der Einstellungen mit zwei Projekten und einem Ordner samt jedem Projekt darin, jeweils mit einem Knopf Entfernen, der Tooltip eines davon sagt, dass TrenchDot erneut fragt, bevor Godot hier die Skripte ausführt](../assets/running-godot/trusted-projects.png)

<div class="td-mcp" markdown>

## MCP

Ein Agent bekommt die Abfrage nie in den Weg. `open_project` und `get_state` melden `restricted`, und die Tools, die Godot das Projekt ausführen oder bauen ließen, schlagen mit einem Fehler fehl, bis du ihm über den Chip vertraust. `map_file` `open` behält bei einer Karte eines nicht vertrauten Projekts das offene Projekt und gibt dieses Projekt als `other_project` zurück.

`trust` in beiden Ergebnissen sagt, warum ein Projekt nicht im eingeschränkten Modus ist: `stored`, `session` für `--project` oder `no_code`. Speicherstände eines Agenten, mit `map_file`, `run_action` oder in `run_script`, speichern nie Vertrauen, ein Projekt ohne Code, in dem ein Agent arbeitet, wird also erneut abgefragt, sobald es Skripte hat.

</div>
