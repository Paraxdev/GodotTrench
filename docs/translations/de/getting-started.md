---
source: 18fb3370f830b9fda2728d12b95c3b5cef06be64
---

# Erste Schritte { #getting-started }

Schritte 1 bis 3: die Ersteinrichtung. Wenn du schon ein Godot-Projekt hast, das mit TrenchDot eingerichtet ist, kannst du sie überspringen.

Schritte 4 und 5: den ersten Raum in TrenchDot bauen und in Godot ansehen.

## Was du brauchst { #what-you-need }

* Godot 4.7 oder neuer, die .NET-Version, wenn du Entitäten in C# schreiben willst
* Das neueste [TrenchDot-Release](https://github.com/Paraxdev/TrenchDot/releases/tag/beta)

## Sorg dafür, dass TrenchDot die Godot-Programmdatei findet { #ensure-trenchdot-can-see-the-godot-executable }

Stell sicher, dass TrenchDot deine Godot-Installation sieht. Ob alles richtig erkannt wurde, prüfst du unter
*Datei > Einstellungen… > Godot-Programm*.

Wenn TrenchDot Godot von selbst gefunden hat, steht in der Zeile unter dem Eingabefeld schon „Verwendet“ und der Pfad. Steht dort
stattdessen „Nicht gefunden“, musst du die Programmdatei auf deiner Platte suchen.

Klick dafür auf die Schaltfläche „…“ neben dem Eingabefeld

![Die Schaltfläche … neben dem Eingabefeld Godot-Programm](assets/getting-started/godot-path-browse.png)

und such die Datei, die mit „Godot_v“ beginnt.

Wenn du die richtige Datei gewählt hast, sollte dort „Verwendet“ und der Pfad stehen, den du gerade gewählt hast.

![Die Zeile unter dem Eingabefeld, die Verwendet und den Pfad zur Godot-Programmdatei zeigt](assets/getting-started/godot-path-using.png)

## 1. Ein Projekt in Godot erstellen { #1-create-a-project-in-godot }

Öffne Godot und klick links oben auf „Create“.

![Godots Projektmanager, die Schaltfläche Create mit einem Plus-Symbol links oben](assets/getting-started/godot-create-project.png)

Wähl einen Projektnamen, für dieses Tutorial TrenchDotTest, und (optional) einen Ordner, in dem das Projekt liegen
soll.

Klick dann auf „Create“.

## 2. Das Projekt in TrenchDot öffnen { #2-opening-the-project-within-trenchdot }

Wähl *Godot > Godot-Projekt öffnen…*

![Das Menü Godot in TrenchDot mit hervorgehobenem Eintrag Godot-Projekt öffnen…](assets/getting-started/menu-open-godot-project.png)

oder die Schaltfläche **Godot-Projekt öffnen…** rechts oben in der Werkzeugleiste

![Die Schaltfläche Godot-Projekt öffnen… rechts oben in der TrenchDot-Werkzeugleiste, die den Projektnamen zeigt, sobald eins geöffnet ist](assets/getting-started/toolbar-open-godot-project.png)

und such den Projektordner oder einen beliebigen Ordner darin aus. Der Editor öffnet es beim nächsten Start wieder.

## 3. Der Willkommensbildschirm { #3-the-welcome-screen }

Sobald TrenchDot erkennt, dass es ein neues Godot-Projekt ist, zeigt es das Fenster *Dieses Projekt einrichten*.

![Das Fenster Dieses Projekt einrichten mit den Bereichen Addon, Gameplay-Entitäten und fertiger Inhalt](assets/getting-started/setup-window.png)

Hier kannst du das Addon herunterladen, das Godot braucht, um deine Karten in Szenen zu verwandeln und mit TrenchDot zu sprechen, und
auf Wunsch zusätzlichen Inhalt. Du kannst natürlich auch ohne alles anfangen.

Ein paar einfache Entitäten sind immer dabei, etwa ein schlichtes `info_player_start` und `light`.

1. Das Addon installieren

    Klick oben auf „Addon installieren“, um TrenchDots Fork von FuncGodot in dein Projekt herunterzuladen, das
    ist für TrenchDot unverzichtbar.
    (Hinweis: Das lädt von GitHub-Releases. Hast du keinen Internetzugang, lade zuerst
    `func_godot-trenchdot-addon.zip` herunter und entpacke sie in den Projektordner, um es von Hand zu installieren.)

    Danach klickst du auf „Plugin aktivieren“, damit Godot es lädt.
    (Hinweis: Schließ zuerst Godot, falls es das Projekt offen hat, sonst schreibt Godot seine eigene Kopie von `project.godot` zurück
    und das Plugin bleibt aus.)

2. Die Gameplay-Entitäten installieren

    Klick unter dem Addon auf **Gameplay-Entitäten installieren**.

    ![Der Bereich Gameplay-Entitäten im Fenster Dieses Projekt einrichten mit der Schaltfläche Gameplay-Entitäten installieren](assets/getting-started/setup-gameplay-entities.png)

    Türen, Knöpfe, Aufzüge, Trigger, Props und Spawner kommen mit diesem Paket. Ohne es baut Godot sie als schlichte
    Formen, die keine Kollision haben und nichts tun. Die Tür- und Trigger-Helfer im Menü Gameplay installieren das Paket
    beim ersten Benutzen von selbst, aber es jetzt zu installieren, bevor Godot das Projekt zum ersten Mal öffnet, heißt, dass
    nie eine Szene ohne es gebaut wird.

3. (Optional) Zusätzlicher Inhalt

    TrenchDot hat optionalen Inhalt, den du vorab laden kannst: CC0-Texturen und -Modelle von Poly Haven und ambientCG sowie Modelle,
    die für TrenchDot gemacht wurden. Die Lizenzen und eine Liste, woher alles stammt, sind dabei.

    Willst du keinen zusätzlichen Inhalt, nimm „Nichts zusätzlich“, die minimalen Entitäten sind trotzdem enthalten.

4. Das Projekt einmal in Godot öffnen

    Sobald das Plugin aktiviert ist, öffne das Projekt in Godot (dafür hat das Fenster eine Schaltfläche „Projekt im Godot-Editor öffnen“). Das Addon legt dann eine Datei namens `trenchdot_game.json` im
    Projektordner an. Daraus erfährt TrenchDot, welche Texturen und Entitäten du hast, und TrenchDot lädt sie innerhalb einer Sekunde.

    Das Addon schreibt sie immer wieder neu, wenn sich Dateien ändern, du musst sie also nicht von Hand bearbeiten. Solange sie fehlt, sagt die Statusleiste,
    dass es keine `trenchdot_game.json` gibt, und der Editor bietet die Kern-Entitäten an, dazu die Gameplay-Entitäten,
    wenn du sie installiert hast.

Eigene Entitäten findest du unter [Eigene Entitäten in GDScript](gameplay/custom-entities.md) und
[C#-Entitäten](gameplay/csharp.md). Die Ressourcen, die sie auflisten, erklärt [Projekteinrichtung](godot/project-setup.md).

## 4. Einen Raum bauen { #4-build-a-room }

Der Editor hat 4 Hauptansichten: eine 3D-Ansicht, in der du herumfliegen kannst wie in Godot, und 3 Drahtgitter-Ansichten von oben,
vorne und von der Seite.

Brushes und Operationen hängen von der Rastergröße ab. In der Werkzeugleiste stellst du deine Rastergröße ein (kleiner ist feiner).

![Die Einstellung der Rastergröße in der Werkzeugleiste, auf 16 gesetzt](assets/getting-started/toolbar-grid-size.png)

Auf einem leeren Level siehst du einen kleinen Infotext, der das Zeichnen erklärt. Du kannst ihn mit „Tipps ausblenden“ verstecken
oder einfach loszeichnen, er verschwindet, sobald etwas in der Karte ist.

![Eine neue Karte: Die Karte in der 3D-Ansicht sagt, du sollst in der Ansicht Oben ziehen, um die erste Box zu zeichnen, und die Ansicht Oben zeigt ein gelbes Label „Hier ziehen, um die erste Box zu zeichnen“](assets/getting-started/empty-map.png)

1. Deine erste Box zeichnen

    Zieh mit dem Werkzeug Auswählen (Q) in einer beliebigen Ansicht auf leerem Raum.
    Du solltest sehen, wie eine Box entsteht. Bewege die Maus bei gedrückter linker Taste, um ein Rechteck zu formen.

    ![Alle vier Ansichten mit der ersten gezeichneten Box, die Ansicht Oben schreibt ihre Größe, 512 mal 256](assets/getting-started/draw-first-box.png)

2. Höhe hinzufügen

    Halte in der Ansicht Seite die Maus über die Oberkante der Box, bis sie gelb wird, drück die linke Taste und
    bewege die Maus nach oben, bis die Box 256 Einheiten hoch ist.

    ![Die Ansicht Seite mit der nach oben gezogenen Oberkante der Box, 256 steht darüber und daneben](assets/getting-started/side-view-depth.png)

3. Aushöhlen

    Drück Ctrl+Shift+K oder nutze die Schaltfläche Aushöhlen in der Werkzeugleiste.

    ![Die Werkzeugleiste mit der Schaltfläche Aushöhlen unter der Maus und ihrem Tooltip](assets/getting-started/toolbar-hollow.png)

    Die Box ist jetzt in Wände, einen Boden und eine Decke zerlegt, das ist ideal für einen kleineren Raum oder
    Korridor. Flieg nun mit den Flugsteuerungen (rechte Maustaste halten und WASD benutzen) in die hohle Box hinein.

4. Einen Türdurchgang ausschneiden

    Mit Escape hebst du die Auswahl der gewählten Brushes auf. Ist nichts mehr gewählt, zeichne in der Ansicht Vorne ein kleineres senkrechtes Rechteck innerhalb
    der Box, geformt wie ein Türdurchgang.

    ![Der Tür-Brush in der hohlen Box, 64 breit und 96 hoch in der Ansicht Vorne](assets/getting-started/doorway-brush.png)

    Zieh dann in der Ansicht Seite eine Seite des Tür-Brushs auseinander, sodass er Tiefe bekommt, ganz durch die Wand.

    ![Eine Seite des Tür-Brushs in der Ansicht Seite ziehen, sodass er die Wand durchquert](assets/getting-started/doorway-extend.gif)

    Drück jetzt bei gewähltem Brush Ctrl+K oder nutze die Schaltfläche Subtrahieren in der Werkzeugleiste.

    ![Die Werkzeugleiste mit der Schaltfläche CSG Subtrahieren unter der Maus und ihrem Tooltip](assets/getting-started/toolbar-subtract.png)

    Du solltest sehen, wie der Brush verschwindet und der Türdurchgang aus der Wand ausgeschnitten ist.

    ![Die Wand in der 3D-Ansicht mit ausgeschnittenem Türdurchgang](assets/getting-started/doorway-cut.png)

5. Texturieren

    Du solltest 5 mitgelieferte Texturen zum Prototyping haben. Um eine Textur nur auf eine Fläche zu legen (zum Beispiel den Türrahmen), zieh
    die Textur aus dem Tab Materialien auf die Fläche (halte Shift, um den ganzen Brush zu bedecken).

    ![dev/blue wird aus dem Tab Materialien auf eine Fläche gezogen, mit der Vorschau des Ablegens](assets/getting-started/texture-drop.png)

6. Die Karte speichern

    Um die Änderungen in Godot zu sehen, müssen wir die Karte speichern. Drück Ctrl+S oder nutze *Datei > Speichern*.

    Der Speichern-Dialog öffnet sich im Ordner deines Godot-Projekts. Speichere sie irgendwo im Projekt, ein Ordner `maps` im
    Projektordner hält die Dinge ordentlich, zum Beispiel als `res://maps/room.tdm`.
    (Hinweis: Der Ordnername ist dir überlassen, aber behalte die Karte im Projekt, damit Godot sie über ihren `res://`-Pfad
    laden kann.)

    Nach dem Speichern erscheint sie unter *Datei > Karten im Projekt*.

    ![Das Menü Datei mit Speichern und Karten im Projekt, das room unter res://maps auflistet](assets/getting-started/file-menu-maps-in-project.png)

## 5. In Godot ansehen { #5-see-it-in-godot }

In Godot ist ein Level eine Szene: ein Baum aus Nodes, gespeichert als `.tscn`-Datei, wobei jeder Node eine Sache ist, etwa ein Mesh, ein
Licht oder eine Kamera. Godot schreibt Pfade im Projekt mit `res://`, also ist `res://maps/room.tdm` die Datei
`maps/room.tdm` in deinem Projektordner. Die Karte wird von einem Node des Addons in Nodes verwandelt, `FuncGodotMap`.

1. Wähl im Godot-Editor *Scene > New Scene* und klick im Scene-Panel links auf **3D Scene**.
2. Drück Ctrl+A (*Add Child Node*), such nach `FuncGodotMap` und füge ihn hinzu.
3. Setz bei gewähltem neuem Node *Local Map File* im Inspektor von Godot selbst auf deine `.tdm`, das ist das Panel rechts im
   Godot-Editor. Er funktioniert wie der Inspektor in TrenchDot, gehört aber zu Godot.
4. Drück oben in diesem Inspektor auf **Build Map** und speichere die Szene dann mit Ctrl+S.

Aus Brushes werden Meshes mit Kollision und aus Entitäten Nodes mit ihren Skripten, gespeichert mit der Szene.
[Karten bauen](godot/building.md) erklärt die anderen Einstellungen des Nodes. Kommt eine Entität als bloßes Mesh ohne
Kollision heraus, sagt die Statusleiste von TrenchDot, warum, siehe [Entitäten, die nichts tun](godot/entities-troubleshooting.md).

Lass diese Szene in Godot offen, und jedes Speichern in TrenchDot baut sie neu, sofort, solange ihr Tab der aktuelle ist,
oder wenn du zu ihm zurückwechselst. Speichere die Szene in Godot, um das Ergebnis zu behalten. Baut ein Speichern nicht neu, sagt
die Statusleiste, warum, siehe [Neu bauen beim Speichern](godot/live.md#rebuild-on-save).

> [!WARNING]
> Ein Build löscht zuerst die Kinder der `FuncGodotMap`. Setz deine eigenen Nodes neben den Karten-Node oder in
> ein [Overlay](godot/overlays.md).

## Das Demo-Projekt ausprobieren { #try-the-demo-project }

Der Ordner `godot` des Repositorys, auch im Release als `trenchdot-demo-project.zip`, ist ein Projekt mit installiertem Addon.
Seine Hauptszene spielt die Showcase-Karten ab, die in `godot/demo/maps/showcase` gespeichert sind. Öffne das Projekt im
Editor und wähl eine unter *Datei > Karten im Projekt*, um zu sehen, wie eine fertige Karte aufgebaut ist.

Die Demo-Auswahl von *Godot > Inhalt zum Projekt hinzufügen…* legt dieselben Karten und Szenen in dein eigenes Projekt. Öffne in Godot
`res://demo/demo.tscn` und drück F6, um sie abzuspielen. In TrenchDot aus deinem Projekt geöffnet, erscheinen sie ohne ihre
Texturen, weil sie den Texturordner `res://demo/textures` der Demo nutzen und nicht den deines Projekts. Das Demo-Projekt
zeigt sie so, wie sie gebaut wurden.

| Taste | Aktion |
| --- | --- |
| WASD, Space, Shift | Gehen, springen, rennen |
| E | Türen und Knöpfe benutzen |
| 1 bis 5 | Zwischen den Showcase-Karten wechseln |
| F3 | Das I/O-Debug-Overlay zeigen, ein Protokoll jedes Inputs und Outputs, sobald er feuert, mit eingezeichneten Trigger-Volumen |

## Weiter { #next }

Damit der Raum etwas tut, lies [So funktioniert Entity-I/O](gameplay/io.md) und baue
[Knopf öffnet eine Tür](tutorials/button-door.md).

Dein eigener Spieler braucht für die Entitäten des Pakets zwei Dinge. Trigger reagieren nur auf Körper in der Godot-Gruppe `player`
auf Physik-Layer 1, setz deinen Spieler also dorthin. Türen und Knöpfe bewegen sich, wenn dein Spielercode `use()` an dem aufruft, den er
ansieht, siehe [Dem Spieler das Drücken ermöglichen](tutorials/button-door.md#let-the-player-press-it).

<div class="td-mcp" markdown>

## MCP { #mcp }

Die Inhaltsfrage öffnet sich nicht von selbst in einem Editor, den ein MCP-Client steuert: Der erste Tool-Aufruf außer
`get_state`, `screenshot` und `simulate_input` schließt sie, und ein Projekt, das mit `open_project` geöffnet wurde, wird nicht gefragt.
`project_content` fügt denselben Inhalt hinzu, siehe [MCP](mcp.md).

</div>
