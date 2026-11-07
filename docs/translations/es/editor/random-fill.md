---
source: 7bb83afeb1e5ebd8423b507ac79851c690337899
---

# Relleno aleatorio { #random-fill }

*Brush > Relleno aleatorio…* cubre una cuadrícula de celdas con piezas de la [biblioteca de prefabs](prefab-library.md), eligiendo la pieza de cada celda por peso. Construye un pequeño kit de piezas, como una oficina, un pasillo, una cuadrícula de pilares y un suelo abierto, y el relleno traza una planta entera con ellas. Cada semilla da una planta distinta, y la misma semilla da siempre la misma.

1. Construye tus piezas

    Construye cada pieza de una celda de ancho (128 unidades por defecto), centrada en el origen. Mantén las paredes lejos del borde de la celda, para que las piezas vecinas siempre conecten.

    Luego añádela a la biblioteca: selecciónala, abre **Añadir la selección** arriba de la pestaña Prefabs, marca **Mantener posición**, ponle un nombre y haz clic en **Añadir** (mira [Tus propias piezas](prefab-library.md#your-own-pieces)).

    ![La pestaña Prefabs con Añadir la selección abierto y Mantener posición marcado, con su tooltip diciendo que conserva el origen del mapa como origen de la pieza, como un kit para Relleno aleatorio](../assets/random-fill/prefabs-keep-position.png)

2. Abre el diálogo

    Ve a *Brush > Relleno aleatorio…*.

    ![El menú Brush con Relleno aleatorio… bajo el cursor](../assets/random-fill/menu-random-fill.png)

3. Añade las piezas

    Elige cada pieza en **Añadir pieza…** y dale un **peso**: una pieza con peso 6 llena el doble de celdas aproximadamente que una con peso 3. **girar** deja que una pieza gire un múltiplo aleatorio de 90 grados.

    ![El diálogo Relleno aleatorio con tres piezas, sus pesos, peso vacío 4, una cuadrícula de 12 por 8 y su vista previa de colores](../assets/random-fill/random-fill-dialog.png)

4. (Opcional) Deja algunas celdas abiertas

    **peso vacío** funciona como una pieza que no coloca nada.

5. Coloca la cuadrícula

    Selecciona el suelo y haz clic en **Ajustar a la selección**, o fija tú mismo el tamaño de **Celda**, el número de **Celdas** y la **Esquina**.

6. (Opcional) Agrupa las piezas

    **Agrupación** reúne cada pieza en manchas de unas **Tamaño de mancha** celdas, así un ala de oficinas queda junto a una sala de pilares en lugar de que todo esté mezclado por igual.

7. Rellénalo

    La cuadrícula de colores muestra una vista previa de la distribución, un color por pieza. Haz clic en **Rellenar**, y **Volver a tirar** reemplaza el último relleno con la semilla siguiente.

    ![Un suelo relleno con pilares cuadrados y redondos y pilas de cajas, con algunas celdas vacías](../assets/random-fill/random-fill-result.jpg)

El relleno es un grupo de copias simples, así que se precalcula y se construye como cualquier cosa que coloques a mano, y puedes editar cualquier celda después. Los nombres de entidad de cada copia reciben su propio prefijo, `rf<seed>_<n>-`, así que una pieza con entidades conectadas, como una lámpara que parpadea con un temporizador, funciona por sí sola en cada celda.

> [!TIP]
> Rellena distintos tipos de detalle en pasadas separadas sobre la misma cuadrícula, por ejemplo un relleno para las paredes y otro para las luces del techo, con sus propios pesos, como unas pocas luces fundidas o parpadeantes entre las que funcionan.

<div class="td-mcp" markdown>

## MCP { #mcp }

La herramienta `random_fill` toma `pieces` como `[{name, weight, rotate}]`, `empty`, `min`, `cell`, `cells`, `seed`, `cluster` y `cluster_size`. `preview: true` solo devuelve la distribución como un boceto, `replace` quita primero un relleno anterior.

</div>
