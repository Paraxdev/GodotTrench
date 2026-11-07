---
source: 62dc81f5978f85583c7c7cd6083e20459a306923
---

# Glossaire { #glossary }

## Curseur 3D { #3d-cursor data-forms="curseurs 3D" }
Le point du monde sous votre souris, dans la vue que vous avez survolée en dernier. Tout ce que vous placez au curseur atterrit là, comme une entité du bouton Placer du panneau Entités ou un nouveau terrain.

## activateur { #activator data-forms="activateurs" }
Le joueur ou le corps qui a déclenché une chaîne d'E/S d'entités, en général celui qui a appuyé sur le bouton ou qui est entré dans le déclencheur. Les étapes suivantes de la chaîne peuvent cibler !activator pour agir exactement sur lui.

## albédo { #albedo data-forms="albedo" }
La couleur de base d'un matériau, la texture simple avant l'ajout de l'éclairage, des cartes normales ou de la lueur. Godot appelle la texture de couleur d'un matériau sa texture d'albédo.

## Alpha Scissor { #alpha-scissor }
Un mode de transparence de Godot qui coupe net une texture à un seuil d'alpha, si bien que chaque pixel est soit entièrement visible, soit entièrement caché. Il convient aux grilles, aux feuilles et aux decals à bords durs, et contrairement au mélange alpha il projette encore des ombres.

## Autoload { #autoload data-forms="Autoloads" }
Un script ou une scène que Godot charge une fois au démarrage et garde en vie pendant tout le jeu, réglé sous Project Settings > Globals > Autoload. Le code valable pour tout le jeu y vit, comme un compteur de score ou l'écouteur du hot reload.

## Éclairage précalculé { #baked-lighting }
De la lumière calculée une seule fois à l'avance et stockée, au lieu d'être recalculée à chaque image. Godot > Précalculer l'éclairage précalcule le soleil, le ciel, les lampes et les matériaux lumineux dans une lightmap, et Godot l'affiche avec un LightmapGI.

## bloom { #bloom }
La lueur douce que les lampes vives et les matériaux lumineux répandent dans les pixels autour d'eux. On l'active avec la clé worldspawn glow_intensity.

## booléen { #boolean data-forms="booléens" }
Une valeur qui vaut soit vrai, soit faux, par exemple si le courant est allumé. Les entités logic_branch, logic_gate et logic_flipflop stockent et combinent des booléens.

## brush { #brush data-forms="brushes" }
Un solide convexe, comme une boîte ou un coin, la brique de base d'un niveau. Vous dessinez des brushes dans les vues et vous les combinez avec le CSG, et Godot transforme chacun en mesh avec une collision correspondante.

## entité brush { #brush-entity data-forms="entités brush" }
Une entité dont la forme vient de brushes que vous avez construits, comme une porte, un bouton ou un déclencheur. Vous dessinez d'abord les brushes, puis vous en faites l'entité, contrairement à une entité point, qui se trouve à un seul endroit.

## convexe { #convex data-forms="convexes" }
Bombé vers l'extérieur partout, sans creux ni bosses rentrantes, comme une boîte, un coin ou un cylindre. Chaque brush doit être convexe, c'est pourquoi l'éditeur refuse les déplacements de sommets qui en feraient un creux. Pour les formes concaves, convertissez le brush en mesh.

## cordon { #cordon data-forms="cordons" }
Une boîte qui limite la carte à une zone pendant que vous travaillez dessus. Quand le cordon est activé, les objets hors de la boîte sont masqués et exclus d'un export avec cordon.

## CSG { #csg }
Constructive solid geometry, c'est-à-dire construire des formes en combinant des solides, par exemple en soustrayant un brush d'un autre pour percer une porte. Les commandes CSG de l'éditeur sont Soustraire, Évider, Fusion convexe et Intersection.

## Decal { #decal data-forms="decals" }
Une texture posée sur une surface, comme une tache, une fissure ou une affiche. Dans TrenchDot, un decal est un mesh fin qui se pose sur la face sans clignoter, placé en maintenant Alt pendant que vous déposez un matériau sur une face. Un decal projeté est une entité qui projette plutôt un matériau sur tout ce qui se trouve dans sa boîte.

## détuilage { #detile }
Un réglage de calque de terrain de 0 à 1 qui tourne et décale au hasard chaque répétition de la texture, pour qu'un grand champ ne montre pas de quadrillage évident. Il coûte des lectures de texture en plus, laissez-le à 0 là où la répétition ne se voit pas.

## Displacement { #displacement data-forms="displacements" }
Une face de brush à quatre côtés découpée en grille de sommets que l'on peut chacun monter ou descendre, comme dans Hammer. Elle donne un sol irrégulier dans un niveau construit en brushes. Pour les grandes zones en extérieur, mieux vaut construire un terrain.

