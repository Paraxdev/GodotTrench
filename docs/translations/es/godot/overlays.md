---
source: 5a39bcf2beab2f567018f04963913fd06af5df16
---

# Overlays { #overlays }

Cada construcción reemplaza lo que generó el mapa, así que todo lo que añadas a mano bajo el nodo del mapa desaparece en el siguiente guardado. Un overlay es donde va el contenido de Godot que pertenece al mapa y tiene que sobrevivir a cada reconstrucción: una máquina expendedora con su propio script, un efecto de partículas ajustado en Godot, una lámpara con un AnimationPlayer.

![El patio del distrito nocturno: guirnaldas de luces, luciérnagas y un cartel del overlay, lámparas del mapa](../assets/overlays/night-district-courtyard.jpg)

## Añadir uno { #adding-one }

1. Añade el nodo overlay

    En Godot, añade un nodo `TrenchDotOverlay` como hijo directo del `FuncGodotMap`.

2. Pon tus nodos debajo

    Las construcciones, el modo en vivo y la recarga en caliente nunca lo tocan, así que sus nodos conservan su estado y sus conexiones de señales y se guardan con tu escena.

(Para un solo nodo puedes saltarte el overlay: un hijo directo del mapa en el grupo `trenchdot_keep` también sobrevive a las construcciones. Un overlay en otra parte de la escena puede apuntar a su mapa con *Map Path*, y entonces sigue la transformación del mapa.)

El contenido del overlay usa el espacio local del mapa en metros. A 32 unidades por metro, un hijo en `(2, 0, -3)` queda donde TrenchDot muestra `(64, 0, -96)`.

## Un overlay en su propia escena { #an-overlay-in-its-own-scene }

Un overlay también puede ser una escena propia. Compensa cuando varias escenas usan el mismo mapa y deben compartir el overlay, cuando una persona trabaja en la escena del nivel y otra en el overlay, o cuando un script genera la escena del nivel, como en la demo.

1. Haz una escena cuya raíz sea un `TrenchDotOverlay`.
2. Arrastra el `.tscn` al nodo `FuncGodotMap`, para que se instancie como hijo directo del mapa.

Godot muestra una escena instanciada como un solo nodo, sin flecha para expandirlo. Para llegar a su contenido:

| Quieres | Haz esto |
| --- | --- |
| Editar el overlay en todos los sitios donde se usa | Haz clic en el icono *Open in Editor* junto al nodo en el dock Scene, o doble clic en el `.tscn` en el dock FileSystem |
| Ver o cambiar sus nodos solo en esta escena | Clic derecho en el nodo y activa *Editable Children* |

Abierta por sí sola, la escena del overlay muestra una advertencia en su raíz porque no está bajo un mapa. Es lo esperado. Las cajas discontinuas de TrenchDot se ponen al día la próxima vez que se construya o guarde la escena con el mapa, ya que solo allí el overlay conoce su mapa.

## Cablear overlays { #wiring-overlays }

Los nodos del overlay y las entidades del mapa pueden activarse entre sí con las mismas salidas y entradas que usan entre sí las entidades del mapa.

| Quieres | Haz esto |
| --- | --- |
| Que una salida del mapa llegue a un nodo del overlay | Dale al nodo un hijo `TrenchDotOverlayIO` y pon su *Targetname* |
| Que un nodo del overlay dispare entidades del mapa | Añade un hijo `TrenchDotOutput`, elige la señal en *Output* y rellena destino, input y parámetro |
| Que un contenido viaje sobre una entidad en movimiento | Ponlo bajo un `TrenchDotAnchor` con *Target* puesto al targetname de la entidad |

Un input enviado a un `TrenchDotOverlayIO` llama al método del nodo con ese nombre, pone su propiedad con ese nombre, o ejecuta `show`, `hide` o `toggle`. Así que también funcionan los nodos sin script: el input `emitting` con parámetro `true` arranca un `GPUParticles3D`. Cada input emite además `input_received(input, parameter, activator)`.

Un `TrenchDotOutput` escucha cualquier señal de su padre, así que un `Area3D` dispara en `body_entered` sin código. Para tus propios eventos, añade una señal al script del padre y elígela:

```gdscript
extends StaticBody3D

signal flipped_on(activator: Node)
signal flipped_off(activator: Node)

var on := false

func use(activator: Node = null) -> void:
	on = not on
	if on:
		flipped_on.emit(activator)
	else:
		flipped_off.emit(activator)
```

Un ancla mantiene el desplazamiento donde la colocaste y sigue a la entidad a través de reconstrucciones, ediciones en vivo y movimiento, así que un cartel de advertencia sube con su puerta enrollable.

> [!WARNING]
> El cableado sobrevive a las reconstrucciones, el estado no. Las entidades del mapa vuelven a su estado inicial, así que conéctate a la señal `map_rebuilt(map)` del overlay y devuelve tu estado, por ejemplo vuelve a encender las lámparas.

## Verlo en la demo { #see-it-in-the-demo }

El distrito nocturno del proyecto demo tiene todo esto. Su escena `res://demo/showcase/night_district.scn` instancia la escena de overlay `res://demo/overlays/night_district_overlay.tscn` bajo su mapa, porque un script reconstruye las escenas de showcase. Abre la escena del overlay y encontrarás una caja de fusibles que enciende las lámparas del patio, guirnaldas de luces que enciende un trigger del mapa y un cartel que viaja sobre la puerta enrollable del garaje.
