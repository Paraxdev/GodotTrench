---
source: 2de051fc5fd93ade8349b194b97a6c34a8399111
---

# Projizierte Decals { #projected-decals }

Ein projiziertes Decal ist eine Box, die ein Material auf alles wirft, was in ihr liegt. Anders als ein [normales Decal](decals.md), das ein flaches Blatt ist, folgt es allem, was es trifft. So kann sich ein Fleck über eine Stufe ergießen, um ein Rohr legen oder auf holprigem Boden liegen. Props und Modelle in der Box bekommen es ebenfalls ab.

Im gebauten Szenenbaum ist es ein Godot-`Decal`-Node, und es kostet beim Zeichnen mehr als ein normales Decal. Nimm es dort, wo ein flaches Blatt nicht reicht.

> [!WARNING]
> Projizierte Decals belasten die GPU stärker als [normale Decals](decals.md). Godot rechnet das Decal für jedes Pixel aus, das seine Box auf dem Bildschirm abdeckt, also bremsen viele Boxen oder große Boxen das Spiel aus, vor allem auf schwächeren Grafikkarten. Nimm auf einer ebenen Fläche ein normales Decal, halte die Box so klein, wie die Aufgabe es erlaubt, und heb dir projizierte Decals für Stellen auf, die den Umschlag wirklich brauchen.

## In dein Projekt holen { #adding-it-to-your-project }

