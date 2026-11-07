---
source: 2de051fc5fd93ade8349b194b97a6c34a8399111
---

# Decals proyectados { #projected-decals }

Un decal proyectado es una caja que proyecta un material sobre todo lo que hay dentro de ella. A diferencia de un [decal normal](decals.md), que es una lámina plana, este sigue lo que toca, así que una mancha puede derramarse sobre un escalón, rodear una tubería o tumbarse sobre un suelo irregular. Los props y modelos dentro de la caja también lo reciben.

En la escena construida es un nodo `Decal` de Godot, y cuesta más de dibujar que un decal normal. Úsalo donde una lámina plana no sirva.

> [!WARNING]
> Los decals proyectados son más duros para la GPU que los [decals normales](decals.md). Godot calcula el decal en cada píxel que su caja cubre en pantalla, así que muchas cajas, o cajas grandes, ralentizan el juego, sobre todo en tarjetas gráficas más modestas. Usa un decal normal sobre una superficie plana, mantén la caja lo más pequeña que permita el trabajo, y reserva los decals proyectados para los sitios que necesiten que el material se adapte a la forma.

## Añadirlo a tu proyecto { #adding-it-to-your-project }

El decal proyectado viene con el [paquete de entidades de gameplay](../godot/project-setup.md#core-entities-and-the-gameplay-entities-pack) como `infodecal`, así que prepararlo es instalar el paquete.

1. Instala o actualiza el paquete

    Haz clic en **Instalar entidades de gameplay** en la ventana *Set up this project*, o en **Actualizar entidades de gameplay** en el menú Gameplay cuando el paquete ya está en tu proyecto. [Instalar y actualizar el paquete](../godot/project-setup.md#installing-and-updating-the-pack) te guía paso a paso.

2. Comprueba que el editor lo ve

    Deberías ver una tarjeta `infodecal` en la pestaña **Entidades**, junto al Inspector. Escribe `infodecal` en el cuadro de búsqueda de arriba de la pestaña para encontrarla rápido.

    ![La pestaña Entidades buscando infodecal, con la tarjeta bajo el cursor y su tooltip diciendo Projected decal](../assets/projected-decals/entities-infodecal-card.png)

> [!NOTE]
> Si hiciste tu propia definición de `infodecal` antes de que el paquete tuviera una, quítala de tu FGD después de actualizar. Si no, la clase queda definida dos veces y el editor te avisa.

> [!TIP]
> ¿No usas el paquete? Copia su definición,
> [`infodecal.tres`](https://github.com/Paraxdev/trenchdot_func/blob/main/gameplay_pack/definitions/infodecal.tres),
> a tu proyecto y añádela a *Entity Definitions* de tu propio archivo FGD, mira [Entidades propias](../gameplay/custom-entities.md#making-it-placeable). Deja *Node Class* en `Decal`, el script lo necesita y es como el editor sabe que debe tratar la entidad como un decal.

## Colocar uno { #placing-one }

1. Arrastra la tarjeta sobre una superficie

    Abre la pestaña **Entidades** junto al Inspector y arrastra la tarjeta `infodecal` a una vista, como colocas cualquier entidad (mira [Colocar entidades](../gameplay/placing.md#placing-entities)). Aterriza donde el cursor toca la primera superficie sólida, girada para proyectar hacia esa superficie. Una caja muestra el espacio que cubre, la mitad delante de la superficie y la mitad detrás.

    ![Un infodecal en el borde de un escalón de ladrillo en la vista 3D, con su caja pasando por encima y por el frente, y el material del cartel previsualizado dentro](../assets/projected-decals/decal-box-on-step.png)

2. Elige el material

    En el Inspector, escribe un nombre de material en `material`, el mismo nombre que usa una cara, como `base/stain`. Su color, mapa normal y brillo se aplican.

    ![El Inspector con el infodecal seleccionado, night/shop_sign en material y 128 64 64 en size](../assets/projected-decals/decal-inspector-material.png)

3. Dale tamaño

    `size` toma tres números en unidades de mapa: ancho, profundidad y alto. El ancho y el alto son el parche que cubre sobre la superficie, la profundidad es cuánto entra y sale de ella la caja. El valor por defecto es 64 32 64, un cuadrado de dos metros que alcanza medio metro hacia cada lado. Si lo dejas en el valor por defecto, el parche toma en cambio el [tamaño de textura](../godot/materials.md#texture-size) del material, si lo tiene, tanto en el editor como en Godot.

4. (Opcional) Oriéntalo

    Rótalo como cualquier otra entidad. Siempre proyecta a lo largo de su propio eje hacia abajo, -Y local, que el soltado apuntó hacia la superficie.

5. Construye el mapa en Godot

    El material se ve en cada superficie dentro de la caja cuando Godot construye el mapa, mira [Construir mapas](../godot/building.md).

    El editor solo muestra una vista previa plana del material dentro de la caja, Godot dibuja la proyección real, doblando el cartel sobre el borde del escalón y por su frente.

    ![El mismo cartel en un juego de Godot en ejecución, doblado sobre el borde del escalón de ladrillo y sobre su cara frontal](../assets/projected-decals/decal-in-game.png)

## Claves { #keys }

| Clave | Qué hace |
| --- | --- |
| `material` | El material que proyectar, un nombre como `base/stain` o una ruta `res://`. Se usan sus texturas de albedo, normal, ORM y emisión y su color de albedo |
| `texture` | Una ruta `res://` a una imagen, que se proyecta cuando `material` está vacío |
| `normal_texture` | Una ruta `res://` a un mapa normal, para usar con `texture`. El mapa normal propio de un material lo reemplaza |
| `size` | Ancho, profundidad y alto de la caja en unidades de mapa, 64 32 64 salvo que lo cambies |
| `modulate` | Un color por el que se multiplica la imagen proyectada |

## Cosas a tener en cuenta { #things-to-watch-out-for }

| Situación | Qué pasa |
| --- | --- |
| El proyecto usa el renderizador Compatibility | Godot no dibuja ningún decal. Cambia *Rendering Method* a Forward+ o Mobile en *Project Settings > Rendering > Renderer* |
| El proyecto usa el renderizador Mobile | Solo se ven 8 decals en cada malla, el resto se omite |
| Caras que van a lo largo de la proyección | La textura se estira en vetas, como en el frente de un escalón bajo un decal que apunta recto hacia abajo. Inclina la caja para que encuentre ambas caras en ángulo |
| Superficies cerca de los extremos de la caja | El decal se desvanece hacia la parte de arriba y de abajo de su caja, así que mantén la superficie cerca del medio, donde la deja el soltado |
| Algo se mueve por la caja | También recibe el decal, un jugador que pase incluido. Mantén la caja no más profunda de lo que necesita la superficie |
| Cristal y recortes | Siguen igual de transparentes que antes, un decal no puede rellenar las partes transparentes |
| Muchos decals grandes | Cada uno añade trabajo a cada píxel que cubre, así que ralentizan el juego donde se acumulan |

<div class="td-mcp" markdown>

## MCP { #mcp }

Por el [servidor MCP](../mcp.md), `run_action` con `action: "place_entities"` y `args: {"classnames": ["infodecal"], "at": [x, y, z], "normal": [x, y, z]}` lo coloca girado hacia la superficie, como hace un arrastre. Define sus claves después con `update_entity` `properties`.

</div>
