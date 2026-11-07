---
source: d6310713b2a9d65be25e506f0179132b8e0e10fd
---

# Kollision { #collision }

Jede Brush-Entität des Gameplay-Pakets und jede Modell-Entität kollidiert, ohne dass du etwas einstellst. Türen, Knöpfe, Aufzüge, Loren und Trigger bekommen eine konvexe Hülle pro Brush, und `prop_model`, `prop_physics`, `npc_walker` und Scatter-Props bekommen eine aus ihrem Modell gebaute Form. Du fasst die Schlüssel unten nur an, wenn der Standard nicht passt.

## Die Schlüssel { #the-keys }

Wähle die Entität aus und suche diese Zeilen im Inspektor. Fahre über eine Zeile, um zu lesen, was jede Auswahl bewirkt.

![Der Inspektor eines func_door mit geöffnetem Kollisionsmenü auf default, none, box, convex hull, convex decomposition und trimesh, darunter die Zeilen collision_layer und collision_mask](../../assets/collision/inspector.png)

| Schlüssel | Standard | Was er tut |
| --- | --- | --- |
| `collision` | default | Die Form: default, none, box, convex hull, convex decomposition oder trimesh |
| `collision_layer` | leer | Die Physikebenen, auf denen der Körper liegt, als Ebenennummern wie `1, 3`. `none` legt ihn auf keine Ebene |
| `collision_mask` | leer | Die Physikebenen, die der Körper erkennt, in derselben Form. `none` erkennt nichts |

Eine leere Ebene oder Maske behält den Standard: bei Brush-Entitäten Ebene und Maske der Entitätsdefinition, wobei Trigger *Trigger Collision Mask* aus den Karteneinstellungen übernehmen, und bei Modell-Entitäten die Projekteinstellungen unten.

## Eine Form wählen { #picking-a-shape }

| Form | Brush-Entitäten | Modelle |
| --- | --- | --- |
| default | Die Form der Klasse, eine konvexe Hülle pro Brush | Die Projekteinstellung, sofern nicht geändert eine konvexe Hülle pro Mesh. `prop_physics` nutzt eine Box aus `size` |
| none | Keine Kollision | Keine Kollision |
| box | Eine Box um alle Brushes | Eine Box um das Modell, die günstigste Form |
| convex hull | Eine Hülle pro Brush | Eine Hülle pro Mesh, gut für kantige Modelle |
| convex decomposition | Wie convex hull, ein Brush ist schon konvex | Mehrere Hüllen, die einem konkaven Modell wie einem Bogen folgen, langsamer zu bauen |
| trimesh | Jede Fläche, nur für statische Körper | Jedes Dreieck, exakt, aber am teuersten |

Trimesh funktioniert nur bei statischen Körpern. Türen, Knöpfe, Aufzüge und Loren sind animierbare Körper, die als statisch zählen, sie können es also nutzen. Trigger, `prop_physics` und `npc_walker` bauen stattdessen konvexe Hüllen, und ein Trigger mit trimesh meldet beim Bauen der Karte eine Warnung.

Ein `prop_model`, dessen Modell schon einen Kollisionskörper enthält, behält diesen Körper und baut keine eigene Form.

## Projektstandards { #project-defaults }

Modell-Entitäten mit leeren Schlüsseln folgen drei Einstellungen in Godots *Project Settings* unter *Trenchdot > Collision*. Sie erscheinen, sobald das TrenchDot-Plugin aktiviert ist.

| Einstellung | Standard | Was sie tut |
| --- | --- | --- |
| *Model Shape* | convex | Die Form, für die default bei `prop_model` und `npc_walker` steht |
| *Model Layer* | 1 | Die Ebene, auf der Modellkörper liegen |
| *Model Mask* | 1 | Die Ebenen, die Modellkörper erkennen |

Scatter-Sets wählen ihre Form in der eigenen Zeile *Collision* des Scatter-Sets, siehe [Scatter](../../editor/scatter.md).

## Deine eigenen Brush-Entitäten { #your-own-brush-entities }

Eine Brush-Entität, die du selbst definierst, kollidiert als der Körper, den ihre *Node Class* benennt. Bleibt *Node Class* leer, nimmt sie den Körper, den ihr Skript erweitert, und ohne Skript oder mit einem Skript, das `Node3D` erweitert, wird sie zu einem `StaticBody3D`. Nur ein Skript, das einen anderen Node wie einen `Marker3D` erweitert, lässt sie ohne Kollision, und der Build warnt davor. Siehe [Eigene Entitäten in GDScript](../custom-entities.md).

Der Editor-Viewport zeichnet keine Kollisionsformen. Godots *Debug > Visible Collision Shapes* zeigt sie, während das Spiel läuft, in der Live-Sitzung und in einem vollständigen Build gleichermaßen. Brush-Entitäten und `prop_model` speichern ihre Formen in der gebauten Szene, `prop_physics` und `npc_walker` bauen ihre bei jedem Laden der Szene, so wie sie ihr Modell laden. Ein `prop_model` oder `npc_walker`, dessen Modellszene schon einen Physikkörper enthält, behält diesen Körper und baut keine eigenen Formen.
