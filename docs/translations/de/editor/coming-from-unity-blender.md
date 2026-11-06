---
source: 84383292fb022d6e6369e9ad8b643a67a07d4a67
---

# Von Unity oder Blender kommen { #coming-from-unity-or-blender }

TrenchDot ist ein Brush-Editor in der Linie von TrenchBroom und Hammer. Ein Level beginnt als massive Blöcke, die du auf einem Raster zeichnest, nicht als importierte Meshes. Das meiste, was du aus Unity und Blender kennst, hat hier weiter seinen Platz, nur unter anderen Namen und Tasten. Diese Seite ordnet es zu, und [Erste Schritte](../getting-started.md) führt dich durch einen ersten Raum.

## Wo was ist { #where-things-are }

| In Unity oder Blender | In TrenchDot |
| --- | --- |
| Hierarchy, Outliner | Der **Outliner**, mit [Ebenen und Gruppen](organizing.md) statt Eltern-Objekten |
| Inspector, N-Panel | Der **Inspektor**. Seine Felder für Position und Größe nehmen getippte Werte, Enter übernimmt sie |
| Scene View, Viewport | Die 3D-Ansicht plus die Ansichten Oben, Vorne und Seite. Zeichnen und Ausrichten geht in 2D am leichtesten |
| ProBuilder-Formen, Add-Menü | In einer beliebigen Ansicht ziehen, um einen Box-[Brush](brushes.md) zu zeichnen, Ctrl+Shift+B für Bögen, Treppen und Zylinder |
| Boolean-Modifier | *Brush > CSG*: subtrahieren (Ctrl+K), vereinen, schneiden und aushöhlen. Sie ändern die Brushes sofort, einen Modifier-Stack gibt es nicht |
| Edit Mode | Das **Vertex**-Werkzeug für Brushes, das **Mesh**-Werkzeug für [Meshes](meshes.md), beide auf Tab |
| Prefabs | [Prefabs](organizing.md#prefabs), als eigene Map gespeichert und in jeder Instanz aktualisiert |
| Verknüpftes Duplikat (Alt+D) | Verknüpft duplizieren, Ctrl+Shift+D, damit die Kopien ihre Geometrie teilen |
| Einheiten, 1 m | 32 Karteneinheiten sind ein Meter, der Standard-Rasterschritt ist 16 |

Genaue Zahlen gehören in den Inspektor rechts. Markier etwas, tipp eine neue Position oder Größe und drück Enter. Die Position ist die unterste Ecke der Auswahl, nicht ihre Mitte oder ein Pivot.

![Der Inspektor bei markiertem Brush, seine Position und Größe als Felder X, Y und Z über dem Abschnitt Auswahl](../assets/coming-from-unity-blender/inspector-position-size.png)

## Tasten, die anders sind { #keys-that-differ }

Die Standardtasten folgen TrenchBroom, und drei davon überraschen Blender-Nutzer:

* **G** ist das Terrain-Werkzeug Modellieren, nicht Greifen.
* **R** und **T** wählen die Werkzeuge Drehen und Skalieren, die mit Griffen auf die Auswahl wirken, statt der Maus zu folgen.
* **Tab** bearbeitet die Auswahl dort, wo sie steht.

Zum Verschieben ziehst du etwas in einer beliebigen Ansicht oder nimmst das Gizmo in der 3D-Ansicht, und für genaue Zahlen nimmst du den Inspektor.

Lieber deine Blender-Tasten behalten? Stell *Datei > Einstellungen > Tastenbelegung* auf Blender um.

![Das Dropdown Tastenbelegung im Fenster Einstellungen, geöffnet mit TrenchBroom, Hammer und Blender](../assets/coming-from-unity-blender/keymap-preset.png)

Es ergänzt A zum Alles-Auswählen, X zum Löschen, Shift+D zum Duplizieren, H und Alt+H zum Ausblenden und Einblenden, S für das Skalieren-Werkzeug und Shift+A für den Formgenerator. G bleibt Modellieren, und das Tippen einer Zahl während einer Verschiebung funktioniert nur im Mesh-Werkzeug. Die ganze Liste steht unter [Andere Presets](shortcuts.md#other-presets).

Es gibt keine Numpad-Ansichten. Die Ansichten Oben, Vorne und Seite sind immer offen, Shift+Space maximiert die unter dem Mauszeiger, und *Ansicht > Ansichten* schaltet zwischen einer, zwei und vier um.

## Gewohnheiten, die bleiben { #habits-that-carry-over }

Im Mesh-Werkzeug folgen die Tasten Blender: 1, 2 und 3 für Vertices, Kanten und Flächen, G, R und S mit X, Y oder Z, um eine Achse zu sperren, E zum Extrudieren, Ctrl+R für einen Loop Cut und Ctrl+B zum Abfasen.

Rechts ziehen mit WASD fliegt die 3D-Kamera wie die Scene View in Unity, und F zentriert auf die Auswahl. Ctrl+Z und Ctrl+Shift+Z machen rückgängig und stellen wieder her, und F2 benennt um.
