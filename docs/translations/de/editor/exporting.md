---
source: 5b25efdda7a0b559f9cff81871758a2f55e33a9c
---

# Export nach Blender und in andere 3D-Tools { #exporting-to-blender-and-other-3d-tools }

Du kannst speichern, wie deine Karte aussieht, als eine einzige 3D-Modelldatei: Brushes, Meshes, Terrain und Modelle, mit ihren Texturen darin. Praktisch, um einen Level in Blender zu rendern, eine Trailer-Einstellung aufzubauen oder einer Künstlerin oder einem Künstler ein Blockout zu geben, die oder der darüber die fertigen Meshes baut.

> [!WARNING]
> Exportiert werden nur Geometrie und Materialien. Entitätslogik, I/O-Verdrahtung, Trigger, Skripte und Gameplay-Verhalten bleiben draußen, und die Datei lässt sich nicht wieder in TrenchDot als Karte öffnen. Die `.tdm` bleibt die Karte, bearbeite weiter sie.

## Schritte { #steps }

1. Den Export öffnen

    Geh auf *Datei > Exportieren > glTF Binary (.glb) für Blender und andere 3D-Werkzeuge…*, oder tipp "glTF" in die Befehlspalette (Ctrl+Shift+P oder F1).

    ![Das Menü Datei mit geöffnetem Exportieren und glTF Binary (.glb) für Blender und andere 3D-Werkzeuge unter dem Mauszeiger](../assets/exporting/menu-export.png)

