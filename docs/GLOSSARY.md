# Glossary

## 3D cursor
The point in the world under the mouse pointer, in whichever view you hovered last. Commands that place something at the cursor, such as the Place button in the Entities panel or a new terrain, put it there.

## activator
The player or body that started a chain of entity I/O, usually whoever pressed the button or walked into the trigger. Later steps in the chain can target !activator to act on exactly that one.

## albedo
The base color of a material, the plain texture before lighting, normal maps or glow are added. Godot calls the color texture of a material its albedo texture.

## alpha scissor
A Godot transparency mode that cuts a texture off hard at an alpha threshold, so every pixel is either fully shown or fully hidden. It suits grates, leaves and hard edged decals, and unlike alpha blending it still casts shadows.

## autoload
A script or scene that Godot loads once at startup and keeps alive for the whole game, set up under Project Settings > Globals > Autoload. Game wide code, such as a score keeper or the hot reload listener, lives in one.

## bloom
The soft glow that bright lamps and glowing materials spread into the pixels around them. It is switched on with the worldspawn key glow_intensity.

## brush
A convex solid, such as a box or a wedge, the basic building block of a level. You draw brushes in the views and combine them with CSG, and Godot turns each one into a mesh with matching collision.

## brush entity
An entity that takes its shape from brushes you built, such as a door, a button or a trigger. You draw the brushes first and then turn them into the entity, unlike a point entity, which sits at a single spot.

## convex
Bulging outward everywhere, with no dents or hollows, like a box, a wedge or a cylinder. Every brush has to be convex, which is why the editor refuses vertex moves that would dent one. For concave shapes, convert the brush to a mesh.

## cordon
A box that limits the map to one area while you work on it. With the cordon enabled, objects outside the box are hidden and left out of a cordoned export.

## CSG
Constructive solid geometry, building shapes by combining solids, such as subtracting one brush from another to cut a doorway. The editor's CSG commands are Subtract, Hollow, Convex Merge and Intersect.

## decal
A texture laid over a surface, like a stain, a crack or a poster. In GodotTrench a decal is a thin mesh that sits on the face without flickering, placed by holding Alt while you drop a material on a face.

## detile
A terrain layer setting from 0 to 1 that turns and shifts every repeat of the texture at random, so a large field does not show an obvious grid. It costs extra texture reads, so leave it at 0 where the repeat does not show.

## displacement
A four sided brush face split into a grid of vertices that can each be raised or lowered, as in Hammer. It gives uneven ground inside a brush built level. Large outdoor areas are better built as a terrain.

## falloff
How a paint brush, such as the Blend tool's, fades from its center to its edge. A smooth falloff gives soft borders, a constant one paints the whole disc at the same strength.

## FGD
The definition of a game's entity classes, with their names, keys, inputs and outputs. The name comes from Hammer's Forge Game Data files. In Godot it is a FuncGodotFGDFile resource, and the game config passes it on to the editor.

## fixup
A name prefix set on a prefab instance. With a fixup of p1, an entity named door inside it becomes p1-door, so several copies of one prefab keep their wiring separate.

## func_detail
A FuncGodot brush entity for static geometry with collision, built as its own node apart from the world brushes. It adds no occluder, so it never hides the objects behind it from rendering.

## func_illusionary
A FuncGodot brush entity that is visible but has no collision, so players pass through it. The sea island tutorial uses one for the water.

## FuncGodot
The Godot plugin for building levels from Quake style map files. The GodotTrench addon is a fork of it, which is why its node is called FuncGodotMap and its resources start with FuncGodot.

## game config
The file godottrench_game.json in the Godot project root, written by the addon. It tells the editor which entities, texture folders and unit scale the project uses, and the editor reloads it whenever the addon exports it again.

## Godot group
A named tag on nodes in Godot, set under Node > Groups or from code, that lets code and entity I/O find all of them at once. A target such as @enemies reaches every node in the enemies group.

## Hammer
Valve's level editor for Source engine games such as Half-Life 2. GodotTrench borrows its model of entity inputs and outputs and can import its .vmf maps.

## hot reload
Rebuilding the maps of a game that is already running each time you save in GodotTrench, so you do not have to restart it to see a change. It needs the godottrench_hot_reload.gd autoload in the game.

## hotspot
A rectangle marked on a trim sheet and stored in the texture's .hotspots.json file. Hotspot Fit snaps the selected faces to the rectangle that matches them best, so a trim sheet can texture a room quickly.

