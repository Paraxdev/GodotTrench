---
source: 8586c5cf175329bb5dcb3e351f21f472800d70ab
---

# Der Logikgraph { #the-logic-graph }

Das Panel **Logik** zeigt das Entity-I/O der Karte als Graph. Jede verdrahtete Entität und jede Logik-Entität ist ein Node, und jede Verbindung ist eine Leitung von einem Output-Pin rechts an einem Node zu einem Input-Pin links an einem anderen.

Der Graph zeigt dieselben Outputs, die der Inspektor bearbeitet, eine Änderung im einen erscheint also sofort im anderen. Die Karte, der Godot-Build und das Spiel funktionieren genau gleich, ob du den Graphen nutzt oder nicht.

## Öffnen { #opening-it }

Wähle *Ansicht > Panels > Logik*.

![Das Menü Ansicht mit geöffnetem Panels und Logik unter dem Mauszeiger](../assets/logic-graph/menu-panels-logic.png)

Es öffnet sich neben Materialien unterhalb der Ansichten und vergrößert dieses Dock auf etwa die Hälfte der Fenstermitte, denn ein Graph braucht Platz. Ziehe den Teiler oder den Tab, um das zu ändern.

![Das Panel Logik der Demokarte Night District: Knöpfe, die mit Türen, einem Trigger und einem Timer verdrahtet sind, mit nach Kategorie gefärbten Kopfzeilen, und ein Rahmen mit dem Titel Hall lights um einen Trigger, der drei Lichter mit Verzögerung einschaltet](../assets/logic-graph/graph.png)

## Den Graphen lesen { #reading-the-graph }

