---
source: fb9984bb419921807428d1c050d90fa918ed7b18
---

# Texturieren { #texturing }

## Materialien anwenden { #applying-materials }

Deine Texturen liegen im Panel **Materialien**, unten im Fenster unter den Ansichten. Das Suchfeld oben links filtert sie nach Namen, **In Karte verwendet** und **Favoriten** engen die Liste ein, und der Regler stellt die Größe der Vorschaubilder ein.

![Das Panel Materialien mit dem Suchfeld, der Ordnerliste, In Karte verwendet, Favoriten und dem Größenregler, darunter die Vorschaubilder der Materialien, wobei dev/blue überfahren ist und seinen Namen und seine Größe zeigt](../assets/texturing/materials-panel.png)

| So geht's | Ergebnis |
| --- | --- |
| Ein Vorschaubild anklicken | Wendet es auf die Auswahl an und nutzt es für neue Brushes |
| Ein Vorschaubild auf eine Fläche ziehen | Wendet es auf diese Fläche an |
| Shift beim Ablegen | Wendet es auf den ganzen Brush oder das ganze Mesh an |
| Alt beim Ablegen | Legt es als [Decal](decals.md) ab |

Beim Ziehen erinnert dich eine kleine Karte neben dem Cursor an die Modifikatoren, und die Fläche ändert sich, sobald du loslässt.

![Withered/mossy_plaster wird auf die Vorderseite einer Backsteinwand gezogen, dann dieselbe Wand mit dem Putz nur an ihrer Vorderfläche](../assets/texturing/drop-on-face.png)

Um eine Fläche ohne Ziehen zu texturieren, klick sie zuerst in einer Ansicht mit Shift an und klick dann das Vorschaubild an. Mit Ctrl+Shift+Klick fügst du weitere Flächen hinzu oder nimmst sie heraus.

Rechtsklick auf ein Vorschaubild bietet mehr: es zu deinen Favoriten hinzufügen, die Flächen auswählen, die es nutzen, es für das aktuelle Material in der ganzen Karte einwechseln, den Hotspot-Editor öffnen oder mit *Im Dateimanager zeigen* den Ordner seiner Textur mit ausgewählter Datei öffnen, zum Beispiel um sie zu bearbeiten.

![Das Rechtsklick-Menü eines Vorschaubilds mit Auf Auswahl anwenden, Zu Favoriten hinzufügen, Flächen damit auswählen, showcase/red_bricks in der Karte dadurch ersetzen, Hotspot-Editor, Namen kopieren und Im Dateimanager zeigen](../assets/texturing/material-context-menu.png)

Legst du ein Material auf einem Terrain ab, setzt es stattdessen eine Terrainebene, siehe [Terrainebenen](terrain-layers.md).

Die UV-Sperre (Ctrl+Shift+U) ist standardmäßig an, damit Texturen an einem Brush haften bleiben, während du ihn verschiebst.

## Eingebaute Farben { #built-in-colors }

Das Panel Materialien bietet immer `dev/grey`, `dev/dark`, `dev/orange`, `dev/blue` und `dev/green`, Rastertexturen, um ein Level abzustecken, bevor es Grafik hat. Räume damit farblich zu kodieren übersteht den Build, denn das Addon liefert dieselben Raster mit, und Godot nimmt sie, wenn das Projekt keine Textur dieses Namens hat. Eine eigene Textur unter zum Beispiel `res://textures/dev/grey.png` ersetzt eine davon.

## Werkzeugtexturen { #tool-textures }

Der Editor zeichnet Werkzeugtexturen durchsichtig mit Streifen, und der Godot-Build gibt ihnen kein sichtbares Mesh. Jede Textur im Ordner `special/`, `tools/` oder `td/` zählt, ebenso jede Textur, die nach einem der Werkzeuge unten benannt ist, etwa `common/caulk`.

![Ein dev/dark-Block, davor eine durchsichtige lila special/clip-Wand und eine orange gestreifte special/trigger-Box](../assets/texturing/tool-textures.png)

