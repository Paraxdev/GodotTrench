---
source: 2bf892f3f08e81fde13de0d18bb8d2c0a06c8753
---

# Decals { #decals }

Los decals ponen mugre, manchas, grietas, pósteres y carteles sobre una superficie. Se dibujan un pelo más cerca de la cámara que la pared que tienen detrás y se mezclan sobre ella, así que nunca parpadean contra la pared ni entre ellos.

## Colocar un decal { #placing-a-decal }

1. Encuentra el material

    Búscalo en el panel Materiales, el cuadro de búsqueda arriba a la izquierda ayuda.

2. Mantén Alt y arrástralo sobre una cara

    La tarjeta junto al cursor te recuerda que Alt suelta un decal.

    ![Arrastrando night/shop_sign sobre una pared de ladrillo, la tarjeta junto al cursor dice Soltar sobre una cara, Shift cubre todo el brush, Alt suelta un decal](../assets/decals/decal-alt-drop.png)

3. Suelta

    Deberías ver cómo el decal aterriza donde lo soltaste, como un cuadrado de un metro, 32 por 32 unidades, mirando hacia fuera de la cara.

    ![El decal del cartel de la tienda sobre la pared de ladrillo](../assets/decals/decal-on-wall.png)

Un decal es una malla normal, así que muévelo, rótalo y escálalo como tal. Con él seleccionado, el Inspector muestra **Decal (mezclado, doble cara)** marcado, y los campos de posición y tamaño para colocarlo con exactitud.

![El Inspector con el decal seleccionado, Decal (mezclado, doble cara) marcado y su tamaño de 32 por 32](../assets/decals/decal-inspector.png)

Esa misma casilla convierte cualquier malla fina en un decal, para uno con un contorno propio.

Los bordes suaves se quedan suaves. Para un corte duro, usa un material en modo alpha scissor, mira [Transparencia](../godot/materials.md#transparency).

## Por qué no un brush fino { #why-not-a-thin-brush }

> [!WARNING]
> No finjas decals con brushes finos ni con mallas a unos milímetros de una pared. Parpadean.

De cerca un brush fino se ve bien. Aléjate y empieza a parpadear a través de la pared en franjas que se arrastran al moverte. Esto se llama z-fighting.

La GPU calcula qué superficie está más cerca en cada píxel comparando sus profundidades, y guarda la profundidad en un número limitado de pasos. Los pasos son más gruesos cuanto más lejos está una superficie y más rasante la miras. A unos metros, el brush y la pared se redondean al mismo paso, así que cada píxel elige uno de los dos casi al azar, y la elección cambia cada vez que se mueve la cámara. Separar más el brush de la pared solo empuja el parpadeo más lejos, y de cerca entonces flota a la vista delante de la pared.

Un decal no tiene ese problema. Cuando Godot construye el mapa, acerca cada decal a la cámara un 0,05 por ciento de su distancia, medio milímetro a 1 metro y 5 centímetros a 100 metros, así que se queda delante de la pared a cualquier distancia. El desplazamiento va a lo largo de tu línea de visión, así que el decal no se mueve en pantalla. Además no escribe profundidad propia, así que los decals que se solapan se mezclan entre sí en lugar de pelearse.

Una malla fina sirve si marcas **Decal (mezclado, doble cara)** en ella, eso la convierte en un decal de verdad.

## Decals proyectados { #projected-decals }

Un decal normal es plano. Para una mancha que se curva sobre un escalón, una tubería o un suelo irregular, usa en cambio un decal proyectado. Es una entidad que proyecta un material sobre todo lo que hay dentro de su caja, props incluidos. Cuesta más de dibujar, así que quédate con los decals normales en superficies planas.

Viene con el paquete de entidades de gameplay como `infodecal`. [Decals proyectados](projected-decals.md) te enseña a configurarlo, colocarlo y a qué prestar atención.

<div class="td-mcp" markdown>

## MCP { #mcp }

Por el [servidor MCP](../mcp.md), coloca un decal con `run_action` `create_decal`, pasando `material`, `at`, `normal` y `size` como `[width, height]`.

</div>