| Was du siehst | Was es bedeutet |
| --- | --- |
| Node-Titel | Der targetname der Entität, sonst ihr Name aus *Umbenennen*, sonst ihr Classname. Darunter stehen die Kategorie und der Classname |
| Graue Zeilen unter dem Titel | Die Einstellungen, die von den Standardwerten der Entität abweichen, wie `max 3`, `wait 2 s` oder `once`, auf zwei kurze Zeilen gekürzt. Ein Node ohne sie hat die Standardwerte. Der Inspektor zeigt alle |
| Kopfzeilenfarbe | Die Kategorie: die Gruppe der Definition, sonst das erste Wort des Classnames, `logic_relay` ist also *logic*. Siehe [Farben](#colors) |
| Pins | Inputs links, Outputs rechts, aus der [Entitätsdefinition](entities/README.md) plus jedem Namen, den die Karte an dieser Entität schon benutzt. Ein gefüllter Pin hat eine Leitung. Fahre über einen Pin, um seinen Typ und die Werte zu sehen, die er weitergibt |
| Gelber Pin-Name | Der Name steht nicht in der Definition der Entität, das Panel Probleme warnt ebenfalls davor |
| Leitungsbeschriftung | Die Verzögerung, `once` oder wie oft sie auslösen darf, und der Parameter, wenn sie gesetzt sind. Fahre über eine Leitung, um Output, Ziel, Input und Verzögerung vollständig zu sehen |
| Blauer Node und blaue Leitung | Ein [Ziel](parameters.md#targets), das nur das laufende Spiel auflöst, etwa `!player`, `@enemies`, ein Node-Pfad oder ein Platzhalter wie `door_*` |
| Roter Node, gestrichelte rote Leitung | Ein targetname, den keine Entität hat, meistens ein Tippfehler |

Nur verdrahtete Entitäten sowie Logik- und Mathe-Entitäten bekommen einen Node. Wähle eine beliebige andere Entität, in einer Ansicht oder im Outliner, dann erscheint sie ebenfalls, bereit zum Verdrahten.

## Farben { #colors }

Farben helfen dir, einen großen Graphen zu überfliegen, aber jede Farbe kommt auch mit einem Namen oder einer Form, und die Paletten bleiben für die häufigen Formen der Farbfehlsichtigkeit auseinanderzuhalten.

| Kategorie | Farbe |
| --- | --- |
| logic | blau |
| math | orange |
| trigger | bläulich grün |
| func, Türen, Knöpfe und Beweger | zinnoberrot |
| light | gelb |
| env | türkis |
| game | himmelblau |
| prop | rosa |
| npc | rötlich violett |
| info | grau |

| Pin-Typ | Farbe |
| --- | --- |
| pulse, ein Ereignis ohne Wert | weiß |
| bool | zinnoberrot |
| int | himmelblau |
| float | bläulich grün |
| string | rosa |
| vector3 | gelb |
| color | rötlich violett |
| node, etwa der Aktivator | blau |
| variant, ein beliebiger Wert | grau |

Pins, die einen Wert weitergeben, sind Kreise, Pulse sind Dreiecke. Eine Leitung nimmt die Farbe des Outputs an, an dem sie beginnt. Eine eigene Kategorie, die du mit der `group` einer Definition setzt, bekommt ebenfalls eine eigene Farbe.

## Verdrahten { #wiring }

Ziehe von einem Output-Pin zu einem Input-Pin. Während du ziehst, bekommen die Pins, zu denen die Leitung passt, einen Ring, und der Pin, an dem sie enden würde, füllt sich. Lässt du los, wird die Verbindung zu den Outputs der Quellentität hinzugefügt.

![Eine Leitung wird vom Output released von button_1 zum Input turn_off von hall_light gezogen, die passenden Pins zeigen Ringe](../assets/logic-graph/wire-drag.gif)

Ein Puls passt an jeden Pin, und ein Wert passt an einen Pin seines eigenen Typs, ein `activator`-Node passt also nicht an einen `int`-Input. (Ein int passt auch an einen float- oder bool-Pin und umgekehrt, und ein variant passt an alles.) Jeder Pin nimmt die Leitung trotzdem an, denn die Namen im Entity-I/O werden erst geprüft, wenn das Spiel läuft.

Ein Ziel ohne targetname bekommt im selben Schritt einen: den Namen, den du ihm mit *Umbenennen* im Outliner gegeben hast, ein Licht, das du `lamp` genannt hast, wird also zum Ziel `lamp`, sonst ein freier wie `door_2`. Einem Namen, den eine andere Entität hat, wird eine Zahl angehängt.

Du kannst die Leitung auch woanders fallen lassen:

* Auf dem Titel eines Nodes, um aus allen seinen Inputs zu wählen oder einen beliebigen Namen zu tippen.
* Auf leerem Raum, um dort eine Logik-Entität hinzuzufügen, die schon mit dem Pin verdrahtet ist, von dem du gezogen hast.

### Logik-Entitäten hinzufügen { #adding-logic-entities }

Klicke mit rechts auf leeren Raum oder drücke **Hinzufügen** oben links im Panel, um eine Logik-Entität ohne Leitung hinzuzufügen.

![Die Liste Hinzufügen: ein Suchfeld, dann die Gruppe logic mit hervorgehobenem logic_animate, logic_auto, logic_branch und mehr](../assets/logic-graph/add-menu.png)

Ohne Suche ist die Liste nach Kategorie gruppiert, logic und math zuerst, dann andere Entitäten mit Inputs oder Outputs. Tippe, um in allen zu suchen: Klassen, deren Name mit dem Text beginnt oder ihn enthält, kommen zuerst, dann die, deren Kategorie oder Beschreibung es tut. Die Pfeiltasten nach oben und unten bewegen die Hervorhebung, und Enter fügt diese Klasse hinzu.

Neue Entitäten sind Punktentitäten, und der Godot-Build macht für jede einen Node, sie brauchen also einen Platz in der Karte. Sie landen auf einer Ebene namens *Logic*, neben der Entität, mit der sie verdrahtet sind, oder vor der Kamera der 3D-Ansicht. Jede bekommt sofort einen freien targetname, etwa `relay_2`, der ihren Node betitelt. Ändere ihn im Inspektor, denn *Umbenennen* setzt nur den Namen im Outliner.

> [!NOTE]
> Ohne das Gameplay-Entitätenpaket gibt es keine Logik-Entitäten zum Hinzufügen. Das Menü Hinzufügen bietet dann an, es zu installieren.

### Eine Verbindung bearbeiten { #editing-a-connection }

Klicke eine Leitung an, und ein Kasten oben rechts im Graphen lässt dich ihren Output, ihr Ziel, ihren Input, Parameter, die Verzögerung und die Anzahl bearbeiten. Seine Schaltfläche **Löschen** oder die Taste Entf, solange der Zeiger über dem Graphen ist, entfernt sie.

![Eine gewählte Leitung von button_1 zu hall_light und der Kasten Verbindung von button_1 oben rechts mit Output released, Ziel hall_light, Input turn_off, Verzögerung 0.00 s, Mal jedes Mal und Löschen](../assets/logic-graph/connection-box.png)

Klicke mit rechts irgendwo auf eine Leitung, nicht nur auf ihre Beschriftung, für **Verbindung bearbeiten** und **Verbindung löschen**. Jede Änderung ist ein Schritt im *Verlauf*, Rückgängig funktioniert also wie überall.

### Das Node-Menü { #the-node-menu }

Klicke einen Node mit rechts an, um zu sehen, was du mit seiner Entität tun kannst.

![Das Rechtsklickmenü des Nodes hall_light: Rahmen in Ansichten, Umbenennen, Duplizieren, Verbundene wählen, Alle trennen, Entität löschen und Simulieren mit seinem Output switched](../assets/logic-graph/node-menu.png)

| Eintrag | Tut |
| --- | --- |
| Rahmen in Ansichten | Wählt sie und bewegt die Ansichten dorthin, wie ein Doppelklick |
| Umbenennen | Benennt die Entität im Outliner um. Der Node wird nach dem targetname betitelt, wenn es einen gibt, der Name erscheint dort also nur bei einer Entität ohne |
| Duplizieren | Kopiert die Entität einen Rasterschritt daneben, Leitungen von ihr eingeschlossen |
| Verbundene wählen | Wählt den Node und jeden Node, mit dem eine Leitung ihn verbindet |
| Alle trennen | Löscht in einem Schritt jede Leitung in den Node hinein oder aus ihm heraus. Die Entitäten bleiben |
| Entität löschen | Entfernt die Entität aus der Karte, siehe [Kopieren, Einfügen und Löschen](#copy-paste-and-delete) |
| Simulieren | Löst einen ihrer Outputs auf dem Papier aus, siehe [Simulieren](#simulate) |

Sind mehrere Nodes gewählt, wirken Duplizieren und Löschen auf alle, wenn du einen der gewählten mit rechts anklickst.

## Kopieren, Einfügen und Löschen { #copy-paste-and-delete }

Der Graph zeigt Entitäten der Karte, diese Tasten wirken also auf die gewählten Entitäten, Brushes eingeschlossen, solange der Zeiger über dem Graphen ist.

| Taste | Tut |
| --- | --- |
| Strg+C, Strg+V | Kopieren und Einfügen. Die Kopie landet einen Rasterschritt vom Original, samt Leitungen, da unter dem Zeiger keine Ansicht ist, in die sie passt. Füge mit dem Zeiger über einer Ansicht ein, um sie dort zu platzieren |
| Strg+D | Duplizieren, ebenfalls einen Rasterschritt vom Original |
| Entf | Entfernt die gewählte Leitung oder den gewählten Rahmen. Ist nichts davon gewählt, löscht es die gewählten **Entitäten aus der Karte**, nicht nur ihre Nodes. Leitungen anderer Entitäten zu einer gelöschten werden rot. Rückgängig holt alles zurück |
| Strg+A | Wählt alles in der Karte, nicht nur die gezeigten Nodes |

## Sich bewegen { #moving-around }

| Um | Tu das |
| --- | --- |
| Zu zoomen | Drehe das Mausrad |
| Zu verschieben | Ziehe mit der mittleren oder rechten Maustaste. Ein Rechtsklick ohne Bewegung öffnet ein Menü, ein wackliges Verschieben kann es also aufpoppen lassen, Esc schließt es |
| Zu wählen | Klicke einen Node, Umschalt- oder Strg-Klick fügt hinzu, auf leerem Raum ziehen wählt per Auswahlrahmen. Die Entitäten werden auch in den Ansichten und im Outliner gewählt, und umgekehrt |
| In den Ansichten zu zentrieren | Doppelklicke einen Node |
| Zu bewegen | Ziehe Nodes. Ihre Positionen werden in der Karte gespeichert |
| Aufzuräumen | **Anordnen** ordnet die gewählten Nodes nach Signalfluss an. Sind weniger als zwei gewählt, kehrt jeder Node zum automatischen Layout zurück |
| Alles zu sehen | **Anpassen** oder Pos1, solange der Zeiger über dem Graphen ist |
| Die Auswahl zu sehen | F, solange der Zeiger über dem Graphen ist, zentriert die gewählten Nodes |

Nodes, die niemand bewegt hat, folgen dem automatischen Layout, von links nach rechts, wie die Signale fließen, und die Ansicht zoomt so, dass sie ins Panel passt, bis du selbst zoomst oder verschiebst. Sobald du einen Node bewegst oder im Graphen etwas verdrahtest, behalten die anderen ihre Plätze, und ein neuer Node erscheint neben dem, mit dem er verdrahtet ist. Blaue und rote Markierungsnodes sind keine Entitäten, du kannst sie also nicht bewegen. Sie sitzen neben dem Node, der sie anzielt.

## Rahmen { #frames }

Ein Rahmen ist ein betitelter Kasten hinter einer Gruppe von Nodes, der einen großen Graphen ordentlich hält: die Türen eines Gebäudes, ein Rätsel, eine Zwischensequenz. Wähle die Nodes und drücke **C** über dem Graphen oder **Zentrieren** in der Werkzeugleiste des Panels, dann tippe seinen Titel. Der Kasten *Hall lights* im Bild oben ist einer.

| Um | Tu das |
| --- | --- |
| Einen Rahmen samt Inhalt zu verschieben | Ziehe seinen Titel |
| Seine Größe zu ändern | Ziehe die Ecke unten rechts |
| Ihn umzubenennen | Doppelklicke seinen Titel oder wähle ihn und drücke F2 |
| Ihn umzufärben oder zu löschen | Klicke seinen Titel mit rechts an. Das Löschen eines Rahmens lässt seine Nodes in Ruhe |

Rahmen werden als Editordaten in der Karte gespeichert, der Godot-Build ignoriert sie.

## Simulieren { #simulate }

Wähle eine Entität, drücke **Simulieren** und wähle einen ihrer Outputs, oder klicke einen Node mit rechts an. Der Graph nummeriert jede Leitung, die der Output auslösen würde, in Gelb, und eine Liste unter dem Graphen zeigt jeden Schritt mit den Node-Titeln, in der Reihenfolge, in der die Schritte eintreffen, mit der Zeit in Sekunden nach den Verzögerungen. **Leeren** beendet die Vorschau. Klicke einen Schritt an, um seine Leitung zu wählen.

![Simulieren an button_2: die Leitungen zu garage_door und weiter zu garage_beacon sind gelb und von 1 bis 3 nummeriert, und die Liste unter dem Graphen nennt jeden Schritt](../assets/logic-graph/simulate.png)

Es folgt den Regeln der Laufzeit: Ziele werden über den targetname gefunden, und Relays, Timer, Counter, Gates, Türen und die anderen gängigen Entitäten geben die Kette weiter. Laufzeitziele wie `!player` werden gezeigt, aber nicht weiter verfolgt.

Eine Entität, die einen von mehreren Outputs auslöst, zeigt jede Leitung, die sie auslösen könnte. Diese Schritte und alles danach enden in der Liste auf `(maybe)` und an der Nummer der Leitung auf `?`, und ein Zeiger darüber sagt, worauf sie warten. Ein Counter, der per `add` oder `subtract` gespeist wird, läuft auf diese Weise weiter zu `hit_max` und `hit_min`, da die Vorschau keine Tastendrücke zählt: Der Schritt ist da, und er passiert, sobald der Counter sein `max` oder `min` erreicht. Ein `logic_sequence` löst seine Schritte zu den Zeiten seiner Eigenschaften `interval`, `times` und `steps` aus, eine Zwischensequenz liest sich also wie eine Zeitleiste.

Simulieren arbeitet auf dem Papier, ohne Godot. Was im Spiel wirklich passiert, behandelt [Verdrahtung debuggen](debugging.md).