2. Auswählen, was hineinkommt

    Das Fenster *Für 3D-Werkzeuge exportieren* öffnet sich. Hak an, was du willst, siehe [die Optionen](#options) unten. Die Zahlen neben *Prop-Modelle* und *Scatter-Instanzen* sagen dir, wie viele Objekte jede Option hinzufügt. Oben kannst du das Format auf OBJ umstellen.

    ![Das Fenster Für 3D-Werkzeuge exportieren mit gewähltem glTF Binary, Ganze Karte, angehakten Prop-Modellen (5) und nicht angehakten Scatter-Instanzen (4081)](../assets/exporting/export-dialog.png)

3. Speichern

    Klick auf **Exportieren…** und wähl, wohin die Datei kommt. Der Export läuft im Hintergrund mit einem Fortschrittsfenster, du kannst also weiterarbeiten, und **Abbrechen** stoppt ihn, ohne etwas zu schreiben. Ist er fertig, sagt die Statusleiste, wie viele Objekte, Materialien und Dreiecke hinausgegangen sind.

4. In Blender öffnen

    Nimm *File > Import > glTF 2.0 (.glb/.gltf)* und wähl die Datei. Die Standardwerte stimmen, der Importer dreht das Modell von allein mit Z nach oben.

Die Datei enthält die Karte so, wie sie war, als du auf **Exportieren…** geklickt hast. Ungespeicherte Änderungen werden vorher automatisch gespeichert, außer das Autospeichern ist unter *Datei > Einstellungen* aus. Nennt die Statusleiste ein Material, für das das Projekt kein Bild hat, kommt dieses ohne Textur heraus.

## Optionen { #options }

| Option | Bewirkt |
| --- | --- |
| Ganze Karte, Nur Auswahl | Alles, oder nur die gewählten Objekte, weiterhin in ihren Ebenen und Gruppen |
| Ausgeblendete Ebenen und Objekte | Exportiert auch, was in den Ansichten ausgeblendet ist. Aus bekommst du, was du siehst |
| Prop-Modelle | Die Modelle, die Props und andere Punkt-Entitäten zeigen, an ihrem Platz, mit ihrer Drehung und Skalierung |
| Scatter-Instanzen | Jeder Baum, Fels und Grasbüschel der [Scatter-Sets](scatter.md) als eigenes Objekt. Standardmäßig aus, denn Tausende davon ergeben eine große Datei, die sich langsam öffnet |
| Punkt-Entitäten als leere Marker | Ein leeres Objekt, benannt wie die Entität, dort, wo jede Punkt-Entität ohne Modell steht, praktisch, um in Blender Lichter oder Kameras zu platzieren |
| Unbenannte Brushes zusammenführen | Brushes, die du nicht umbenannt hast, werden zu einem Objekt pro Ebene, Gruppe oder Entität, was den Outliner von Blender kurz hält. Benannte Brushes bleiben getrennt |

## Was exportiert wird { #what-is-exported }

| In der Karte | In der Datei |
| --- | --- |
| Ebenen und Gruppen | Leere Elternobjekte mit denselben Namen, sodass die Anordnung des Outliners übernommen wird |
| Brushes | Je ein Objekt, benannt wie im Outliner, `brush12` oder der Name, den du vergeben hast |
| Brush-Entitäten wie Türen und Aufzüge | Ein Elternobjekt, benannt nach der Entität, das ihre Brushes hält, dort, wo sie im Editor stehen |
| Meshes, Decals | Je ein Objekt. Decals bekommen geblendete, zweiseitige Materialien |
| Terrain | Ein Objekt. Jedes Dreieck bekommt die Textur seiner stärksten gemalten Ebene, gemalte Übergänge wirken also gestuft |
| Prefab-Instanzen | Der Inhalt des Prefabs, an seinen Platz verschoben, unter einem Objekt, das nach der Instanz benannt ist |
| Materialien | Eins pro Material mit seiner Albedo-Textur, dazu die Normal-, Roughness-, Metallic-, Ambient-Occlusion- und Emissions-Maps, die es hat. Ausgeschnittene und durchsichtige Materialien behalten ihr Alpha, Pixel Art bleibt scharf. Eingebaute Dev-Farben kommen als ihre Gittertexturen heraus |

Die Datei lässt weg, was der Godot-Build nicht zeichnet: Flächen mit Werkzeugtexturen (Clip, Skip, Origin, Sky, Trigger, nodraw, hint), Flächen, die gegen einen anderen Brush verdeckt sind, Trigger und andere Volumen-Entitäten sowie Ebenen, die auf *Beim Export auslassen* stehen. Lichter, Geräusche, Spawnpunkte, der Himmel und Nebel werden ebenfalls nicht exportiert.

Nur Objekte, die in die Datei kommen, verdecken gegenseitig ihre Flächen. Ein Pfeiler, der mit *Nur Auswahl* exportiert wird, behält also seine Unterseite, die auf dem Boden liegt, und eine Ebene, die auf *Beim Export auslassen* steht, hinterlässt keine Löcher in ihren Nachbarn.

## Einheiten und Achsen { #units-and-axes }

Ein Meter in Godot ist ein Meter in Blender. Die Karteneinheiten werden durch die Einheiten pro Meter des Projekts geteilt, 32, außer die Karteneinstellungen sagen etwas anderes. Die Datei ist wie Godot Y nach oben, was glTF erwartet. Jeder Brush, jedes Mesh und jedes Terrain hat seinen Ursprung in der Mitte seiner Begrenzung, und Modelle behalten den Ursprung der Modelldatei.

## Wann OBJ nutzen { #when-to-use-obj }

*Datei > Exportieren > Wavefront-OBJ (.obj)…* ist für Tools, die glTF nicht lesen können. Es schreibt drei Dinge nebeneinander, zum Beispiel `level.obj`, `level.mtl` mit den Materialien und einen Ordner `level_textures` mit Kopien der Texturen. Halte sie zusammen, wenn du sie verschiebst. In Blender nimm *File > Import > Wavefront (.obj)*.

OBJ kennt keine Elternobjekte und keine Möglichkeit, ein Mesh zwischen Kopien zu teilen, deshalb wird jedes Objekt flach im Weltraum geschrieben, und jede Scatter-Instanz ist eine vollständige Kopie. Sind Scatter-Instanzen angehakt, warnt das Fenster, sobald sie zusammen eine Million Dreiecke erreichen, und sagt, wie groß die Datei wird. Ein OBJ-Export lehnt mehr als 10 Millionen Dreiecke ab, eine Datei von mehr als einem Gigabyte. Materialien behalten die Albedo-, Normal-, Roughness- und Emissionstexturen. Nimm glTF, wann immer das andere Tool es liest.

Eine `.glb`-Datei fasst höchstens 4 GB. glTF schreibt jedes Modell einmal, egal wie viele Scatter-Instanzen es zeigen, deshalb erreicht nur eine riesige Karte diese Grenze, und der Export sagt es dann, statt eine kaputte Datei zu schreiben.

## Kommandozeile { #command-line }

`trenchdot --export-glb <map> <out.glb>` und `--export-obj <map> <out.obj>` exportieren mit den Standardoptionen, ohne ein Fenster zu öffnen. Das Godot-Projekt der Karte wird über ihren Ordner gefunden. Siehe [Kommandozeile](../command-line.md).

<div class="td-mcp" markdown>

## MCP { #mcp }

`map_file {op: export_glb, path}` schreibt eine `.glb` und `map_file {op: export_obj, path}` eine `.obj` mit ihrer `.mtl` und dem Texturordner, aus der offenen Karte, wie sie ist, gespeichert oder nicht. Die Optionen sind Flags: `selection_only`, `hidden`, `scatter`, `models` (wahr, wenn nicht gesetzt), `markers` und `merge_brushes`. Das Ergebnis listet `objects`, `meshes`, `materials`, `images`, `triangles`, `bytes` und `missing_textures`. `run_action` lehnt `export_glb` und `export_obj` ab, da die Menüeinträge einen Dateidialog öffnen.

</div>
