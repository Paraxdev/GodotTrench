---
source: 70bacdb1e7200d36894611dfa449da2b79911911
---

# Shader-Warm-up { #shader-warm-up }

Godot kompiliert Shader erst, wenn es zum ersten Mal etwas zeichnet, deshalb ruckelt das erste Bild beim Spielen. Ein Warm-up erledigt diese Arbeit stattdessen hinter deinem Ladebildschirm.

Besteht dein Spiel nur aus der Karte, musst du nichts tun. Jede Karte wärmt sich selbst auf, nachdem sie im laufenden Spiel gebaut oder geladen wurde, und sendet `warmed_up`, wenn sie fertig ist.

Das Warm-up einer Karte deckt aber nur die Karte ab. Fügt dein Spiel eigene Inhalte wie einen Spieler, Gegner oder Waffen hinzu, ruckelt es bei deren erstem Auftritt trotzdem. Entferne in diesem Fall im Inspector von Godot das Häkchen bei **Warm Up Shaders** am `FuncGodotMap` und wärme alles in einem Durchgang auf, sobald alles drin ist:

```gdscript
loading.show()
await build_maps()
spawn_player()
await TrenchDotWarmUp.warm_shaders([get_tree().current_scene, preload("res://enemies/crawler.tscn")]).finished
loading.hide()
```

`warm_shaders` nimmt Nodes, Szenen, Meshes und Materialien. Gib auch Szenen mit, die noch nicht gespawnt sind, wie den Gegner oben, damit sie bereit sind, wenn sie auftauchen. Es zeichnet nichts auf den Bildschirm und führt keines ihrer Skripte aus. Verbinde sein Signal `progress(done, total)`, um einen Ladebalken anzutreiben.

In einem Testspiel mit fünf Karten sank das erste Bild beim Spielen von 360 ms auf 22 ms, bei einer halben Sekunde Warm-up.

> [!TIP]
> Schalte in deinen Export-Presets auch **Shader Baker** ein, damit Spieler beim ersten Start kein Shader-Kompilieren abwarten müssen.
