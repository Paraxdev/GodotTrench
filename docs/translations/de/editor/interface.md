---
source: 7982e4d62890afb0393dc32addda8d5b4d33f84a
---

# Oberfläche und Navigation { #interface-and-navigation }

TrenchDot orientiert sich an TrenchBroom und Hammer, wenn du eins davon schon benutzt hast, tragen die meisten Gewohnheiten also weiter. Wenn nicht, führt dich diese Seite herum, und [Von Unity oder Blender kommen](coming-from-unity-blender.md) ordnet die Ideen zu, die du schon kennst.

## Das Fenster { #the-window }

![Der Editor mit einem kleinen Raum: links der Outliner, in der Mitte die Ansichten 3D, Oben, Vorne und Seite, rechts der Inspektor und unten das Panel Materialien](../assets/interface/window.png)

Die Menüs sind oben. Ein Befehl steht unter der Art von Objekt, die er ändert, CSG liegt also unter *Brush* und das Modellieren von Terrain unter *Terrain*.

Findest du etwas nicht? Drück Ctrl+Shift+P (oder F1) für die Befehlspalette und tipp seinen Namen. Befehle mit einem Fachnamen, etwa *Aushöhlen* oder *UV-Sperre*, erklären sich beim Überfahren selbst, und die Palette zeigt dieselbe Zeile unter ihrer Liste. Escape oder ein Klick daneben schließt sie, ohne etwas auszuführen.

![Die Befehlspalette mit eingetipptem hollow, Brush: Aushöhlen gewählt mit seinem Kurzbefehl, und darunter die Zeile, die es erklärt](../assets/interface/command-palette.png)

Direkt unter der Werkzeugleiste liegt die **Optionsleiste des Werkzeugs**. Sie enthält die Einstellungen des aktiven Werkzeugs, oder eine Zeile dazu, wie man ein Werkzeug ohne Einstellungen benutzt. Fahr über den Namen des Werkzeugs dort oder über seine Schaltfläche in der Werkzeugleiste, um eine kurze Beschreibung zu sehen.

![Die Optionsleiste des Werkzeugs, Auswählen-Werkzeug überfahren, sein Tooltip beschreibt das Werkzeug und endet auf F1: Handbuch](../assets/interface/tool-options-tooltip.png)

### Die Karte für die leere Map { #the-empty-map-card }

Auf einer neuen, leeren Map sagt dir eine Karte in der 3D-Ansicht, wo du anfängst: Zieh in der Ansicht Oben, um deine erste Box zu zeichnen, und die Ansicht Oben markiert die Stelle mit einem gelben Schriftzug. Darunter erklärt sie die 3D-Kamera und verlinkt die [Anleitung für den Einstieg](../getting-started.md). Hat das geöffnete Projekt Maps, listet **Eine Karte aus diesem Projekt öffnen** sie auf, sodass du stattdessen eine fertige öffnen kannst.

![Die Karte für die leere Map in der 3D-Ansicht mit Einstieg, Eine Karte aus diesem Projekt öffnen und Tipps ausblenden, neben dem gelben Schriftzug in der Ansicht Oben](../assets/interface/empty-map-card.png)

Die Karte verschwindet, sobald die Map etwas enthält. **Tipps ausblenden** schaltet die Tipps ab, und *Tipps bei leerer Karte* in *Datei > Einstellungen* holt sie zurück.

### Panels { #panels }

| Panel | Andocken | Wofür es da ist |
| --- | --- | --- |
| Outliner | Links | Die Map als Baum aus [Ebenen, Gruppen](organizing.md) und Objekten |
| Verlauf | Links | Jede Bearbeitung. Ein Klick darauf macht sie und alles danach rückgängig, ein Klick auf eine abgeblendete stellt bis dorthin wieder her |
| Probleme | Links | Probleme, die beim Arbeiten auffallen, etwa Flächen ohne Material. Klick auf eins wählt das Objekt aus, die meisten bieten eine Lösung an |
| Inspektor | Rechts | Die Eigenschaften der Auswahl, etwa die Keys einer Entität oder die genaue Position und Größe eines Brushes |
| Entitäten | Rechts | Entitätsklassen zum Ziehen in eine Ansicht |
| UV-Editor | Rechts | [Genaue Texturplatzierung](uv-editor.md) |
| Scatter | Rechts | Die Modelle, die das Werkzeug [Scatter](scatter.md) malt |
| Materialien | Unten | Die Texturen deines Projekts |
| Modelle | Unten | [Modelldateien](models.md) zum Platzieren |
| Prefabs | Unten | Fertige Teile aus der [Prefab-Bibliothek](prefab-library.md) zum Einfügen |
| Logik | Startet geschlossen | Die [Entitätsverdrahtung als Graph](../gameplay/logic-graph.md), um Entitäten zu verbinden und vorab zu sehen, was ein Output auslöst |
| Referenz | Startet geschlossen | Code, um eine Entitätsklasse aus deinen Skripten zu nutzen |

