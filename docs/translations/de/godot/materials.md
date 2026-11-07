---
source: 6f3b7a09923ac7b7e6e7e7d49ae11666c6d47d02
---

# Materialien { #materials }

Jede Fläche zeigt über ihren Pfad innerhalb des *Base Texture Dir* der Karteneinstellungen ohne Dateiendung auf ein Material, zum Beispiel `showcase/planks`.

Baut Godot die Karte, sucht es `showcase/planks.tres` in diesem Ordner oder stattdessen im *Base Material Dir*, falls du eines gesetzt hast, und nutzt sie unverändert. Keine `.tres`? Dann macht das Addon aus dem Bild ein einfaches Material. TrenchDot zeigt dieselbe `.tres` in der Vorschau, beide Seiten sehen also gleich aus.

Eine `.tres` brauchst du nur, wenn ein einfaches Bild nicht reicht, zum Beispiel um den Maßstab zu ändern, Leuchten hinzuzufügen oder es durchsichtig zu machen. Eine `.tres` mit nur einer `albedo_color` und ohne Textur funktioniert auch, der Editor zeichnet sie in dieser Farbe.

Neue Texturen und Materialien erscheinen im Tab Materialien binnen weniger Sekunden, ein Neuladen ist nicht nötig.

## Texturgröße { #texture-size }

Standardmäßig deckt ein Texturpixel eine Karteneinheit ab, eine Textur mit 64 Pixeln wiederholt sich also alle zwei Meter. Super für Pixelart, aber ein Foto mit 1024 Pixeln von einer zwei Meter breiten Wand würde sich über 32 Meter strecken.

Um das zu beheben, sag dem Material, wie viele Karteneinheiten eine Wiederholung abdeckt:

```
[resource]
albedo_texture = ExtResource("1_albedo")
metadata/texture_size = Vector2(64, 64)
```

(Du kannst es auch im Inspector von Godot unter *Metadata* hinzufügen, als `Vector2` oder als einzelne Zahl.)

Fahr im Tab Materialien mit der Maus über das Material, um es zu prüfen, der Tooltip zeigt die Bildgröße und wie viele Einheiten eine Wiederholung abdeckt.

![Der Tab Materialien mit dem Tooltip von night/shop_sign: 1024 x 256 px, 256 x 64 Einheiten je Wiederholung, emissiv x3](../assets/materials/tooltip-texture-size.png)

Jetzt kannst du ein schärferes Bild einsetzen, ohne die Karte zu ändern. Terrain-Ebenen ignorieren das, ihre Einstellung `tile` legt die Wiederholung schon fest.

## Leuchtende Materialien { #glowing-materials }

Ein Material leuchtet, wenn seine Datei `emission_enabled` setzt. Bei einer schwarzen Emissionsfarbe leuchtet die Emissionstextur genau so, wie sie gemalt ist:

```
[resource]
albedo_texture = ExtResource("1_albedo")
emission_enabled = true
emission = Color(0, 0, 0, 1)
emission_energy_multiplier = 2.5
emission_texture = ExtResource("2_emission")
```

Keine Materialdatei? Leg ein Bild, das wie das Albedo heißt und mit `_emission` ergänzt ist, daneben, `night/sign_emission.png` neben `night/sign.png`, und es wird zur Leuchtkarte.

Leuchtende Materialien bekommen im Tab Materialien ein gelbes Label *Leuchten*:

![Materialien des Ordners night im Tab Materialien, wobei apartment_lit, neon_red und shop_sign ein gelbes Label Leuchten tragen](../assets/materials/glow-tags.png)

Drück F3 für die Beleuchtete Vorschau, um sie in der 3D-Ansicht leuchten zu sehen. Der Bloom um sie herum zeigt sich nur in Godot, siehe [Nachtbeleuchtung](night-lighting.md).

## Transparenz { #transparency }

Setz `transparency` in der Materialdatei, wenn der Alphakanal der Textur Dinge durchscheinen lassen soll. Jeder Modus tauscht Qualität und Kosten anders:

| `transparency` | So funktioniert es | Nimm es für | Achtung |
| --- | --- | --- | --- |
| `1` blend | Mischt die Oberfläche mit dem, was dahinter ist | Glas, weiche Flecken, Rauch | Wirft keine Schatten |
| `2` scissor | Pixel werden entweder ganz gezeigt oder weggeschnitten | Harte Ausschnitte aus der Nähe, etwa Gitter und Blätter | Dünne Teile verschwinden in der Ferne |
| `3` hash | Verrauscht Pixel per Dithering, je niedriger das Alpha, desto weniger werden gezeigt | Dünne Ausschnitte aus der Ferne, etwa Drahtzäune | Weiche Kanten werden zu Rauschen |

Der Tooltip im Tab Materialien sagt dir, welchen Modus ein Material nutzt:

![Der Tab Materialien mit dem Tooltip von night/chainlink: Alpha-Hash, Normal-Map, zweiseitig](../assets/materials/tooltip-alpha-hash.png)
