---
source: b549cb4d1d3fd2ab8e0ac144848e50eac2b168fc
---

# Eine Karte organisieren { #organizing-a-map }

Sobald eine Karte größer wird, willst du sie in Teile sortieren, Dinge wiederfinden und Teile, die du wiederverwendest, an einem Ort halten. Das meiste davon beginnt im Outliner, dem Panel links.

## Ebenen { #layers }

Der Outliner zeigt die Karte als Baum aus Ebenen, Gruppen und Objekten. Mit Ebenen teilst du eine Karte in Teile auf, zum Beispiel ein Gebäude pro Ebene, sodass du einen Teil ausblenden oder sperren kannst, während du am Rest arbeitest.

Klick oben im Outliner auf **Ebene hinzufügen**, um eine neue anzulegen, sie wird sofort zur aktuellen Ebene. Neue Objekte landen in der aktuellen Ebene, der mit der Markierung *(aktuell)*, und ein Klick auf eine Ebene macht sie zur aktuellen. Das Auge vor einer Zeile blendet sie aus, das Schloss verhindert, dass sie ausgewählt oder bearbeitet wird.

Fahr über eine Zeile, um ihre Materialien, Größe und Begrenzung zu sehen. Das Objekt wird in jeder Ansicht gleichzeitig umrandet, sodass du siehst, welche Wand `brush7` ist.

![Der Outliner mit geöffneter Ebene House und dem Mauszeiger auf der Zeile brush7, ein Tooltip zeigt Material, Größe und Begrenzung, während die Ansichten 3D und Vorne die Rückwand umranden](../assets/organizing/outliner-hover.png)

Brushes und Meshes werden nach Art und ID aufgelistet, etwa `brush12` oder `mesh40`, damit sich die Wände, die ein Aushöhlen oder ein Subtrahieren zurücklässt, auseinanderhalten lassen. Die ID bleibt gleich, wenn das Objekt verschoben wird, und über das Speichern hinweg.

### Dinge benennen { #naming-things }

Wähl ein Objekt aus und drück F2, oder klick im Outliner oder in einer Ansicht mit rechts darauf und wähl *Umbenennen*. Die Zeile wird zu einem Textfeld. Enter behält den Namen, Escape bricht ab, und ein leerer Name bringt den Standardnamen zurück.

![Eine Brush-Zeile im Outliner, zu einem Textfeld geworden, mit "front wall" darin](../assets/organizing/outliner-rename.png)

Namen werden mit der Karte gespeichert. Entitäten behalten ihren `targetname` getrennt von diesem Namen, ein Umbenennen ändert also nie, was ein Output erreicht. Die Teile, die ein Clip, eine CSG-Operation oder eine Umwandlung zurücklässt, behalten den Namen, und wenn du ein Objekt in einer verknüpften Gruppe benennst, heißt es in jeder Kopie so.

(Ebenen, Gruppen und Scatter-Sets behalten immer einen Namen, ein leerer ändert bei ihnen also nichts. Wenn du ein Scatter-Set umbenennst, wird auch die Ebene umbenannt, die das Scatter-Werkzeug dafür angelegt hat.)

### Objekte im Outliner auswählen { #picking-objects-in-the-outliner }

Ctrl+Klick auf eine Zeile fügt sie der Auswahl hinzu oder nimmt sie heraus. Shift+Klick wählt jede Zeile zwischen der zuletzt angeklickten und dieser aus, und Ctrl+Shift+Klick fügt diesen Bereich zu dem hinzu, was schon ausgewählt ist. Der Outliner ist oft der einfachere Ort, um Objekte zu wählen, die in den Ansichten hinter anderen stecken.

### Eine Ebene aus Godot heraushalten { #keeping-a-layer-out-of-godot }