## définition d'entité { #entity-definition data-forms="définitions d'entité|définition d'entités|définitions d'entités" }
La description d'une classe d'entité dans un FGD : son classname, ses clés, ses inputs et outputs, et ce que Godot construit pour elle, comme la classe de nœud et le script. Dans Godot, c'est une ressource FuncGodotFGDPointClass ou FuncGodotFGDSolidClass, listée sous Entity Definitions d'un fichier FGD.

## Atténuation { #falloff data-forms="atténuations" }
La façon dont un pinceau de peinture, comme celui de l'outil Mélange, s'estompe de son centre vers son bord. Une atténuation douce donne des bords souples, une atténuation constante peint tout le disque avec la même force.

## FGD { #fgd }
La définition des classes d'entités d'un jeu, avec leurs noms, clés, inputs et outputs. Le nom vient des fichiers Forge Game Data de Hammer. Dans Godot, c'est une ressource FuncGodotFGDFile, et la config de jeu la transmet à l'éditeur.

## fixup { #fixup data-forms="fixups" }
Un préfixe de nom réglé sur une instance de prefab. Avec un fixup p1, une entité nommée door à l'intérieur devient p1-door, si bien que plusieurs copies d'un même prefab gardent leur câblage séparé.

## func_detail { #func_detail }
Une entité brush de FuncGodot pour la géométrie statique avec collision, construite comme son propre nœud à part des brushes du monde. Elle n'ajoute aucun occluder, elle ne cache donc jamais à l'affichage les objets situés derrière.

## func_illusionary { #func_illusionary }
Une entité brush de FuncGodot visible mais sans collision, que les joueurs traversent. Le tutoriel de l'île en mer en utilise une pour l'eau.

## FuncGodot { #funcgodot }
Le plugin Godot pour construire des niveaux à partir de fichiers de carte de style Quake. L'addon TrenchDot en est un fork, c'est pourquoi son nœud s'appelle FuncGodotMap et ses ressources commencent par FuncGodot.

## Config de jeu { #game-config data-forms="configs de jeu" }
Le fichier trenchdot_game.json à la racine du projet Godot, écrit par l'addon. Il indique à l'éditeur quelles entités, quels dossiers de textures et quelle échelle d'unités votre projet utilise, et l'éditeur le recharge chaque fois que l'addon le réécrit.

## pack d'entités Gameplay { #gameplay-entities-pack data-forms="packs d'entités Gameplay" }
Les portes, boutons, ascenseurs, déclencheurs, logique, spawners, props et effets de la référence des entités. Ils ne font pas partie de l'addon lui-même. Installer le pack copie leurs définitions et leurs scripts dans res://trenchdot/entities, où ils appartiennent à votre projet et où vous pouvez les modifier librement.

## glTF { #gltf }
Un format de fichier ouvert pour les modèles et scènes 3D que Blender, Godot et la plupart des outils 3D savent lire. Un fichier .glb en est la forme binaire, qui contient la géométrie et les textures dans un seul fichier.

## groupe Godot { #godot-group data-forms="groupes Godot" }
Une étiquette nommée sur des nœuds dans Godot, réglée sous Node > Groups ou depuis le code, qui permet au code et aux E/S d'entités de tous les trouver d'un coup. Une cible comme @enemies atteint chaque nœud du groupe enemies.

## Hammer { #hammer }
L'éditeur de niveaux de Valve pour les jeux du moteur Source comme Half-Life 2. TrenchDot lui emprunte son modèle d'inputs et d'outputs d'entités et peut importer ses cartes .vmf.

## hot reload { #hot-reload }
Reconstruire les cartes d'un jeu déjà lancé à chaque enregistrement dans TrenchDot, pour que vous n'ayez pas à le redémarrer pour voir un changement. Il faut pour cela l'autoload trenchdot_hot_reload.gd dans le jeu.

## Hotspot { #hotspot data-forms="hotspots" }
Un rectangle marqué sur une trim sheet et stocké dans le fichier .hotspots.json de la texture. Ajustement au hotspot cale les faces sélectionnées sur le rectangle qui leur convient le mieux, si bien qu'une trim sheet permet de texturer une pièce rapidement.

## lightmap { #light-map data-forms="lightmaps|light map|light maps" }
Une texture qui contient la lumière arrivant en chaque point de la géométrie statique, rebonds compris. Chaque face en reçoit sa propre zone.

## sonde de lumière { #light-probe data-forms="sondes de lumière" }
Un point dans l'espace libre qui stocke la lumière arrivant de toutes les directions. Les objets mobiles tirent leur lumière des sondes autour d'eux, pour qu'ils s'accordent à l'éclairage précalculé de la géométrie statique.

## LightmapGI { #lightmapgi }
Le nœud Godot qui affiche une lightmap et ses sondes de lumière. Une carte construite à partir d'un .tdm précalculé en reçoit un nommé baked_lighting.

