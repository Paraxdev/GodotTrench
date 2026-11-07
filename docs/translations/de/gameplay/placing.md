---
source: 059c4546c8d47d8cdd4e33a900932ff4ff2b3833
---

# Platzieren und Verdrahten { #placing-and-wiring }

## Entitäten platzieren { #placing-entities }

Öffne den Tab **Entitäten**, er sitzt rechts neben dem Inspektor. Jede Klasse, die dein Projekt kennt, hat dort eine Karte, nach Kategorie gruppiert. Fahre mit der Maus über eine Karte, um zu sehen, was sie tut.

![Das Entitäten-Panel mit der Karte func_door unter dem Mauszeiger, ihr Tooltip sagt Brush-Entität, Schiebetür](../assets/placing/entities-panel.png)

Ziehe eine Karte in eine beliebige Ansicht, um sie zu platzieren, oder doppelklicke sie, um sie am 3D-Cursor abzulegen. (Mit Strg- oder Umschalt-Klick wählst du mehrere Karten, ziehst du sie zusammen hinein, landen sie in einer Reihe.) Eine Karte einer Brush-Entität kommt mit einer Box. Sind Brushes gewählt, doppelklicke sie, um stattdessen diese Brushes in die Entität zu packen.

Die Entität landet auf der ersten festen Oberfläche unter dem Zeiger. Trigger-Volumen, Brushes, die nur aus Werkzeugtexturen bestehen, und andere Entitäten werden übersprungen. Beim Ziehen zeigt eine cyanfarbene Markierung, wo sie landen wird. Eine 2D-Ansicht kann keine Tiefe zeigen, deshalb trägt die Markierung dort die Höhe (oder Tiefe), auf der sie landet. In der Ansicht Oben steht das Dach eines Raums im Weg, halte also beim Ablegen Alt gedrückt, dann geht die Entität am Dach vorbei und landet auf dem Boden darin. In den Ansichten Vorne und Seite geht sie mit Alt an der nahen Wand vorbei und landet in der Mitte des Raums.

### Brush-Entitäten { #brush-entities }

Türen, Knöpfe und Trigger sind Brush-Entitäten, sie bekommen ihre Form von Brushes, die du gebaut hast.

1. Baue die Form, zum Beispiel eine dünne Box dort, wo die Tür hin soll, und wähle sie.
2. Klicke mit rechts in eine Ansicht und wähle unter *Brush-Entität erstellen* eine Klasse. *Brush > Brush-Entität* hat dieselbe Liste.

![Das Rechtsklickmenü einer Ansicht mit geöffnetem Brush-Entität erstellen und trigger_once unter dem Mauszeiger](../assets/placing/context-create-brush-entity.png)

Für die gängigen gibt es schnellere Wege:

| Werkzeug oder Menü | Was es macht |
| --- | --- |
| *Gameplay > Türen und Beweger* | Eine Dreh- oder Schiebetür mit ihrem Trigger oder einen Aufzug aus den gewählten Brushes |
| *Gameplay > Trigger* | Ein Trigger-Volumen in der Größe der Auswahl |
| Werkzeug **Volumen** (Umschalt+E) | Zieht ein beliebiges Trigger-Volumen in einem Zug auf, vorher ist kein Brush nötig |
| Werkzeug **Pfad** (Umschalt+P) | Klick für Klick eine Kette aus `path_corner`-Entitäten, denen Züge und Läufer folgen, Enter beendet |

![Das Gameplay-Menü mit geöffnetem Türen und Beweger: Drehtüren, Schiebetüren und Aufzug, bewegliche Plattform](../assets/placing/gameplay-doors-menu.png)

![Die Werkzeugleiste mit dem Volumen-Werkzeug unter dem Mauszeiger und seinem Tooltip, Volumen-Werkzeug (Umschalt+E), ein Trigger-, Spawn-, Schadens-, Teleport- oder Schubvolumen aufziehen](../assets/placing/toolbar-volume-tool.png)

### Griffe { #handles }

Wähle eine Tür, ein Tor, eine Plattform, einen Knopf, einen Schub-Trigger, einen Spawner, eine Explosion oder ein Physik-Prop, dann zeigt es in den Ansichten Griffe. Ziehe daran, um Angel, Öffnungsweg oder Fahrstrecke, Radius oder Größe einzustellen, statt Zahlen zu tippen. Ein Angelwinkel rastet in 15-Grad-Schritten ein, mit Umschalt stellst du einzelne Grad ein.

