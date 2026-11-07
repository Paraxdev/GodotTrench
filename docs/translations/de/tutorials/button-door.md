---
source: 19de62e149bf31bc7f0453a04737b8fbd5510db4
---

# Knopf öffnet eine Tür { #button-opens-a-door }

Ein Knopf an der Wand, der eine Tür aufschiebt, wenn der Spieler ihn drückt. Das ist das kleinste nützliche Stück Entity-I/O: Der Knopf löst einen Output namens `pressed` aus, und eine Verbindung macht daraus einen Aufruf des Inputs `open` der Tür.

> [!TIP]
> Wenn Outputs, Inputs und `targetname` neu für dich sind, lies zuerst [So funktioniert Entity-I/O](../gameplay/io.md). Das dauert nur ein paar Minuten, und diese Seite baut darauf auf.

## Verdrahten { #wire-it }

`func_door` und `func_button` kommen mit dem Gameplay-Entitätenpaket. Hat dein Projekt es noch nicht, nutze *Gameplay > Gameplay-Entitäten installieren* oder **Gameplay-Entitäten installieren** im Fenster *Dieses Projekt einrichten* (*Godot > Addon installieren oder aktualisieren…*).

1. Die Tür bauen

    Zeichne einen türgroßen Brush in einen Durchgang, wie du einen ausschneidest, zeigt [Erste Schritte](../getting-started.md#4-build-a-room). Klicke den Brush mit der rechten Maustaste an und wähle *Brush-Entität erstellen > func_door*.

    ![Das Rechtsklickmenü an einem orangen Türbrush, Brush-Entität erstellen ist geöffnet und func_door markiert](../assets/button-door/create-brush-entity-func-door.png)

2. Benennen

    Tippe im Inspektor rechts `main_door` in `targetname` ein. Über den Namen finden andere Entitäten die Tür. Lass `travel` bei `0 64 0`, dann schiebt sich die Tür beim Öffnen 64 Einheiten (2 m) nach oben.

    ![Der Inspektor der func_door mit targetname main_door und travel 0 64 0](../assets/button-door/inspector-targetname.png)

3. Den Knopf bauen

    Zeichne einen kleinen Brush an die Wand neben der Tür und mache ihn genauso zu einer `func_button`.

4. Beide wählen

    Klicke den Knopf an, dann mit Ctrl+Klick die Tür, in einer Ansicht oder im Outliner links. Stehen Wände im Weg? Tippe `func` in das Feld *Filter* oben im Outliner, dann werden nur diese beiden aufgelistet.

    ![Der nach func gefilterte Outliner, func_door (main_door) und func_button sind beide gewählt](../assets/button-door/outliner-filter-func.png)

5. Verknüpfen

    Gehe zu *Gameplay > Logik > Zwei gewählte Entitäten verknüpfen…* oder klicke oben im Inspektor auf **Verknüpfen…**.

    ![Das Gameplay-Menü mit geöffnetem Logik und markiertem Zwei gewählte Entitäten verknüpfen…](../assets/button-door/menu-link-two-selected.png)

    Die Entität, die auslöst, muss links stehen, hier ist das der Knopf. Der Dialog setzt die zuerst erstellte nach links, und das ist die Tür, also klicke auf den kleinen Tauschknopf zwischen den beiden Namen. Wähle `pressed` unter *Wenn* und `open` unter *Aufruf*, dann klicke auf **Verknüpfen**.

    ![Das Fenster Entitäten verknüpfen mit func_button links, func_door (main_door) rechts, Wenn pressed und Aufruf open](../assets/button-door/link-dialog.png)

Die Statusleiste unten links sollte jetzt `pressed > main_door.open` zeigen. Lies es als "Wenn pressed, rufe open an main_door auf".

![Die Statusleiste mit pressed > main_door.open](../assets/button-door/status-linked.png)

Wählst du den Knopf allein, findest du die Verbindung im Inspektor unter *Ausgänge*, wo du sie später ändern oder entfernen kannst.

![Der Abschnitt Ausgänge der func_button: Output pressed, Ziel main_door, Input open](../assets/button-door/inspector-button-output.png)

> [!TIP]
> *Gameplay > Türen und Beweger*, auch im Abschnitt *Gameplay* des Inspektors, macht den gewählten Brush mit einem Klick zu einer Dreh- oder Schiebetür. Es benennt die Tür, legt fest, wie sie sich öffnet, und stellt ein `trigger_multiple` davor, das die Tür öffnet, wenn der Spieler herankommt. Lösche diesen Trigger, wenn nur der Knopf die Tür öffnen soll.
>
> ![Das Gameplay-Menü mit geöffnetem Türen und Beweger: Drehtür mit linker oder rechter Angel, Schiebetür nach oben oder zur Seite und Aufzug, bewegliche Plattform](../assets/button-door/menu-doors-and-movers.png)

## Den Spieler drücken lassen { #let-the-player-press-it }

Das Addon bringt kein eigenes Interaktionssystem mit, denn jedes Spiel handhabt "benutzen" anders. Türen und Knöpfe haben eine Methode `use(activator)`, und dein Spielerskript ruft sie auf. Der Demo-Spieler tut das bei **E**: Er schießt einen Strahl von der Kamera und ruft `use` am ersten Treffer auf, der die Methode hat ([godot/demo/player.gd](https://github.com/Paraxdev/TrenchDot/blob/main/godot/demo/player.gd)):

```gdscript
func use_target() -> void:
	var from := camera.global_position
	var query := PhysicsRayQueryParameters3D.create(from, from - camera.global_basis.z * use_distance)
	query.exclude = [get_rid()]
	query.collide_with_areas = false
	var hit := get_world_3d().direct_space_state.intersect_ray(query)
	var node: Node = hit.get("collider")
	while node:
		if node.has_method(&"use"):
			node.call(&"use", self)
			return
		node = node.get_parent()
```

Der Spieler übergibt sich selbst als Aktivator, also trägt `pressed(activator)` den Spieler mit. Spätere Outputs in der Kette können `!activator` als Ziel nehmen, um an dem anzusetzen, der den Knopf gedrückt hat.

## Varianten { #variations }

| Du willst | Was du änderst |
| --- | --- |
| Tür öffnet sich nur über den Knopf | Hake `interact` der Tür ab. Sonst öffnet auch **E** an der Tür sie |
| Tür schließt sich selbst | Setze `wait` der Tür auf die Sekunden, die sie offen bleibt. Der Standard -1 hält sie offen |
| Verschlossene Tür | Hake `locked` an. Die Tür ignoriert `open` und `use` und löst stattdessen `locked_use` aus, bis ein Output ihren Input `unlock` aufruft, zum Beispiel von einem Schlüssel-Pickup |
| Ein Knopf, mehrere Türen | Nenne die Türen `gate_1`, `gate_2` und so weiter und ziele auf `gate_*`. Ein `*` am Ende passt auf jeden Namen, der mit `gate_` beginnt |
| Schwingtür | Nimm stattdessen `func_door_rotating` und ziehe ihren Angelgriff in der Ansicht dorthin, wo die Angel sitzen soll |

## Dieselbe Verdrahtung im Code { #the-same-wiring-in-code }

Verbindest du lieber in GDScript als im Editor? Das hier macht dasselbe wie die Verbindung oben:

```gdscript
# pressed(activator) > main_door.open
button.pressed.connect(func(activator):
	for door in TrenchDotIO.find_targets(button, "main_door", activator):
		TrenchDotIO.invoke(door, &"open", "", activator))
```

`pressed` direkt mit `door.open` zu verbinden schlägt fehl, weil `pressed` ein Argument übergibt und `open()` keines nimmt. `invoke` passt die Argumente an die Methode an, genau wie Karten-Verbindungen es tun.
