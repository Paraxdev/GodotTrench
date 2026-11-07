---
source: 0215c0eddc11970f765e58b4ce44056eebf2815e
---

# Große Karten streamen { #streaming-big-maps }

Große Außenkarten können verbergen, was weit weg ist, sodass die Karte nur kostet, was der Spieler sehen kann, statt alles, was sie enthält. Der Build sortiert die Meshes der Karte in Chunks auf einem Raster, und nur die Chunks nahe der Kamera werden gezeichnet.

## Einschalten { #turning-it-on }

1. Nichts auswählen

    Drück Escape, damit nichts ausgewählt ist. Der Inspector rechts zeigt dann *Karte (worldspawn)*.

2. `chunk_streaming` hinzufügen

    Tipp `chunk_streaming` unten ins Feld *Schlüssel*, `1` in *Wert* und klick auf **Hinzufügen**. (Steht es schon in der Liste, schalte es dort einfach ein.)

    ![Der Inspector von worldspawn mit chunk_streaming im Feld Schlüssel, 1 im Feld Wert und dem Knopf Hinzufügen unter dem Mauszeiger](../assets/streaming/worldspawn-add-key.png)

    Du solltest es zusammen mit den anderen worldspawn-Schlüsseln aufgelistet sehen:

    ![Der Inspector von worldspawn mit angehaktem chunk_streaming](../assets/streaming/worldspawn-chunk-streaming.png)

3. Die Karte speichern

    Der nächste Build fügt der Karte einen `TrenchDotStreamer` hinzu.

(Optional) Füg `chunk_size` und `load_radius` genauso hinzu:

| Schlüssel | Standard | Bedeutung |
| --- | --- | --- |
| `chunk_size` | 2048 | Chunk-Größe in Karteneinheiten, 64 m. Kleinere Chunks verbergen genauer, kosten aber mehr Prüfaufwand |
| `load_radius` | 8192 | Wie weit von der Kamera Chunks sichtbar bleiben, in Karteneinheiten, 256 m |

Streaming verbirgt nur Meshes. Kollision und Skripte laufen überall weiter, es spart also Renderkosten, nicht Physik oder Logik. Overlays, zur Laufzeit gespawnte Inhalte und alles an einer beweglichen Tür oder einem Zug werden nie verborgen.

Der Streamer folgt der aktuellen Kamera des Viewports oder einem `info_player_start`, solange es keine Kamera gibt. Um ihn an eine Kamera zu binden, setz nach dem Build per Code seinen `camera_path`, denn ein Neubau ersetzt den Streamer. Er läuft nur im Spiel, der Godot-Editor zeigt also immer die ganze Karte.

Während Chunks kommen und gehen, sendet der Streamer `area_loaded(key, bounds)` und `area_unloaded(key, bounds)`, damit du eigene Inhalte daneben streamen kannst:

```gdscript
streamer.area_loaded.connect(func(key, bounds): spawn_wildlife(bounds))
```

## Abschnitte streamen { #streaming-sections }

Du willst mehrere Karten als eine Welt ohne Ladebildschirme spielen? Teil sie in Abschnitte und verbinde sie mit Verbindungsräumen. Immer nur ein Abschnitt ist im Szenenbaum. Der nächste lädt im Hintergrund, während der Spieler erkundet, und wird eingetauscht, während der Spieler durch einen Verbinder geht, so ausgerichtet, dass der Raum um ihn herum exakt gleich bleibt. Dafür brauchst du das [Gameplay-Entitätenpaket](../gameplay/entities/README.md#installing-and-customizing).

1. Einen Verbindungsraum bauen

    Ein Korridor mit je einer Biegung auf jeder Seite, sodass man von seiner Mitte aus keines der beiden Enden sieht. Setz einen `info_landmark` in die Mitte und zwei `func_section_stream`-Volumen, je eins über jeder Hälfte, die sich in der Mitte treffen. Füg ihn der [Prefab-Bibliothek](../editor/prefab-library.md) hinzu, damit jede Kopie gleich ist.

2. Ihn dort einfügen, wo zwei Karten aufeinandertreffen

    Füg in jeder Karte, die an eine andere grenzt, den Verbinder dort ein, wo die beiden aufeinandertreffen, und benenne seine Landmarke, zum Beispiel `lm_hall_cellar`. Setz bei jeder Hälfte `section` auf den Abschnitt dieser Seite und `landmark` auf den Namen der Landmarke. Beide Karten bekommen dieselbe Kopie mit denselben Namen, gleich gedreht.

3. In jeder Abschnittskarte die Beleuchtung backen

    SDFGI bemerkt Geometrie, die ein Tausch hereinbringt, erst, wenn seine Kaskaden scrollen, ein eingetauschter Abschnitt bliebe also unbeleuchtet. Gebackene Beleuchtung kommt mit dem Abschnitt.

4. Pro Abschnitt eine Szene anlegen

    Eine Szene pro Abschnitt mit ihrer gebauten Karte, wie üblich.

5. Den Streamer hinzufügen

    Füg deiner Weltszene einen `TDSectionStreamer`-Node hinzu und setz seine `first_section` und seinen Spieler, siehe [TDSectionStreamer](../gameplay/entities/sections.md#tdsectionstreamer).

> [!TIP]
> Halte die Position einer Landmarke bei einem Vielfachen von 480 Karteneinheiten oder einer beliebigen Größe, in der sich alle Texturen des Verbinders wiederholen, damit die Texturen in beiden Kopien gleich ausgerichtet sind.
