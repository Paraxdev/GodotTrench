---
source: d8695a266e0b8802237d74d9393aab6b51f516c4
---

# Geometría { #geometry }

## Brush { #brush }

Un sólido convexo, guardado como sus esquinas exactas más una lista de caras que las indexan, así obtienes las esquinas sin intersectar planos. Los planos de las caras se recalculan al cargar.

| Clave | Tipo | Significado |
| --- | --- | --- |
| `vertices` | array de `[x, y, z]` | Esquinas en unidades de mapa |
| `faces` | array de caras | Un polígono por cara |

Cada cara:

| Clave | Tipo | Por defecto | Significado |
| --- | --- | --- | --- |
| `indices` | array de enteros | obligatoria | Índices en `vertices`, al menos tres, en sentido antihorario visto desde fuera |
| `material` | string | `""` | Siempre se escribe. Nombre de la textura, por ejemplo `base/wall` |
| `uv` | object | paraxial | Siempre se escribe. Cómo se coloca la textura en la cara, mira [Proyección de textura](#texture-projection) |
| `props` | object de string a string | se omite | Atributos libres de la cara, como las claves de mezcla de más abajo |
| `disp` | object | se omite | Convierte la cara en un [displacement](#displacement) |
| `colors` | array de `[r, g, b, a]` | se omite | [Pintura de vértices](../editor/texturing.md#vertex-paint), un color por índice |

La construcción de Godot se salta un brush con menos de cuatro vértices o caras, ya que ningún sólido cerrado tiene menos.

Una cara también puede mezclar dos materiales. Las props de cara `blend_material`, `blend_detile`, `blend_uv_scale` y `blend_detile_sharpen` lo configuran, y después el alfa del color de vértice pasa de `material` hacia `blend_material` a lo largo de la cara.

## Proyección de textura { #texture-projection }

`uv` es una proyección al estilo Valve 220 con ejes explícitos: dos direcciones del mundo dicen hacia dónde corre la textura por la cara, y un desplazamiento y una escala la colocan.

| Clave | Tipo | Significado |
| --- | --- | --- |
| `u_axis`, `v_axis` | `[x, y, z]` | Dirección en el mundo de la u y la v de la textura |
| `offset` | `[u, v]` | Desplazamiento en texels |
| `scale` | `[u, v]` | Unidades de mapa por texel |
| `rotation` | number | Opcional y solo informativa, los ejes ya contienen la rotación |

Un punto `p` cae en el texel `dot(p, u_axis) / scale.x + offset.x`, y lo mismo para v. Para el UV final, divide eso entre el tamaño de textura. Es el `texture_size` del material cuando lo tiene, mira [Materiales](../godot/materials.md), y si no el tamaño en píxeles de la imagen.

Si falta `uv`, significa `u_axis = [1, 0, 0]` y `v_axis = [0, 0, 1]` con escala 1. Cuando `uv` está presente, todas las claves menos `rotation` son obligatorias.

## Ajustes de textura { #texture-settings }

El objeto `textures` de nivel superior contiene los ajustes compartidos por todas las caras con un material, indexados por el nombre del material. Solo se escriben los ajustes que difieren del valor por defecto, y un material sin ninguno se deja fuera.

| Clave | Tipo | Por defecto | Significado |
| --- | --- | --- | --- |
| `bake` | bool | `true` | Las caras reciben un sitio en el lightmap. Desactivado, Godot les da la luz precalculada media |
| `casts` | bool | `true` | Las caras bloquean la luz en el precalculado |
| `texel_scale` | number | `1` | Multiplica el tamaño de texel del lightmap en estas caras, de 0,125 a 16 |
| `size` | `[w, h]` | ninguno | Unidades de mapa que cubre una repetición, se usa en lugar del tamaño de textura de arriba |
| `projection` | string | `"face"` | `"world"` ignora el `uv` de cada cara de brush y usa la proyección reiniciada de su normal, desplazamiento 0 y escala 1 |

`world` solo se aplica a las caras de brush, las caras de malla conservan su proyección.

## Displacement { #displacement }

Una cara cuadrangular puede llevar un displacement al estilo Hammer, una cuadrícula de vértices que se empujan cada uno a lo largo de la normal de la cara.

| Clave | Tipo | Por defecto | Significado |
| --- | --- | --- | --- |
| `power` | integer | obligatoria | De 1 a 4. La cuadrícula tiene `2^power + 1` vértices por lado |
| `heights` | array de números | obligatoria | Cuánto se empuja cada vértice de la cuadrícula a lo largo de la normal |
| `alphas` | array de números | se omite | Peso de mezcla por vértice de la cuadrícula, 0 muestra el material de la cara |

Los dos arrays van por filas con `(2^power + 1)^2` entradas. Las filas van desde la primera esquina de la cara hacia la cuarta, las columnas desde la primera esquina hacia la segunda.

## Malla { #mesh }

Una malla de polígonos editable. A diferencia de un brush puede ser cóncava, abierta o tener caras que no son planas.

| Clave | Tipo | Por defecto | Significado |
| --- | --- | --- | --- |
| `vertices` | array de `[x, y, z]` | obligatoria | Esquinas en unidades de mapa |
| `faces` | array de caras | obligatoria | Las mismas claves que las caras de brush, más `uvs` |
| `smooth_angle` | number | `0`, se omite | Las caras que se encuentran en menos de estos grados comparten normales y se sombrean suavemente. 0 es sombreado plano |
| `decal` | bool | `false`, se omite | Marca la malla como un [decal](../editor/decals.md) |

El suavizado compara cada esquina de cara con cada cara alrededor de su vértice. Una malla que necesita más de 64 comparaciones por esquina, o 262.144 en total, se sombrea plana en su lugar. Solo un vértice compartido por cientos de caras, como la punta de un cono de más de unos 650 lados, llega a eso.

El `uvs` de una cara de malla es un array de `[u, v]`, uno por índice, en anchos de textura. Cuando está presente reemplaza a `uv`, y un recuento que no coincide con `indices` hace que el editor rechace el mapa.
