---
source: 80f0288013e71dba5bd67c791a5e3e3272a71653
---

# Props, Lichter und Effekte { #props-lights-and-effects }

> [!NOTE]
> Diese Entitäten gehören zum Gameplay-Entitätenpaket, siehe
> [Installieren und anpassen](README.md#installing-and-customizing). `light`, `light_spot` und die Sonden
> (`light_probe`, `env_reflection_probe` und `env_voxel_gi`) gehören zum Kern des Addons und funktionieren auch ohne es.

## prop_physics { #prop_physics }

Eine Kiste oder ein Fass, das geworfen, beschädigt und zerbrochen werden kann. Es zerbricht bei 0 Gesundheit, bei einem Aufprall schneller als `impact_speed`, bei `smash` oder `fuse` Sekunden nach `ignite`. Ein explosives Prop sprengt, wenn es zerbricht.

| Schlüssel | Standard | Was er tut |
| --- | --- | --- |
| `model` | | Die Szene, wie es aussieht, nur fürs Aussehen |
| `size` | 16 16 16 | Halbe Ausdehnung seiner Box-Kollision, der Standard ist also einen Meter breit |
| `collision` | default | default und box nutzen `size`, convex hull und convex decomposition folgen dem Modell, siehe [Kollision](collision.md) |
| `collision_layer`, `collision_mask` | leer | Ebenennummern wie `1, 3`, leer behält den Projektstandard |
| `mass` | 5.0 | Masse in Kilogramm |
| `health` | 30 | Schaden, den es verträgt, bevor es zerbricht |
| `explosive` | 0 | Sprengt, wenn es zerbricht |
| `explosion_radius`, `explosion_damage` | 192, 40 | Größe und Stärke dieser Explosion |
| `fuse` | 0.6 | Sekunden, die ein entzündetes Prop brennt, bevor es zerbricht |
| `impact_speed` | 0 | Zerbricht, wenn es mit mehr als dieser Geschwindigkeit in m/s gegen etwas prallt. 0 zerbricht nie durch Aufprall |
| `debris_scene` | | Eine Szene, die dort gespawnt wird, wo es zerbrochen ist, etwa Bretter |

* **Inputs:** `smash`, `ignite(activator)`, `push(direction)`, `take_damage(amount, source)`
* **Outputs:** `damaged(hp)` mit der verbleibenden Gesundheit, `broken`

`push` nimmt eine Geschwindigkeit in Karteneinheiten pro Sekunde. `broken` löst nach der Explosion aus, was es anstößt, sieht den Schaden also schon.

Das Prop baut sein Modell und seine Kollisionsbox bei jedem Laden der Szene. Godots Editor zeigt sie als Vorschau, die nicht mit der Szene gespeichert wird.

## prop_model { #prop_model }

Ein statisches Modell mit optionaler Kollision, für Möbel, Felsen und andere Props, die sich nicht bewegen sollen.

* **Schlüssel:** `model` (.bbmodel, .glb, .gltf oder .tscn), `model_node`, `collision`, `collision_layer`,
  `collision_mask`, `scale` 1.0

`collision` default gibt jedem Mesh eine konvexe Hülle, gut für kantige Formen. Wähle convex decomposition oder trimesh für Formen, durch die oder auf denen der Spieler läuft, etwa einen Bogen, oder box für die günstigste Form, siehe [Kollision](collision.md). Kollision wird nur erzeugt, wenn das Modell keine eigene hat. Modelle, die mit *Modell-Prop* platziert wurden, und Modelle in Prefabs sind prop_model-Entitäten, sie kollidieren also genauso. Für eine Datei, die mehrere Varianten nebeneinander enthält, behält `model_node` nur den benannten Node und verschiebt ihn an den Ursprung des Props.

## env_explosion { #env_explosion }

Eine geskriptete Explosion, etwa eine Wandladung. Bei `explode` ruft sie `damage_method` an jedem Node innerhalb von `radius` mit `damage * (1 - distance / radius)` auf, stößt Starrkörper weg und spawnt `effect_scene` für Blitz und Knall.

* **Inputs:** `explode(activator)`
* **Outputs:** `exploded`
* **Schlüssel:** `radius` 256, `damage` 50, `force` 8 (m/s im Zentrum, egal bei welcher Masse), `damage_method` take_damage,
  `effect_scene`

Die Entfernung wird bis zum Ursprung jedes Nodes gemessen, ohne Sichtlinienprüfung, sie verletzt also durch Wände hindurch.

## game_text { #game_text }

Zeigt eine Textzeile in der Welt, im HUD oder in beidem, für Hinweise, Schilder oder Untertitel von Zwischensequenzen.

| Schlüssel | Standard | Was er tut |
| --- | --- | --- |
| `text` | | Der Text, der angezeigt wird |
| `place` | world | `world` schwebt an der Entität, `hud` zeigt ihn auf dem Bildschirm, `both` macht beides |
| `start_visible` | 0 | Ab dem Laden der Karte sichtbar |
| `duration` | 0 | Blendet ihn nach so vielen Sekunden aus, 0 lässt ihn stehen, bis `hide` kommt |
| `text_color` | 255 255 255 | Textfarbe |
| `world_size` | 0.01 | Größe des Labels in der Welt in Metern pro Pixel |
| `billboard` | 1 | Das Label in der Welt schaut zur Kamera und wird durch Wände gezeichnet. Schalte es für Schilder aus, sie behalten dann die Winkel der Entität und verschwinden hinter Wänden |

* **Inputs:** `show`, `hide`, `set_text(text)`, `flash(seconds)`
* **Outputs:** `shown`, `hidden`

`show` und `hide` schalten das Label, nicht den Node. `flash` zeigt es für die Sekunden seines Parameters, sonst `duration`, sonst 2.

## light und light_spot { #light-and-light_spot }

Omni- und Spotlichter, die Inputs ein- und ausschalten, für Lichtschalter, Alarme und flackernde Lampen. `switched` löst nur aus, wenn sich der Zustand wirklich ändert. Sie gehören zum Kern des Addons, jedes Projekt hat sie also.

* **Inputs:** `turn_on`, `turn_off`, `toggle`
* **Outputs:** `switched(on)`
* **Schlüssel:** `light_energy` 1.0 (1.5 bei Spots), `light_color` 255 255 255, `start_on` 1, `shadows` 0 (kostet mehr
  beim Rendern), `fixture`, `fixture_off` hide, und `omni_range` 10, oder `spot_range` 15 und `spot_angle` 35 (der halbe
  Winkel des Kegels in Grad), `bake_mode` auto, `light_size` 0, `light_indirect_energy` 1, und `omni_attenuation` 1, oder
  `spot_attenuation` 1 und `spot_angle_attenuation` 1. Diese letzten Schlüssel setzen die gleichnamigen Godot-Lichteigenschaften, und das Backen nutzt sie ebenfalls

**Fixtures** halten das Lampenmodell im Gleichschritt mit dem Licht. Gib der leuchtenden Geometrie der Lampe einen targetname und trage ihn in `fixture` ein, ein abschließendes `*` passt auf ein Präfix, sodass `hall_tube_*` jede Röhre abdeckt. Sie wird angezeigt, solange das Licht an ist, und ausgeblendet, solange es aus ist, ganz ohne Verdrahtung. Mit `fixture_off` auf `dark` bleibt sie sichtbar, ihre Emission ist aber aus. `fixture` nimmt auch die anderen [Ziel](../parameters.md#targets)-Formen, `@lamps` für eine Node-Gruppe und `/root/...` für einen Node-Pfad. Der Editor zeigt jede Form außer dem Node-Pfad in der Vorschau, den nur Godot auflösen kann.

`bake_mode` und `light_size` bestimmen, wie [Beleuchtung backen](../../editor/light-baking.md#light-keys) das Licht behandelt. Mit `auto` bleibt ein Licht, das I/O schalten kann, also eines mit targetname oder `start_on` 0, in Echtzeit.

## env_sound { #env_sound }

Ein Sound, der von einem Punkt im Level abgespielt wird, für Alarme, Maschinen oder einen Krach, der mit einem Ereignis verdrahtet ist.

* **Inputs:** `play`, `stop`, `toggle`
* **Outputs:** `finished`
* **Schlüssel:** `sound` (eine `res://`-Audiodatei), `volume_db` 0, `max_distance` 0 (keine Grenze), `autoplay_on` 0 (spielt ab
  dem Laden der Karte), `loop_sound` 0

## env_particles { #env_particles }

Ein Partikeleffekt für Feuer, Rauch, Funken, Staub und Regen, mit eingebauten Looks, sodass du keinen selbst bauen musst. `burst` sendet einen Schub aus, ein laufender Effekt läuft dann weiter.

* **Inputs:** `start`, `stop`, `toggle`, `burst`
* **Outputs:** `finished`
* **Schlüssel:** `amount` 32 (gleichzeitig lebende Partikel), `lifetime` 2.0, `one_shot` 0, `start_emitting` 0, `effect` rise,
  `color` 255 255 255, `size` 0, `blend` auto, `area` 10 1 10

`effect` ist `rise` (leuchtender Staub oder Glut), `fire`, `smoke`, `sparks` oder `rain` (fällt aus einer Box `area`, so viele Meter breit, hoch und tief). `color` färbt ihn ein, und Weiß behält die eigenen Farben des Effekts. `size` setzt die Partikelgröße in Metern, 0 nutzt die des Effekts. `blend` ist `add` für Leuchten oder `mix` für Rauch. Die Standardwerte jedes Effekts sehen von sich aus richtig aus, beginne also nur mit `effect`.

## infodecal { #infodecal }

Ein projiziertes Decal, eine Box, die ein Material auf alles darin projiziert, für Flecken, die sich über Stufen, Rohre oder holprigen Boden legen. [Projizierte Decals](../../editor/projected-decals.md) zeigt dir, wie du eines platzierst und worauf du achten musst.

* **Schlüssel:** `material` (ein Name wie `base/stain` oder ein `res://`-Pfad), `texture`, `normal_texture`, `size` 64 32 64,
  `modulate` 255 255 255

`material` bringt seine Texturen und Farbe mit, `texture` und `normal_texture` sind `res://`-Bilder, die ohne eines genutzt werden. `size` ist Breite, Tiefe und Höhe der Box in Karteneinheiten. Bleibt es bei 64 32 64, bestimmt stattdessen die [Texturgröße](../../godot/materials.md#texture-size) des Materials Breite und Höhe. `modulate` färbt das Decal ein.

## light_probe, env_reflection_probe und env_voxel_gi { #light_probe-env_reflection_probe-and-env_voxel_gi }

Sonden, die bewegte Objekte beleuchten und spiegeln, siehe [Gebackene Beleuchtung in Godot](../../godot/baked-lighting.md). Keine hat Inputs oder Outputs.

| Entität | Was sie ist | Schlüssel |
| --- | --- | --- |
| `light_probe` | Ein Punkt, der dem Backen eine Lichtsonde hinzufügt, für Stellen, die das Sondenraster verfehlt | Keine |
| `env_reflection_probe` | Ein Brush-Volumen, das zu einer `ReflectionProbe` in der Größe des Brushes wird | `intensity` 1.0, `update` once, `box_projection` 1, `interior` 0, `max_distance` 0 |
| `env_voxel_gi` | Ein Brush-Volumen, das zu einem `VoxelGI` in der Größe des Brushes wird, bei jedem Build im Godot-Editor gebacken | `subdiv` 128, `energy` 1.0, `bake_on_build` 1 |

`update` auf `always` nimmt eine Reflexionssonde jeden Frame für bewegte Objekte auf, was viel kostet. `box_projection` richtet Spiegelungen an den Wänden eines Raums aus, `interior` lässt den Himmel für Räume ohne Fenster weg, und `max_distance` gibt an, wie viele Meter die Sonde über ihre Box hinaus sieht. `subdiv` sind 64, 128, 256 oder 512 Voxel entlang der längsten Seite, mehr ist schärfer und braucht länger zum Backen.
