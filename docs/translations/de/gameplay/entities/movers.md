---
source: 63266bb349fb8b02f543d43498bfdee3d8c3f3cd
---

# Türen, Mover und Knöpfe { #doors-movers-and-buttons }

> [!NOTE]
> Diese Entitäten gehören zum Gameplay-Entitätenpaket, siehe
> [Installieren und anpassen](README.md#installing-and-customizing).

Brush-Entitäten, die sich als ein Körper bewegen und alles mitnehmen, was darauf steht, sodass ein Aufzug den Spieler mitnimmt.

> [!NOTE]
> Türen und Knöpfe reagieren nicht von selbst auf den Spieler. Dein Spielerskript ruft `use(activator)` an ihnen auf, zum Beispiel über einen Interaktionsstrahl, siehe [Ein Knopf öffnet eine Tür](../../tutorials/button-door.md). Setze `interact` auf 0 für eine, die nur I/O oder ein Skript bewegen soll.

## func_door { #func_door }

Eine Schiebetür, Luke oder ein Rollladen. Beim Öffnen bewegt sie sich um `travel`, beim Schließen zurück. Ziehe den Griff im Viewport, um den Wert zu setzen. Eine verschlossene Tür ignoriert `open` und `use` und löst stattdessen `locked_use` aus, für eine Meldung wie "verschlossen".

| Schlüssel | Standard | Was er tut |
| --- | --- | --- |
| `travel` | 0 64 0 | Versatz, um den sie schiebt, in Karteneinheiten |
| `speed` | 2.0 | Meter pro Sekunde |
| `wait` | -1 | -1 bleibt offen, ein positiver Wert schließt sie nach so vielen Sekunden wieder |
| `locked` | 0 | Startet verschlossen |
| `start_open` | 0 | Startet in der offenen Stellung |
| `interact` | 1 | Öffnet sich, wenn der Spieler `use` an ihr aufruft |

* **Inputs:** `open`, `close`, `toggle`, `lock`, `unlock`, `use(activator)`
* **Outputs:** `opened`, `closed`, `started_opening`, `started_closing`, `locked_use(activator)`

## func_door_rotating { #func_door_rotating }

Eine Tür oder ein Tor, das um ein Scharnier schwingt, das du im Viewport an die richtige Stelle ziehst. `open` nimmt den Aktivator entgegen, damit `open_away` sie von ihm wegschwingen lassen kann, statt ihm ins Gesicht.

| Schlüssel | Standard | Was er tut |
| --- | --- | --- |
| `hinge` | 0 0 0 | Position des Scharniers relativ zur Mitte der Tür |
| `axis` | y | Schwenkachse. `y` ist ein senkrechtes Scharnier, mit `x` oder `z` wird es eine Luke oder Zugbrücke |
| `open_angle` | 95 | Grad, um die sie schwingt, negativ schwingt sie in die andere Richtung |
| `speed` | 120 | Grad pro Sekunde |
| `open_away` | 1 | Schwingt von dem weg, der sie öffnet |
| `wait`, `locked`, `start_open`, `interact` | -1, 0, 0, 1 | Wie bei `func_door` |

* **Inputs:** `open(activator)`, `close`, `toggle(activator)`, `lock`, `unlock`, `use(activator)`
* **Outputs:** wie bei `func_door`

`func_gate` ist ein älterer Alias mit nur `hinge`, `open_angle` 100 und `speed` 90 als Schlüsseln. Nutze `func_door_rotating`.

## func_platform { #func_platform }

Ein Aufzug oder eine bewegte Plattform zwischen ihrem Start und `start + travel`, für Fahrstühle oder Böden, die über eine Grube fahren.

* **Inputs:** `start`, `stop`, `toggle`, `go_to_end`, `go_to_start`
* **Outputs:** `reached_end`, `reached_start`, `started`
* **Schlüssel:** `travel` 0 128 0, `speed` 2.0, `wait` 1.0, `mode` 0, `start_active` 0 (bewegt sich ab dem Laden der Karte)

| `mode` | `start` macht | Typische Verwendung |
| --- | --- | --- |
| 0, toggle | Fährt zum anderen Ende | Ein Aufzug, den ein Knopf ruft |
| 1, ping pong | Fährt hin und her, bis `stop`, und wartet an jedem Ende `wait` Sekunden | Eine Plattform, die ständig pendelt |
| 2, once | Fährt zum Ende und bleibt dort | Eine Brücke, die sich einmal ausfährt |

## func_train { #func_train }

Folgt ab `target` einer Kette von `path_corner`-Entitäten, für alles, was eine Route mit mehreren Halten fährt, etwa eine Straßenbahn oder eine Lore auf einer Schiene. `wait` und `speed` jeder Ecke gelten bei der Ankunft. Sie fährt beim Laden der Karte los und läuft in Schleife, setze also `start_active` oder `loop` auf 0 für eine einmalige Fahrt.

* **Inputs:** `start`, `stop`, `toggle`
* **Outputs:** `arrived(corner)`, `finished` (nur ohne Schleife)
* **Schlüssel:** `target`, `speed` 3.0, `loop` 1, `start_active` 1, `orient` 0 (1 dreht sie in Fahrtrichtung)

## func_button { #func_button }

Sinkt um `travel` ein, solange er gedrückt ist, und löst `pressed` aus. Nach `wait` Sekunden springt er wieder heraus, mit -1 bleibt er gedrückt, für einen Schalter, der nur einmal funktioniert. Ein verschlossener Knopf löst stattdessen `locked_use` aus, praktisch für ein Bedienfeld, das erst Strom braucht.

* **Inputs:** `press(activator)`, `release`, `lock`, `unlock`, `use(activator)`
* **Outputs:** `pressed(activator)`, `released`, `locked_use(activator)`
* **Schlüssel:** `travel` 0 -2 0, `wait` 1.0, `locked` 0, `interact` 1