Ein Panel versehentlich geschlossen? Öffne es wieder über *Ansicht > Panels*. *Layout zurücksetzen* unten im selben Menü stellt alles wieder so hin, wie es am Anfang war.

![Das Menü Ansicht mit offenem Panels, die sichtbaren Panels angehakt und Layout zurücksetzen ganz unten](../assets/interface/view-panels-menu.png)

## Hilfe und das Handbuch { #help-and-the-manual }

Tooltips erklären Schaltflächen, Tabs und Einstellungen. Für mehr zeigst du auf ein Werkzeug, einen Panel-Tab, einen Menüeintrag oder einen Abschnitt des Inspektors und drückst **F1**, und die passende Seite dieses Handbuchs öffnet sich. Ein Ctrl+Klick auf ein Werkzeug in der Werkzeugleiste (Cmd+Klick auf macOS) tut dasselbe. Ein Tooltip, der auf *F1: Handbuch* endet, hat eine Seite dahinter, über allem anderen öffnet F1 die Befehlspalette.

*Hilfe > Handbuch* öffnet den Anfang des Handbuchs.

![Das Menü Hilfe mit überfahrenem Handbuch](../assets/interface/help-menu.png)

Kennst du dich aus, hak *Hilfetext* in *Datei > Einstellungen* ab, um die kurzen Hinweiszeilen in der Optionsleiste des Werkzeugs und in den Panels auszublenden. Ihre Tooltips bleiben.

(Hinweis: Links öffnen sich in deinem Webbrowser. Öffnet sich kein Browser, sagt das die Statusleiste, und der Link wird in die Zwischenablage kopiert.)

## Ansichten { #views }

Die Ansichten sind 3D, Oben, Vorne und Seite. Die 2D-Ansichten schauen gerade entlang einer Achse, was das Ausrichten erleichtert. Ist etwas markiert, schreiben sie seine Breite darüber und seine Höhe daneben.

![Die Ansicht Vorne mit einem markierten Box-Brush, 96 darüber und 48 daneben geschrieben](../assets/interface/front-view-size-labels.png)

Ein Klick in einer 2D-Ansicht wählt das nächste Objekt entlang der Ansicht, in der Ansicht Oben verdeckt eine Decke also die Wände darunter. Klick innerhalb von etwa anderthalb Sekunden noch einmal an dieselbe Stelle, oder Alt+Klick jederzeit, um zum nächsten Objekt unter dem Cursor zu springen.

| So geht's | Wirkung |
| --- | --- |
| Den Punkt ziehen, an dem sich die Trennlinien kreuzen | Ändert die Größe aller vier Ansichten gleichzeitig |
| Shift+Space | Maximiert die Ansicht unter dem Mauszeiger, noch einmal stellt sie wieder her |
| *Ansicht > Ansichten* | Schaltet zwischen einer, zwei und vier Ansichten um oder holt eine geschlossene Ansicht zurück. Eine Ansicht behält die Ansicht, in der du zuletzt gearbeitet hast, zu Beginn die 3D-Ansicht, und solange nur eine Ansicht zu sehen ist, wechselt die Wahl des Namens einer anderen Ansicht zu ihr |

![Das Menü Ansicht mit offenem Ansichten: die vier Ansichten angehakt, Eine, Zwei und Vier Ansichten und Ansicht maximieren auf Shift+Space](../assets/interface/view-views-menu.png)

## Tabs { #tabs }

Jede offene Map bekommt einen Tab, und die Tab-Leiste erscheint unter der Optionsleiste des Werkzeugs, sobald zwei offen sind. Ctrl+Shift+N (oder die Schaltfläche **+**) öffnet einen neuen Tab, Ctrl+Tab wechselt und Ctrl+W schließt einen.

![Die Tab-Leiste mit geöffneten room.tdm und untitled.tdm und der überfahrenen Schaltfläche +, die Neuer Tab zeigt](../assets/interface/map-tabs.png)

*Neue Karte* und das Importieren öffnen in einem neuen Tab, wenn die aktuelle Map ungespeicherte Änderungen hat. Das Schließen eines geänderten Tabs fragt, ob gespeichert, verworfen oder abgebrochen werden soll, und beim Beenden werden alle ungespeicherten Maps aufgelistet.

## Kamera { #camera }

