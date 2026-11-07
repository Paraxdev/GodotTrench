---
source: cb5ad8fa1eb282094d0cdec538d72382b939f070
---

# Precalcular la iluminación { #baking-lighting }

Precalcular calcula una sola vez la luz del sol, del cielo, de tus lámparas y de los materiales brillantes, rebotes incluidos, y la guarda en el mapa como un lightmap. Godot entonces dibuja esa luz en lugar de calcularla en cada fotograma. Obtienes luz de rebote suave, derrames de color de las lámparas y esquinas más oscuras casi sin coste en tiempo de ejecución. Las luces que quieras encender, apagar o mover siguen en tiempo real junto a la iluminación precalculada.

![Izquierda: una sala en Vista previa iluminada, gris plano con manchas de sol de bordes duros bajo las ventanas. Derecha: la misma sala tras precalcular, luz cálida de lámpara, esquinas más oscuras y manchas de sol suaves](../assets/light-baking/lit-preview-and-baked.png)

El precálculo cubre la geometría estática: brushes del mundo, mallas, terrenos, displacements y entidades brush que no se mueven. Las entidades brush que se mueven, como puertas y plataformas, los volúmenes de activación, los decals, las texturas de herramienta y las capas excluidas de la exportación ni reciben luz del precálculo ni proyectan sombras precalculadas.

## Precalcular un mapa { #baking-a-map }

1. Abre la ventana Precalcular iluminación

    Elige *Godot > Precalcular iluminación…*.

    ![El menú Godot con Precalcular iluminación bajo el cursor y su tooltip visible](../assets/light-baking/menu-bake-lighting.png)

2. Revisa qué entra

    La línea de arriba lo resume, por ejemplo "Precalcula el sol, 1 luz, el cielo y los materiales brillantes en 102 superficies", y dice qué luces siguen en tiempo real.

    ![La ventana Precalcular iluminación con Calidad en Medio, Tamaño de texel 16,0 u, Suavidad de las sombras 0,30 y Se ejecuta en CPU](../assets/light-baking/bake-dialog.png)

