---
source: b890aefb21f6d8de673ab4ebc70a71d93fd598c5
---

# Ziele, Parameter und Inputs { #targets-parameters-and-inputs }

Eine Verbindung nennt ihr Ziel, ihren Input und einen optionalen Parameter als einfachen Text. Diese Seite zeigt, wie aus diesem Text die Nodes, die Methode und die Werte werden, die im Spiel gelten.

## Ziele { #targets }

Das Ziel einer Verbindung, und von Schlüsseln wie `call_target`, entscheidet, welche Nodes den Input bekommen. Meistens ist es ein targetname, den du einer anderen Entität gegeben hast. Die anderen Formen erreichen Gruppen von Dingen und Nodes außerhalb der Karte.

| Ziel | Erreicht | Dafür nutzen |
| --- | --- | --- |
| `main_door` | Jede Entität mit diesem targetname in derselben Karte | Der Normalfall, eine benannte Entität oder mehrere mit gleichem Namen |
| `door_*` | Jeden targetname, der mit `door_` beginnt | Eine ganze Reihe auf einmal öffnen. Nur ein `*` am Ende funktioniert |
| `@enemies` | Jeden Node in der Godot-Gruppe `enemies`, irgendwo im Baum | Dinge, die zur Laufzeit gespawnt oder vom Spielcode platziert werden |
| `!player` | Den ersten Node in der Gruppe `player` | Den Spieler erreichen, wenn die Kette den Aktivator verloren hat |
| `!activator` | Wer die Kette gestartet hat, `!caller` bedeutet dasselbe | Den verletzen oder teleportieren, der einen Trigger berührt hat |
| `!self` | Die Entität, der der Output gehört | Eine Entität, die auf sich selbst wirkt, etwa ein Knopf, der sich selbst deaktiviert |
| `/root/Game` | Ein absoluter Node-Pfad, er muss mit `/` beginnen | Autoloads und anderer Spielcode außerhalb der Karte |

Targetnames werden innerhalb der nächsten `FuncGodotMap` nachgeschlagen, zwei Karten in einer Szene können die Namen der jeweils anderen also nicht erreichen. Gruppen, Node-Pfade und `!player` funktionieren im ganzen Baum, nutze sie, wenn eine Karte mit einer anderen reden muss.

> [!NOTE]
> In Source ist `!caller` die Entität, die den Output ausgelöst hat. Hier bedeutet es dasselbe wie `!activator`, für die Entität selbst nimmst du `!self`.

### Änderungen zur Laufzeit { #runtime-changes }

Targetname-Abfragen werden pro Karte zwischengespeichert, damit sie schnell sind. Wenn du benannte Entitäten zur Laufzeit spawnst oder umbenennst, rufe `TrenchDotIO.invalidate(node)` mit einem beliebigen Node dieser Karte auf, damit die nächste Abfrage sie sieht. Umbenennen heißt, die Meta `td_targetname` zu ändern, der `name` des Nodes spielt keine Rolle. Das Töten einer Entität braucht keinen Aufruf, der Cache bemerkt freigegebene Nodes.

Nodes, die ein Spawner erzeugt hat, haben keinen targetname. Erreiche sie über ihre Gruppe, zum Beispiel `@enemies`.

## Parameter { #parameters }

Der Parameter ist ein Wert, den du in die Verbindung tippst und an den Input übergibst, etwa `25` für `take_damage` oder eine Geschwindigkeit für eine Tür. Der Text wird zum ersten Typ, der passt:

| Du tippst | Der Input bekommt |
| --- | --- |
| `25` | Die Ganzzahl 25 |
| `0.5` | Den Float 0.5 |
| `true` oder `false` | Einen Bool |
| `0 64 0` | Einen `Vector3`, aus drei durch Leerzeichen getrennten Zahlen |
| `hello` | Den String `"hello"`, wenn nichts anderes passt |

Ein JSON-Array wie `[10, "$activator"]` wird auf mehrere Argumente verteilt, für Inputs, die mehr als eins nehmen. Werte werden in die Typen umgewandelt, die die Methode deklariert, also kommt `1234` als Text bei einem `String`-Argument an. Nimmt die Methode vor allen anderen Argumenten einen Node, wird Text wie `!player` oder ein targetname nachgeschlagen und als dieser Node übergeben.

Ein Parameter kann auch einer dieser Platzhalter sein, der beim Aufruf des Inputs eingesetzt wird:

| Platzhalter | Wird zu |
| --- | --- |
| `$activator` | Dem Aktivator, dem Node, der die Kette gestartet hat |
| `$self`, `$caller` | Der empfangenden Entität selbst |
| `$position` | Der globalen Position der empfangenden Entität |
| `$caller_name` | Dem targetname der empfangenden Entität, sonst ihrem Node-Namen |

Platzhalter werden an der **empfangenden** Entität eingesetzt, nicht an der, die ausgelöst hat. Sie müssen der ganze Parameter oder ein ganzes Array-Element sein, in längerem Text werden sie nicht ersetzt.

## Kein Parameter { #no-parameter }

Lässt du den Parameter leer, geht der Aktivator in das erste Argument, das als Objekt typisiert ist oder untypisiert `activator` heißt, egal wo es steht. Deshalb bekommt `take_damage(amount, source)` ihn in `source`. Andere Werte, die der Output mitgebracht hat, füllen die übrigen Argumente der Reihe nach, `changed(value)` an `set_value` verdrahtet gibt den Wert also weiter.

Das reicht auch `false` weiter, sodass `switched(false)` an das `add` eines Counters verdrahtet 0 addiert. Setze einen Parameter, wenn du einen festen Wert willst.

## Wie ein Input gefunden wird { #how-an-input-is-found }

Wenn eine Verbindung auslöst, wird der Input-Name in dieser Reihenfolge mit dem Ziel abgeglichen:

1. Eine Methode dieses Namens oder ihre PascalCase- oder camelCase-Schreibweise, ein C#-`AddScore` hört also auf `add_score`.
2. Eine Eigenschaft dieses Namens, gesetzt auf den Parameter oder den Wert, den der Output mitgebracht hat. Der Input `speed` mit Parameter `4` ändert die Geschwindigkeit einer Tür, eine zusätzliche Methode ist nicht nötig.
3. Einer der eingebauten Inputs unten, die jeder Node versteht.

| Input | Wirkung |
| --- | --- |
| `kill` | Gibt den Node frei und entfernt ihn aus dem Spiel |
| `show`, `enable` | Macht ihn sichtbar und lässt ihn wieder verarbeiten |
| `hide`, `disable` | Macht ihn unsichtbar und pausiert seine Verarbeitung |
| `toggle` | Schaltet nur die Sichtbarkeit um, die Verarbeitung bleibt, wie sie ist |

Die eigenen Methoden der Entität haben Vorrang, `disable` an einem Trigger schaltet den Trigger also ab, statt ihn zu verstecken. `show` und `hide` sind die eingebauten, sofern das Skript der Entität keine eigenen definiert. Jeder andere Input gibt `[TD I/O] <node> has no input '<name>'` aus.