| Textur | In Godot |
| --- | --- |
| `special/clip`, `playerclip`, `nodraw`, `caulk`, `trigger`, `sky` und jedes andere Werkzeug | Kollidiert, zeichnet aber nichts. Ein Trigger behält seine Trigger-Form |
| `special/skip`, `origin`, `hint`, `hintskip`, `null`, `areaportal` | Baut gar nichts. Ein Brush nur aus diesen Flächen kollidiert nicht |
| `special/occluder` | Zeichnet nichts und verdeckt keine Nachbarn, trägt aber zum Occluder einer Entität bei, die einen baut, etwa `func_geo` |

Die Karteneinstellungen nennen die Texturen für Clip, Skip, Origin und Sky, standardmäßig `special/clip` und so weiter. Ein Origin-Brush setzt den Ursprung seiner Entität nur mit der Textur, die die Karteneinstellungen nennen.

## Das Textur-Werkzeug { #the-texture-tool }

Die Ausrichtung einer Fläche ist, wo ihre Textur auf ihr sitzt: Versatz, Skalierung und Drehung. Drück **Shift+T** für das Textur-Werkzeug, das sie an Ort und Stelle bearbeitet, wie der Flächenbearbeitungsmodus von Hammer. Klick eine Fläche an, um sie zu markieren. Sie wird rot getönt und zeigt eine blaue Box um eine Wiederholung der Textur, dazu zwei kurze Linien für die Richtungen U und V. Die Fläche unter dem Cursor bekommt einen gelben Umriss.

![Das Textur-Werkzeug mit markierter Vorderfläche einer Backsteinwand, eine blaue Box markiert eine Texturwiederholung](../assets/texturing/texture-tool-face.png)

| Eingabe | Ergebnis |
| --- | --- |
| Klick, Ctrl+Klick | Klick markiert eine Fläche, Ctrl+Klick fügt eine hinzu oder nimmt sie heraus |
| Eine markierte Fläche ziehen | Verschiebt die Textur, das Pixel unter dem Cursor bleibt darunter |
| Pfeiltasten | Um die Rastergröße in Pixeln verschieben, mit Shift um ein Pixel |
| Ctrl+Mausrad | Skaliert um 2, mit Shift in feinen Schritten |
| Alt+Mausrad | Dreht um 15°, mit Shift um 1° |
| Alt+Klick | Nimmt Material und Ausrichtung einer Fläche auf |
| Rechtsklick | Wendet das aktuelle Material an |
| Ctrl+Rechtsklick | Wendet es an und setzt die Ausrichtung zurück |
| Shift+Rechtsklick | Fügt das aufgenommene Material und seine Ausrichtung ein |
| Alt+Rechtsklick | Wendet das aktuelle Material an und setzt die Ausrichtung der markierten Fläche über die Kante hinweg fort |

Mit Alt+Rechtsklick läuft eine Textur, etwa ein Trim oder eine Reihe Backsteine, sauber um eine Ecke.

Die Optionsleiste des Werkzeugs oben hat die Schaltflächen **Ausrichten**. **Links**, **Rechts**, **Oben**, **Unten** und **Mitte** schieben die Textur an diese Seite der Fläche, **Anpassen** streckt sie über die Fläche, **Breite anp.** und **Höhe anp.** nur in die Breite oder nur in die Höhe. Ist **Als Einheit behandeln** angehakt, wirken sie auf alle markierten Flächen als eine Fläche, sodass eine Textur sich über alle erstreckt. **An Ansicht ausrichten** projiziert die Textur entlang der 3D-Kamera.

![Die Optionsleiste des Textur-Werkzeugs mit Ausrichten Links, Rechts, Oben, Unten, Mitte, Anpassen, Breite anp., Höhe anp., Als Einheit behandeln, An Ansicht ausrichten und Hotspot-Anpassung](../assets/texturing/texture-tool-options.png)

