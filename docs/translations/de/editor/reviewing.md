---
source: 77812fdfe3757a8be2d8e1be3d7fbd73a1ff678a
---

# Änderungen prüfen { #reviewing-changes }

Halte deine Karten in git, dann siehst du, was sich zwischen zwei Versionen geändert hat, genau wie beim Review von Code. Für Änderungen innerhalb der aktuellen Sitzung listet das [Panel Verlauf](organizing.md#history) jede Bearbeitung auf und macht bis zu jeder beliebigen davon alles rückgängig.

## Karten in git { #maps-in-git }

`.tdm`-Dateien sind binär, aber git kann dir trotzdem lesbare Diffs zeigen. Es bittet den Editor, jede Version als JSON auszugeben, und vergleicht diese. Das richtest du einmal ein:

1. Füg diese Zeile in die `.gitattributes` deines Projekts ein:

    ```
    *.tdm binary diff=tdm
    ```

2. Sag git einmal pro Clone, wie es eine Karte ausgibt:

    ```sh
    git config diff.tdm.textconv "trenchdot --dump"
    ```

3. (Optional) Halte Backups und Autosaves mit diesen Zeilen in der `.gitignore` aus dem Repository heraus:

    ```
    *.tdm.bak
    *.tdm.autosave
    ```

`git diff` und `git log -p` sollten Kartenänderungen jetzt als JSON zeigen, statt nur zu sagen, dass sich die Binärdateien unterscheiden.

> [!WARNING]
> `trenchdot` muss in deinem `PATH` liegen, sonst schlägt `git diff` bei Karten fehl.

Um dieses JSON als Datei zu bekommen, schreibt `trenchdot --to-json map.tdm map.json` es, und `trenchdot --to-tdm map.json
map.tdm` macht daraus wieder eine binäre Karte. Siehe [Kommandozeile](../command-line.md).

<div class="td-mcp" markdown>

## MCP { #mcp }

Die MCP-Review-Tools zeigen dir, was ein KI-Agent mit deiner Karte gemacht hat.

### Bearbeitungen durch einen KI-Agenten { #edits-made-by-an-ai-agent }

Wenn ein Agent deine Karte über [MCP](../mcp.md) bearbeitet, ist jeder Tool-Aufruf ein Undo-Schritt namens `MCP: ...` im [Panel Verlauf](organizing.md#history). Auch ein ganzes `run_script` ist ein Schritt, etwa `MCP: Add crates, 6 steps`. Mach diesen Schritt rückgängig, um die Arbeit des Agenten zurückzunehmen.

![Das Panel Verlauf mit MCP: Add crates, 6 steps ganz oben, darunter von Hand gemachte Bearbeitungen wie Umbenennen, Gruppieren und Aushöhlen](../assets/reviewing/history-mcp-script.png)

| Du willst sehen | Frag den Agenten nach |
| --- | --- |
| Was sich seit dem letzten Speichern geändert hat | `changes_since`, das hinzugefügte, entfernte, verschobene oder anderweitig bearbeitete Nodes listet |
| Was sein letzter Aufruf oder sein letztes Skript geändert hat | `changes_since` mit `undo_steps: 1` |
| Was sich seit einem git-Commit geändert hat | `changes_since` mit `file` auf dieser Version, geschrieben mit `git show HEAD:maps/level.tdm > old.tdm` |

> [!TIP]
> Bitte den Agenten, einen Auftrag mit mehreren Schritten in ein `run_script` mit einem `label` zu packen, damit sich der ganze Auftrag in einem Schritt unter einem Namen rückgängig machen lässt, den du erkennst.

</div>
