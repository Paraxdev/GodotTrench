---
source: 7e5d26a9fedbc9f43e9edcb92409992b19d24676
---

# Navigation { #navigation }

`TrenchDotNav` backt aus einer gebauten Karte ein Navigationsmesh, damit Gegner und NPCs ihren Weg hindurch finden.

```gdscript
map.build()
var region := await TrenchDotNav.bake_region(map)
```

`bake_region` backt in einem Worker-Thread, fügt unter der Karte eine `NavigationRegion3D` namens `navigation` hinzu und kehrt zurück, sobald Pfade darüber funktionieren. Gib einen Node mit mehreren Karten, um sie alle in ein Mesh zu backen. Ein Neubau gibt die Region frei, back also danach erneut.

## Was zählt { #what-counts }

Das Backen liest die Kollision der Karte, nicht ihre Meshes. Alles, wogegen ein Spieler stoßen kann, ist also das, worum ein Agent herumläuft.

| In der Karte | Im Navigationsmesh |
| --- | --- |
| Welt-Brushes, `func_detail`, Terrain und Prop-Kollision | Ja |
| Clip-Brushes | Ja, sie sperren wie Wände |
| Illusionary-Brushes, Trigger und Physik-Props | Nein |
| Türen, Buttons, Plattformen und Züge | Nein, eine geschlossene Tür versiegelt ihren Durchgang also nicht |
| Körper in der Gruppe `navigation_ignore` | Nein |

Das heißt, Pfade führen durch geschlossene Türen, ein Agent muss also eine Tür öffnen, in die er hineinläuft.

## Agentengröße { #agent-size }

`TrenchDotNav.profile(radius, height, max_climb, max_slope)` liefert dir die Einstellungen für eine Agentengröße. Der Radius hält Pfade von Wänden fern, die Höhe schließt niedrige Decken aus, *max_climb* ist die höchste Stufe, die der Agent hinaufsteigt, und *max_slope* die steilste Rampe. Größen sind in Metern, die Neigung in Grad:

| Argument | Standard |
| --- | --- |
| `radius` | 0,3 |
| `height` | 1,8 |
| `max_climb` | 0,3 |
| `max_slope` | 45 |

Das Mesh nutzt Zellen von 0,1 m und rundet die Größen auf sie. Gib ein Profil als zweites Argument mit, zum Beispiel für ein breites, niedriges Wesen:

```gdscript
var region := await TrenchDotNav.bake_region(map, TrenchDotNav.profile(0.5, 1.2))
```

(`bake_region` setzt die Zellgröße der Navigationskarte auf die des Backens, und andere Regionen auf dieser Karte müssen sie teilen.)

## Zwischenspeicher { #caching }

Das erste Backen einer großen Karte dauert ein paar Sekunden. Das Ergebnis wird in `user://trenchdot_navigation` zwischengespeichert, abhängig von der Kollision und den Einstellungen, beim nächsten Start mit derselben Karte lädt es also in einem Bruchteil einer Sekunde. Gib `false` als drittes Argument mit, um ohne den Zwischenspeicher zu backen.

## Eine Karte prüfen { #checking-a-map }

`TrenchDotNav.bake(map)` backt im aufrufenden Thread und gibt das `NavigationMesh` zurück, praktisch für Werkzeuge und Tests. `TrenchDotNav.islands(nav)` listet seine nicht verbundenen Teile als Polygonindizes auf, den größten zuerst.

Ein Raum, der erreichbar sein sollte, aber auf einer eigenen Insel liegt, hat meist einen Durchgang, den etwas Niedriges versperrt, oder eine Stufe, die höher ist als `max_climb`. Auch Wandkronen und der Boden im Inneren dicker Wände tauchen als kleine Inseln auf.

> [!WARNING]
> Godot bricht eine Pfadsuche nach 4096 Polygonen ab und gibt den Pfad bis dahin zurück, was auf einer großen Karte wie ein unerreichbarer Punkt aussieht. `TrenchDotNav.path(node, from, to)` hat kein Limit. Setz bei Agenten `path_search_max_polygons` an der `NavigationAgent3D` auf 0.

## In TrenchDot ansehen { #seeing-it-in-trenchdot }

Du kannst direkt in TrenchDot sehen, wo ein Agent läuft, ohne das Spiel zu starten.

1. Die Szene in Godot öffnen

    Speichere die Karte im Projekt und öffne das Projekt im Godot-Editor mit einer Szene, deren `FuncGodotMap` diese Karte baut.

2. Den begehbaren Bereich einschalten

    Wähl *Ansicht > Begehbarer Bereich > Begehbaren Bereich zeigen*. Die Agentengröße steht direkt darunter, und die unterste Zeile sagt dir, wie viel begehbar ist und wie viele Inseln abgeschnitten sind.

    ![Das Menü Ansicht mit geöffnetem Untermenü Begehbarer Bereich: Begehbaren Bereich zeigen, dann Radius 0.30 m, Höhe 1.80 m, Max. Steighöhe 0.30 m, Max. Steigung 45.00 Grad und Noch nicht gebacken](../assets/navigation/menu-walkable-area.png)

Godot backt die Szene über den [Live Link](live-link.md), und TrenchDot zeichnet das Ergebnis in die 3D- und 2D-Ansicht. (Kann es nicht backen, sagt die Statusleiste warum, zum Beispiel dass die Karte erst im Projekt gespeichert werden muss.)

Die Farben bedeuten:

| Farbe | Bedeutung |
| --- | --- |
| Bläuliches Grün | Die größte zusammenhängende Fläche, auf der die meisten Agenten sein werden |
| Zinnoberrot | Inseln, die ein Agent auf der grünen Fläche nicht erreichen kann. Such nach einer Stufe, die höher ist als *Max. Steighöhe*, einer Lücke, die schmaler ist als der doppelte *Radius*, oder einer Steigung, die steiler ist als *Max. Steigung* |

Das Overlay backt kurz nach jeder Änderung erneut. Ohne [Live-Modus](live.md) baut Godot die Karte zuerst wie gezeigt, genau wie bei *In Godot bauen*.

> [!TIP]
> Wand- und Dachkanten tauchen als kleine zinnoberrote Inseln auf. Achte auf zinnoberroten Boden neben grünem Boden.

<div class="td-mcp" markdown>

## MCP

Über den [MCP-Server](../mcp.md) steuern Agenten und Skripte den Editor. Sie prüfen eine Karte mit dem Tool `walkability`, das die Inseln mit ihren Flächen und Grenzen in Karteneinheiten zurückgibt.

</div>