Wenn du lieber Zahlen nutzt, zeigt der **Inspektor** dieselbe Fläche unter **Ausrichtung**, mit Feldern für Versatz, Skalierung und Drehung und Schaltflächen zum Spiegeln, Drehen und Zurücksetzen. **Textur-Werkzeuge** darunter stellt die Texeldichte ein und kopiert die Ausrichtung von Fläche zu Fläche.

![Der Inspektor bei einer markierten Fläche, der Abschnitt Ausrichtung mit Versatz, Skalierung und Drehung, die Schaltflächen Ausrichten und der Abschnitt Textur-Werkzeuge](../assets/texturing/inspector-alignment.png)

Für Flächen mit ausdrücklichen UVs und für Meshes nimmst du den [UV-Editor](uv-editor.md).

## Texturmaßstab { #texture-scale }

Standardmäßig deckt ein Texturpixel eine Karteneinheit ab, eine Textur mit 64 Pixeln wiederholt sich also alle zwei Meter. Das passt zu Texturen mit niedriger Auflösung, aber ein Foto mit 1024 Pixeln würde sich über 32 Meter dehnen. Setz `metadata/texture_size` am Material auf die Weltgröße, die eine Wiederholung abdecken soll, siehe [Texturgröße](../godot/materials.md#texture-size). Die Zeile im Inspektor unter dem Materialnamen sagt dir, was eine Fläche bekommen hat, etwa „1024 x 1024 px, 64 x 64 Einheiten pro Wiederholung“.

Um sie nur für eine Karte zu setzen oder eine Textur nahtlos über alle Flächen laufen zu lassen, nimmst du die [Textureinstellungen](light-baking.md#texture-settings) im Inspektor. Ihre Wiederholungsgröße schlägt die des Materials.

## Hotspots { #hotspots }

Mit Hotspots texturiert ein einziges Trim Sheet viele verschieden geformte Flächen. Du markierst seine Rechtecke einmal, dann wählt TrenchDot für jede Fläche das passende.

1. Klick mit rechts im Panel Materialien auf das Trim Sheet und wähl **Hotspot-Editor**. *Textur > Hotspot-Editor…*
    öffnet ihn auch, für das Material der markierten Fläche oder sonst das aktuelle.

    ![Das Menü Textur mit überfahrenem Hotspot-Editor…, unten Hotspot-Anpassung (Alt+H)](../assets/texturing/menu-texture.png)

2. Zieh auf der Textur, um ein Rechteck zu zeichnen, oder stell ein Raster ein und klick auf **In Raster teilen**. Zieh ein Rechteck, um es zu verschieben, an seinem Eckgriff, um die Größe zu ändern, Rechtsklick löscht es.
3. Klick auf **Speichern**. Die Rechtecke landen in `<texture>.hotspots.json` neben der Texturdatei.
4. Markier die Flächen und nutze *Textur > Hotspot-Anpassung* (Alt+H) oder **Auf Auswahl anwenden** im Editor.

![Der Hotspot-Editor auf withered/concrete_panels, in ein 4 mal 4 großes Raster geteilt, 16 Rechtecke, nicht gespeichert](../assets/texturing/hotspot-editor.png)

Die Hotspot-Anpassung gibt jeder Fläche das Rechteck, dessen Form am besten passt, und bevorzugt eines, das die Textur nahe an ihrem normalen Maßstab hält.

## Vertex-Malen { #vertex-paint }

Das Werkzeug **Malen** (P) malt Vertex-Farben auf Brush-Flächen, für Schmutz, Tönung und gefälschte Beleuchtung. Wähl die Farbe in der Optionsleiste des Werkzeugs, neben Pinselradius und Stärke, und zieh dann über die Flächen. Es malt auf die markierten Brushes, oder auf jeden Brush, wenn nichts markiert ist. Ctrl+Mausrad ändert die Pinselgröße.

![Die Optionsleiste des Werkzeugs Malen mit dem Farbfeld Farbe, Radius 48 u und Stärke 4.0](../assets/texturing/paint-tool-options.png)
