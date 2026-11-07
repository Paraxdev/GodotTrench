---
source: 5d04cb3d9a7e005f4a371160c23d337e8ee707a7
---

# Iluminación precalculada en Godot { #baked-lighting-in-godot }

Cuando [precalculas un mapa en el editor](../editor/light-baking.md), el lightmap se guarda dentro del `.tdm`. Godot no precalcula nada por sí mismo, cada construcción solo convierte el precálculo guardado en el nodo de lightmap propio de Godot.

## Qué añade la construcción { #what-the-build-adds }

| Parte | Qué recibe |
| --- | --- |
| Un `LightmapGI` llamado `baked_lighting` | Hijo del `FuncGodotMap`, que guarda el lightmap, la máscara de sombras del sol y las sondas de luz |
| Mallas precalculadas | Sus coordenadas de lightmap como UV2 y un modo de GI estático, chunks en streaming incluidos |
| Luces y el sol | El modo de precálculo de Godot a partir de su `bake_mode` o `sun_bake_mode`, mira [Claves de luz](../editor/light-baking.md#light-keys) |

Con el sol en `bounce`, el sol sigue en tiempo real y la máscara de sombras precalculada sombrea lo que queda más allá de su distancia de sombra. Las caras que cambiaste después del precálculo recurren a la luz media del mapa, así que se ven plausibles pero planas hasta que precalcules de nuevo.

## Sondas de luz { #light-probes }

Los objetos en movimiento, como el jugador, los props y las puertas, toman su luz de las sondas de luz que los rodean, así que reciben la luz de rebote precalculada y se integran.

El precálculo coloca sondas en una cuadrícula de 2 metros sobre la caja alrededor de la geometría precalculada y descarta las que quedan dentro de sólidos. Donde la cuadrícula es demasiado gruesa, como un pasillo estrecho o una escalera, añade una entidad `light_probe` para tener una más. Un mapa recibe unas 1000 sondas como máximo, las colocadas a mano incluidas, así que en un mapa más grande la cuadrícula se espacia.

## Entidades de sonda { #probe-entities }

| Entidad | Tipo | Nodo de Godot | Úsala para |
| --- | --- | --- | --- |
| `light_probe` | Punto | `LightmapProbe` | Una sonda de luz extra para el precálculo, mira arriba |
| `env_reflection_probe` | Volumen de brush | `ReflectionProbe` del tamaño del brush | Reflejos que coincidan con una sala en suelos brillantes y metal |
| `env_voxel_gi` | Volumen de brush | `VoxelGI` del tamaño del brush | Luz de rebote en tiempo real que sigue a luces y objetos en movimiento |

Encuentras las tres en la pestaña Entidades. Arrastra `light_probe` a una vista. Para los dos volúmenes, dibuja una caja alrededor de la sala, haz clic derecho en la tarjeta y elige **Convertir en ella los brushes seleccionados**.

![La pestaña Entidades filtrada por env_, con la tarjeta env_voxel_gi con clic derecho y Convertir en ella los brushes seleccionados resaltado](../assets/baked-lighting/entities-turn-brushes-into.png)

El brush solo da la caja, la construcción lo elimina. Las claves de las tres están en [Props, luces y efectos](../gameplay/entities/effects.md#light_probe-env_reflection_probe-and-env_voxel_gi).

Un `env_voxel_gi` se precalcula en el editor de Godot cada vez que se construye el mapa. En un volumen grande, pon `bake_on_build` a 0 y precalcúlalo a mano con el botón **Precalcular** ("Bake" en un Godot en inglés) del nodo `VoxelGI`. Los mapas construidos mientras corre el juego se saltan ese precálculo.

## Usar otro lightmapper { #using-another-lightmapper }

¿Quieres precalcular con el `LightmapGI` propio de Godot? Desmarca **Use Baked Lighting** en el `FuncGodotMap` para dejar fuera el precálculo guardado. La construcción no añade entonces ningún nodo `baked_lighting` y deja en paz los modos de precálculo de las luces. Marca *Unwrap UV2* en *Build Flags* para que las mallas reciban el UV2 que necesita un precálculo de Godot.