3. Elige una calidad y un tamaño de texel

    Mira [las opciones](#dialog-options) más abajo. Los valores por defecto sirven para un primer vistazo.

4. Haz clic en **Precalcular**

    Una ventana de progreso muestra la etapa, y mientras tanto puedes seguir editando. Los cambios hechos durante el precálculo no entran en él. **Cancelar** lo detiene y el mapa conserva la iluminación que tenía.

5. Mira el resultado

    Cuando termina, la vista 3D cambia al sombreado **Iluminación precalculada**, y la barra de estado dice cuánto tardó y qué tamaño tiene el lightmap.

El precálculo se guarda dentro del `.tdm`, y precalcular es un solo paso de deshacer. **Quitar precálculo** en la misma ventana lo descarta, y Godot vuelve a iluminar el mapa en tiempo real.

## Opciones del diálogo { #dialog-options }

| Opción | Qué hace |
| --- | --- |
| Calidad | Cuántos rayos y rebotes gasta el trazador, mira la tabla de abajo |
| Tamaño de texel | Unidades de mapa que cubre un texel del lightmap, de 2 a 128, 16 (50 cm) por defecto. Más pequeño es más nítido y más lento |
| Suavidad de las sombras | De 0 a 1. Extiende las luces que no tienen `light_size` propio para que las sombras tengan bordes suaves. 0 las mantiene duras |
| Se ejecuta en | CPU, GPU o CPU + GPU, mira abajo |

| Calidad | Rayos por texel | Rebotes | Rayos de sombra por luz | Úsala para |
| --- | --- | --- | --- | --- |
| Vista previa | 32 | 1 | 4 | Colocar luces, con ruido pero rápido |
| Medio | 96 | 2 | 8 | El trabajo diario, el valor por defecto |
| Alto | 192 | 3 | 12 | Un resultado limpio |
| Final | 384 | 4 | 16 | La compilación que publicas, lenta en mapas grandes |

El lightmap mide como mucho 4096 texels por lado. Cuando un mapa no cabe con el tamaño de texel que pediste, el precálculo usa uno más grueso y lo dice en la barra de estado. El mapa recuerda las opciones con las que se precalculó por última vez.

### CPU, GPU o ambas { #cpu-gpu-or-both }

| Se ejecuta en | Elígelo cuando |
| --- | --- |
| CPU | Siempre funciona, en todos los núcleos. El valor seguro por defecto |
| GPU | El equipo tiene una tarjeta gráfica dedicada, que suele ser mucho más rápida |
| CPU + GPU | Ambas son potentes. La tarjeta y todos los núcleos toman trabajo de una sola cola, cada uno tanto como aguante |

Cuando no se puede usar la GPU, por ejemplo porque no hay ninguna, es un renderizador por software o el mapa es demasiado grande para sus búferes, la CPU precalcula el mapa y la barra de estado dice por qué. Una GPU que falla a medias pasa el resto a la CPU.

## Mirar el precálculo { #looking-at-the-bake }

*Ver > Sombreado > Iluminación precalculada* muestra el precálculo, y cuando el mapa ya tiene uno, F4 lo incluye al recorrer los sombreados. *Ver > Sombreado > Vista precalculada* elige qué muestra.

![Ver > Sombreado > Vista precalculada abierto, con Texturas con luz precalculada marcado y Solo lightmap bajo el cursor](../assets/light-baking/menu-baked-view.png)

| Vista precalculada | Muestra |
| --- | --- |
| Texturas con luz precalculada | Las texturas iluminadas por el lightmap, cerca de lo que muestra Godot |
| Solo lightmap | Solo la luz, sin texturas, para juzgar el rebote y el ruido |
| Máscara de sombra del sol | Cuánto sol llega a cada punto, blanco a pleno sol |
| Oclusión ambiental | Oscurecimiento en esquinas y bajo los objetos, solo para comprobar, Godot no lo usa |

![La misma sala en las cuatro vistas precalculadas: texturas con luz precalculada arriba a la izquierda, solo el lightmap arriba a la derecha, la máscara de sombra del sol abajo a la izquierda y la oclusión ambiental abajo a la derecha](../assets/light-baking/baked-views.png)

## Claves de luz { #light-keys }

Cada `light` y `light_spot` tiene una clave `bake_mode`, y worldspawn tiene `sun_bake_mode` para el sol:

| Valor | En el precálculo | En Godot |
| --- | --- | --- |
| `auto` | Como `baked`, pero una luz con `targetname` o `start_on` 0 sigue en tiempo real, porque la E/S puede encenderla o apagarla. El sol siempre se precalcula | Estática o desactivada |
| `baked` | Luz directa y rebotada y sus sombras | Estática, el lightmap la lleva |
| `bounce` | Solo la luz rebotada | Dinámica, Godot dibuja la luz directa y sus sombras en tiempo real |
| `realtime` | Se deja fuera | Desactivada, totalmente en tiempo real |

Usa `bounce` para una lámpara cuyas sombras deben seguir a los objetos en movimiento mientras su rebote aún llena la sala. Los nombres propios de Godot `static`, `dynamic` y `disabled` también funcionan. Una luz en tiempo real también ilumina los objetos en movimiento mediante las [sondas de luz](../godot/baked-lighting.md#light-probes).

`light_size` es el radio de la lámpara en metros. Una lámpara más grande proyecta sombras más suaves, y 0 usa en cambio la suavidad de las sombras de la ventana. `light_indirect_energy` escala la luz rebotada, y `omni_attenuation`, `spot_attenuation` y `spot_angle_attenuation` dan forma a la atenuación. La luz de Godot recibe todas ellas como sus propiedades del mismo nombre, así la luz en tiempo real coincide con el precálculo.

## Cuando un precálculo queda desfasado { #when-a-bake-goes-out-of-date }

El precálculo recuerda la geometría y las luces con las que se hizo. Mover, remodelar, añadir o borrar geometría estática, o cambiar una luz, una sonda de luz, las claves de sol y cielo de worldspawn, las texturas en uso o sus ajustes, lo deja desfasado. La ventana Precalcular iluminación entonces avisa de que el mapa ha cambiado desde el último precálculo.

![La ventana Precalcular iluminación con la línea amarilla "El mapa ha cambiado desde el último precálculo. Vuelve a precalcular para actualizar la iluminación." y los botones Precalcular y Quitar precálculo](../assets/light-baking/bake-dialog-out-of-date.png)

Hasta que vuelvas a precalcular, las caras que cambiaron muestran la Vista previa iluminada en el editor y la luz media del mapa en Godot, mientras el resto conserva su luz precalculada. Aquí el pilar se movió después del precálculo: muestra la Vista previa iluminada, y su sombra antigua sigue en el suelo. Vuelve a precalcular antes de publicar.

![La sala precalculada tras mover el pilar, el pilar está iluminado de manera uniforme y su sombra precalculada antigua se queda donde estaba](../assets/light-baking/bake-out-of-date.png)

## Ajustes de textura { #texture-settings }

Selecciona una cara y abre **Ajustes de textura** en el Inspector para fijar cómo se trata cada cara con esa textura, en todo el mapa. Los ajustes se guardan con el mapa.

![El Inspector con una cara seleccionada y Ajustes de textura abierto: Precalcular luz en ella, Proyecta sombras precalculadas, Detalle del lightmap, Repetir cada y Proyección](../assets/light-baking/texture-settings.png)

| Ajuste | Qué hace | Úsalo para |
| --- | --- | --- |
| Precalcular luz en ella | Desactivado deja las caras fuera del lightmap, Godot les da la luz media del mapa | Cielo, agua, caras que nadie ve de cerca |
| Proyecta sombras precalculadas | Desactivado deja pasar la luz por las caras en el precálculo | Cristal, rejillas, decals finos |
| Detalle del lightmap | Multiplica el tamaño de texel en estas caras, por debajo de 1 es más nítido, por encima de 1 ahorra espacio | Sombras nítidas en un suelo, borrosas en un techo lejano |
| Repetir cada | Unidades de mapa que cubre una repetición de la textura, en lugar de su tamaño en píxeles | Una sola escala para una textura en todas partes |
| Proyección | Mundo ignora el desplazamiento, la escala y la rotación propios de cada cara, así la textura continúa sin costuras | Papel pintado, baldosas, hormigón |

> [!TIP]
> Para un mapa al estilo backrooms, pon el papel pintado en proyección Mundo y un tamaño de repetición una vez, y cada pared que dibujes con él queda alineada sin tocar la herramienta Textura.

Una cara con las dos opciones de precálculo desactivadas queda fuera del precálculo por completo. El tamaño de repetición y la proyección también se aplican en Godot.

<div class="td-mcp" markdown>

## MCP { #mcp }

La herramienta `bake_lighting` precalcula con las mismas opciones que la ventana, `quality`, `texel_size`, `softness` y `backend` (`cpu`, `gpu` o `hybrid`), y `view` cambia la Vista precalculada. Espera a que termine el precálculo y devuelve el tamaño del lightmap, los recuentos, la GPU usada o por qué no se usó, y `out_of_date`. La operación `settings` de la herramienta `texture` lee y fija los ajustes de textura, con `material`, `bake`, `casts`, `texel_scale`, `size` (`null` lo borra) y `projection` (`face` o `world`). Mira [servidor MCP](../mcp.md).

</div>
