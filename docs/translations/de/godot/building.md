---
source: 755d17f52aebf0b32513b6e22a04abb368badeab
---

# Karten bauen { #building-maps }

TrenchDot speichert eine `.tdm`-Datei und schreibt nie selbst Godot-Szenen. In Godot liest der Node `FuncGodotMap` des Addons diese Datei und erzeugt die Meshes, die Kollision und die Entitäts-Nodes. Das Ergebnis wird mit deiner Szene gespeichert wie jeder von Hand gemachte Node, deshalb wird zur Laufzeit nichts neu gebaut, außer du verlangst es.

## Der Node FuncGodotMap { #the-funcgodotmap-node }

1. Den Node hinzufügen

    Drück im Godot-Editor in deiner Szene Ctrl+A, such nach `FuncGodotMap` und füg ihn hinzu.

2. Auf deine Karte zeigen

    Setz bei gewähltem Node *Local Map File* im Inspector von Godot auf die `.tdm`, die du gespeichert hast.

3. Bauen

    Klick oben im Inspector auf **Build Map**. Du solltest die Karte im Viewport von Godot sehen, und der Node bekommt Kinder für die Meshes und Entitäten. Speichere die Szene mit Ctrl+S, um sie zu behalten. **Clear Map** entfernt sie wieder.

Das sind die Einstellungen, die du im Inspector siehst:

