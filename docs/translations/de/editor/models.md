---
source: d2dc61c8ee306263eda3f36520611e0790a27732
---

# Modelle { #models }

Modelle sind 3D-Dateien, die in einem anderen Programm entstanden sind, etwa Blender oder Blockbench. TrenchDot liest glTF-, OBJ-, Blockbench-, STL-, MD2- und MD3-Dateien.

Du findest sie im Tab **Modelle** unten, neben Materialien und Prefabs. Er listet alles unter `res://models` auf, dazu die Naturmodelle in `res://trenchdot/nature`, sobald du sie installiert hast.

![Der Tab Modelle: Suchfeld, Ordner- und Quellenfilter, Sortieren, Slugs, der Regler für die Vorschaugröße, Anpassen, der Größenmultiplikator, die Schaltfläche zum erneuten Einlesen und Importieren…, darüber ein Raster aus Vorschaubildern von Naturmodellen](../assets/models/models-panel.png)

Ein neues Projekt startet ohne Modelle. *Godot > Inhalt zum Projekt hinzufügen…* fügt die Naturmodelle hinzu, und die Demo-Auswahl bringt außerdem die Poly-Haven-Props in `res://models/polyhaven` mit. Oder kopier deine eigenen Dateien nach `res://models` und drück die Schaltfläche ⟳, um neu einzulesen.

## Ein Modell platzieren { #placing-a-model }

Ein Modell kommt auf zwei Wegen in die Map. Such dir einen aus, je nachdem, ob du seine Form im Editor ändern willst.

| Platziert als | Gespeichert in der Map | Dafür gedacht |
| --- | --- | --- |
| Bearbeitbares Mesh | Eine Kopie der Geometrie, die der Datei nicht mehr folgt | Formen, die du mit den [Mesh-Werkzeugen](meshes.md) bearbeiten willst |
| `prop_model`-Entität | Nur ein `res://`-Pfad, Godot lädt die Datei | Props, die du so lässt, wie sie sind. Jede Kopie folgt der Datei, wenn du sie änderst |

### Als bearbeitbares Mesh { #as-an-editable-mesh }

1. (Optional) Die Größe festlegen

    **Anpassen** in der Werkzeugleiste Modelle ist standardmäßig angehakt. Es skaliert ein Modell, dessen längste Seite unter einem Viertelmeter oder über 256 Metern liegt, auf 2 Meter, damit auch winzige oder riesige Dateien brauchbar ankommen. Das Feld × daneben multipliziert die Größe zusätzlich. Beides gilt beim Platzieren, stell es also ein, bevor du ziehst.

2. In eine Ansicht ziehen

    Zieh ein Vorschaubild aus dem Tab Modelle in eine beliebige Ansicht. Es landet auf der Fläche unter dem Mauszeiger.

    ![Ein Weinfass wird aus dem Tab Modelle in die 3D-Ansicht gezogen und landet dort als bearbeitbares Mesh auf dem Boden](../assets/models/drag-model-into-view.gif)

Für eine Datei irgendwo auf der Platte nimmst du **Importieren…** am Ende der Werkzeugleiste Modelle.

### Als Prop { #as-a-prop }

1. Die Datei wählen

    Geh auf *Datei > Importieren > Modell-Prop (.bbmodel, .glb)…* und wähl das Modell.

    ![Das Menü Datei mit offenem Importieren und überfahrenem Modell-Prop (.bbmodel, .glb)…](../assets/models/file-import-model-prop.png)