Für Referenzgeometrie, die nie ins Spiel kommen soll, klick mit rechts auf ihre Ebene und wähl *Beim Export auslassen umschalten*. Die Ebene zeigt dann *[ausgelassen]* hinter ihrem Namen. Ausblenden reicht nicht, Godot baut auch ausgeblendete Objekte.

![Das Rechtsklickmenü der Referenzebene Blockout mit Beim Export auslassen umschalten unter dem Mauszeiger](../assets/organizing/layer-menu-omit.png)

## Gruppen { #groups }

Eine Gruppe hält Objekte zusammen, damit sie sich als eines auswählen und verschieben lassen, etwa die Brushes eines Tischs. Alles rund um Gruppen findest du unter *Bearbeiten > Gruppen*.

![Das Menü Bearbeiten mit geöffnetem Gruppen und Verknüpft duplizieren unter dem Mauszeiger](../assets/organizing/menu-groups.png)

| Eingabe | Ergebnis |
| --- | --- |
| Ctrl+G, Ctrl+Shift+G | Die Auswahl gruppieren, die Gruppierung aufheben |
| Klick | Das Objekt in der Gruppe auswählen |
| Doppelklick | Die ganze Gruppe auswählen |

**Verknüpft duplizieren** (Ctrl+Shift+D) macht eine Kopie, die mit ihrem Original synchron bleibt, sodass die Reparatur eines Fensterrahmens alle repariert. Verknüpfte Kopien zeigen im Outliner *(verknüpft)*, und ein Klick auf eine wählt die ganze Gruppe aus, da ihre Kopien als eines bearbeitet werden. *Bearbeiten > Gruppen > Gruppen trennen* löst die Verknüpfung, wenn sich eine Kopie unterscheiden muss.

## Prefabs { #prefabs }

Für Teile, die du in mehreren Karten wiederverwendest, wähl sie aus und nimm *Bearbeiten > Prefabs > Aus Auswahl erstellen…*. Das speichert die Auswahl als eigene `.tdm` und setzt an ihre Stelle eine Instanz. Jede Instanz aktualisiert sich, wenn sich die Prefab-Datei ändert.

![Das Menü Bearbeiten mit geöffnetem Prefabs und Aus Auswahl erstellen unter dem Mauszeiger](../assets/organizing/menu-prefabs.png)

> [!NOTE]
> Speichere die Karte einmal, bevor du ein Prefab erstellst. Prefab-Pfade werden relativ zur Karte gespeichert, deshalb weigert sich der Editor, bis die Karte eine Datei hat.

Für Teile, die du einmal einfügst und dann frei formst, nimm stattdessen die [Prefab-Bibliothek](prefab-library.md).

Wenn du ein Prefab zweimal platzierst, tragen zwei Entitäten denselben targetname. Das Fixup einer Instanz löst das: Bei einem Fixup `p1` bekommt jeder Name darin dieses Präfix, aus `door` wird `p1-door`, und jede Kopie hält ihre Verdrahtung für sich. Die Kartendatei und der Godot-Build kennen das Fixup, aber der Editor hat noch kein Feld, um es zu setzen.

## Cordon { #cordon }

Ein Cordon schrumpft den Editor auf einen Teil einer großen Karte zusammen.

1. Wähl aus, woran du arbeiten willst.
2. Wähl *Ansicht > Cordon > Cordon aus Auswahl setzen*, oder klick im Inspektor auf **Cordon**.

    ![Das Menü Ansicht mit geöffnetem Cordon und Cordon aus Auswahl setzen unter dem Mauszeiger](../assets/organizing/menu-cordon.png)

3. Alles, was die gelbe Box um deine Auswahl nicht berührt, ist jetzt ausgeblendet und lässt sich nicht auswählen, und die Statusleiste sagt *Cordon*.

    ![Vorher: die ganze Karte mit Haus, Rasen, einer Gartenmauer, einem Pfeiler und einem hohen Referenzblock. Nachher: nur das Haus in einer gelben Box und der Rasen, der es berührt](../assets/organizing/cordon-before-after.png)