| Eigenschaft | Was sie tut |
| --- | --- |
| Local Map File | Der `res://`-Pfad der `.tdm`, die gebaut werden soll. Das ist die normale Wahl. |
| Global Map File | Eine Kartendatei außerhalb des Projekts. Sind beide gesetzt, gewinnt diese. |
| Auto Rebuild On Save | Baut den Node neu, sobald TrenchDot diese Karte speichert, solange ihre Szene der aktuelle Tab im Godot-Editor ist, siehe [Neu bauen beim Speichern](live.md#rebuild-on-save). Standardmäßig an. |
| Map Settings | Woher Texturen, Materialien und Entitätsdefinitionen kommen und wie groß eine Karteneinheit ist. Bleibt es leer, gilt die Projekteinstellung `func_godot/default_map_settings`, siehe [Projekteinrichtung](project-setup.md). Ein Karten-Node, der mit einem älteren Addon hinzugefügt wurde, hat hier die Standard-Karteneinstellungen des Addons gespeichert, und sein Build warnt in der Ausgabe von Godot davor, leere das Feld, um der Projekteinstellung zu folgen. |
| Use Baked Lighting | Beleuchtet die Karte mit der [im Editor gebackenen](baked-lighting.md) Beleuchtung. Standardmäßig an, nimm den Haken heraus, um einen anderen Lightmapper zu nutzen. |
| Warm Up Shaders | Kompiliert die Shader der Karte beim Laden, damit die ersten Sekunden des Spiels nicht ruckeln. Siehe [Shader-Warm-up](warm-up.md). |

Das Addon baut nur `.tdm`-Dateien. Um eine Quake-`.map` oder eine Hammer-`.vmf` zu nutzen, öffne sie in TrenchDot mit *Datei > Importieren* und speichere sie zuerst als `.tdm`.

> [!WARNING]
> Ein Build löscht jedes Kind des Karten-Nodes außer den [Overlays](overlays.md). Halte deine eigenen Nodes neben dem Karten-Node oder in einem Overlay.

## Drei Wege zu bauen { #three-ways-to-build }

| Auslöser | Was gebaut wird | Wann du ihn nimmst |
| --- | --- | --- |
| In TrenchDot speichern | Jeder `FuncGodotMap`, der diese Datei nutzt, in der Szene, die im Godot-Editor offen ist | Die tägliche Schleife, in TrenchDot speichern und das Ergebnis in Godot ansehen |
| *Godot > In Godot bauen* | Die Karte, wie sie gerade in TrenchDot ist, gespeichert oder nicht, ohne die Datei zu schreiben | Um eine ungespeicherte Änderung in Godot auszuprobieren, ohne dich festzulegen |
| **Build Map** in Godot | Die gespeicherte Datei | Wenn TrenchDot geschlossen ist, oder um eine Szene neu zu bauen, die du gerade geöffnet hast |

Die ersten beiden brauchen einen Godot-Editor mit diesem Projekt offen, und sie berühren nur die Szene, die dort offen ist. Ein Neubau markiert diese Szene als geändert, speichere sie also in Godot, um das Ergebnis zu behalten.

Solange Godot das Projekt nicht offen hat, bleibt *In Godot bauen* ausgegraut. Es braucht außerdem, dass die Karte einmal im Projekt gespeichert wurde, damit Godot weiß, welcher `FuncGodotMap` sie nutzt.

![Das Menü Godot mit In Godot bauen unter dem Mauszeiger](../assets/building/menu-build-in-godot.png)

## Aus Code bauen { #building-from-code }

Du kannst eine Karte auch bauen, während das Spiel läuft, zum Beispiel um einen Level zu laden, der in einem Menü gewählt wurde:

```gdscript
var map := FuncGodotMap.new()
map.map_settings = load("res://demo/demo_map_settings.tres")
map.local_map_file = "res://demo/maps/scripted_scene.tdm"
add_child(map)
map.build()
```

Der Node sendet `build_complete`, wenn er fertig ist, oder `build_failed`, wenn die Kartendatei fehlt oder keine `.tdm` ist.

## Umgebung und Sonne { #environment-and-sun }

Die Keys für Himmel, Nebel und Sonne an worldspawn bauen eine `WorldEnvironment` und ein Licht namens `sun` in die Karte, sodass eine Karte ohne jede Einrichtung in Godot beleuchtet aussieht. Drück Escape, damit nichts gewählt ist, der Inspektor zeigt dann *Karte (worldspawn)* mit diesen Keys. Sein Key `environment` bestimmt, was gebaut wird. Ein Key, den du nicht setzt, bleibt aus der Karte heraus, eine Karte ohne einen davon baut also gar keine Umgebung. Wenn ein Projekt, das mit einem älteren Addon gemacht wurde, die Keys nicht auflistet, aktualisiere das Addon und [exportiere die Spielkonfiguration](project-setup.md#the-game-config) erneut.

| `environment` | Baut |
| --- | --- |
| `1` | Die Umgebung und die Sonne. Das ist der Standard. |
| `sun_only` | Nur die Sonne, für ein Spiel, das eine eigene `WorldEnvironment` mitbringt |
| `0` oder `none` | Nichts |

Hat die Szene schon eine eigene `WorldEnvironment` oder `DirectionalLight3D`, behält die Karte sie und fügt keine hinzu, ein Spiel mit eigenem Look gewinnt also immer. Teilen sich mehrere Karten eine Szene, beleuchtet die zuerst gebaute sie.

## Build-Bericht { #build-report }

`build()` gibt ein Dictionary zurück, das auch in `build_report` bleibt. Es sagt dir, wie lange jeder Schritt gedauert hat, wie viele Entitäten jeder Klasse gebaut wurden und wie viele Vertices die Karte hat. Am nützlichsten ist die Liste dessen, was der Build nicht finden konnte, meist ein Tippfehler oder eine Datei, die umbenannt oder nie eingecheckt wurde:

| Key | Was er auflistet | Was du im Spiel siehst |
| --- | --- | --- |
| `missing_classes` | Entitätsklassen, die keine Entitätsdefinition beschreibt | Eine Brush-Entität behält ihr Mesh, hat aber kein Skript und keine Kollision, eine Punkt-Entität ist ein leerer `Marker3D`, siehe [Entitäten, die nichts tun](entities-troubleshooting.md) |
| `missing_models` | Modellpfade, die nicht existieren, mit den Entitäten, die jeden nutzen | Das Prop hat kein Modell |
| `missing_materials` | Flächentexturen, die weder eine Materialdatei noch ein Bild haben | Die Flächen zeigen die Platzhaltertextur |
| `unknown_tool_textures` | Werkzeugtexturen wie `special/clip`, die die Karteneinstellungen anders nennen | Die Flächen zeichnen weiterhin nichts, aber ein Origin-Brush setzt den Ursprung seiner Entität nicht |
| `unresolved_targets` | Outputs und Target-Keys, deren Ziel zu keinem targetname in der Karte oder ihren Overlays passt | Der Output feuert ins Leere |

Um ihn nach jedem Build auszugeben, hak am Node unter *Build Flags* *Print Build Report* an, oder füg im Code `FuncGodotMap.BuildFlags.PRINT_REPORT` zu `build_flags` hinzu. Ein Test oder eine CI-Prüfung kann stattdessen das Dictionary lesen:

```gdscript
var report := map.build()
if not report["missing_models"].is_empty():
	push_error("missing models: %s" % [report["missing_models"]])
```

Jeder Build gibt außerdem in der Ausgabe von Godot eine Warnung für das aus, was er nicht finden konnte: eine für alle fehlenden Klassen und eine für jedes fehlende Modell, Material oder jede fehlende Werkzeugtextur, egal wie viele Props den Pfad teilen. Nach einem Speichern listet TrenchDot dieselben Warnungen in seinem Panel Probleme.

> [!NOTE]
> Overlays, die du nach `build()` hinzufügst, sind nicht da, wenn der Bericht nach Targets sucht, ihre targetnames tauchen also unter `unresolved_targets` auf.

## Beschädigte Karten { #damaged-maps }

Eine beschädigte Karte wird weiterhin aus den Teilen gebaut, die sich lesen ließen, mit einer `[TDM]`-Warnung für jeden Teil, der es nicht tat. Siehe [das .tdm-Kartenformat](../format/recovery.md).