## Lit Preview
A shading mode of the editor's 3D view, toggled with F3, that shows the map's lights, sun shadows, sky and fog. It lets you judge lighting before building in Godot, but it draws only the 64 strongest point and spot lights.

## live link
The connection between GodotTrench and the Godot editor, a local server the addon runs on port 7842. Rebuilding on save, live mode and the Godot status in the toolbar all go through it.

## live mode
Pushes your edits into the scene open in the Godot editor as you make them, before you save, so you can judge lighting and scale in Godot's own renderer. Turn it on with the link button in the toolbar or Godot > Live Mode.

## map settings
A FuncGodotMapSettings resource that says where textures, materials and entity definitions come from and how many map units make a meter. A FuncGodotMap can have its own, otherwise the project default is used.

## MCP
Model Context Protocol, a standard way for AI agents and scripts to call a program's tools. The editor can run an MCP server so an agent can inspect and edit maps. You do not need it to build levels, and these docs hide the MCP parts until you turn on the MCP switch in the header.

## MultiMeshes
Godot's way of drawing many copies of one mesh in a single pass. Scattered trees, rocks and grass are built as MultiMeshes, which keeps thousands of them cheap to render.

## navigation mesh
A simplified outline of the floors that AI agents can walk on, which Godot uses to find paths. GodotTrenchNav bakes one from the collision of a built map.

## normal map
A texture that stores which way each pixel of a surface faces, so flat faces catch the light as if they had bumps and grooves. It adds fine detail without extra geometry.

## ORM
A texture that packs three grayscale maps into its color channels, ambient occlusion in red, roughness in green and metallic in blue. Godot materials and GodotTrench terrain layers read it as one file.

## point entity
An entity that sits at a single spot instead of taking its shape from brushes, such as a light, a sound or a spawn point. Place one by dragging it from the Entities panel into a view.

## prefab
A reusable piece of a map saved as its own .gtm file and placed in other maps as instances. Editing the prefab file updates every instance of it.

## scatter set
A group of models painted together with the Scatter tool, such as a forest or a meadow of grass, with its own layer, density and target surfaces. Godot builds each set as MultiMeshes.

## screen space reflections
A Godot effect that mirrors what is already on screen in shiny surfaces, so wet streets reflect the lamps. Turn it on with the worldspawn key ssr set to 1.

## signal
Godot's name for an event a node announces, such as a button being pressed. Every entity output is a signal, and wiring an output connects that signal to an input method on the target.

## targetname
The name you give an entity so other entities can reach it, like main_door. Outputs, prefab fixups and keys such as fixture find entities by it.

## texel
One pixel of a texture as it lands on a surface. By default one texel covers one map unit, and the texel density, how many texels fit on an area, decides how sharp a texture looks.

## texture size
The world size that one repeat of a texture covers, set as metadata/texture_size on a Godot material. Without it one pixel covers one map unit, which stretches a high resolution photo over a huge area.

## tool textures
Special textures that tell the build what a brush face is for instead of drawing it. Clip faces only collide, skip faces are left out of the mesh, and origin faces set where a brush entity's origin is. The map settings name them, special/clip, special/skip and special/origin by default.

## TrenchBroom
A free level editor for Quake style games. GodotTrench works much like it and can import its .map files.

## trigger volume
An invisible brush entity that fires outputs when a body enters or leaves it, for example to start a cutscene when the player walks into a room. Draw one with the Volume tool (Shift+E).

## trim sheet
A texture that packs strips of detail, such as edges, pipes and panels, into one image. Faces are fitted to its parts with hotspots, so a few trim sheets can texture a whole building.

## triplanar
A way of projecting a texture onto a surface from three directions, from above and from two sides, blended by the angle of the surface. Terrain layers use it so cliffs show rock instead of a stretched smear.

## UV lock
A setting, toggled with Ctrl+Shift+U and on by default, that keeps textures attached to a brush while you move it. With it off, the texture stays put in the world and slides across the moved faces.

## UV
The coordinates that say which part of a texture lands on each point of a face, U across the texture and V down it. Brush faces usually get them from a projection, while meshes and some faces store their own for each corner.

## worldspawn
The map's root entity, which holds the world brushes. Its keys hold settings for the whole map, such as the sky, fog, sun and streaming, and the Inspector shows it when nothing is selected.

## yaw
Rotation around the vertical axis, the direction something faces when seen from above. A teleport destination's yaw sets which way bodies face after arriving.
