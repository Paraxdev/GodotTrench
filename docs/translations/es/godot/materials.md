---
source: 6f3b7a09923ac7b7e6e7e7d49ae11666c6d47d02
---

# Materiales { #materials }

Cada cara apunta a un material por su ruta dentro del *Base Texture Dir* de los ajustes del mapa, sin la extensión, por ejemplo `showcase/planks`.

Cuando Godot construye el mapa busca `showcase/planks.tres` en esa carpeta, o en *Base Material Dir* si pusiste uno, y lo usa tal cual. ¿Sin `.tres`? Entonces el addon crea un material simple a partir de la imagen. TrenchDot previsualiza el mismo `.tres`, así que los dos lados se ven igual.

Solo necesitas un `.tres` cuando una imagen simple no basta, por ejemplo para cambiar la escala, añadir brillo o hacerlo transparente. Un `.tres` con solo un `albedo_color` y sin textura también funciona, el editor lo dibuja de ese color.

Las texturas y los materiales nuevos aparecen en la pestaña Materiales en un par de segundos, sin recargar.

## Tamaño de textura { #texture-size }

Por defecto un píxel de textura cubre una unidad de mapa, así que una textura de 64 píxeles se repite cada dos metros. Genial para pixel art, pero una foto de 1024 píxeles de una pared de dos metros se estiraría por 32 metros.

Para arreglarlo, dile al material cuántas unidades de mapa cubre una repetición:

```
[resource]
albedo_texture = ExtResource("1_albedo")
metadata/texture_size = Vector2(64, 64)
```

(También puedes añadirlo en el inspector de Godot, en *Metadata*, como un `Vector2` o un solo número.)

Pasa el ratón por encima del material en la pestaña Materiales para comprobarlo, el tooltip muestra el tamaño de la imagen y cuántas unidades cubre una repetición.

![La pestaña Materiales con el tooltip de night/shop_sign: 1024 x 256 px, 256 x 64 unidades por repetición, emisivo x3](../assets/materials/tooltip-texture-size.png)

Ahora puedes poner una imagen más nítida sin cambiar el mapa. Las capas de terreno lo ignoran, su ajuste `tile` ya fija la repetición.

## Materiales brillantes { #glowing-materials }

Un material brilla cuando su archivo pone `emission_enabled`. Con un color de emisión negro, la textura de emisión brilla exactamente como está pintada:

```
[resource]
albedo_texture = ExtResource("1_albedo")
emission_enabled = true
emission = Color(0, 0, 0, 1)
emission_energy_multiplier = 2.5
emission_texture = ExtResource("2_emission")
```

¿Sin archivo de material? Pon una imagen con el nombre del albedo más `_emission` al lado, `night/sign_emission.png` junto a `night/sign.png`, y se convierte en el mapa de brillo.

Los materiales brillantes llevan una etiqueta amarilla *brillo* en la pestaña Materiales:

![Materiales de la carpeta night en la pestaña Materiales, donde apartment_lit, neon_red y shop_sign llevan una etiqueta amarilla de brillo](../assets/materials/glow-tags.png)

Pulsa F3 para la Vista previa iluminada y verlos brillar en la vista 3D. El bloom a su alrededor solo se ve en Godot, mira [Iluminación nocturna](night-lighting.md).

## Transparencia { #transparency }

Pon `transparency` en el archivo del material cuando el canal alfa de la textura deba dejar ver lo de detrás. Cada modo cambia calidad por coste de forma distinta:

| `transparency` | Cómo funciona | Úsalo para | Ojo con |
| --- | --- | --- | --- |
| `1` blend | Mezcla la superficie con lo que hay detrás | Cristal, manchas suaves, humo | No proyecta sombras |
| `2` scissor | Los píxeles se muestran enteros o se recortan | Recortes duros de cerca, como rejillas y hojas | Las partes finas desaparecen a distancia |
| `3` hash | Tramea los píxeles, cuanto menor el alfa menos se muestran | Recortes finos vistos de lejos, como vallas de alambre | Los bordes suaves se vuelven ruido |

El tooltip de la pestaña Materiales te dice qué modo usa un material:

![La pestaña Materiales con el tooltip de night/chainlink: alfa hash, mapa normal, doble cara](../assets/materials/tooltip-alpha-hash.png)
