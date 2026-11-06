# Projected decals

A projected decal is a box that shines a material onto everything inside it. Unlike a [regular decal](decals.md),
which is a flat sheet, it follows whatever it hits, so a stain can spill over a step, wrap around a pipe or lie on
bumpy ground. Props and models inside the box get it too.

It is a Godot `Decal` node in the built scene, and it costs more to draw than a regular decal. Use it where a flat
sheet will not do.

## Adding it to your project

TrenchDot ships the script, `trenchdot_decal.gd`, but no entity class that uses it, so you add one once per project.
(The demo project has one already, called `infodecal`.)

1. Save this definition as `res://entities/infodecal.tres`

    ```
    [gd_resource type="Resource" script_class="FuncGodotFGDPointClass" load_steps=3 format=3]

    [ext_resource type="Script" path="res://addons/func_godot/src/fgd/func_godot_fgd_point_class.gd" id="1_point"]
    [ext_resource type="Script" path="res://addons/func_godot/src/trenchdot/runtime/trenchdot_decal.gd" id="2_decal"]

    [resource]
    script = ExtResource("1_point")
    classname = "infodecal"
    description = "Projected decal, shines a material onto everything inside its box"
    script_class = ExtResource("2_decal")
    class_properties = Dictionary[String, Variant]({
    "material": "",
    "texture": "",
    "normal_texture": "",
    "size": Vector3(64, 32, 64),
    "modulate": Color(1, 1, 1, 1)
    })
    meta_properties = Dictionary[String, Variant]({
    "size": AABB(-8, -8, -8, 8, 8, 8),
    "color": Color(0.9, 0.3, 0.9, 1)
    })
    node_class = "Decal"
    ```

    Any classname works. Keep *Node Class* at `Decal`, the script needs it and it is how the editor knows to treat the
    entity as a decal.

2. Add it to your FGD

    Add the `.tres` to *Entity Definitions* of your own FGD file, the one your map settings and game config both use.
    The addon's default FGD gets replaced on every update, so if you do not have your own yet, set one up as shown in
    [Project setup](../godot/project-setup.md#your-own-resources). More on definitions is in
    [Custom entities](../gameplay/custom-entities.md#making-it-placeable).

3. Check that the editor sees it

    Godot exports the game config again when it notices the new file, and the editor reloads it within a second. You
    should see an `infodecal` card in the **Entities** tab.

## Placing one

1. Drag the card onto a surface

    Open the **Entities** tab next to the Inspector and drag the `infodecal` card into a view, the way you place any
    entity (see [Placing entities](../gameplay/placing.md#placing-entities)). It lands where the pointer meets the
    first solid surface, turned so it projects into that surface. A box shows the space it covers, half in front of
    the surface and half behind it.

2. Pick the material

    In the Inspector, type a material name into `material`, the same name a face uses, like `base/stain`. Its color,
    normal map and glow carry over.

3. Size it

    `size` takes three numbers in map units: width, depth and height. Width and height are the patch it covers on the
    surface, depth is how far the box reaches into and out of it. The default is 64 32 64, a two meter square that
    reaches half a meter each way. Leave it at the default and Godot sizes the patch by the material's
    [texture size](../godot/materials.md#texture-size) instead, if it has one. (Note: the editor still draws the
    default box then.)

4. (Optionally) aim it

    Rotate it like any other entity. It always projects along its own down axis, local -Y, which the drop pointed
    into the surface.

5. Build the map in Godot

    The material shows on every surface inside the box once Godot builds the map, see
    [Building maps](../godot/building.md).

## Keys

| Key | What it does |
| --- | --- |
| `material` | The material to project, a name like `base/stain` or a `res://` path. Its albedo, normal, ORM and emission textures and its albedo color are used |
| `texture` | A `res://` path to an image, projected when `material` is empty |
| `normal_texture` | A `res://` path to a normal map, for use with `texture` |
| `size` | Width, depth and height of the box in map units |
| `modulate` | A color the projected image is multiplied by |

## Things to watch out for

| Situation | What happens |
| --- | --- |
| The project uses the Compatibility renderer | Godot draws no decals at all. Switch *Rendering Method* to Forward+ or Mobile under *Project Settings > Rendering > Renderer* |
| The project uses the Mobile renderer | Only 8 decals show on any one mesh, the rest are left out |
| Faces that run along the projection | The texture smears into streaks, like on the front of a step under a decal that points straight down. Tilt the box so it meets both faces at an angle |
| Surfaces near the ends of the box | The decal fades out toward the top and bottom of its box, so keep the surface near the middle, where the drop puts it |
| Something moves through the box | It picks up the decal too, a player walking past included. Keep the box no deeper than the surface needs |
| Glass and cutouts | They stay as see-through as before, a decal cannot fill in transparent parts |
| Many large decals | Each one adds work to every pixel it covers, so they slow the game down where they stack up |

<div class="td-mcp" markdown>

## MCP

Over the [MCP server](../mcp.md), `run_action` `place_entities` with `classnames: ["infodecal"]`, `at` and `normal`
places it turned to the surface, like a drag does. Set its keys afterwards with `update_entity` `properties`.

</div>
