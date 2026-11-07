---
source: 3df4b9a4462dd78307ebb473881223aa02ea1321
---

# Iluminación nocturna { #night-lighting }

Para un mapa de noche bajas la luz global, así que hacen el trabajo las lámparas que colocas.

## Oscurece el mapa { #darken-the-map }

Pulsa Escape, o haz clic en un espacio vacío de una vista, para que no haya nada seleccionado. El Inspector de la derecha muestra entonces *Mapa (worldspawn)*, los ajustes de todo el mapa.

Pon las claves de abajo. Si una clave aún no aparece en la lista, escríbela en el campo *key* de abajo, su valor al lado, y haz clic en **Añadir**.

| Clave | Por defecto | Qué hacer para la noche |
| --- | --- | --- |
| `sun_energy`, `sun_color` | 1.1, `255 245 224` | Baja la energía y elige un azul pálido, y el sol se lee como una luna |
| `ambient_energy`, `sky_energy` | 1.0, 1.0 | Baja las dos, para que las zonas sin luz y el cielo se oscurezcan |
| `glow_intensity` | 0 | Por encima de 0, las lámparas y los [materiales brillantes](materials.md#glowing-materials) hacen bloom en Godot |
| `ssr` | 0 | 1 activa los reflejos en espacio de pantalla, para que las calles mojadas y los suelos brillantes reflejen las lámparas |

Esto es lo que usa el showcase del distrito nocturno:

![El Inspector mostrando Mapa (worldspawn) del distrito nocturno: ambient_energy 0.8, glow_intensity 0.9, sky_energy 0.6, un sun_color azul pálido y sun_energy 0.16](../assets/night-lighting/worldspawn-night.png)

## Compruébalo en la Vista previa iluminada { #check-it-in-lit-preview }

Pulsa F3, o haz clic en Vista previa iluminada en la barra de herramientas, para ver el mapa con sus luces, su cielo y su niebla.

![La barra de herramientas con el botón Vista previa iluminada resaltado y su tooltip, Sombreado: Vista previa iluminada (F3)](../assets/night-lighting/toolbar-lit-preview.png)

Deberías ver las lámparas, las ventanas iluminadas y los carteles brillantes destacar contra un cielo oscuro:

![La calle del distrito nocturno en la Vista previa iluminada: farolas, ventanas iluminadas y el cartel brillante Corner Shop bajo un cielo oscuro](../assets/night-lighting/street-lit-preview.jpg)

La Vista previa iluminada muestra todas las claves de arriba salvo el brillo y los reflejos, así que compruébalos en Godot. Para obtener la luz de rebote de las lámparas sin pagarla en cada fotograma, [precalcula la iluminación](../editor/light-baking.md).

## Lámparas { #lamp-fixtures }

Una lámpara es una entidad `light` o `light_spot` más la geometría de la lámpara con su bombilla brillante. Dale a esa geometría un targetname y ponlo en la clave `fixture` de la luz (un `*` al final coincide con un prefijo).

La geometría sigue entonces a la luz sin cableado: se muestra mientras la luz está encendida y se oculta mientras está apagada. Pon `fixture_off` a `dark` para mantenerla visible con su brillo apagado.

![El Inspector de una luz con targetname alley_lamp, fixture alley_bulb y fixture_off puesto a dark (emisión apagada)](../assets/night-lighting/light-fixture-keys.png)

Mira [Props, luces y efectos](../gameplay/entities/effects.md) para las demás claves de luz.

<div class="td-mcp" markdown>

## MCP { #mcp }

Los mapas de showcase se construyen con [scripts MCP](../mcp-scripts.md) que controlan el editor mediante su [servidor MCP](../mcp.md). `examples/mcp/night_district.json` y `examples/mcp/withered_city.json` son mapas nocturnos completos construidos así.

</div>
