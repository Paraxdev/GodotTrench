---
source: ba7a98139fdce301c2c6a3f48c26f62837b5d180
---

# Scatter { #scatter }

La herramienta **Scatter** (B) pinta árboles, rocas y hierba sobre las superficies que elijas, para que cubras una ladera en segundos en lugar de colocar cada modelo a mano. El [tutorial de la isla del mar](../tutorials/sea-island-scatter.md) recorre una configuración completa.

![Pintando una franja de bosque por una isla con la herramienta Scatter](../assets/scatter/scatter-paint.gif)

## Conjuntos de scatter { #scatter-sets }

Todo lo que dispersas vive en un **conjunto de scatter** en su propia capa, por ejemplo un conjunto para árboles y otro para hierba. Pulsa B y el panel Scatter se abre a la derecha, con una tarjeta por cada modelo del conjunto activo.

![El panel Scatter con el conjunto arbustos activo: el menú de conjuntos, +, Preajuste, Renombrar y una tarjeta por modelo](../assets/scatter/scatter-panel.png)

| Para | Haz esto |
| --- | --- |
| Empezar un conjunto vacío | **+** junto al menú de conjuntos |
| Empezar desde un preajuste | **Preajuste**, y elige uno de los diez conjuntos incluidos |
| Añadir modelos | Arrástralos desde el panel Modelos, o **Añadir modelos…**. **Añadir props seleccionados** toma los modelos de las entidades seleccionadas |
| Dejar de pintar un modelo | Desmarca **pintar** en su tarjeta. Lo que ya colocó se queda |
| Ajustar un modelo | Haz clic en la flecha a la derecha de su tarjeta |

![El menú Preajuste abierto con bosque, pinos, bosque mixto, bosque detallado, árboles low-poly, arbustos, sotobosque, rocas, peñascos y hierba, con rocas bajo el cursor](../assets/scatter/preset-menu.png)

Los preajustes de rocas, peñascos, hierba, sotobosque y árboles low-poly usan modelos de Blockbench incluidos en el editor. Elegir uno añade los modelos que necesita a `res://trenchdot/nature`, sin conexión. Los preajustes de bosque, pinos, bosque mixto, bosque detallado y arbustos usan los árboles y arbustos glTF del paquete de naturaleza, una descarga de unos 23 MB. En un proyecto que no lo tiene, elegir uno de esos abre *Godot > Añadir contenido al proyecto…* en los modelos de naturaleza, en lugar de crear un conjunto con modelos que faltan. *Terreno > Scatter > Instalar modelos de naturaleza…*, o **Instalar modelos de naturaleza…** abajo del panel, también lo abre. Los archivos que ya están en el proyecto nunca se sobrescriben.

### Tarjetas de modelo { #model-cards }

Abre una tarjeta con su flecha para ajustar ese modelo.

![Una tarjeta abierta de bush_round con Escalar, Espaciado, Alinear, Inclinación, Hundimiento, Giro aleatorio y Material](../assets/scatter/model-card.png)

| Ajuste de la tarjeta | Qué hace |
| --- | --- |
| peso | Con qué frecuencia se elige este modelo comparado con los demás del conjunto |
| Escalar | Un tamaño aleatorio entre los dos valores |
| Espaciado | La distancia más pequeña en unidades de mapa a cualquier otra instancia |
| Alinear | 0 deja el modelo vertical, 1 lo inclina para seguir la superficie |
| Inclinación | La mayor inclinación aleatoria en grados |
| Hundimiento | Cuánto se hunde el modelo en el suelo, para ocultar la base de un tronco o una roca |
| Giro aleatorio | Gira cada instancia hacia una dirección aleatoria |
| Material | Se usa en lugar de los materiales propios del modelo. Si se deja vacío, se aplica el material del conjunto, si lo tiene |

## Destinos { #targets }

Un conjunto solo cae sobre sus **destinos**, así que un bosque pintado en un terreno nunca cubre la casa que hay en él. Alt+clic en una vista, o haz clic en el cuentagotas a la derecha de *Destinos* y luego clic en una vista, para añadir o quitar un destino. Un conjunto sin destinos toma lo que sea que su primer trazo toque. Otro conjunto también puede ser un destino, que es como crece la hierba sobre los peñascos.

![La sección Destinos del panel Scatter con el terreno como destino, el cuentagotas arriba a la derecha](../assets/scatter/targets.png)

Marcar **Seguir el cursor** ignora los destinos y pinta sobre cualquier superficie o prop disperso que haya bajo el pincel.

## Pintar { #painting }

Los ajustes rápidos están en la barra de opciones de herramienta arriba a la izquierda: el conjunto, **Pintar** o **Borrar**, radio, densidad, pendiente máx., **Seguir el cursor**, el cuentagotas y **Rellenar**. **Panel scatter** recupera el panel si lo cerraste.

![La barra de opciones de la herramienta Scatter con el conjunto arbustos, Pintar, radio 512 u, densidad 1,00 y pendiente máx. 40°](../assets/scatter/scatter-toolbar.png)

| Entrada | Acción |
| --- | --- |
| Arrastre con el botón izquierdo | Pintar |
| Shift+arrastre con el botón izquierdo | Borrar, solo del conjunto activo |
| Ctrl+rueda | Cambiar el tamaño del pincel |
| **Rellenar objetivos** | Cubrir todos los destinos de una vez, dentro de los límites de pendiente y altura |

La sección *Brush* del panel guarda el resto.

![La sección Brush del panel Scatter: Radio, Densidad, Pendiente, Altura, Atenuación, Solo expuesto, Cantidad de borrado, Semilla, Salida y las casillas bajo ellos](../assets/scatter/brush-settings.png)

**Densidad** es cuántas colocaciones intenta el pincel por cada 64 por 64 unidades, y el espaciado de cada modelo limita cuántas caben de verdad. **Solo expuesto** salta los puntos con geometría encima, lo que mantiene la hierba fuera de debajo de balcones y tejados. **Mantener espaciado con otros conjuntos**, activado por defecto, evita que un arbusto crezca atravesando el tronco de otro conjunto.

## En Godot { #in-godot }

El **Tipo** de un conjunto decide qué construye Godot.

| Tipo | Se construye como |
| --- | --- |
| props | MultiMeshes con colisión y sombras. Los modelos cuyas escenas llevan scripts se instancian uno a uno para que los scripts se ejecuten |
| foliage | MultiMeshes sin colisión ni sombras, ocultos más allá de 2400 unidades por defecto. Pensado para hierba y plantas pequeñas |

Selecciona el conjunto en el Outliner y su fila **Colisión** en el Inspector elige la forma con la que colisiona cada instancia. Los conjuntos de props nuevos empiezan en convexa y los de foliage en ninguno.

| Colisión | Forma por instancia |
| --- | --- |
| ninguno | Sin colisión |
| caja | Una caja alrededor de la instancia |
| convexa | Un casco convexo alrededor de la instancia |
| descomposición | Varios cascos convexos que siguen los modelos cóncavos, como un arco, más lento de construir |
| trimesh | Cada cara del modelo, exacto pero el más costoso |

Cuando necesites editar props sueltos, **A entidades** abajo del panel Scatter, o *Terreno > Scatter > Sets scatter a entidades* para los conjuntos seleccionados, reemplaza un conjunto por una entidad `prop_model` por cada instancia.

![La parte de abajo del panel Scatter con Tipo, Pintar, Borrar, Rellenar objetivos, Borrar instancias, A entidades e Instalar modelos de naturaleza…](../assets/scatter/panel-buttons.png)
