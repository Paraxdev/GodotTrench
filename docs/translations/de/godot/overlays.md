---
source: 5a39bcf2beab2f567018f04963913fd06af5df16
---

# Overlays { #overlays }

Jeder Build ersetzt, was die Karte erzeugt hat, alles, was du von Hand unter dem Karten-Node hinzufügst, ist also beim nächsten Speichern weg. Ein Overlay ist der Ort für Godot-Inhalte, die zur Karte gehören und jeden Neubau überleben müssen: ein Verkaufsautomat mit eigenem Skript, ein in Godot abgestimmter Partikeleffekt, eine Lampe mit einem AnimationPlayer.

![Der Hof des nächtlichen Viertels: Lichterketten, Glühwürmchen und ein Schild aus dem Overlay, Lampen aus der Karte](../assets/overlays/night-district-courtyard.jpg)

## Eines hinzufügen { #adding-one }

1. Den Overlay-Node hinzufügen

    Füg in Godot einen `TrenchDotOverlay`-Node als direktes Kind der `FuncGodotMap` hinzu.

2. Deine Nodes darunter legen

    Builds, Live-Modus und Hot Reload rühren ihn nie an, seine Nodes behalten also ihren Zustand und ihre Signal-Verbindungen und werden mit deiner Szene gespeichert.

(Für einen einzelnen Node kannst du das Overlay weglassen: Ein direktes Kind der Karte in der Gruppe `trenchdot_keep` überlebt Builds ebenfalls. Ein Overlay an anderer Stelle in der Szene kann stattdessen mit *Map Path* auf seine Karte zeigen und folgt dann der Transformation der Karte.)

Overlay-Inhalte nutzen den lokalen Raum der Karte in Metern. Bei 32 Einheiten pro Meter sitzt ein Kind bei `(2, 0, -3)` dort, wo TrenchDot `(64, 0, -96)` zeigt.

## Ein Overlay in einer eigenen Szene { #an-overlay-in-its-own-scene }

Ein Overlay kann auch eine eigene Szene sein. Das lohnt sich, wenn mehrere Szenen dieselbe Karte nutzen und sich das Overlay teilen sollen, wenn eine Person an der Levelszene arbeitet und eine andere am Overlay oder wenn ein Skript die Levelszene erzeugt, wie in der Demo.

1. Leg eine Szene an, deren Wurzel ein `TrenchDotOverlay` ist.
2. Zieh die `.tscn` auf den `FuncGodotMap`-Node, damit sie als direktes Kind der Karte instanziiert wird.

Godot zeigt eine instanziierte Szene als einzelnen Node ohne Pfeil zum Aufklappen. So kommst du an ihren Inhalt:

| Du willst | Tu das |
| --- | --- |
| Das Overlay überall bearbeiten, wo es genutzt wird | Klick im Szenen-Dock auf das Symbol *Open in Editor* neben dem Node oder doppelklick die `.tscn` im FileSystem-Dock |
| Seine Nodes nur in dieser Szene sehen oder ändern | Klick mit rechts auf den Node und schalte *Editable Children* ein |

Für sich allein geöffnet zeigt die Overlay-Szene an ihrer Wurzel eine Warnung, weil sie nicht unter einer Karte liegt. Das ist erwartet. Die gestrichelten Kästen in TrenchDot ziehen nach, sobald die Szene mit der Karte das nächste Mal gebaut oder gespeichert wird, denn nur dort kennt das Overlay seine Karte.

## Overlays verdrahten { #wiring-overlays }

Overlay-Nodes und Karten-Entitäten können einander über dieselben Ausgänge und Eingänge auslösen, die Karten-Entitäten untereinander nutzen.

| Du willst | Tu das |
| --- | --- |
| Dass ein Ausgang der Karte einen Overlay-Node erreicht | Gib dem Node ein Kind `TrenchDotOverlayIO` und setz seinen *Targetname* |
| Dass ein Overlay-Node auf Karten-Entitäten feuert | Füg ein Kind `TrenchDotOutput` hinzu, wähl das Signal bei *Output* und trag Ziel, Eingang und Parameter ein |
| Dass Inhalt auf einer bewegten Entität mitfährt | Leg ihn unter einen `TrenchDotAnchor`, dessen *Target* auf den targetname der Entität zeigt |

Ein Eingang, der an ein `TrenchDotOverlayIO` gesendet wird, ruft die Methode des Nodes mit diesem Namen auf, setzt seine Eigenschaft mit diesem Namen oder führt `show`, `hide` oder `toggle` aus. Auch Nodes ohne Skript funktionieren also: Der Eingang `emitting` mit dem Parameter `true` startet einen `GPUParticles3D`. Jeder Eingang sendet außerdem `input_received(input, parameter, activator)`.

Ein `TrenchDotOutput` lauscht auf jedes Signal seines Elternteils, ein `Area3D` feuert also ohne Code bei `body_entered`. Für eigene Ereignisse füg dem Skript des Elternteils ein Signal hinzu und wähl es:

```gdscript
extends StaticBody3D

signal flipped_on(activator: Node)
signal flipped_off(activator: Node)

var on := false

func use(activator: Node = null) -> void:
	on = not on
	if on:
		flipped_on.emit(activator)
	else:
		flipped_off.emit(activator)
```

Ein Anker behält den Versatz, an dem du ihn platziert hast, und folgt der Entität durch Neubauten, Live-Änderungen und Bewegung, ein Warnschild fährt also mit seinem Rolltor nach oben.

> [!WARNING]
> Verdrahtung überlebt Neubauten, Zustand nicht. Karten-Entitäten kommen in ihrem Startzustand zurück, verbinde dich also mit dem Signal `map_rebuilt(map)` des Overlays und schieb deinen Zustand zurück, schalte zum Beispiel die Lampen wieder ein.

## In der Demo ansehen { #see-it-in-the-demo }

Das nächtliche Viertel des Demoprojekts hat all das. Seine Szene `res://demo/showcase/night_district.scn` instanziiert die Overlay-Szene `res://demo/overlays/night_district_overlay.tscn` unter ihrer Karte, weil ein Skript die Showcase-Szenen neu baut. Öffne die Overlay-Szene, und du findest einen Sicherungskasten, der die Hoflampen schaltet, Lichterketten, die ein Trigger der Karte einschaltet, und ein Schild, das am Garagenrolltor mitfährt.