Das projizierte Decal kommt mit dem [Gameplay-Entitätenpaket](../godot/project-setup.md#core-entities-and-the-gameplay-entities-pack) als `infodecal`, die Einrichtung besteht also darin, das Paket zu installieren.

1. Das Paket installieren oder aktualisieren

    Klick im Fenster *Dieses Projekt einrichten* auf **Gameplay-Entitäten installieren**, oder im Menü Gameplay auf **Gameplay-Entitäten aktualisieren**, wenn das Paket schon in deinem Projekt liegt.
    [Das Paket installieren und aktualisieren](../godot/project-setup.md#installing-and-updating-the-pack) führt dich Schritt für Schritt durch.

2. Prüfen, ob der Editor es sieht

    Im Tab **Entitäten** neben dem Inspektor solltest du eine `infodecal`-Karte sehen. Tipp `infodecal` ins Suchfeld oben im Tab, dann findest du sie schnell.

    ![Der Tab Entitäten mit der Suche nach infodecal, die Karte darunter mit dem Mauszeiger darauf und ihrem Tooltip Projected decal](../assets/projected-decals/entities-infodecal-card.png)

> [!NOTE]
> Falls du dir eine eigene `infodecal`-Definition gebaut hast, bevor das Paket eine mitbrachte, nimm sie nach dem Aktualisieren aus deiner FGD heraus. Sonst ist die Klasse doppelt definiert, und der Editor warnt davor.

> [!TIP]
> Du nutzt das Paket nicht? Kopier seine Definition,
> [`infodecal.tres`](https://github.com/Paraxdev/trenchdot_func/blob/main/gameplay_pack/definitions/infodecal.tres),
> in dein Projekt und füg sie *Entity Definitions* deiner eigenen FGD-Datei hinzu, siehe
> [Eigene Entitäten](../gameplay/custom-entities.md#making-it-placeable). Lass *Node Class* auf `Decal`, das Skript
> braucht das, und daran erkennt der Editor, dass er die Entität als Decal behandeln soll.

## Eins platzieren { #placing-one }

1. Die Karte auf eine Fläche ziehen

    Öffne den Tab **Entitäten** neben dem Inspektor und zieh die `infodecal`-Karte in eine Ansicht, so wie du jede Entität platzierst (siehe [Entitäten platzieren](../gameplay/placing.md#placing-entities)). Es landet dort, wo der Zeiger auf die erste feste Fläche trifft, und ist so gedreht, dass es in diese Fläche hineinprojiziert. Eine Box zeigt den Raum, den es abdeckt, zur Hälfte vor der Fläche und zur Hälfte dahinter.

    ![Ein infodecal an der Kante einer Backsteinstufe in der 3D-Ansicht, seine Box reicht über die Oberseite und an der Vorderseite hinunter, darin die Vorschau des Schildmaterials](../assets/projected-decals/decal-box-on-step.png)

2. Das Material wählen

    Gib im Inspektor bei `material` einen Materialnamen ein, denselben Namen, den auch eine Fläche verwendet, etwa `base/stain`. Farbe, Normal-Map und Leuchten werden übernommen.

    ![Der Inspektor mit ausgewähltem infodecal, night/shop_sign bei material und 128 64 64 bei size](../assets/projected-decals/decal-inspector-material.png)

3. Die Größe festlegen

    `size` nimmt drei Zahlen in Karteneinheiten: Breite, Tiefe und Höhe. Breite und Höhe sind der Fleck, den es auf der Fläche abdeckt, die Tiefe ist, wie weit die Box in die Fläche hinein und aus ihr heraus reicht. Standard ist 64 32 64, ein Quadrat von zwei Metern Kantenlänge, das einen halben Meter in jede Richtung reicht. Lässt du es auf dem Standard, nimmt der Fleck stattdessen die [Texturgröße](../godot/materials.md#texture-size) des Materials, falls es eine hat, im Editor wie in Godot.

4. (Optional) Es ausrichten

    Dreh es wie jede andere Entität. Es projiziert immer entlang seiner eigenen Abwärtsachse, der lokalen -Y, die beim Ablegen in die Fläche zeigte.

5. Die Karte in Godot bauen

    Das Material erscheint auf jeder Fläche in der Box, sobald Godot die Karte baut, siehe [Karten bauen](../godot/building.md).

    Der Editor zeigt in der Box nur eine flache Vorschau des Materials, die echte Projektion zeichnet Godot, das Schild biegt sich dabei über die Stufenkante und an ihrer Vorderseite hinunter.

    ![Dasselbe Schild in einem laufenden Godot-Spiel, über die Kante der Backsteinstufe und auf ihre Vorderseite gebogen](../assets/projected-decals/decal-in-game.png)

## Schlüssel { #keys }

| Schlüssel | Was er tut |
| --- | --- |
| `material` | Das Material, das projiziert wird, ein Name wie `base/stain` oder ein `res://`-Pfad. Seine Albedo-, Normal-, ORM- und Emissionstexturen und seine Albedo-Farbe werden verwendet |
| `texture` | Ein `res://`-Pfad zu einem Bild, das projiziert wird, wenn `material` leer ist |
| `normal_texture` | Ein `res://`-Pfad zu einer Normal-Map, zur Verwendung mit `texture`. Die eigene Normal-Map eines Materials ersetzt sie |
| `size` | Breite, Tiefe und Höhe der Box in Karteneinheiten, 64 32 64, solange du es nicht änderst |
| `modulate` | Eine Farbe, mit der das projizierte Bild multipliziert wird |

## Worauf du achten solltest { #things-to-watch-out-for }

| Situation | Was passiert |
| --- | --- |
| Das Projekt nutzt den Compatibility-Renderer | Godot zeichnet gar keine Decals. Stell *Rendering Method* unter *Project Settings > Rendering > Renderer* auf Forward+ oder Mobile um |
| Das Projekt nutzt den Mobile-Renderer | Auf einem einzelnen Mesh erscheinen nur 8 Decals, der Rest bleibt weg |
| Flächen, die entlang der Projektion verlaufen | Die Textur verschmiert zu Streifen, etwa an der Vorderseite einer Stufe unter einem Decal, das senkrecht nach unten zeigt. Kipp die Box, damit sie beide Flächen schräg trifft |
| Flächen nahe den Enden der Box | Das Decal blendet zum oberen und unteren Rand seiner Box hin aus, halte die Fläche also nahe der Mitte, wo das Ablegen sie hinsetzt |
| Etwas bewegt sich durch die Box | Es bekommt das Decal ebenfalls ab, auch ein Spieler, der vorbeiläuft. Mach die Box nicht tiefer, als die Fläche es braucht |
| Glas und Cutouts | Sie bleiben so durchsichtig wie vorher, ein Decal kann durchsichtige Stellen nicht füllen |
| Viele große Decals | Jedes macht für jedes Pixel, das es abdeckt, mehr Arbeit, sie bremsen das Spiel also dort aus, wo sie sich häufen |

<div class="td-mcp" markdown>

## MCP { #mcp }

Über den [MCP-Server](../mcp.md) platziert `run_action` mit `action: "place_entities"` und
`args: {"classnames": ["infodecal"], "at": [x, y, z], "normal": [x, y, z]}` es zur Fläche gedreht, genau wie ein Ziehen. Setz seine Schlüssel danach mit `update_entity` `properties`.

</div>
