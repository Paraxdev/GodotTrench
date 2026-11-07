---
source: 70bacdb1e7200d36894611dfa449da2b79911911
---

# Precalentamiento de shaders { #shader-warm-up }

Godot compila los shaders la primera vez que dibuja algo, así que el primer fotograma del juego se atasca. Un precalentamiento hace ese trabajo detrás de tu pantalla de carga.

Si tu juego es solo el mapa, no tienes que hacer nada. Cada mapa se precalienta solo después de construirse o cargarse en el juego en marcha, y emite `warmed_up` cuando termina.

El precalentamiento de un mapa solo cubre el mapa, eso sí. Si tu juego añade contenido propio como un jugador, enemigos o armas, su primera aparición sigue atascándose. En ese caso desmarca **Warm Up Shaders** en el `FuncGodotMap`, en el Inspector de Godot, y precalienta todo de una vez cuando ya esté todo dentro:

```gdscript
loading.show()
await build_maps()
spawn_player()
await TrenchDotWarmUp.warm_shaders([get_tree().current_scene, preload("res://enemies/crawler.tscn")]).finished
loading.hide()
```

`warm_shaders` acepta nodos, escenas, mallas y materiales. Pásale escenas que aún no estén instanciadas, como el enemigo de arriba, para que estén listas cuando aparezcan. No dibuja nada en pantalla y no ejecuta ninguno de sus scripts. Conecta su señal `progress(done, total)` para mover una barra de carga.

En un juego de prueba con cinco mapas, bajó el primer fotograma del juego de 360 ms a 22 ms, a cambio de medio segundo de precalentamiento.

> [!TIP]
> Activa también **Shader Baker** en tus presets de exportación, para que los jugadores no compilen shaders en el primer arranque.