*Ansicht > Cordon > Cordon aktiv* schaltet ihn aus und wieder ein, und *Cordon entfernen* löscht ihn. Der Cordon wird mit der Karte gespeichert.

Er ändert nur, was der Editor zeigt, Godot baut weiterhin die ganze Karte. Um einen Bereich an ein anderes Tool zu übergeben, schreibt *Datei > Exportieren > .map, nur Cordon…* nur die Objekte darin.

## Probleme { #issues }

Das Panel Probleme, der Tab neben Outliner und Verlauf, prüft die Karte, während du arbeitest. Es listet Probleme auf, die in Godot kaputtgehen oder falsch aussehen würden, etwa Brushes ohne Volumen, Flächen ohne Material oder Outputs, die auf einen targetname zielen, den nichts hat.

![Das Panel Probleme mit einem Fehler für ein fehlendes Ziel, drei Warnungen für einen doppelten Brush, eine Entität ohne Definition und zwei Flächen mit Z-Fighting und einer Info-Zeile für eine leere Gruppe, die meisten mit einer Schaltfläche Beheben](../assets/organizing/issues-panel.png)

Klick auf ein Problem, um das Objekt auszuwählen und einzurahmen. Viele haben eine Schaltfläche **Beheben**, und **Alle beheben** in der Kopfzeile behebt jedes, das es kann.

Eine Warnung, dass eine Fläche eine Fläche eines anderen Brushes auf derselben Ebene überlappt, heißt, dass die beiden in Godot flackern werden. Schieb eine Fläche von der Ebene weg oder beschneide die Überlappung.

Eine Warnung, dass ein Modell nicht existiert, heißt, dass das `model` eines Props eine Datei nennt, die es nicht gibt, das Prop wird also ohne Mesh gebaut. Gibt es eine Datei mit demselben Namen und anderer Endung, `chair.gltf` für `chair.glb`, nennt die Warnung sie.

Eine Warnung, dass etwas GDScript aus der Karte ausführt, heißt, dass die Karte eigenen Code mitbringt, der läuft, sobald die Szene in Godot abgespielt wird. Du bekommst eine für jedes `logic_script` mit einer `source` oder `expression`, für jeden Output oder `logic_call`, der ihm Code zuspielen kann, etwa ein Output, der `source` an ihn schickt, und für jedes Prefab, das etwas davon enthält. Lies sie, bevor du eine Karte spielst, die jemand anderes gemacht hat.

Warnungen, die mit "Godot" beginnen, stammen vom letzten Build in Godot, nach einem Speichern oder *In Godot bauen*: die Klassen, die es ohne ihre Definition gebaut hat, und die anderen Warnungen dieses Builds. Siehe [Entitäten, die nichts tun](../godot/entities-troubleshooting.md).

## Verlauf { #history }

Das Panel Verlauf listet jede Bearbeitung auf, die neueste oben. Klick auf einen älteren Eintrag, um bis dorthin rückgängig zu machen, dieser Eintrag eingeschlossen. Rückgängig gemachte Bearbeitungen bleiben abgeblendet über *aktuell*, klick auf eine, um bis dorthin wiederherzustellen.

![Das Panel Verlauf mit Umbenennen und Gruppieren abgeblendet über der Markierung aktuell und dem Eintrag Aushöhlen unter dem Mauszeiger](../assets/organizing/history-panel.png)

<div class="td-mcp" markdown>

## MCP { #mcp }

Ein Agent oder Skript, der den Editor über den [MCP-Server](../mcp.md) steuert, führt dieselben Prüfungen wie das Panel Probleme mit dem Tool `validate_map` aus, das sie mit ihrem Code nennt, etwa `coplanar_faces` und `missing_model`. Die `run_action`-Aktion `insert_prefab` nimmt ein `fixup`, ein Agent kann einer Instanz ihr Fixup also beim Platzieren geben.

</div>