![Der Winkelgriff einer Drehtür wird in der Ansicht Oben gezogen, die Beschriftung geht von 95° auf 30°](../assets/placing/door-hinge-handle.gif)

## Verdrahten { #wiring }

Eine Verbindung liegt an der Entität, die sie auslöst. Wähle diese Entität und schau dir **Ausgänge** unten im Inspektor an. **+ Output hinzufügen** fügt eine Verbindung hinzu, dann füllst du Output, Ziel und Input aus. Was jedes Feld tut, steht in [So funktioniert Entity-I/O](io.md).

![Der Abschnitt Ausgänge im Inspektor eines Knopfs: Output pressed, Ziel front_door, Input open](../assets/io/inspector-outputs.png)

Um zwei Entitäten schneller zu verbinden, wähle beide und nimm *Gameplay > Logik > Zwei gewählte Entitäten verknüpfen…*. Der Inspektor zeigt außerdem eine Schaltfläche **Verknüpfen…**, solange genau zwei Entitäten gewählt sind.

![Das Gameplay-Menü mit geöffnetem Logik und Zwei gewählte Entitäten verknüpfen… unter dem Mauszeiger](../assets/placing/menu-link-two-entities.png)

Das Fenster *Entitäten verknüpfen* zeigt links die Quelle und rechts das Ziel. Wähle den Output unter *Wenn*, den Input unter *Aufruf* und drücke **Verknüpfen**.

![Das Fenster Entitäten verknüpfen: func_button links, light rechts, Wenn pressed, Aufruf turn_on, und der Tooltip von Tauschen](../assets/placing/link-dialog.png)

> [!WARNING]
> Die Quelle ist nicht immer die Entität, die du zuerst angeklickt hast. Prüfe die beiden Namen oben und drücke die Schaltfläche **Tauschen** dazwischen, wenn sie falsch herum stehen.

Der [Logikgraph](logic-graph.md) zeigt die ganze Verdrahtung auf einmal und verbindet Entitäten, indem du von Pin zu Pin ziehst.

## Verdrahtung ohne Godot prüfen { #checking-wiring-without-godot }

Wähle im [Logikgraph](logic-graph.md#simulate) eine Entität, drücke **Simulieren** und wähle einen Output. Der Graph hebt jeden Schritt `source.output → target.input` hervor, der folgen würde, und listet sie mit ihren Verzögerungen auf, sodass du einer langen Kette folgen kannst, ohne das Spiel zu bauen und zu starten. Rot markiert ein Ziel, das es nicht gibt, meistens ein Tippfehler in einem targetname. Blau markiert ein Laufzeitziel wie `!player` oder `@enemies`, das nur im Spiel existiert und nicht weiter verfolgt wird.

## Code für jede Entität { #code-for-any-entity }

Das Panel **Referenz** zeigt GDScript und C# für jede Entitätsklasse. Es ist praktisch, wenn Spielcode mit einer Karten-Entität reden muss, etwa um eine Tür zu öffnen oder auf einen Knopf zu lauschen. Öffne es mit *Ansicht > Panels > Referenz*, mit *Hilfe > Entitäts- und Code-Referenz* oder mit **Code-Referenz** im Inspektor, was direkt zur Klasse der gewählten Entität springt. Ziehe den Teiler zwischen der Klassenliste und den Details, um die Liste zu skalieren, doppelklicke ihn, damit sie zu den Namen passt.

![Das Panel Referenz mit gewähltem func_button: seine Inputs und Outputs und die Verwendung in GDScript](../assets/placing/reference-panel.png)

Wähle links eine Klasse, dann einen Tab. *Verwendung in GDScript* und *Verwendung in C#* zeigen, wie du die Entität aufrufst, *GDScript-Klasse*, *C#-Klasse* und *FGD-.tres* sind Gerüste für eigene Entitäten. **Kopieren** legt den Code in die Zwischenablage. Auf den Gerüst-Tabs schreibt **Im Projekt erstellen** die Datei unter `res://entities/` und lässt eine bereits vorhandene Datei in Ruhe.
