---
source: 5d04cb3d9a7e005f4a371160c23d337e8ee707a7
---

# Gebackene Beleuchtung in Godot { #baked-lighting-in-godot }

Wenn du [eine Karte im Editor backst](../editor/light-baking.md), wird die Lightmap in der `.tdm` gespeichert. Godot backt selbst nichts, jeder Build macht nur aus dem gespeicherten Backergebnis Godots eigenen Lightmap-Node.

## Was der Build hinzufügt { #what-the-build-adds }

| Teil | Was es bekommt |
| --- | --- |
| Eine `LightmapGI` namens `baked_lighting` | Ein Kind der `FuncGodotMap`, das die Lightmap, die Sonnenschattenmaske und die Lichtsonden hält |
| Gebackene Meshes | Ihre Lightmap-Koordinaten als UV2 und ein statischer GI-Modus, gestreamte Chunks eingeschlossen |
| Lichter und die Sonne | Godots Backmodus aus ihrem `bake_mode` oder `sun_bake_mode`, siehe [Licht-Schlüssel](../editor/light-baking.md#light-keys) |

Steht die Sonne auf `bounce`, bleibt die Sonne in Echtzeit, und die gebackene Schattenmaske schattiert, was jenseits ihrer Schattenreichweite liegt.

Flächen, die du nach dem Backen geändert hast, fallen auf das durchschnittliche Licht der Karte zurück, sie sehen also plausibel, aber flach aus, bis du erneut backst.

## Lichtsonden { #light-probes }

Bewegte Objekte wie der Spieler, Props und Türen beziehen ihr Licht von den Lichtsonden um sie herum, sie nehmen also das gebackene Streulicht auf und fügen sich ein.

Das Backen setzt Sonden auf ein 2-Meter-Raster über die Box um die gebackene Geometrie und lässt die in Festkörpern weg. Wo das Raster zu grob ist, etwa in einem schmalen Korridor oder auf einer Treppe, füg eine Entität `light_probe` für eine weitere hinzu. Eine Karte bekommt höchstens etwa 1000 Sonden, von Hand gesetzte eingeschlossen, auf einer größeren Karte wird das Raster also weiter.

## Sonden-Entitäten { #probe-entities }

| Entität | Art | Godot-Node | Nimm sie für |
| --- | --- | --- | --- |
| `light_probe` | Punkt | `LightmapProbe` | Eine zusätzliche Lichtsonde für das Backen, siehe oben |
| `env_reflection_probe` | Brush-Volumen | `ReflectionProbe` in der Größe des Brushes | Reflexionen, die zu einem Raum passen, auf glänzenden Böden und Metall |
| `env_voxel_gi` | Brush-Volumen | `VoxelGI` in der Größe des Brushes | Streulicht in Echtzeit, das bewegten Lichtern und Objekten folgt |

Alle drei findest du im Tab Entitäten. Zieh `light_probe` in eine Ansicht. Zeichne bei den beiden Volumen eine Box um den Raum, klick dann mit rechts auf die Karte und wähl **Gewählte Brushes dazu machen**.

![Der Tab Entitäten, auf env_ gefiltert, mit rechts angeklickter Karte env_voxel_gi und hervorgehobenem Gewählte Brushes dazu machen](../assets/baked-lighting/entities-turn-brushes-into.png)

Der Brush gibt nur die Box, der Build entfernt ihn. Die Schlüssel aller drei stehen unter [Props, Lichter und Effekte](../gameplay/entities/effects.md#light_probe-env_reflection_probe-and-env_voxel_gi).

Ein `env_voxel_gi` wird bei jedem Bauen der Karte im Godot-Editor gebacken. Setz bei einem großen Volumen `bake_on_build` auf 0 und back es stattdessen von Hand mit dem Knopf **Backen** ("Bake" in einem englischen Godot) des `VoxelGI`-Nodes. Karten, die gebaut werden, während das Spiel läuft, überspringen dieses Backen.

## Einen anderen Lightmapper nutzen { #using-another-lightmapper }

Du willst lieber mit Godots eigener `LightmapGI` backen? Entferne bei der `FuncGodotMap` das Häkchen bei **Use Baked Lighting**, um das gespeicherte Backergebnis wegzulassen. Der Build fügt dann keinen Node `baked_lighting` hinzu und lässt die Backmodi der Lichter in Ruhe. Hak *Unwrap UV2* in den *Build Flags* an, damit die Meshes das UV2 bekommen, das ein Godot-Backen braucht.
