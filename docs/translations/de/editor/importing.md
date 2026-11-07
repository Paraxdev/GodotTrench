---
source: e0934aeaf4aaffb804a36e5ff5821a38614fc413
---

# Hammer- und TrenchBroom-Karten importieren { #importing-hammer-and-trenchbroom-maps }

Du hast einen Level aus Hammer oder TrenchBroom? Bring ihn rüber, statt ihn neu zu bauen. *Datei > Importieren* öffnet eine Hammer-`.vmf` oder eine `.map` aus TrenchBroom, Quake oder Qodot als neue, ungespeicherte Karte (in einem neuen Tab, wenn die offene Karte ungespeicherte Änderungen hat). Speichere sie als `.tdm`, und Godot baut sie wie jede andere Karte.

## Schritte { #steps }

1. Zuerst das Godot-Projekt öffnen

    Öffne das Projekt, für das die Karte gedacht ist, mit *Godot > Godot-Projekt öffnen…*, denn konvertierte Texturen landen in seinem Texturordner.

2. Die Karte importieren

    Geh auf *Datei > Importieren* und wähl *.vmf-Datei (Hammer)…* oder *.map-Datei (TrenchBroom, Quake)…*, dann die Datei.

    ![Das Menü Datei mit geöffnetem Importieren und .vmf-Datei (Hammer)… unter dem Mauszeiger, darunter Texturen konvertieren und die Modellimporte](../assets/importing/file-import-menu.png)

3. Die Texturen konvertieren

    Wenn Texturen der Karte in ihrer Nähe liegen, fragt der Editor "Texturen der Karte konvertieren?" für die, die dem Projekt fehlen. Sag Ja, siehe [Texturen konvertieren](importing-textures.md).

4. Den Tab Probleme prüfen

    Öffne oben links den Tab Probleme, neben Outliner und Verlauf. Jedes Material, das dem Projekt noch fehlt, erscheint als Warnung, und seine Flächen werden als Schachbrett gezeichnet.

    ![Der Tab Probleme mit zwei Warnungen, Material 'castle/brick_wall' ist nicht im Projekt (12 Flächen), und die 3D-Ansicht mit diesen Wänden und Stufen als graues Schachbrett](../assets/importing/issues-missing-materials.png)

    Die Flächen behalten den Materialnamen, wenn du die Textur später hinzufügst, ist das Problem also behoben. Sobald jede Textur da ist, sind diese Warnungen weg.

    ![Dieselbe Karte nach dem Konvertieren ihrer Texturen: Der Tab Probleme sagt Keine Probleme gefunden, und die Wände zeigen ihre Ziegeltextur](../assets/importing/textures-converted.png)

5. Speichern

    Nimm *Datei > Speichern unter…* und speichere sie als `.tdm` im Projekt.

> [!NOTE]
> Einheiten bleiben, wie sie sind. Source-Karten sind für etwa 40 Einheiten pro Meter gebaut und Quake-Karten für etwa 32, eine Source-Karte wird also etwas größer als in ihrem Spiel, solange das Projekt keinen anderen Maßstab nutzt.

## Was der Import behält { #what-the-import-keeps }

| Hammer | In TrenchDot |
| --- | --- |
| Werkzeugmaterialien (`tools/toolsnodraw`, `toolsclip`, `toolsplayerclip`, `toolstrigger`, `toolsorigin`, ...) | Die Skip-, Clip- und Origin-Texturen des Projekts und `special/trigger` |
| Sky-Flächen | Die Sky-Textur des Projekts, standardmäßig `special/sky` |
| Displacements | Displacements mit ihren Höhen und Blend-Alphas |
| `func_instance` | Eine Gruppe mit dem Inhalt der Instanz, an ihre Stelle verschoben. Namen darin bekommen das Fixup der Instanz, seine Ersetzungen mit `$` und `#` werden eingesetzt, und Outputs, die auf `instance:name;Input` zielen, werden mit der echten Entität verdrahtet |
| Visgroups | Ebenen |
| Versteckte Objekte | Versteckte Nodes |
| Entitäts-Keys und Outputs | Entitäts-Eigenschaften und Outputs |

TrenchBroom-Ebenen und -Gruppen bleiben Ebenen und Gruppen, und Quake-Werkzeugtexturen werden auf dieselbe Weise zu den Werkzeugtexturen des Projekts.

Du brauchst nur ein paar Brushes? Kopier sie in TrenchBroom und drück Ctrl+V in TrenchDot, sie kommen genauso konvertiert herüber, siehe [Prefab-Bibliothek](prefab-library.md#pasting-from-trenchbroom).

Sky-Flächen kollidieren in Godot, zeichnen aber nichts. Der worldspawn-Key `sky_source` behält die Textur, die sie hatten, und das Konvertieren der Texturen macht daraus einen prozeduralen Himmel, siehe [Texturen konvertieren](importing-textures.md#skies).

## Bekannte Einschränkungen { #known-limitations }

| Einschränkung | Was du tun kannst |
| --- | --- |
| Texturen in Archiven wie `.vpk` oder `.pak` werden nicht gelesen | Entpack sie zuerst, zum Beispiel mit VPKEdit |
| Instanzen werden neben der Karte und in den Ordnern darüber gesucht | Behalte die Ordnerstruktur `maps` des Spiels bei. Instanzen, die nicht gefunden werden, bleiben `func_instance`-Entitäten |
| Für Valve- und Quake-Entitätsklassen gibt es in einem neuen Projekt keine Definitionen, der Editor kennt ihre Keys also nicht | Füg sie deiner FGD hinzu oder ersetz sie durch eigene Klassen. Der Tab Probleme listet sie auf |
| Manche Source-Materialfunktionen fehlen: `$basetexture2`-Blends, Detailtexturen, Env-Maps und Proxys | Richte sie in Godot ein |
| Quake-3-Patches (gekrümmte Flächen) und Brush-Primitives werden übersprungen. Die Statusleiste sagt, wie viele | Bau sie als Meshes nach |
| Displacements, deren Vertices sich seitlich bewegen und nicht nur nach oben und unten, werden zu Meshes ohne Blend-Alphas | Bearbeite sie als Meshes |
