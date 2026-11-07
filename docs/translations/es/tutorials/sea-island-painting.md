---
source: 6da371c3430389f428469b408b32808d1d2b0af6
---

# Isla en el mar 3: pintar { #sea-island-3-painting }

Cambia a la herramienta Mezcla con **Shift+G**, o con el botón Mezcla justo al lado de Esculpir en la barra de herramientas. Igual que Esculpir, funciona sobre el terreno seleccionado o sobre todo cuando no hay nada seleccionado. Sus ajustes están en la barra de opciones de la herramienta, arriba a la izquierda.

![La barra de herramientas con el botón Herramienta de mezcla resaltado y su tooltip: arrastre para pintar mezclas en capas de terreno, displacements y caras con material de mezcla](../assets/sea-island/toolbar-blend.png)

Los modos **pendiente** y **altura** solo pintan donde el suelo está dentro de un rango de inclinación o de altura. Un radio de 12000 cubre toda la isla, así que un clic en estos modos pinta de una vez todas las costas o todos los acantilados.

## Arena y roca { #sand-and-rock }

| Ajuste | Arena | Roca |
| --- | --- | --- |
| Modo | altura, de `-3000` a `40` | pendiente, de `32` a `90` |
| Atenuación | constante, fuerza total en todo el pincel | constante |
| Capa | 3 | 1 |
| Radio, fuerza | 12000, 1 | 12000, 1 |

Ajústalos y haz un clic en el centro de la isla para cada uno. Deberías ver que todo lo que está por debajo de 40 se vuelve arena, y que toda pendiente empinada se vuelve roca:

![Modo altura: todo lo que queda por debajo de 40 unidades se vuelve arena](../assets/sea-island/sand-height.jpg)

![Modo pendiente: todo lo más empinado que 32° se vuelve roca](../assets/sea-island/rock-slope.jpg)

Para ensanchar la playa, pinta la capa 3 a lo largo de la costa sur en modo **pintar**, atenuación suavizar, radio 700, fuerza 0.7. La atenuación suave se desvanece hacia el borde del pincel, así que el borde de la arena queda suave.

## Caminos { #paths }

Modo **pintar**, atenuación suavizar, capa 2, radio 140, fuerza 0.8. Arrastra:

1. Desde lo alto de la playa hasta la colina principal en curvas largas.
2. Desde la cima de la colina principal a lo largo de la loma hasta lo alto del acantilado.
3. Desde el extremo este de la playa hasta la segunda colina.

![Pintando el camino que sube a la colina principal](../assets/sea-island/paths.jpg)

![El camino de cerca](../assets/sea-island/path-close.jpg)

## Rompe la repetición { #break-up-the-repeat }

Desde lejos una textura repetida se ve como una cuadrícula. Selecciona el terreno en el Outliner (un clic en una vista pintaría con la herramienta Mezcla), luego ajusta **desrepetir** y **nitidez** en cada fila Capa del Inspector. La nitidez mantiene definida la textura desrepetida en vez de borrosa.

| Capa | Desrepetir | Nitidez |
| --- | --- | --- |
| 0 hierba | 0.5 | 0.5 |
| 1 acantilado | 0.7 | 0.4 |
| 2 tierra | 0.5 | 0.5 |
| 3 arena | 0.4 | 0.5 |

![El Inspector del terreno con las cuatro filas Capa, cada una con su material, mosaico, desrepetir y nitidez](../assets/sea-island/inspector-terrain-layers.png)

Siguiente: [Isla en el mar 4: scatter](sea-island-scatter.md).
