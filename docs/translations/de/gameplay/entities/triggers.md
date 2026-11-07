---
source: c7fdbd8dcaa1ac8625c1d6e7014b686194c50f36
---

# Trigger { #triggers }

> [!NOTE]
> Diese Entitäten gehören zum Gameplay-Entitätenpaket, siehe
> [Installieren und anpassen](README.md#installing-and-customizing).

Unsichtbare Brush-Volumen, die reagieren, wenn ein Körper hineinkommt. So bemerkt ein Level den Spieler: Ein Türrahmen startet eine Zwischensequenz, eine Grube verletzt, ein Pad katapultiert. Zeichne eines mit dem Werkzeug **Volumen** (Shift+E).

## Gemeinsame Inputs, Outputs und Schlüssel { #shared-inputs-outputs-and-keys }

Jeder Trigger unten hat diese zusätzlich zu seinen eigenen.

* **Inputs:** `enable`, `disable`, `toggle`
* **Outputs:** `triggered(activator)`, `entered(activator)`, `exited(activator)`
* **Schlüssel:** `filter_group` player, `start_disabled` 0

`filter_group` beschränkt den Trigger auf Körper in dieser Godot-Gruppe, damit Gegner und Kisten keinen Spieler-Trigger auslösen. Lass es leer, um jeden Körper zu akzeptieren.

| Output | Löst aus |
| --- | --- |
| `entered` | Bei jedem akzeptierten Eintritt, solange er aktiviert ist |
| `triggered` | Bei einem akzeptierten Eintritt, höchstens einmal pro `cooldown` für jeden Körper. Die meiste Verdrahtung nutzt diesen |
| `exited` | Beim Austritt, auch wenn der Trigger deaktiviert ist, um etwas rückgängig zu machen, wenn der Körper geht |

Trigger erkennen Körper auf Physikebene 1. Liegt der Spieler auf einer anderen Ebene, füge sie in den Karteneinstellungen (die Ressource `FuncGodotMapSettings` in Godot) zu **Trigger Collision Mask** hinzu, sonst lösen die Trigger nie aus.

## trigger_once { #trigger_once }

Löst einmal aus und deaktiviert sich dann selbst, für einmalige Ereignisse wie eine Zwischensequenz oder einen Hinweis. `enable` schärft ihn wieder.

* **Schlüssel:** `once` 1

## trigger_multiple { #trigger_multiple }

Löst bei jedem Eintritt aus, höchstens einmal pro `cooldown` für jeden Körper, für Dinge, die jedes Mal passieren, etwa eine Ladenglocke.

* **Schlüssel:** `cooldown` 0.5

## trigger_hurt { #trigger_hurt }

Fügt Körpern Schaden zu, solange sie darin bleiben, für Lava, Stacheln oder eine Todeszone unter der Karte. Er ruft alle `interval` Sekunden `damage_method(amount, source)` an ihnen auf, mit `damage * interval` als Menge.

* **Outputs:** `hurt(activator)`, pro verletztem Körper und Tick
* **Schlüssel:** `damage` 10 (pro Sekunde), `damage_method` take_damage, `interval` 0.5, `filter_group` leer

Mit dem leeren Standardwert von `filter_group` verletzt er alles, was die Methode hat, nicht nur den Spieler. Ist eine Gruppe gesetzt, löst `hurt` für jeden Körper der Gruppe aus, auch für einen ohne die Methode.

## trigger_teleport { #trigger_teleport }

Bewegt Körper zu der Entität, die in `destination` steht, meist ein [`info_teleport_destination`](actors.md#info_teleport_destination), und dreht sie in deren Yaw. Er bremst sie ab, außer `keep_velocity` ist 1, für ein Portal, durch das der Spieler fallen oder rennen soll, ohne Tempo zu verlieren.

* **Outputs:** `teleported(activator)`
* **Schlüssel:** `destination`, `keep_velocity` 0, `cooldown` 0.5

## trigger_push { #trigger_push }

Schubst Körper mit `push`, einer Geschwindigkeit in Karteneinheiten pro Sekunde, die du mit ihrem Viewport-Griff ausrichten kannst. Mit `once` 1 bekommt jeder Eintritt einen Impuls, wie bei einem Sprungpad. Mit 0 schiebt er gleichmäßig wie ein Windkanal oder ein Förderband, und ein Starrkörper beschleunigt nicht über die Geschwindigkeit `push` hinaus.

* **Outputs:** `pushed(activator)`, einmal pro Eintritt
* **Schlüssel:** `push` 0 384 0, `filter_group` leer, `once` 1

## trigger_call { #trigger_call }

Ruft beim Auslösen eine Methode auf, standardmäßig an `!activator`, sodass ein Trigger direkt ins Skript des Spielers rufen kann, zum Beispiel um einen Checkpoint zu speichern. `call_target` und `arguments` funktionieren wie bei [`logic_call`](scripting.md#logic_call).

* **Inputs:** `trigger(activator)`
* **Outputs:** `called(result)`
* **Schlüssel:** `call_target` `!activator`, `method`, `arguments` `[]`, `once` 0, `cooldown` 0.5

## trigger_spawn_area { #trigger_spawn_area }

Spawnt Szenen an zufälligen Stellen in seinem Volumen, standardmäßig wenn der Spieler eintritt, für einen Hinterhalt, wenn der Spieler einen Raum betritt. Seine Schlüssel funktionieren wie bei [`info_spawner`](actors.md#info_spawner). `start` startet einen Spawn-Timer, wenn `interval` über 0 liegt, und `toggle` schaltet hier diesen Timer um statt des Triggers.

* **Inputs:** `spawn`, `start`, `stop`, `toggle`, `kill_all`
* **Outputs:** `spawned(node)`, `all_dead`, `exhausted` (einmal, wenn `total` zum ersten Mal erreicht wird)
* **Schlüssel:** `scene`, `count` 3, `max_alive` 10, `total` 0, `interval` 0, `spawn_on_enter` 1, `spawn_group` enemies,
  `snap_to_ground` 1, `cooldown` 0.5

## trigger_area { #trigger_area }

Ein einfaches `Area3D`-Volumen ohne Skript, um die Ereignisse beim Betreten und Verlassen eines Raums direkt mit anderen Entitäten zu verdrahten. Es hat keinen der gemeinsamen Schlüssel von oben und löst für jeden Körper aus, filtere also in der Entität, die es aufruft, wenn das wichtig ist.

* **Outputs:** `body_entered(body)`, `body_exited(body)`
* **Schlüssel:** `targetname`