2. Den Inspektor prüfen

    Es landet am 3D-Cursor als `prop_model`-Entität, und ihr Key `model` enthält den `res://`-Pfad.

    ![Der Inspektor einer prop_model-Entität, der Key model auf einen Pfad unter res://models/polyhaven gesetzt](../assets/models/inspector-prop-model.png)

Ein Prop muss auf eine Datei zeigen, die Godot laden kann. Wählst du eine außerhalb des Projekts, bietet der Editor an, sie zuerst nach `res://models` zu kopieren.

Blockbench-Modelle haben zwei weitere Wege hinein, *Datei > Importieren > Blockbench-Modell als Mesh…* und *Blockbench-Modell als Brushes…*. Als Brushes wird jeder Würfel zu einem Brush, gruppiert unter dem Namen der Datei.

## Texturen eines platzierten Meshes { #textures-of-a-placed-mesh }

Ein bearbeitbares Mesh behält das Aussehen des Modells. Seine Texturen werden nach `models/<name>` in den Texturordner des Projekts kopiert, wobei `<name>` der Pfad des Modells im Tab Modelle ist, oder bei einem Modell von anderswo sein Dateiname. Markier das Mesh, und der Inspektor listet die Materialien auf, mit denen es gezeichnet wird:

![Der Inspektor eines platzierten Meshes: 5966 Vertices und das Material models/polyhaven/containers/wine_barrel_01/tex0](../assets/models/inspector-mesh-materials.png)

Wo das schlichte Bild nicht wie das Modell zeichnen würde, kommt zusätzlich ein Godot-Material in den Materialordner des Projekts. So werden ausgeschnittene Blätter, Pixel Art und leuchtende Teile (auch Emissionsfarbe und Emissions-Map eines glTF-Modells) in Godot so gezeichnet wie hier. Um die Materialien zu tauschen, klick im Tab Materialien eines an, während das Mesh markiert ist, *Bearbeiten > Rückgängig* bringt sie zurück.

Das Platzieren überschreibt nie eine Datei. Platzierst du dasselbe Modell noch einmal, nutzt es seine Texturen wieder und behält ein Material, das du in Godot abgestimmt hast. Liegen im Ordner schon die Texturen eines anderen Modells, etwa die von `rock.bbmodel` neben `rock.glb`, kommen die neuen in einen nummerierten Ordner wie `models/rock_2`. Rückgängig lässt die Dateien, wo sie sind. Lassen sich die Texturen nicht schreiben, etwa in einem schreibgeschützten Projekt, kommt das Mesh ohne Texturen an, und die Statusleiste sagt, warum.

Texturen gelten anhand ihrer Pixel als gleich, eine, die ein anderes Programm nur neu gespeichert hat, wird also trotzdem wiederverwendet. Hast du eine übermalt? Zuvor platzierte Meshes behalten deine Version, und die nächste Platzierung bekommt frische Kopien in einem nummerierten Ordner, mit einem neuen Material. Damit spätere Platzierungen deine Änderungen bekommen, wende dein Material im Tab Materialien auf sie an.

> [!NOTE]
> FuncGodot baut das Material einer Fläche nur dann aus dem `default_material` des Projekts, wenn es keine Materialdatei gleichen Namens gibt. Flächen eines platzierten Modells, die eine Materialdatei bekommen haben, werden mit dieser Datei gezeichnet, ein eigener Shader als `default_material` erreicht sie also nicht. Bearbeite die Datei in Godot, um sie zu ändern. Ein Projekt, dessen Materialendung nicht `tres` ist, bekommt keine Materialdateien, und sein Standardmaterial gilt.

## Mehr zu einem Vorschaubild { #more-on-a-thumbnail }

Rechtsklick auf ein Vorschaubild platziert es am Cursor, kopiert seinen Pfad oder zeigt die Datei auf der Platte. *Paketseite öffnen* erscheint, wenn sein Paket eine URL hat.

![Das Rechtsklick-Menü am Vorschaubild wine_barrel_01 mit überfahrenem Am Cursor platzieren, dann Pfad kopieren, Im Dateimanager zeigen und Paketseite öffnen](../assets/models/thumbnail-menu.png)

## Paket-Credits { #pack-credits }

Kostenlose Modellpakete verlangen meistens eine Namensnennung. Leg eine `pack.json` in den Ordner eines Modells oder in einen Ordner darüber (die nächste gewinnt), und das Panel weiß, aus welchem Paket das Modell stammt, wer es gemacht hat und unter welcher Lizenz. Fahr über ein Vorschaubild, um es zu sehen:

![Der Tooltip des Vorschaubilds wine_barrel_01: sein Pfad, Paket: Poly Haven, der Autor, Lizenz: CC0, die URL des Pakets und die Nennung des Modells](../assets/models/thumbnail-credit-tooltip.png)

```json
{
  "name": "Nature Kit",
  "short": "Nature",
  "author": "Some Artist",
  "license": "CC0",
  "url": "https://example.com/nature-kit",
  "credits": [{ "asset": "pine", "author": "Other Artist" }]
}
```

`short` ist die Beschriftung auf jedem Vorschaubild, solange Slugs angehakt ist, und fällt auf `name` zurück. `credits` überschreibt Autor, Lizenz oder URL einzelner Modelle, zugeordnet über den Dateinamen ohne Endung. Ordner ohne `pack.json` werden unter dem Namen ihres obersten Ordners aufgelistet.