| Eingabe | 3D-Ansicht | 2D-Ansichten |
| --- | --- | --- |
| Mit rechts ziehen | Umsehen. WASD bewegt, Q und E gehen nach unten und oben, Shift verdreifacht die Geschwindigkeit | Verschieben |
| Mit der Mitte ziehen | Verschieben | Verschieben |
| Mausrad | Vor und zurück bewegen, schneller, je weiter die Fläche direkt voraus entfernt ist | Zum Cursor hin zoomen |
| Alt+Ziehen mit links | Umkreisen, außer das Ziehen beginnt auf der Auswahl, die sich dann senkrecht bewegt | |

WASD fliegt nur, solange du die rechte Taste hältst, sonst sind diese Tasten Kurzbefehle. Die Steuerung vergessen? Fahr über den Tab einer Ansicht. Bis du dich zum ersten Mal umgesehen hast, zeigt die 3D-Ansicht sie auch unter ihrem Namen, solange der Zeiger darüber ist (**Tipps ausblenden** auf der Karte für die leere Map schaltet das ab).

**F** zentriert die Auswahl in jeder Ansicht, oder die ganze Map, wenn nichts markiert ist. Eine Rasterung des Mausrads zoomt eine 2D-Ansicht bei jeder Zoomstufe um denselben Faktor, von einem ganzen großen Terrain hinunter zu einem einzelnen Raum sind es also etwa 20 Rasterungen, und **F** auf dem Raum geht schneller.

Ctrl+Shift+1 bis 9 speichert die 3D-Kamera als Lesezeichen, und Ctrl+1 bis 9 springt zurück. Lesezeichen werden in der Map gespeichert, und Rückgängig rührt sie nicht an.

## Schattierung { #shading }

F4 schaltet die 3D-Ansicht durch Texturiert, Flach, Beleuchtete Vorschau und Drahtgitter, und die Ansicht schreibt den Modus oben links hin. Sobald die Map ein [Backen](light-baking.md) hat, kommt **Gebackene Beleuchtung** nach der Beleuchteten Vorschau. F3 springt direkt auf die **Beleuchtete Vorschau** und zurück auf Texturiert, ohne die anderen zu durchlaufen.

![Die 3D-Ansicht, die durch texturierte, flache, beleuchtete und Drahtgitter-Schattierung wechselt](../assets/interface/shading-cycle.gif)

Die Beleuchtete Vorschau zeigt Lichter, Sonnenschatten, Himmel und Nebel der Map, ohne in Godot zu bauen. Sie zeichnet nur die 64 stärksten Punkt- und Spotlichter, eine Map mit deutlich mehr kann hier also dunkler wirken als in Godot.

## Design { #theme }

TrenchDot gibt es in einem dunklen und einem hellen Look. Eine frische Installation startet dunkel.

1. Öffne **Datei > Einstellungen**. **Design** ist die erste Zeile, oben im Fenster.
2. Klick auf das Dropdown und wähl **Dunkel**, **Hell** oder **System**.

    ![Das Dropdown Design in den Einstellungen, geöffnet mit Dunkel, Hell und System](../assets/interface/preferences-theme.png)

3. Der ganze Editor wechselt im nächsten Frame, ohne Neustart. Die Wahl wird mit deinen anderen Einstellungen gespeichert.

    ![Der Editor im hellen Design, mit der Menüleiste, dem Outliner, der 3D-Ansicht und der Ansicht Oben](../assets/interface/theme-light-window.png)

**System** folgt dem hellen oder dunklen App-Modus deines Betriebssystems und ändert sich mit ihm. Solange es gewählt ist, steht im Dropdown *Systemvorgabe (Hell)* oder *Systemvorgabe (Dunkel)*, damit du siehst, was dein System geliefert hat. Meldet das System keinen Modus, bleibt der Editor dunkel.

Du kannst es auch unter **Ansicht > Design** wählen, direkt unter *Oberflächengröße*, oder in der Befehlspalette als *Ansicht: Design: Dunkel*, *Hell* oder *System*. Standardmäßig ist keine Taste belegt, aber im Tastenbelegungs-Editor kannst du jedem eine Taste geben.

Im hellen Design werden die 2D-Ansichten (Oben, Vorne und Seite) zu Papier, ein helles Grau als Hintergrund mit dunklen Rasterlinien und Brush-Kanten, damit sich die Linien, die du zeichnest, auf einen Blick lesen lassen. Die 3D-Ansicht behält in beiden Designs ihre dunkle Umgebung, ebenso die Texturflächen des UV-Editors und des Hotspot-Editors. Eine Textur sieht in beiden Designs gleich aus, und eine helle Umgebung würde verändern, wie du ihre Farben beurteilst.

<div class="td-mcp" markdown>

