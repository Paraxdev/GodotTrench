---
source: 7732f7c6b2b4601df9d9b8913237f6914fb6ca69
---

# Capas de terreno { #terrain-layers }

Un terreno tiene de una a cuatro capas, cada una un material más unos ajustes de cómo se repite. Selecciona el terreno y las encuentras en el Inspector. Las capas se reparten el suelo entre ellas, así que pintar arena sobre hierba le quita peso a la hierba.

![El Inspector de un terreno con cuatro capas, cada una con su material, mosaico, desrepetir y nitidez](../assets/terrain-layers/layers-inspector.png)

| Para | Haz esto |
| --- | --- |
| Añadir una capa | **+ capa (material actual)** bajo las capas, que usa el material que pulsaste por última vez en el panel Materiales. Se muestra mientras el terreno tiene menos de cuatro |
| Quitar la última capa | **- última capa** |
| Cambiar el material de una capa | Escribe su nombre en el campo de la capa, o arrastra un material del panel Materiales sobre el terreno. La línea de abajo del Inspector dice a qué capa va un soltado, y con la herramienta Mezcla activa es la capa que pinta la herramienta |

**La capa 0 es el suelo base.** El suelo sin pintar muestra la capa 0 y borrar vuelve a pintar con ella, así que pon ahí tu suelo más común, normalmente hierba.

Un terreno nuevo recibe las capas nombradas en el diálogo *Crear terreno*, cuatro por defecto. Soltar un material para una capa que el terreno aún no tiene la añade como la capa siguiente.

## Mosaico y desrepetición { #tile-and-de-tiling }

Cualquier textura repetida sobre una zona grande muestra un patrón de cuadrícula desde lejos. Estos ajustes cambian nitidez de cerca por ocultar ese patrón.

| Ajuste | Significado |
| --- | --- |
| mosaico | Cuántas unidades de mapa cubre una repetición de la textura. De 128 a 384 funciona bien para suelo por el que caminas |
| desrepetir | De 0 a 1. Gira y desplaza cada repetición al azar para que la cuadrícula deje de verse |
| nitidez | De 0 a 1, aparece cuando desrepetir está por encima de 0. 0 mezcla las repeticiones desplazadas de manera uniforme, lo que se ve suave. 1 las mezcla solo a lo largo de sus costuras, lo que mantiene la textura nítida |

Un mosaico pequeño es nítido de cerca pero se repite a la vista de lejos, uno grande se desenfoca a tus pies. Desrepetir oculta la repetición sin un mosaico más grande, pero cuesta lecturas de textura extra, así que déjalo en 0 donde la repetición no se vea.

![Mosaico 256 con desrepetir 0 a la izquierda y 0,7 a la derecha](../assets/terrain-painting/detile-compare.jpg)

> [!NOTE]
> El `metadata/texture_size` del material no se aplica al terreno, solo **mosaico**.

## Cómo se ve en Godot { #how-it-looks-in-godot }

Cada capa se proyecta desde arriba y desde los lados, así que los acantilados muestran roca en lugar de un borrón, y coincide con lo que ves en el editor. Cada capa usa la textura de color, el mapa normal, la rugosidad y la emisión de su material. La oclusión ambiental se usa cuando está en la textura de rugosidad, como en una textura ORM, o cuando la capa no tiene textura de rugosidad.

![Los acantilados pintados de la isla del mar en Godot](../assets/terrain-painting/godot-cliffs.jpg)
