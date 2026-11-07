---
source: 5d131fe08a12db6615a97a4cbd455746685c5f02
---

# Isla en el mar 5: muelle y Godot { #sea-island-5-pier-and-godot }

## El muelle { #the-pier }

1. Dale su propia capa

    Haz clic en **Añadir capa** en la parte superior del Outliner. La capa nueva pasa a ser la actual, así que el muelle va a parar a ella y puedes ocultarla o bloquearla aparte del terreno.

    ![El Outliner con Añadir capa resaltado y su tooltip: capa nueva, pasa a ser la capa actual en la que van los objetos nuevos](../assets/sea-island/outliner-add-layer.png)

2. Constrúyelo

    Pulsa **Q** para la herramienta Seleccionar, ya que un arrastre con la herramienta Scatter sigue pintando modelos. Luego dibuja brushes simples que salgan hacia el mar desde la mitad de la playa, con las medidas de la tabla de abajo.

3. Añade props

    Ve a *Archivo > Importar > Prop de modelo (.bbmodel, .glb)…* y elige modelos de la carpeta `models/polyhaven` del proyecto de demo: dos `containers/wooden_crate_01` y un `containers/wine_barrel_01` sobre la cubierta, un `nautical/lifebuoy` contra un poste y un `nautical/ocean_buoy` en el agua. Cada uno aparece en el cursor 3D, muévelo a su sitio desde ahí.

    ![El menú Archivo con Importar abierto y Prop de modelo (.bbmodel, .glb)… resaltado](../assets/sea-island/menu-import-model-prop.png)

| Parte | Tamaño | Material |
| --- | --- | --- |
| Cubierta | 128 de ancho, 1100 de largo, 8 de grosor, cara superior 48 sobre el agua | `showcase/planks` |
| Postes | 16 unidades de lado, dos filas cada 200 unidades, desde 320 bajo el agua hasta la cubierta | `showcase/beam` |
| Barandilla | Dos postes cortos en el extremo lejano, de 40 de alto | `showcase/beam` |

Un prop referencia su modelo por una ruta `res://`, así que el archivo tiene que estar dentro del proyecto de Godot abierto. Si eliges uno de otro sitio, el editor ofrece copiarlo antes a `res://models`.

![El muelle visto desde el mar](../assets/sea-island/pier.jpg)

## Velo en Godot { #see-it-in-godot }

Guarda el mapa dentro de tu proyecto de Godot y constrúyelo con un `FuncGodotMap`, mira [Construir mapas](../godot/building.md). Cada parte del mapa se convierte en un tipo distinto de nodo:

| Parte del mapa | Qué obtienes en Godot |
| --- | --- |
| Terreno | Un `TrenchDotTerrain` dividido en chunks, cada uno una malla que mezcla las cuatro capas y tiene su propia colisión |
| Mar | El brush `func_illusionary`, visible y sin colisión |
| Conjuntos de árboles, arbustos y peñascos | MultiMeshes, más un cuerpo estático por modelo con una forma de colisión para cada instancia, todas compartiendo una sola forma |
| Conjuntos de hierba | MultiMeshes sin colisión que se desvanecen con la distancia |

![La isla terminada en Godot](../assets/sea-island/godot-overview.jpg)

![El muelle en Godot](../assets/sea-island/godot-pier.jpg)