Über den [MCP-Server](../mcp.md) schaltet `set_editor` mit `"theme": "light"` das Design um. Die Werte sind `dark`, `light` und `system`, ein unbekannter ist ein Fehler, der nichts ändert. `get_state` meldet die Wahl als `ui.theme` und was auf dem Bildschirm ist als `ui.theme_active`.

</div>

## Sprache { #language }

Die Menüs, Panels, Tooltips und Statusmeldungen gibt es auf Englisch, Deutsch, Französisch, Spanisch, Portugiesisch, Russisch, in vereinfachtem Chinesisch und auf Japanisch. Eine frische Installation startet auf Englisch.

1. Öffne **Datei > Einstellungen**. **Sprache** ist die zweite Zeile, direkt unter **Design**.
2. Klick auf das Dropdown und wähl deine Sprache. Jede Sprache steht unter ihrem eigenen Namen, du findest deine also auch dann, wenn der Editor in einer Sprache läuft, die du nicht lesen kannst.

    ![Das Dropdown Sprache in den Einstellungen, geöffnet mit English, Deutsch, Français, Español, Português, Русский, 简体中文 und 日本語](../assets/interface/preferences-language.png)

3. Der ganze Editor wechselt sofort, ohne Neustart. Die Wahl wird mit deinen anderen Einstellungen gespeichert.

    ![Das Menü Datei auf Deutsch](../assets/interface/language-file-menu-de.png)

Ist eine andere Sprache als Englisch an, steht in der Zeile zum Beispiel *Sprache (Language)*, damit du immer zurückfindest. Du kannst eine Sprache auch in der Befehlspalette wählen, als *Ansicht: Sprache: Deutsch*, und derselbe Befehl funktioniert in jeder Sprache, denn die Palette durchsucht auch die englischen Namen. Standardmäßig ist keine Taste belegt, aber im Tastenbelegungs-Editor kannst du jeder Sprache eine Taste geben.

Die Übersetzungen stammen von einer KI, und noch hat sie kein Muttersprachler geprüft, ein Hinweis unter der Zeile sagt das. Ein Text ohne Übersetzung erscheint auf Englisch, ebenso eine Sprache, die diese Version nicht kennt. Liest sich eine Übersetzung falsch, kannst du sie in [der Locale-Datei dieser Sprache](https://github.com/Paraxdev/TrenchDot/tree/main/crates/td_editor/locales) korrigieren, siehe [Übersetzungen](../development.md#translations).

> [!NOTE]
> Das Handbuch folgt der Sprache des Editors. Ist eine Sprache gewählt, öffnen **F1** über einem Werkzeug, Panel oder Menü und die Einträge des Menüs **Hilfe** diese Seite des Handbuchs in deiner Sprache. Die Sprachschaltfläche in der Kopfzeile des Handbuchs wechselt die Sprache dort. Eine Seite, die noch nicht übersetzt ist oder sich seit der Übersetzung geändert hat, erscheint auf Englisch mit einem Hinweis oben. Die Screenshots zeigen den englischen Editor, such also denselben Eintrag in deiner Sprache oder tipp den englischen Namen in die Befehlspalette. Klassennamen von Entitäten, Map-Keys, Dateiformate, das Log und der [MCP-Server](../mcp.md) bleiben in jeder Sprache englisch.

<div class="td-mcp" markdown>

Über den [MCP-Server](../mcp.md) schaltet `set_editor` mit `"language": "de"` die Sprache um. Die Werte sind `en`, `de`, `fr`, `es`, `pt`, `ru`, `zh-Hans` und `ja`, in beliebiger Groß- und Kleinschreibung, und ein unbekannter ist ein Fehler, der sie alle nennt und nichts ändert. `get_state` meldet die Wahl als `ui.language`. Tool-Namen, Antworten und Fehler bleiben in jeder Sprache englisch. Ein mit `--default-prefs` gestarteter Editor läuft auf Englisch, worauf sich die e2e-Tests verlassen.

</div>

## Oberflächenskalierung { #interface-scale }

Ctrl+= und Ctrl+- skalieren die Oberfläche, und Ctrl+0 setzt sie zurück. Dasselbe findest du als *Oberflächenskalierung* oben in *Datei > Einstellungen*. Meldet der Monitor die falsche Anzeigeskalierung, hak direkt darunter *dem Monitor folgen* ab.

![Der obere Teil des Fensters Einstellungen: Design auf Dunkel, Sprache auf Englisch, Oberflächenskalierung bei 100%, Anzeigeskalierung mit angehaktem dem Monitor folgen, und darunter Tipps bei leerer Karte und Hilfetext](../assets/interface/preferences-scale-and-tips.png)
