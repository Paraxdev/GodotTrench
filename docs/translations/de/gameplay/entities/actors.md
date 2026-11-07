---
source: 978a2cda95dff6236396e32b6d494bed11f3f94b
---

# Spawner und Pfade { #spawners-and-paths }

> [!NOTE]
> Diese Entitäten gehören zum Gameplay-Entitätenpaket, siehe
> [Installieren und anpassen](README.md#installing-and-customizing).

## info_spawner { #info_spawner }

Spawnt eine Szene, etwa einen Gegner oder ein Pickup, an zufälligen Stellen innerhalb von `radius`. Löse `spawn` von einem Trigger aus für einen Hinterhalt, oder setze ein `interval` und starte es mit `start` für einen stetigen Strom. Um Spawns über einen Raum zu verteilen, nimm [`trigger_spawn_area`](triggers.md#trigger_spawn_area).

| Schlüssel | Standard | Was er tut |
| --- | --- | --- |
| `scene` | | Die `res://`-Szene, die gespawnt wird |
| `count` | 1 | Wie viele jedes Mal gespawnt werden |
| `max_alive` | 5 | Nie mehr gleichzeitig am Leben, überzählige Spawns werden übersprungen |
| `total` | 0 | Hört nach so vielen für immer auf, 0 heißt unbegrenzt |
| `interval` | 0 | Sekunden zwischen Spawns, solange der Timer läuft. 0 spawnt nur bei `spawn` |
| `radius` | 64 | Wie weit vom Spawner entfernt sie erscheinen dürfen. Ziehe seinen Griff im Viewport |
| `start_active` | 0 | Lässt den Timer ab dem Laden der Karte laufen |
| `spawn_on_ready` | 0 | Spawnt einmal, wenn die Karte lädt |
| `spawn_group` | enemies | Godot-Gruppe, die jedem gespawnten Node hinzugefügt wird |
| `snap_to_ground` | 1 | Setzt jeden Spawn auf den Boden darunter |

* **Inputs:** `spawn`, `start`, `stop`, `toggle`, `kill_all`
* **Outputs:** `spawned(node)`, `all_dead`, `exhausted` (einmal, wenn `total` zum ersten Mal erreicht wird)

`all_dead` löst aus, wenn jeder gespawnte Node weg ist, für "räume den Raum, um die Tür zu öffnen". Gespawnte Nodes haben keinen targetname, erreiche sie über ihre `spawn_group`, etwa `@enemies`.

Mit den Standardschlüsseln spawnt er nichts, bis etwas `spawn` an ihm auslöst. Für Spawns ohne Trigger setze `spawn_on_ready` oder ein `interval` mit `start_active`. Gespawnt wird nur im laufenden Spiel, im Godot-Editor bleibt der Spawner ein leerer Node.

## info_teleport_destination { #info_teleport_destination }

Markiert, wohin `trigger_teleport` Körper schickt. Richte ihn so aus, wie der Spieler bei der Ankunft schauen soll, sein Yaw wird zu deren Blickrichtung. Sein einziger Schlüssel ist `targetname`.

## path_corner { #path_corner }

Ein Halt auf der Route eines `func_train` oder `npc_walker`, wobei `target` den nächsten benennt. Klicke eine Kette mit dem Werkzeug **Pfad** (Shift+P) ab. Er löst `reached` aus, wenn ein Zug oder Läufer ankommt, für einen Sound oder ein Ereignis an diesem Halt.

* **Outputs:** `reached(train)`
* **Schlüssel:** `target`, `wait` 0 (Sekunden, die hier gewartet wird), `speed` 0 (neue Zuggeschwindigkeit ab hier, 0 behält sie bei)

`speed` wirkt nur auf Züge, Läufer behalten ihre eigene.

## npc_walker { #npc_walker }

Ein geskripteter Darsteller, der eine Kette von Pfadecken abläuft und dabei Animationen abspielt, für Zwischensequenzen statt KI, etwa eine Wache, die an einem Fenster vorbeigeht. Er bewegt sich in geraden Linien und ignoriert Schwerkraft und Wände, lege die Ecken also dorthin, wo er laufen kann.

| Schlüssel | Standard | Was er tut |
| --- | --- | --- |
| `model` | | Eine Szene, die ihm ein Mesh und einen `AnimationPlayer` gibt |
| `target` | | Die erste Pfadecke, zu der er läuft |
| `speed` | 2.0 | Meter pro Sekunde |
| `loop` | 0 | Kehrt nach der letzten Ecke zur ersten zurück |
| `start_active` | 0 | Läuft ab dem Laden der Karte |
| `walk_anim`, `idle_anim` | walk, idle | Animationen, die beim Laufen und im Stand abgespielt werden |
| `face_travel` | 1 | Dreht sich in Laufrichtung |
| `collision` | default | Die aus seinem Modell gebaute Form, damit andere Körper gegen ihn stoßen, siehe [Kollision](collision.md) |
| `collision_layer`, `collision_mask` | leer | Ebenennummern wie `1, 3`, leer behält den Projektstandard |

* **Inputs:** `start`, `stop`, `walk_to(corner)`, `play_anim(name)`, `face(target)`
* **Outputs:** `arrived(corner)`, `reached_goal`, `finished` (beide an der letzten Ecke, wenn er nicht in Schleife läuft)

`face` und `walk_to` akzeptieren jedes Ziel, etwa `!player`. `walk_to` bricht die aktuelle Etappe ab und läuft direkt dorthin.

Er lädt sein `model` nur im laufenden Spiel, im Godot-Editor ist er also ein leerer Node.
