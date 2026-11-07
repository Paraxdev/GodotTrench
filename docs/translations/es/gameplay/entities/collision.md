---
source: d6310713b2a9d65be25e506f0179132b8e0e10fd
---

# Colisión { #collision }

Toda entidad de brush del paquete Gameplay y toda entidad de modelo colisiona sin que establezcas nada. Las puertas, los botones, los ascensores, los trenes y los triggers reciben un casco convexo por brush, y `prop_model`, `prop_physics`, `npc_walker` y los props de scatter reciben una forma construida a partir de su modelo. Solo tocas las claves de abajo cuando el valor por defecto no encaja.

## Las claves { #the-keys }

Selecciona la entidad y busca estas filas en el Inspector. Pasa el cursor por una fila para leer qué hace cada opción.

![El Inspector de un func_door con el menú de colisión abierto sobre default, none, box, convex hull, convex decomposition y trimesh, y las filas collision_layer y collision_mask debajo](../../assets/collision/inspector.png)

| Clave | Por defecto | Qué hace |
| --- | --- | --- |
| `collision` | default | La forma: `default`, `none`, `box`, `convex hull`, `convex decomposition` o `trimesh` |
| `collision_layer` | vacía | Las capas de física en las que está el cuerpo, como números de capa tipo `1, 3`. `none` lo deja en ninguna capa |
| `collision_mask` | vacía | Las capas de física que detecta el cuerpo, en la misma forma. `none` no detecta nada |

Una capa o máscara vacía conserva el valor por defecto: la capa y la máscara de la definición de entidad para las entidades de brush, con los triggers tomando *Trigger Collision Mask* de los ajustes del mapa, y los ajustes del proyecto de abajo para las entidades de modelo.

## Elegir una forma { #picking-a-shape }

| Forma | Entidades de brush | Modelos |
| --- | --- | --- |
| `default` | La forma de la clase, un casco convexo por brush | El ajuste del proyecto, un casco convexo por malla salvo que se cambie. `prop_physics` usa una caja a partir de `size` |
| `none` | Sin colisión | Sin colisión |
| `box` | Una caja alrededor de todos los brushes | Una caja alrededor del modelo, la forma más barata |
| `convex hull` | Un casco por brush | Un casco por malla, bien para modelos con forma de caja |
| `convex decomposition` | Igual que convex hull, un brush ya es convexo | Varios cascos que siguen un modelo cóncavo como un arco, más lento de construir |
| `trimesh` | Todas las caras, solo para cuerpos estáticos | Todos los triángulos, exacto pero el más costoso |

Trimesh solo funciona en cuerpos estáticos. Las puertas, los botones, los ascensores y los trenes son cuerpos animables, que cuentan como estáticos, así que pueden usarlo. Los triggers, `prop_physics` y `npc_walker` construyen cascos convexos en su lugar, y un trigger establecido en trimesh muestra una advertencia al construir el mapa.

Un `prop_model` cuyo modelo ya contiene un cuerpo de colisión conserva ese cuerpo y no construye ninguna forma propia.

## Valores por defecto del proyecto { #project-defaults }

Las entidades de modelo con las claves vacías siguen tres ajustes de los *Project Settings* de Godot, en *Trenchdot > Collision*. Aparecen cuando el plugin de TrenchDot está activado.

| Ajuste | Por defecto | Qué hace |
| --- | --- | --- |
| *Model Shape* | convex | La forma que representa default en `prop_model` y `npc_walker` |
| *Model Layer* | 1 | La capa en la que están los cuerpos de los modelos |
| *Model Mask* | 1 | Las capas que detectan los cuerpos de los modelos |

Los conjuntos de scatter eligen su forma en la fila **Colisión** del propio conjunto de scatter, mira [Scatter](../../editor/scatter.md).

## Tus propias entidades de brush { #your-own-brush-entities }

Una entidad de brush que defines tú colisiona como el cuerpo que nombra su *Node Class*. Con *Node Class* vacío toma el cuerpo del que extiende su script, y sin script, o con un script que extiende `Node3D`, se convierte en un `StaticBody3D`. Solo un script que extienda algún otro nodo, como un `Marker3D`, la deja sin colisión, y la construcción avisa de ello. Mira [Entidades propias en GDScript](../custom-entities.md).

El viewport del editor no dibuja las formas de colisión. *Debug > Visible Collision Shapes* de Godot las muestra mientras corre el juego, tanto en la sesión en vivo como en una compilación completa. Las entidades de brush y `prop_model` guardan sus formas en la escena construida, `prop_physics` y `npc_walker` construyen las suyas cada vez que se carga la escena, igual que cargan su modelo. Un `prop_model` o un `npc_walker` cuya escena de modelo ya contiene un cuerpo de física conserva ese cuerpo y no construye formas propias.
