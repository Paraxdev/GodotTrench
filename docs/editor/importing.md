# Importing Hammer and TrenchBroom maps

*File > Import* opens a Hammer `.vmf` or a TrenchBroom, Quake or Qodot `.map` as a new unsaved map. Saved as `.gtm`,
Godot builds it like any other map.

## Steps

1. Open the Godot project the map is for, converted textures go into its texture folder.
2. *File > Import > .vmf (Hammer)* or *.map (TrenchBroom, Quake)* and pick the file.
3. When textures for the map are found next to it, the editor offers to convert the ones the project lacks. Say yes.
4. Check the Issues panel. `missing_material` lists every material the project still lacks, with how many faces use
   it. Those faces keep the name, so adding the texture later fixes them.
5. *File > Save As* a `.gtm` inside the project.

Textures are covered in [Converting textures](importing-textures.md).

## What the import keeps

| Hammer | In GodotTrench |
| --- | --- |
| Tool materials (`tools/toolsnodraw`, `toolsskybox`, `toolsclip`, `toolsplayerclip`, `toolstrigger`, `toolsorigin`, ...) | The project's skip, clip and origin textures and `special/trigger` |
| Displacements | Displacements with their heights and blend alphas |
| Displacements whose vertices move sideways | Smooth meshes with the exact positions, since a displacement here only moves along the face normal |
| `func_instance` | A group with the instance's contents, placed, renamed by its fixup and with `$` and `#` replacements applied. `instance:name;Input` wiring is resolved |
| Visgroups | Layers |
| Hidden objects | Hidden nodes |
| Entity keys and outputs | Entity properties and outputs |

TrenchBroom layers and groups stay layers and groups. Quake tool textures (`skip`, `clip`, `hint`, `trigger`,
`origin`, Quake 3 `common/caulk`) become the project's tool textures the same way.

> **Note:** Units are kept as they are. A Source map is built for about 40 units per meter and a Quake map for about
> 32, so a Source map comes out a little larger than in its game unless the project uses a different scale.

## Known limitations

| Limitation | What to do |
| --- | --- |
| Textures inside `.vpk` or `.pak` archives are not read | Extract them first, for example with VPKEdit |
| Instances are looked for next to the map and in the folders above it | Keep the game's `maps` folder layout, missing ones stay `func_instance` entities |
| Valve and Quake entity classes have no definitions in a new project | Add them to your FGD, or replace them, the Issues panel lists them |
| `$basetexture2` blends, detail textures, env maps and proxies are left out | Set them up in Godot |
| Quake 3 patches and brush primitives are skipped, the status bar counts them | Rebuild them as meshes |
| A displacement turned into a mesh keeps no blend alphas and no longer sculpts | Edit it as a mesh |
