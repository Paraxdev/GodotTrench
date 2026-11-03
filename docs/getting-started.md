# Getting started

Steps 1 - 3: First Setup, if you already have a Godot project open you may skip this

Steps 4 - 5: Creating the first room in TrenchDot

## What you need

Godot 4.7 or newer (.NET if you want C# entities)
Use the .NET build if you want to write entities in C#
newest TrenchDot release

## Ensure TrenchDot can see the Editor executable

Make sure TrenchDot can see your Godot install, to check if everything is correctly detected 

go to File > Preferences > Godot Executable

if TrenchDot doesn't find the executable the input field is empty, to continue you need to find the 
executable on your disk.

for this click the "..." button

![screenshot of the ... button besides the input field](assets/getting-started/godot-path-browse.png)

and find the executable starting with "Godot_v"

if you have the correct executable it should say "Using: \<path you just picked>"

![ui text beneath the input field, saying using: and the path to the executable](assets/getting-started/godot-path-using.png)


## 1. Create a Project in Godot

Open Godot and Click "Create" at the top left corner
![godot project manager ui, top left corner button saying create with a plus icon](assets/getting-started/godot-create-project.png)

Pick a Project name, for this tutorial it will be TrenchDotTest
(optionally) select a folder where the project should live

then hit "Create"

## 2. Opening the Project within TrenchDot

Choose *Godot > Open Godot Project…*
![TrenchDot navbar item Godot with the sub element "open godot project" highlighted](assets/getting-started/menu-open-godot-project.png)
or the **Open Godot Project…** button in the toolbar
![button and indicator at the top right of the godot trench ui indicating the project name / status](assets/getting-started/toolbar-open-godot-project.png)
and pick the project folder or any folder inside it. The editor reopens it on the next start.

## 3. The Welcome Screen

Once TrenchDot detects that its a new Godot project it will show a welcome / setup screen
![The Set up this project window with the addon, gameplay entities and ready made content sections](assets/getting-started/setup-window.png)

Here you can download the addon which is needed for the communication between Godot and TrenchDot
and extra content if you want, you can ofc start without anything if you want.

some bare bones content will be included, a simple dummy info_player_start and func_light

1. Install the Addon
   
   Click the "install the addon" button at the top to download TrenchDot's Fork of FuncGodot into your project, this crucial for TrenchDot.
   (Note: This will pull from GitHub releases, if you don't have any internet access make sure to download the addon locally first, extract it into the project root to install it locally) 

   Once you did that click "Enable Plugin" so Godot loads it

2. (Optionally) Extra Content

   TrenchDot has optional CC0 licensed content you may preload, sourced from polyheaven and other sites with retro textures.
   licenses are included and a list from where everything is from

   If you do not want any extra content you can go with "Nothing extra", some minimal entities are still included.
   
if you want to add entities you may use the GitHub source code as reference or go through the addon source and view how the included ones are structured.

## 3. Open the project in Godot once

the addon creates a file called `trenchdot_game.json` in the project root, this is where the shared settings live if you ever want to change anything.
this also is where metadata like textures and entities live, for info on how to add entities that are more complex visit:
[Project setup](godot/project-setup.md) 

The addon then writes `trenchdot_game.json` to the project root, which is how the editor learns your entities and
textures, and the editor loads it within a second. From then on the addon exports again whenever files change. Until
then the status bar says there is no `trenchdot_game.json`, and the editor offers the core entities and the pack if
you installed it. 


## 4. Build a room

The editor has 4 main views, a 3D view you can fly around just like in Godot, and 3 wireframe views from the top, front and side view.
Brushes and operations are dependant on the grid size, in the toolbar you can set your grid size (smaller is finer)
![toolbar showing the grid scale setting](assets/getting-started/toolbar-grid-size.png)

On a empty level you will be shown a small info text explaining how to draw, you can either hide this or just click anywhere to hide it
![A new map: the card in the 3D view says to drag in the Top view to draw the first box, and the Top view shows a yellow "Drag here to draw your first box" label](assets/getting-started/empty-map.png)

1. Draw your first box
   
   With the Select tool (Q), drag on empty space in any view
   you should see a box being drawn, while holding your left click down move the mouse to shape a rect
   ![top view showing a rect being drawn](assets/getting-started/draw-first-box.png)
   
2. Add Depth
   
   In the side view hold your mouse at the top of the box and hold down lclick, then move your mouse up to have a depth of 256 units
   ![side view showing 256 on the side for the depth](assets/getting-started/side-view-depth.png)

3. Making it Hollow
   
   Press Ctrl+Shift+K, or use the Hollow operation in the toolbar
   ![toolbar showing the hover option](assets/getting-started/toolbar-hollow.png)
   The box now turned into a segmented box, this is optimal if you want to make a smaller room or corridor
   now, using the fly controls fly into the created hollow box

4. Cutting a doorway
   
   Using "Escape" you can deselect the selected solids, once nothing is selected draw a smaller vertical rect
   within the box, shaped like a doorway
   ![brush in a shape of a vertical laying rect](assets/getting-started/doorway-brush.png)
   and in the side view extend one side of the door brush so it has a depth
   ![extending the face of the door cutaway](assets/getting-started/doorway-extend.gif)

   now with the brush selected hit Ctrl + K or use the Subtract tool in the toolbar
   ![subtract tool in the toolbar](assets/getting-started/toolbar-subtract.png)

   and you should see the brush should vanish and the intersection of the doorway cutout is removed
   ![cutaway removed and the outline of the door was cut away](assets/getting-started/doorway-cut.png)

5. Texture it 

   You should have 5 included textures used in prototyping, to apply a texture to just one face (e.g. the door frame)
   you drag the texture from the Materials tab to the face
   ![dev texture drop preview](assets/getting-started/texture-drop.png)

6. Saving the Map

   To see the changes in Godot we have to save the map, both the Addon and TrenchDot expect maps (.tdm) files to be within res://maps (maps folder within the project root)



## 5. See it in Godot

In Godot, a level is a scene: a tree of nodes, saved as a `.tscn` file, where each node is one thing such as a mesh, a
light or a camera. Godot writes paths inside the project with `res://`, so `res://maps/room.tdm` is the file
`maps/room.tdm` in your project folder. The map is turned into nodes by one node from the addon, `FuncGodotMap`.

1. In the Godot editor, choose *Scene > New Scene* and click **3D Scene** in the Scene panel on the left.
2. Press Ctrl+A (*Add Child Node*), search for `FuncGodotMap` and add it.
3. With the new node selected, set *Local Map File* to your `.tdm` in Godot's own Inspector, the panel on the right of
   the Godot editor. It works like the Inspector in TrenchDot but belongs to Godot.
4. Press **Build Map** at the top of that Inspector, then save the scene with Ctrl+S.

Brushes become meshes with collision and entities become nodes with their scripts, saved with the scene.
[Building maps](godot/building.md) explains the node's other settings.

Leave that scene open in Godot and every save in TrenchDot rebuilds it, straight away while its tab is the current
one, or when you switch back to it. Save the scene in Godot to keep the result. If a save does not rebuild, the status
bar says why, see [Rebuild on save](godot/live.md#rebuild-on-save).

> **Warning:** A build deletes the children of the `FuncGodotMap` first. Put your own nodes next to the map node, or in
> an [overlay](godot/overlays.md).




## Try the demo project

The repository's `godot` folder, also in the release as `trenchdot-demo-project.zip`, is a project with the addon
installed. Its main scene plays the showcase maps, which are saved in `godot/demo/maps/showcase`. Open the project in
the editor and pick one under *File > Maps in Project* to see how a finished map is put together.

The demo choice of *Godot > Add Content to Project…* puts the same maps and scenes into your own project. In Godot,
open `res://demo/demo.tscn` and press F6 to play them. Opened in TrenchDot from your project they show without their
textures, because they use the demo's texture folder `res://demo/textures` rather than your project's. The demo project
shows them as they were built.

| Key | Action |
| --- | --- |
| WASD, Space, Shift | Walk, jump, run |
| E | Use doors and buttons |
| 1 to 5 | Switch between the showcase maps |
| F3 | Show the I/O debug overlay, a log of every input and output as it fires, with trigger volumes drawn in |

## Next

To make the room do something, read [How entity I/O works](gameplay/io.md) and build
[Button opens a door](tutorials/button-door.md).

{% mcp %}

## MCP

The content question does not open by itself in an editor that an MCP client drives: the first tool call other than
`get_state`, `screenshot` and `simulate_input` closes it, and a project opened with `open_project` is not asked about.
`project_content` adds the same content, see [MCP](mcp.md).

{% endmcp %}
