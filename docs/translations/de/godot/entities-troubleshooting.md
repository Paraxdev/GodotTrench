---
source: 1b2a2ceda473607c8db087c552aff86851d08ef2
---

# Entitäten, die nichts tun { #entities-that-do-nothing }

Wenn Godot eine Entität ohne ihre Definition baut, ist die Entität nur eine Hülle. Eine Brush-Entität wie eine Tür behält ihr Mesh, wird aber ein schlichter `Node3D` ohne Skript und ohne Kollision, und eine Punkt-Entität wird ein leerer `Marker3D`. Dasselbe passiert, wenn eine Definition nicht zu ihrer Entität passt, zum Beispiel bei einer Brush-Entität, deren Definition keinen Physikkörper baut. TrenchDot und Godot sagen dir beide, wenn das passiert, und diese Seite erklärt jede Meldung.

## Wo du es siehst { #where-you-see-it }

1. Die Statusleiste

    Speichere die Karte in TrenchDot, während ihre Szene in Godot offen ist. Hat Godot Klassen gebaut, für die es keine Definition hat, nennt die Statusleiste unten links sie und die Karteneinstellungen, mit denen Godot gebaut hat:

    ![Die Statusleiste mit der Meldung, dass Godot 1 Karten-Node neu gebaut hat, aber res://crypt_settings.tres keine Definition für func_button und func_door hat, sodass sie als einfache Nodes gebaut werden](../assets/entities-troubleshooting/status-missing-classes.png)

2. Das Panel Probleme

    Öffne den Tab **Probleme**, links neben Outliner und Verlauf. Er listet jede Klasse auf, die Godot als einfachen Node gebaut hat, mit der Lösung, und die anderen Warnungen dieses Builds beginnen mit "Godot:". Klick auf eine Zeile, um die Entität auszuwählen.

    ![Das Panel Probleme mit zwei Warnungen, Godot hat func_button und func_door als einfache Nodes gebaut, und jede sagt, welche Datei zu Base Fgd Files hinzugefügt werden soll](../assets/entities-troubleshooting/issues-missing-classes.png)

    Bevor Godot die Karte gebaut hat, vergleicht das Panel Probleme deine Entitäten mit dem FGD, mit dem Godot baut, das das Addon mit der [Spielkonfiguration](project-setup.md#the-game-config) exportiert. Fehlt einer Entität in diesem FGD die Definition, bekommt sie die Warnung "Godot baut mit ..., dessen FGD ... kein ... hat".

    Zeigt Godot eine Szene, die es gebaut hat, bevor eine Definition da war, sagen die Statusleiste und das Panel Probleme "Godot zeigt ..., gebaut mit anderen Entitätsdefinitionen". Wähl *Godot > In Godot bauen*, oder drück in Godot **Build Map** und speichere, um sie erneut zu bauen. **Projekt ausführen** sagt dasselbe über die Szenen, die es ausführt.

3. Das Ausgabe-Panel von Godot

    Drück in Godot am Node `FuncGodotMap` auf **Build Map**. Das Panel Output unten im Godot-Editor zeigt eine `[TrenchDot]`-Warnung für die Klassen ohne Definition und eine für jede Definition, die nicht passt. Hak am Node unter *Build Flags* *Print Build Report* an, um den ganzen [Build-Bericht](building.md#build-report) zu bekommen.

## Entitäten ohne Definition { #entities-without-a-definition }

| Die Meldung sagt | Warum | Das tun |
| --- | --- | --- |
| Installiere das Gameplay-Entitätenpaket | Türen, Knöpfe, Trigger, Props und Spawner kommen mit dem Paket, und das Projekt hat es nicht | Wähl in TrenchDot *Gameplay > Gameplay-Entitäten installieren*, siehe [Projekteinrichtung](project-setup.md#installing-and-updating-the-pack) |
| Deinem FGD fehlt das Gameplay-Entitätenpaket, füge `trenchdot_default_fgd.tres` zu seinen Base Fgd Files hinzu | Dein FGD hat keine *Base Fgd Files* | Füg `res://addons/func_godot/fgd/trenchdot_default_fgd.tres` zu den *Base Fgd Files* des FGD hinzu, das die Karteneinstellungen in *Entity Fgd* nennen |
| Das Paket ist installiert, aber das *Entity Fgd* ist ein FGD des Addons | Die Karteneinstellungen nutzen das eigene FGD von FuncGodot oder eine andere Datei des Addons, die ein Addon-Update ersetzt | Setz *Entity Fgd* dieser Karteneinstellungen auf das FGD, das deine Spielkonfiguration nutzt |
| Eine eigene Klasse hat keine Definition | Sie steht im FGD deiner Spielkonfiguration, aber nicht im *Entity Fgd* der Karteneinstellungen, mit denen Godot baut | Setz *Entity Fgd* dieser Karteneinstellungen auf das FGD, das deine Spielkonfiguration nutzt, siehe [Deine eigenen Ressourcen](project-setup.md#your-own-resources) |
| Der Karten-Node nutzt die Standard-Karteneinstellungen des Addons, nicht die aus den Projekteinstellungen | Der Node wurde mit einem älteren Addon hinzugefügt, das die Standard-Karteneinstellungen in der Szene gespeichert hat | Wähl den Karten-Node in Godot und leere *Map Settings* im Inspector |
| Die Szene wurde mit anderen Entitätsdefinitionen gebaut | Sie wurde gebaut, bevor das Paket oder eine Definition da war, und seitdem nicht mehr | Drück **Build Map** und speichere die Szene. Im [Live-Modus](live.md#when-live-mode-rebuilds-the-whole-map) macht Godot das von selbst |

## Definitionen, die nicht passen { #definitions-that-do-not-fit }

Diese kommen aus dem Ausgabe-Panel von Godot, und nach einem Speichern aus dem Panel Probleme als "Godot:"-Warnungen. Das Panel Probleme markiert die letzten beiden auch schon vor dem Speichern.

| Die Meldung sagt | Warum | Das tun |
| --- | --- | --- |
| `func_crusher builds as a Marker3D, so its brushes have no collision` | Die Definition hat keine *Node Class*, und ihr Skript erweitert einen Node, der kein Physikkörper ist | Setz *Node Class* auf `StaticBody3D`, `AnimatableBody3D` oder `Area3D`, oder lass das Skript einen davon erweitern |
| `func_crusher: Node Class AnimatedBody3D not found, building it as a Node3D` | Ein Tippfehler, oder eine `class_name`, die Godot noch nicht registriert hat | Korrigiere den Namen. Für C# siehe [C#-Entitäten](../gameplay/csharp.md) |
| `alarm: res://entities/alarm.gd has no @tool` | Der Editor führt nur Tool-Skripte aus, die Keys aus der Karte erreichen den Node also nie | Füg `@tool` oben im Skript hinzu |
| `light is a point entity, so its brushes are dropped` | Eine `FuncGodotFGDPointClass` wird für eine Brush-Entität genutzt | Mach die Definition zu einer `FuncGodotFGDSolidClass` |
| `func_door is a brush entity without brushes` | Eine Brush-Entität, aus der sich nichts bauen lässt | Gib ihr Brushes, oder mach die Definition zu einer Punktklasse |

## Entitäten, die gebaut werden, aber nicht reagieren { #entities-that-build-but-do-not-react }

| Was du siehst | Warum | Das tun |
| --- | --- | --- |
| Trigger feuern nie, und die Ausgabe von Godot sagt, der Trigger habe deinen Spieler ignoriert, weil kein Node in seiner `filter_group` ist | Standardmäßig reagieren Trigger nur auf Körper in der Gruppe `player` auf Physikebene 1 | Füg deinen Spieler zur Gruppe `player` hinzu. Auf einer anderen Ebene füg diese Ebene zur *Trigger Collision Mask* in den Karteneinstellungen hinzu |
| Türen und Knöpfe bewegen sich nie | Sie bewegen sich, wenn dein Spielcode `use()` an ihnen aufruft | Siehe [Lass den Spieler drücken](../tutorials/button-door.md#let-the-player-press-it) |
| `info_spawner` und `npc_walker` wirken im Editor von Godot leer | Sie erzeugen oder laden ihr Modell nur im laufenden Spiel | Siehe [Spawner und Pfade](../gameplay/entities/actors.md) |
| Die Ausgabe von Godot zeigt `Parse Error` für ein Skript in `res://trenchdot/entities` | Godot hat ein Paketskript kompiliert, bevor es die Klassen kannte, die es nutzt | Starte Godot neu, drück dann **Build Map** und speichere |