## Aperçu éclairé { #lit-preview }
Un mode d'ombrage de la vue 3D de l'éditeur, activé avec F3, qui montre les lumières de la carte, les ombres du soleil, le ciel et le brouillard. Pratique pour juger l'éclairage avant de construire dans Godot, mais il ne dessine que les 64 lumières ponctuelles et spots les plus fortes.

## live link { #live-link data-forms="live links" }
La connexion entre TrenchDot et l'éditeur Godot, un serveur local que l'addon lance sur le port 7842. La reconstruction à l'enregistrement, le mode live et l'état de Godot dans la barre d'outils passent tous par lui.

## Mode live { #live-mode }
Envoie vos modifications dans la scène ouverte dans l'éditeur Godot au fur et à mesure, avant que vous enregistriez, pour que vous puissiez juger l'éclairage et l'échelle dans le moteur de rendu de Godot. Activez-le avec le bouton de lien dans la barre d'outils ou avec Godot > Mode live.

## porte logique { #logic-gate data-forms="portes logiques" }
Une règle qui transforme deux entrées vrai ou faux en une seule réponse, comme et (les deux sont vraies) ou xor (exactement une l'est). L'entité logic_gate propose les plus courantes.

## scène principale { #main-scene data-forms="scènes principales" }
La scène que Godot joue quand vous lancez le projet, réglée sous Project > Project Settings > Application > Run. Lancer le projet dans TrenchDot et le bouton Play de Godot la démarrent tous les deux.

## réglages de carte { #map-settings }
Une ressource FuncGodotMapSettings qui indique d'où viennent les textures, les matériaux et les définitions d'entités, et combien d'unités de carte font un mètre. Une FuncGodotMap peut avoir les siens, sinon le réglage par défaut du projet est utilisé.

## MCP { #mcp }
Model Context Protocol, une façon standard pour les agents IA et les scripts d'appeler les outils d'un programme. L'éditeur peut lancer un serveur MCP pour qu'un agent puisse inspecter et modifier des cartes. Vous n'en avez pas besoin pour construire des niveaux, et cette documentation cache les parties MCP tant que vous n'activez pas le commutateur MCP dans l'en-tête.

## MultiMeshes { #multimeshes data-forms="MultiMesh" }
La façon dont Godot dessine de nombreuses copies d'un même mesh en une seule passe. Les arbres, rochers et herbes de scatter sont construits en MultiMeshes, ce qui permet d'en afficher des milliers à bas coût.

## maillage de navigation { #navigation-mesh data-forms="maillages de navigation" }
Un contour simplifié des sols sur lesquels les agents IA peuvent marcher, que Godot utilise pour trouver des chemins. TrenchDotNav en précalcule un à partir de la collision d'une carte construite.

## carte normale { #normal-map data-forms="cartes normales" }
Une texture qui stocke dans quelle direction regarde chaque pixel d'une surface, pour que des faces plates captent la lumière comme si elles avaient des bosses et des rainures. Elle ajoute du détail fin sans géométrie supplémentaire.

## opérande { #operand data-forms="opérandes" }
Le nombre avec lequel un calcul travaille en plus de celui qu'il reçoit. Dans un math_calc qui ajoute 5, l'opérande est 5.

## ORM { #orm }
Une texture qui range trois cartes en niveaux de gris dans ses canaux de couleur, l'occlusion ambiante dans le rouge, la rugosité dans le vert et le métal dans le bleu. Les matériaux Godot et les calques de terrain de TrenchDot la lisent comme un seul fichier.

## entité point { #point-entity data-forms="entités point" }
Une entité qui se trouve à un seul endroit au lieu de prendre sa forme de brushes, comme une lumière, un son ou un point d'apparition. Placez-en une en la faisant glisser du panneau Entités dans une vue.

## Prefab { #prefab data-forms="prefabs" }
Un morceau réutilisable de carte enregistré dans son propre fichier .tdm et placé dans d'autres cartes sous forme d'instances. Modifier le fichier du prefab met à jour chacune de ses instances.

## res:// { #res }
La façon dont Godot écrit un chemin dans le dossier du projet, quel que soit le nom du dossier sur le disque. res://maps/room.tdm est room.tdm dans le dossier maps de votre projet Godot. C'est pourquoi les cartes et les modèles doivent être enregistrés dans le projet pour que l'addon les trouve.

## Mode restreint { #restricted-mode }
La façon dont TrenchDot ouvre un projet Godot qui contient du code et auquel vous n'avez pas encore accordé votre confiance. Vous pouvez voir et modifier ses cartes, mais l'éditeur ne lance pas Godot dessus, ne construit pas dans Godot et n'envoie pas de modifications en direct, puisque Godot exécute les scripts d'un projet quand il l'ouvre, le lance ou le construit. Cliquez sur Mode restreint dans la barre d'état pour faire confiance au projet.

## ensemble de scatter { #scatter-set data-forms="ensembles de scatter" }
Un groupe de modèles peints ensemble avec l'outil Scatter, comme une forêt ou une prairie d'herbe, avec son propre calque, sa densité et ses surfaces cibles. Godot construit chaque ensemble en MultiMeshes.

## réflexions en espace écran { #screen-space-reflections data-forms="SSR" }
Un effet de Godot qui reflète ce qui est déjà à l'écran dans les surfaces brillantes, si bien que les rues mouillées reflètent les lampes. Activez-le avec la clé worldspawn ssr réglée sur 1.

## signal { #signal data-forms="signaux" }
Le nom que Godot donne à un événement qu'un nœud annonce, comme l'appui sur un bouton. Chaque output d'entité est un signal, et câbler un output connecte ce signal à une méthode d'input de la cible.

## targetname { #targetname }
Le nom que vous donnez à une entité pour que d'autres entités puissent l'atteindre, comme main_door. Les outputs, les fixups de prefab et des clés comme fixture trouvent les entités grâce à lui.

## texel { #texel data-forms="texels" }
Un pixel d'une texture tel qu'il tombe sur une surface. Par défaut un texel couvre une unité de carte, et la densité de texels, c'est-à-dire combien de texels tiennent sur une surface, décide de la netteté d'une texture. Une lightmap a aussi des texels, et sa taille de texel indique combien d'unités de carte l'un d'eux couvre.

## taille de texture { #texture-size data-forms="tailles de texture" }
La taille dans le monde que couvre une répétition d'une texture, réglée par metadata/texture_size sur un matériau Godot. Sans elle, un pixel couvre une unité de carte, ce qui étire une photo haute résolution sur une zone immense.

## textures d'outil { #tool-textures data-forms="texture d'outil" }
Des textures spéciales qui indiquent à la construction à quoi sert une face de brush au lieu de la dessiner. Les faces clip ne font que collisionner, les faces skip ne construisent rien, et les faces origin fixent où se trouve l'origine d'une entité brush. Toute texture des dossiers special/, tools/ ou td/ ne dessine rien dans Godot, de même que nodraw, hint, trigger et les autres noms d'outils dans n'importe quel dossier.

## TrenchBroom { #trenchbroom }
Un éditeur de niveaux gratuit pour les jeux de style Quake. TrenchDot s'en inspire beaucoup et peut importer ses fichiers .map.

## volume déclencheur { #trigger-volume data-forms="volumes déclencheurs" }
Une entité brush invisible qui émet des outputs quand un corps y entre ou en sort, par exemple pour lancer une cinématique quand le joueur entre dans une pièce. Dessinez-en un avec l'outil Volume (Shift+E).

## trim sheet { #trim-sheet data-forms="trim sheets" }
Une texture qui range des bandes de détail, comme des bords, des tuyaux et des panneaux, dans une seule image. Les faces sont ajustées à ses parties avec des hotspots, si bien que quelques trim sheets suffisent à texturer tout un bâtiment.

## triplanaire { #triplanar data-forms="triplanar" }
Une façon de projeter une texture sur une surface depuis trois directions, du dessus et de deux côtés, mélangées selon l'angle de la surface. Les calques de terrain l'utilisent pour que les falaises montrent de la roche au lieu d'une bavure étirée.

## Verrouillage UV { #uv-lock }
Un réglage, activé par défaut et basculé avec Ctrl+Shift+U, qui garde les textures collées à un brush pendant que vous le déplacez. Désactivé, la texture reste en place dans le monde et glisse sur les faces déplacées.

## UV { #uv }
Les coordonnées qui indiquent quelle partie d'une texture tombe sur chaque point d'une face, U dans le sens de la largeur de la texture et V dans le sens de la hauteur. Les faces de brush les reçoivent en général d'une projection, tandis que les meshes et certaines faces stockent les leurs pour chaque coin.

## worldspawn { #worldspawn }
L'entité racine de la carte, qui contient les brushes du monde. Ses clés contiennent des réglages pour toute la carte, comme le ciel, le brouillard, le soleil et le streaming, et l'Inspecteur l'affiche quand rien n'est sélectionné.

## yaw { #yaw }
La rotation autour de l'axe vertical, la direction vers laquelle quelque chose regarde vu du dessus. Le yaw d'une destination de téléportation détermine vers où les corps regardent après leur arrivée.

## z-fighting { #z-fighting }
Des bandes qui clignotent là où deux surfaces se trouvent sur le même plan ou très près. Le GPU ne peut pas dire laquelle est devant, si bien que chaque pixel montre l'une ou l'autre, et le motif bouge avec la caméra. Les decals l'évitent, des brushes fins posés sur un mur non.
