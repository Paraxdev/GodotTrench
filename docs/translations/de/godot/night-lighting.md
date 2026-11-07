---
source: 3df4b9a4462dd78307ebb473881223aa02ea1321
---

# Nachtbeleuchtung { #night-lighting }

Für eine Nachtkarte drehst du das globale Licht herunter, damit die Lampen, die du platzierst, die Arbeit machen.

## Die Karte abdunkeln { #darken-the-map }

Drück Escape oder klick in einer Ansicht auf leeren Raum, damit nichts ausgewählt ist. Der Inspector rechts zeigt dann *Karte (worldspawn)*, die Einstellungen für die ganze Karte.

Setz die Schlüssel unten. Steht ein Schlüssel noch nicht in der Liste, tipp ihn unten ins Feld *Schlüssel* und seinen Wert daneben und klick auf **Hinzufügen**.

| Schlüssel | Standard | Was du für die Nacht tust |
| --- | --- | --- |
| `sun_energy`, `sun_color` | 1.1, `255 245 224` | Senk die Energie und wähl ein blasses Blau, dann wirkt die Sonne wie ein Mond |
| `ambient_energy`, `sky_energy` | 1.0, 1.0 | Senk beide, damit unbeleuchtete Bereiche und der Himmel dunkel werden |
| `glow_intensity` | 0 | Über 0 bekommen Lampen und [leuchtende Materialien](materials.md#glowing-materials) in Godot einen Bloom |
| `ssr` | 0 | 1 schaltet Bildschirmraum-Reflexionen ein, damit nasse Straßen und glänzende Böden die Lampen spiegeln |

Das nutzt das Showcase des nächtlichen Viertels:

![Der Inspector mit Karte (worldspawn) des nächtlichen Viertels: ambient_energy 0.8, glow_intensity 0.9, sky_energy 0.6, ein blasses Blau als sun_color und sun_energy 0.16](../assets/night-lighting/worldspawn-night.png)

## In der Beleuchteten Vorschau prüfen { #check-it-in-lit-preview }

Drück F3 oder klick in der Werkzeugleiste auf Beleuchtete Vorschau, um die Karte mit ihren Lichtern, dem Himmel und dem Nebel zu sehen.

![Die Werkzeugleiste mit dem Knopf Beleuchtete Vorschau unter dem Mauszeiger und seinem Tooltip, Schattierung: Beleuchtete Vorschau (F3)](../assets/night-lighting/toolbar-lit-preview.png)

Du solltest die Lampen, beleuchteten Fenster und leuchtenden Schilder vor einem dunklen Himmel hervorstechen sehen:

![Die Straße des nächtlichen Viertels in der Beleuchteten Vorschau: Straßenlaternen, beleuchtete Fenster und das leuchtende Schild Corner Shop unter einem dunklen Himmel](../assets/night-lighting/street-lit-preview.jpg)

Die Beleuchtete Vorschau zeigt alle Schlüssel oben außer Glow und Reflexionen, prüf die also in Godot. Um das Streulicht der Lampen zu bekommen, ohne in jedem Frame dafür zu zahlen, [back die Beleuchtung](../editor/light-baking.md).

## Lampenfassungen { #lamp-fixtures }

Eine Lampe ist eine Entität `light` oder `light_spot` plus die Geometrie der Lampe mit ihrer leuchtenden Birne. Gib dieser Geometrie einen targetname und trag ihn im Schlüssel `fixture` des Lichts ein (ein angehängtes `*` passt auf ein Präfix).

Die Geometrie folgt dann dem Licht ohne Verdrahtung: sichtbar, solange das Licht an ist, verborgen, solange es aus ist. Setz `fixture_off` auf `dark`, damit sie stattdessen sichtbar bleibt, aber ohne Leuchten.

![Der Inspector eines Lichts mit dem targetname alley_lamp, fixture alley_bulb und fixture_off auf dark (Emission aus)](../assets/night-lighting/light-fixture-keys.png)

Die anderen Licht-Schlüssel stehen unter [Props, Lichter und Effekte](../gameplay/entities/effects.md).

<div class="td-mcp" markdown>

## MCP

Die Showcase-Karten werden von [MCP-Skripten](../mcp-scripts.md) gebaut, die den Editor über seinen [MCP-Server](../mcp.md) steuern. `examples/mcp/night_district.json` und `examples/mcp/withered_city.json` sind vollständige Nachtkarten, die so gebaut wurden.

</div>
