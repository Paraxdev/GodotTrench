---
source: 078f68d4f428db30dc2dfcb8561a8991ada14154
---

# Texturen konvertieren { #converting-textures }

Godot kann Dateien wie `.vtf`, `.wad` oder `.wal` nicht laden, deshalb macht der Editor daraus PNGs, dazu eine `.tres`, wo ein einfaches Bild die Einstellungen nicht tragen kann. Jede landet im Texturordner des Projekts unter dem Materialnamen der Fläche, sodass die importierten Flächen sie sofort finden.

Beim Import einer Karte wird dir das für die Texturen angeboten, die sie nutzt, siehe [Hammer- und TrenchBroom-Karten importieren](importing.md). Um einen ganzen Ordner zu konvertieren, öffne zuerst das Godot-Projekt, dann:

1. Den Konverter öffnen

    Geh auf *Datei > Importieren > Texturen konvertieren (VTF/VMT, WAD, WAL)…*.

    ![Das Menü Datei mit geöffnetem Importieren und Texturen konvertieren (VTF/VMT, WAD, WAL)… unter dem Mauszeiger](../assets/importing-textures/file-import-convert-textures.png)

2. Den Ordner wählen

    Wähl den Ordner mit den Dateien `.vmt` und `.vtf`, `.wad` oder `.wal`. Fehlen der offenen Karte einige davon, fragt der Editor, ob nur **Die der Karte** oder **Alle** konvertiert werden sollen.

    Die Statusleiste sollte dann sagen, wie viele konvertiert wurden, etwa "Texturen: 12 converted", und sie erscheinen im Tab Materialien unter ihren Materialnamen.

    ![Der Tab Materialien mit der Suche nach castle, er zeigt brick_wall und stone_step aus einer importierten Karte, dazu der Tooltip castle/brick_wall, 1024 x 1024 px](../assets/importing-textures/converted-in-materials.png)

## Wo sie gefunden werden { #where-they-are-found }

Der Editor durchsucht die üblichen Orte, an denen jedes Spiel seine Texturen ablegt, sodass eine Karte, die mit den Ordnern ihres Spiels kopiert wurde, sie ohne Einrichtung findet.

| Quelle | Gesucht wird | Benannt nach |
| --- | --- | --- |
| Valve `.vmt` und `.vtf` | ein Ordner `materials` neben der Karte oder bis zu vier Ordner darüber | dem Pfad unterhalb von `materials`, etwa `brick/brickwall001a` |
| Quake- und Half-Life-`.wad` | dem worldspawn-Key `wad`, dann demselben Dateinamen in Ordnern in der Nähe | dem Texturnamen, etwa `{grate` |
| Quake-2-`.wal` | ein Ordner `textures` in der Nähe | dem Pfad unterhalb von `textures`, etwa `e1u1/floor1_1` |
| Lose `.png`, `.jpg`, `.tga` | ein Ordner `textures` in der Nähe, so wie ihn Qodot- und FuncGodot-Projekte mitbringen | dem Pfad unterhalb von `textures`, unverändert übernommen |

## Valve-Materialien { #valve-materials }

Eine Valve-`.vmt`-Datei beschreibt ein Material als Liste von Parametern. Diese gehen in das Godot-Material über, der Rest bleibt außen vor:

| Valve-Material | Godot-Material |
| --- | --- |
| `$basetexture` | Albedo, ihr Alpha entfällt, außer das Material ist durchsichtig |
| `$bumpmap`, oder `$normalmap` bei Wasser | Normal-Map, Grün von DirectX auf Godot umgedreht |
| `$translucent`, `$additive` | Alpha Blend bei `$translucent`, additive Blend bei `$additive` |
| `$alphatest`, `$alphatestreference` | Alpha Scissor bei diesem Schwellwert, standardmäßig 0,5 |
| `$selfillum`, `$selfillummask` | eine leuchtende Emissionstextur, maskiert durch das Alpha der Basistextur oder durch die Maske |
| `$nocull`, `UnlitGeneric` | zweiseitig bei `$nocull`, unshaded bei `UnlitGeneric` |
| `$color` | Albedo-Farbe |
| `$surfaceprop` | `metadata/surfaceprop` am Material |
| `patch` mit `include` | ein Patch nutzt ein anderes Material mit ein paar Änderungen wieder, wird also zum eingebundenen Material, wobei seine Änderungen aus `insert` und `replace` angewendet werden |

## Quake-Texturen { #quake-textures }

Texturen, die mit `{` beginnen, werden per Alpha Scissor ausgeschnitten, Himmel sind unshaded, und `*water` wird als `water` gespeichert. Animierte Texturen, auch VTFs mit mehreren Frames, behalten ihren ersten Frame.

## Himmel { #skies }

Beim Konvertieren der Texturen einer Karte wird auch ihr Himmel gesetzt, sodass die importierte Karte einen Himmel bekommt, der ihrem Original nahekommt. Das entfällt, wenn die Karte schon `sky_top_color` hat.

Eine Source-Karte mit ihrer `skyname`-Skybox in `materials/skybox` bekommt ein Panorama, `skybox/<skyname>_panorama.png` im Texturordner, gesetzt als `sky_panorama`. Andere Karten bekommen `sky_top_color` und `sky_horizon_color` aus der Textur in `sky_source`. Hat die Szene keine eigene WorldEnvironment (der Godot-Node, der Himmel, Nebel und Umgebungslicht einer Szene hält), baut `TrenchDotEnvironment` aus diesen Keys eine.

## VTF-Formate { #vtf-formats }

Der Konverter liest VTF-Versionen 7.0 bis 7.5 in diesen Pixelformaten: DXT1 (mit einem Bit Alpha), DXT3, DXT5, die 8-Bit-Reihenfolgen RGB und RGBA, I8, IA88, A8, RGB565, BGR565, BGRA4444, BGRA5551, BGRX5551, UV88, UVWQ8888, UVLX8888 und die 16-Bit-RGBA-Formate. Andere Formate und VTF 7.6 werden mit Nennung des Formats abgelehnt, du weißt also, welche Datei du von Hand konvertieren musst.

<div class="td-mcp" markdown>

## MCP { #mcp }

Über den [MCP-Server](../mcp.md) konvertierst du einen Ordner mit Texturen mit `map_file {op: convert_textures, path}`. Füg `only_missing` hinzu, um nur zu konvertieren, was der offenen Karte fehlt, und `overwrite`, um Dateien zu ersetzen, die schon im Projekt liegen.

</div>
