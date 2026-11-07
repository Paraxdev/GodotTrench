---
source: b3ed6fe92fc841e610e206dd64cc4020bf717ee7
---

# Beschädigte Dateien { #damaged-files }

Eine beschädigte binäre Karte verliert nur ihre beschädigten Chunks, alles andere lässt sich weiterhin öffnen. Wenn du einen Leser schreibst, steht auf dieser Seite, was er tun muss, um dasselbe Verhalten zu erreichen.

Ein Leser läuft die [Chunks](container.md#chunks) ab Byte 12 entlang. Ein gültiger Chunk-Header hat den Sync-Marker, eine passende Header-Prüfsumme, eine Rohgröße von höchstens 1 GiB und einen druckbaren Tag. Wo es keinen gültigen Header gibt, sucht der Leser vorwärts nach dem nächsten Sync-Marker und macht dort weiter.

Ein Chunk vom Typ `HEAD`, `NODE` oder `END` wird verworfen, wenn sich seine Nutzlast nicht dekomprimieren lässt, seine Prüfsumme nicht stimmt, die Größe falsch ist oder sie sich nicht dekodieren lässt.

| Schaden | Ergebnis |
| --- | --- |
| Ein `NODE`-Chunk | Die darin gespeicherten Nodes gehen verloren |
| Der Parent eines Nodes ging verloren | Der Node zieht mit allem darunter in eine Ebene namens `Recovered` am Ende der Karte um. Die Ebene bekommt beim Laden eine frische ID |
| Verschachtelung tiefer als 60 Stufen | Wird ebenfalls nach `Recovered` verschoben, damit der Stack beim Lesen nicht ausgehen kann. Das gibt es nur bei einer präparierten Datei |
| Der `HEAD`-Chunk | Worldspawn-Eigenschaften und Editor-Zustand gehen verloren, und die Kartenversion kommt aus `END` |
| `HEAD` und `END` beide | Die Datei wird abgelehnt |
| Kein `END`-Chunk | Die Datei wurde abgeschnitten. Alles bis zum letzten vollständigen Chunk wird geladen |
| `END` zählt mehr, als gelesen wurde | Wird als Anzahl der fehlenden Nodes oder Chunks gemeldet |

Jeder Leser meldet jeden Verlust:

| Leser | Meldung |
| --- | --- |
| Editor | Öffnet, was sich lesen ließ, listet die Verluste in der Statusleiste auf und markiert die Karte als ungespeichert. Beschädigte Prefabs werden ebenfalls in der Statusleiste gemeldet |
| Godot-Addon | Baut, was sich lesen ließ, und gibt für jeden Verlust eine Warnung aus, mit dem Präfix `[TDM]` und dem Dateipfad |
| `trenchdot --dump` | Gibt aus, was sich lesen ließ, und schreibt die Verluste nach stderr |

Vor jedem Speichern kopiert der Editor die alte Datei nach `<map>.tdm.bak`, damit auch beim Speichern einer wiederhergestellten Karte das beschädigte Original daneben erhalten bleibt. Die neue Datei geht zuerst nach `<map>.tdm.tmp` und wird dann über die Karte umbenannt, ein Absturz mitten im Speichern hinterlässt also nie eine halb geschriebene Karte.
